//! Task manager overseeing parallel operations, queuing, worker assignment, and result collection for Javryn V0.5.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

use boa_engine::object::ObjectInitializer;
use boa_engine::object::builtins::JsArray;
use boa_engine::{Context, JsObject, JsValue, js_string};
use javryn_core::RuntimeError;

use crate::workers::id::{OperationId, TaskId};
use crate::workers::{JsMessage, WorkerResponse, with_worker_manager};

use super::task::{ParallelOperation, ParallelTask, TaskStatus};

static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// Generates a new unique `OperationId`.
pub fn next_op_id() -> OperationId {
    OperationId(NEXT_OP_ID.fetch_add(1, Ordering::Relaxed))
}

/// Generates a new unique `TaskId`.
pub fn next_task_id() -> TaskId {
    TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
}

/// Central manager tracking parallel operations and tasks.
pub struct TaskManager {
    pending_tasks: VecDeque<ParallelTask>,
    active_tasks: HashMap<TaskId, ParallelTask>,
    operations: HashMap<OperationId, ParallelOperation>,
    pool_size: usize,
}

impl TaskManager {
    /// Creates a new `TaskManager` with configured worker pool target size.
    pub fn new() -> Self {
        let default_pool_size = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2)
            .max(2);

        Self {
            pending_tasks: VecDeque::new(),
            active_tasks: HashMap::new(),
            operations: HashMap::new(),
            pool_size: default_pool_size,
        }
    }

    /// Submits a `parallel.map` operation to the task manager.
    pub fn submit_map_operation(
        &mut self,
        fn_source: String,
        items: Vec<JsMessage>,
        resolve_fn: JsObject,
        reject_fn: JsObject,
        context: &mut Context,
    ) -> Result<OperationId, RuntimeError> {
        let op_id = next_op_id();

        if items.is_empty() {
            let empty_array = JsArray::new(context);
            let _ = resolve_fn.call(&JsValue::undefined(), &[empty_array.into()], context);
            return Ok(op_id);
        }

        let total_tasks = items.len();
        let mut results = Vec::with_capacity(total_tasks);
        for _ in 0..total_tasks {
            results.push(None);
        }

        let operation = ParallelOperation {
            op_id,
            total_tasks,
            completed_tasks: 0,
            results,
            resolve_fn,
            reject_fn,
            error: None,
            is_cancelled: false,
        };

        self.operations.insert(op_id, operation);

        for (idx, item) in items.into_iter().enumerate() {
            let task_id = next_task_id();
            let task = ParallelTask {
                task_id,
                op_id,
                input_index: idx,
                fn_source: fn_source.clone(),
                arg: item,
                status: TaskStatus::Queued,
                assigned_worker: None,
            };
            self.pending_tasks.push_back(task);
        }

        self.ensure_worker_pool()?;
        self.dispatch_pending_tasks()?;

        Ok(op_id)
    }

    /// Ensures the worker manager pool has spawned up to `self.pool_size` workers.
    fn ensure_worker_pool(&mut self) -> Result<(), RuntimeError> {
        with_worker_manager(|m| {
            while m.len() < self.pool_size {
                let _ = m.spawn_worker(None)?;
            }
            Ok(())
        })
    }

    /// Dispatches pending queued tasks to idle workers in the worker pool.
    pub fn dispatch_pending_tasks(&mut self) -> Result<(), RuntimeError> {
        if self.pending_tasks.is_empty() {
            return Ok(());
        }

        let active_worker_ids = with_worker_manager(|m| m.active_worker_ids());

        for worker_id in active_worker_ids {
            if self.pending_tasks.is_empty() {
                break;
            }

            // Check if worker is currently processing a task
            let is_worker_busy = self
                .active_tasks
                .values()
                .any(|t| t.assigned_worker == Some(worker_id));

            if !is_worker_busy {
                let Some(mut task) = self.pending_tasks.pop_front() else {
                    continue;
                };

                task.status = TaskStatus::Running;
                task.assigned_worker = Some(worker_id);

                let task_id = task.task_id;
                let fn_source = task.fn_source.clone();
                let arg = task.arg.clone();

                let res = with_worker_manager(|m| m.post_task(worker_id, task_id, fn_source, arg));

                if res.is_ok() {
                    self.active_tasks.insert(task_id, task);
                } else {
                    // Put task back into pending queue if worker post failed
                    task.status = TaskStatus::Queued;
                    task.assigned_worker = None;
                    self.pending_tasks.push_front(task);
                }
            }
        }

        Ok(())
    }

    /// Processes worker task response messages.
    pub fn handle_task_response(
        &mut self,
        response: WorkerResponse,
        context: &mut Context,
    ) -> Result<(), RuntimeError> {
        tracing::debug!(response = ?response, "TaskManager::handle_task_response called");
        match response {
            WorkerResponse::TaskCompleted {
                worker_id: _,
                task_id,
                data,
            } => {
                tracing::info!(task_id = task_id.0, "TaskManager received TaskCompleted");
                if let Some(task) = self.active_tasks.remove(&task_id) {
                    let op_id = task.op_id;
                    let idx = task.input_index;

                    if let Some(op) = self.operations.get_mut(&op_id) {
                        if op.is_cancelled {
                            return Ok(());
                        }

                        op.results[idx] = Some(data);
                        op.completed_tasks += 1;

                        if op.completed_tasks == op.total_tasks {
                            // Convert results to ordered JS Array
                            let mut js_results = Vec::with_capacity(op.total_tasks);
                            for item in &op.results {
                                if let Some(val) = item {
                                    js_results.push(val.to_js_value(context)?);
                                } else {
                                    js_results.push(JsValue::undefined());
                                }
                            }

                            let js_array = JsArray::from_iter(js_results, context);
                            if let Err(e) = op.resolve_fn.call(
                                &JsValue::undefined(),
                                &[js_array.into()],
                                context,
                            ) {
                                tracing::error!(error = %e, "failed to call resolve_fn");
                            } else {
                                tracing::debug!("resolve_fn called successfully");
                            }

                            self.operations.remove(&op_id);
                        }
                    }
                }
            }
            WorkerResponse::TaskFailed {
                worker_id: _,
                task_id,
                error,
            } => {
                if let Some(task) = self.active_tasks.remove(&task_id) {
                    let op_id = task.op_id;
                    if let Some(op) = self.operations.remove(&op_id) {
                        let err_obj = ObjectInitializer::new(context)
                            .property(
                                js_string!("message"),
                                js_string!(error.as_str()),
                                boa_engine::property::Attribute::all(),
                            )
                            .build();

                        let _ =
                            op.reject_fn
                                .call(&JsValue::undefined(), &[err_obj.into()], context);

                        // Cancel remaining queued tasks for this operation
                        self.pending_tasks.retain(|t| t.op_id != op_id);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handles worker thread error or sudden termination, failing any active tasks assigned to the worker.
    pub fn handle_worker_failure(
        &mut self,
        worker_id: crate::workers::WorkerId,
        error: &str,
        context: &mut Context,
    ) -> Result<(), RuntimeError> {
        let failed_task_ids: Vec<TaskId> = self
            .active_tasks
            .values()
            .filter(|t| t.assigned_worker == Some(worker_id))
            .map(|t| t.task_id)
            .collect();

        for task_id in failed_task_ids {
            if let Some(task) = self.active_tasks.remove(&task_id) {
                let op_id = task.op_id;
                if let Some(op) = self.operations.remove(&op_id) {
                    let err_obj = ObjectInitializer::new(context)
                        .property(
                            js_string!("message"),
                            js_string!(format!(
                                "Worker thread {worker_id} failed during task execution: {error}"
                            )),
                            boa_engine::property::Attribute::all(),
                        )
                        .build();

                    let _ = op
                        .reject_fn
                        .call(&JsValue::undefined(), &[err_obj.into()], context);

                    self.pending_tasks.retain(|t| t.op_id != op_id);
                }
            }
        }

        Ok(())
    }

    /// Returns `true` if any parallel operations or tasks are active/pending.
    pub fn has_pending(&self) -> bool {
        !self.pending_tasks.is_empty()
            || !self.active_tasks.is_empty()
            || !self.operations.is_empty()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    /// Main thread thread-local TaskManager instance.
    static TASK_MANAGER: std::cell::RefCell<TaskManager> = std::cell::RefCell::new(TaskManager::new());
}

/// Accesses the main thread's local `TaskManager`.
pub fn with_task_manager<F, R>(f: F) -> R
where
    F: FnOnce(&mut TaskManager) -> R,
{
    TASK_MANAGER.with(|m| f(&mut m.borrow_mut()))
}

/// Resets the main thread's local `TaskManager`.
pub fn reset_task_manager() {
    TASK_MANAGER.with(|m| {
        *m.borrow_mut() = TaskManager::new();
    });
}
