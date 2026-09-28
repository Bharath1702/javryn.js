//! Isolated worker thread runner and lifecycle management for Javryn V0.4.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use boa_engine::object::ObjectInitializer;
use boa_engine::{Context, JsError, JsValue, NativeFunction, Source, js_string};
use javryn_core::RuntimeError;

use crate::async_runtime::TimerQueue;

use super::id::{RequestId, WorkerId};
use super::message::{JsMessage, WorkerMessage, WorkerResponse};

/// Lifecycle state values for atomic state tracking.
pub const STATE_CREATED: u8 = 0;
pub const STATE_STARTING: u8 = 1;
pub const STATE_RUNNING: u8 = 2;
pub const STATE_STOPPING: u8 = 3;
pub const STATE_STOPPED: u8 = 4;
pub const STATE_FAILED: u8 = 5;

// Thread-local storage for worker message output channel within the worker thread.
thread_local! {
    static WORKER_SENDER: std::cell::RefCell<Option<(WorkerId, Sender<WorkerResponse>)>> = const { std::cell::RefCell::new(None) };
}

/// Handle to a worker thread managed from the main thread.
pub struct WorkerHandle {
    id: WorkerId,
    sender: Sender<WorkerMessage>,
    receiver: Receiver<WorkerResponse>,
    thread_handle: Option<thread::JoinHandle<()>>,
    state: Arc<AtomicU8>,
}

impl WorkerHandle {
    /// Spawns a new independent JS worker thread with an isolated engine context and event loop.
    pub fn spawn(id: WorkerId, initial_script: Option<PathBuf>) -> Result<Self, RuntimeError> {
        let (main_sender, worker_receiver) = channel::<WorkerMessage>();
        let (worker_sender, main_receiver) = channel::<WorkerResponse>();
        let state = Arc::new(AtomicU8::new(STATE_STARTING));

        let thread_state = Arc::clone(&state);
        let thread_initial_script = initial_script.clone();

        let thread_handle = thread::Builder::new()
            .name(format!("javryn-worker-{}", id.0))
            .spawn(move || {
                let state_ref = thread_state;
                let res = std::panic::catch_unwind(|| {
                    run_worker_thread(
                        id,
                        worker_receiver,
                        worker_sender,
                        &state_ref,
                        thread_initial_script,
                    );
                });

                if res.is_err() {
                    state_ref.store(STATE_FAILED, Ordering::SeqCst);
                    tracing::error!(worker_id = id.0, "worker thread panicked");
                }
            })
            .map_err(|e| RuntimeError::WorkerCreation {
                message: format!("failed to spawn worker thread: {e}"),
            })?;

        let handle = Self {
            id,
            sender: main_sender,
            receiver: main_receiver,
            thread_handle: Some(thread_handle),
            state,
        };

        // Wait for ready handshake
        match handle.receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(WorkerResponse::Ready { .. }) => {
                handle.state.store(STATE_RUNNING, Ordering::SeqCst);
                tracing::debug!(worker_id = id.0, "worker startup handshake successful");
                Ok(handle)
            }
            Ok(WorkerResponse::Error { error, .. }) => {
                handle.state.store(STATE_FAILED, Ordering::SeqCst);
                Err(RuntimeError::WorkerInitialization {
                    message: format!("worker startup failed: {error}"),
                })
            }
            Ok(other) => Err(RuntimeError::WorkerInitialization {
                message: format!("unexpected handshake message: {other:?}"),
            }),
            Err(e) => {
                handle.state.store(STATE_FAILED, Ordering::SeqCst);
                Err(RuntimeError::WorkerInitialization {
                    message: format!("worker startup handshake timed out: {e}"),
                })
            }
        }
    }

    /// Returns the unique worker identifier.
    pub fn id(&self) -> WorkerId {
        self.id
    }

    /// Returns the current lifecycle state code of the worker.
    pub fn state_code(&self) -> u8 {
        self.state.load(Ordering::SeqCst)
    }

    /// Returns `true` if the worker thread is running.
    pub fn is_running(&self) -> bool {
        self.state_code() == STATE_RUNNING
    }

    /// Sends a message payload to the worker thread.
    pub fn post_message(&self, request_id: RequestId, data: JsMessage) -> Result<(), RuntimeError> {
        if !self.is_running() {
            return Err(RuntimeError::WorkerCommunication {
                message: format!(
                    "cannot postMessage to worker {} in state {}",
                    self.id,
                    self.state_code()
                ),
            });
        }

        self.sender
            .send(WorkerMessage::PostMessage { request_id, data })
            .map_err(|e| RuntimeError::WorkerCommunication {
                message: format!("failed to send message to worker: {e}"),
            })
    }

    /// Sends a parallel task execution payload to the worker thread.
    pub fn post_task(
        &self,
        task_id: super::id::TaskId,
        fn_source: String,
        arg: JsMessage,
    ) -> Result<(), RuntimeError> {
        if !self.is_running() {
            return Err(RuntimeError::WorkerCommunication {
                message: format!(
                    "cannot post task to worker {} in state {}",
                    self.id,
                    self.state_code()
                ),
            });
        }

        self.sender
            .send(WorkerMessage::ExecuteTask {
                task_id,
                fn_source,
                arg,
            })
            .map_err(|e| RuntimeError::WorkerCommunication {
                message: format!("failed to send task to worker: {e}"),
            })
    }

    /// Non-blocking check for responses from worker thread.
    pub fn try_recv(&self) -> Result<Option<WorkerResponse>, RuntimeError> {
        match self.receiver.try_recv() {
            Ok(resp) => Ok(Some(resp)),
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => Ok(None),
        }
    }

    /// Terminates the worker thread and waits for thread join.
    pub fn terminate(&mut self) -> Result<(), RuntimeError> {
        if self.state_code() == STATE_STOPPED {
            return Ok(());
        }

        self.state.store(STATE_STOPPING, Ordering::SeqCst);
        let _ = self.sender.send(WorkerMessage::Shutdown);

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        self.state.store(STATE_STOPPED, Ordering::SeqCst);
        tracing::debug!(worker_id = self.id.0, "worker thread terminated and joined");
        Ok(())
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

/// Main execution function running inside the worker thread.
fn run_worker_thread(
    id: WorkerId,
    receiver: Receiver<WorkerMessage>,
    sender: Sender<WorkerResponse>,
    state: &Arc<AtomicU8>,
    initial_script: Option<PathBuf>,
) {
    WORKER_SENDER.with(|s| {
        *s.borrow_mut() = Some((id, sender.clone()));
    });

    let mut context = Context::default();

    // Register host functions inside worker JS environment
    if let Err(e) = crate::engine::boa::register_console(&mut context) {
        let _ = sender.send(WorkerResponse::Error {
            worker_id: id,
            error: format!("failed to register worker console: {e}"),
        });
        state.store(STATE_FAILED, Ordering::SeqCst);
        return;
    }

    if let Err(e) = crate::engine::boa::register_timers(&mut context) {
        let _ = sender.send(WorkerResponse::Error {
            worker_id: id,
            error: format!("failed to register worker timers: {e}"),
        });
        state.store(STATE_FAILED, Ordering::SeqCst);
        return;
    }

    if let Err(e) = register_worker_globals(&mut context) {
        let _ = sender.send(WorkerResponse::Error {
            worker_id: id,
            error: format!("failed to register worker globals: {e}"),
        });
        state.store(STATE_FAILED, Ordering::SeqCst);
        return;
    }

    // Handshake signal
    if sender
        .send(WorkerResponse::Ready { worker_id: id })
        .is_err()
    {
        state.store(STATE_FAILED, Ordering::SeqCst);
        return;
    }

    // If an initial script was requested, evaluate it
    if let Some(script_path) = initial_script.as_ref() {
        let res = execute_worker_file(&mut context, script_path);
        if let Err(err) = res {
            let _ = sender.send(WorkerResponse::Error {
                worker_id: id,
                error: err.to_string(),
            });
            state.store(STATE_FAILED, Ordering::SeqCst);
            return;
        }
    }

    let mut timer_queue = TimerQueue::new();

    loop {
        if state.load(Ordering::SeqCst) == STATE_STOPPING {
            break;
        }

        // Drain Promise microtasks first
        context.run_jobs();

        // Check next timer deadline
        let now = Instant::now();
        let timeout_duration = if let Some(deadline) = timer_queue.next_deadline() {
            if deadline > now {
                deadline.duration_since(now)
            } else {
                Duration::from_millis(0)
            }
        } else {
            Duration::from_millis(100) // Default poll interval when idle
        };

        // Multiplex channel receive with timeout (Zero busy-polling!)
        match receiver.recv_timeout(timeout_duration) {
            Ok(WorkerMessage::Execute { script_path }) => {
                if let Err(err) = execute_worker_file(&mut context, &script_path) {
                    let _ = sender.send(WorkerResponse::Error {
                        worker_id: id,
                        error: err.to_string(),
                    });
                }
            }
            Ok(WorkerMessage::ExecuteTask {
                task_id,
                fn_source,
                arg,
            }) => {
                let task_res = execute_task(&mut context, &fn_source, arg);
                tracing::info!(worker_id = id.0, task_id = task_id.0, result = ?task_res, "execute_task result");
                match task_res {
                    Ok(data) => {
                        let _ = sender.send(WorkerResponse::TaskCompleted {
                            worker_id: id,
                            task_id,
                            data,
                        });
                    }
                    Err(err_msg) => {
                        let _ = sender.send(WorkerResponse::TaskFailed {
                            worker_id: id,
                            task_id,
                            error: err_msg,
                        });
                    }
                }
            }
            Ok(WorkerMessage::PostMessage {
                request_id: _,
                data,
            }) => {
                if let Ok(js_val) = data.to_js_value(&mut context) {
                    dispatch_onmessage(&mut context, js_val);
                }
            }
            Ok(WorkerMessage::Shutdown) => {
                break;
            }
            Err(RecvTimeoutError::Timeout) => {
                // Timer deadline reached or periodic poll
            }
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        }

        // Fire due timers
        let now = Instant::now();
        while let Some(entry) = timer_queue.pop_ready(now) {
            if let Some(obj) = entry.callback.as_object() {
                let _ = obj.call(&JsValue::undefined(), &[], &mut context);
            }
            context.run_jobs();
            timer_queue.reschedule_interval(entry);
        }

        // Check worker activity completion
        let has_worker_timers = timer_queue.has_pending();
        let has_onmessage = context
            .global_object()
            .get(js_string!("onmessage"), &mut context)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .is_some();

        if !has_worker_timers && !has_onmessage && initial_script.is_some() {
            tracing::debug!(
                worker_id = id.0,
                "worker has no pending timers or message handlers: stopping"
            );
            break;
        }
    }

    state.store(STATE_STOPPED, Ordering::SeqCst);
    let _ = sender.send(WorkerResponse::Stopped { worker_id: id });
    WORKER_SENDER.with(|s| {
        *s.borrow_mut() = None;
    });
}

/// Host native function for worker child script: `postMessage(data)`.
fn host_worker_post_message(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let data_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let js_msg = JsMessage::from_js_value(&data_val, context)
        .map_err(|e| JsError::from_opaque(JsValue::from(js_string!(e.to_string()))))?;

    WORKER_SENDER.with(|s| {
        if let Some((worker_id, ref sender)) = *s.borrow() {
            let _ = sender.send(WorkerResponse::PostMessage {
                worker_id,
                request_id: RequestId(0),
                data: js_msg,
            });
        }
    });

    Ok(JsValue::undefined())
}

/// Registers global worker child environment bindings (`postMessage`).
fn register_worker_globals(context: &mut Context) -> Result<(), RuntimeError> {
    let realm = context.realm();
    let fn_post_message =
        NativeFunction::from_fn_ptr(host_worker_post_message).to_js_function(realm);

    let global = context.global_object();
    global
        .set(js_string!("postMessage"), fn_post_message, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set worker postMessage global: {e}"),
        })?;

    Ok(())
}

/// Dispatches an incoming JS message to the global `onmessage` callback in the worker context.
fn dispatch_onmessage(context: &mut Context, message_data: JsValue) {
    let global = context.global_object();
    let maybe_cb = global
        .get(js_string!("onmessage"), context)
        .ok()
        .and_then(|v| v.as_object().cloned());

    if let Some(obj) = maybe_cb {
        let event_obj = ObjectInitializer::new(context)
            .property(
                js_string!("data"),
                message_data,
                boa_engine::property::Attribute::all(),
            )
            .build();
        let _ = obj.call(&JsValue::undefined(), &[event_obj.into()], context);
    }
}

/// Helper to execute a JS source file inside a worker context.
fn execute_worker_file(context: &mut Context, path: &Path) -> Result<(), RuntimeError> {
    let source_code =
        std::fs::read_to_string(path).map_err(|e| RuntimeError::ScriptUnreadable {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

    let source = Source::from_bytes(source_code.as_bytes()).with_path(path);
    context
        .eval(source)
        .map_err(|err| RuntimeError::JavaScriptExecution {
            path: path.to_path_buf(),
            error_type: "WorkerError".to_string(),
            message: err.to_string(),
            stack: None,
        })?;

    Ok(())
}

/// Helper to execute a parallel task callback source with argument inside worker context.
fn execute_task(
    context: &mut Context,
    fn_source: &str,
    arg: JsMessage,
) -> Result<JsMessage, String> {
    let arg_val = arg.to_js_value(context).map_err(|e| e.to_string())?;

    let wrapped_source = format!("({fn_source})");
    let fn_val = context
        .eval(Source::from_bytes(wrapped_source.as_bytes()))
        .map_err(|e| format!("Failed to compile task callback function: {e}"))?;

    let obj = fn_val
        .as_object()
        .ok_or_else(|| "Task callback source is not a callable Function".to_string())?;

    if !obj.is_callable() {
        return Err("Task callback source is not a callable Function".to_string());
    }

    let ret_val = obj
        .call(&JsValue::undefined(), &[arg_val], context)
        .map_err(|e| e.to_string())?;

    JsMessage::from_js_value(&ret_val, context).map_err(|e| e.to_string())
}
