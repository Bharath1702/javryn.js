//! Boa JavaScript engine adapter for Javryn.
//!
//! Wraps `boa_engine::Context` and implements [`JavaScriptEngine`].

use std::cell::RefCell;
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use boa_engine::module::Module;
use boa_engine::object::ObjectInitializer;
use boa_engine::{Context, JsError, JsValue, NativeFunction, Source, js_string};
use javryn_core::{RuntimeError, RuntimeMode};

use super::{ExecutionResult, JavaScriptEngine};
use crate::async_runtime::{EventLoop, TimerId, TimerQueue};

thread_local! {
    /// Thread-local timer queue owned by the single-threaded execution thread.
    static TIMER_QUEUE: RefCell<TimerQueue> = RefCell::new(TimerQueue::new());
}

fn reset_timer_queue() {
    TIMER_QUEUE.with(|q| q.borrow_mut().clear());
}

/// Atomic store for current runtime diagnostic mode (0=Normal, 1=Debug, 2=Verbose, 3=Quiet)
static RUNTIME_MODE: AtomicU8 = AtomicU8::new(0);

fn set_mode_atomic(mode: RuntimeMode) {
    let val = match mode {
        RuntimeMode::Normal => 0,
        RuntimeMode::Debug => 1,
        RuntimeMode::Verbose => 2,
        RuntimeMode::Quiet => 3,
    };
    RUNTIME_MODE.store(val, Ordering::Relaxed);
}

fn get_mode_atomic() -> RuntimeMode {
    match RUNTIME_MODE.load(Ordering::Relaxed) {
        1 => RuntimeMode::Debug,
        2 => RuntimeMode::Verbose,
        3 => RuntimeMode::Quiet,
        _ => RuntimeMode::Normal,
    }
}

/// Adapter wrapping the `boa_engine` JavaScript VM context.
pub struct BoaEngineAdapter {
    context: Option<Context>,
}

impl BoaEngineAdapter {
    /// Creates a new uninitialized `BoaEngineAdapter`.
    pub fn new() -> Self {
        Self { context: None }
    }
}

impl Default for BoaEngineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptEngine for BoaEngineAdapter {
    fn initialize(&mut self, mode: RuntimeMode) -> Result<(), RuntimeError> {
        set_mode_atomic(mode);
        reset_timer_queue();
        crate::workers::reset_worker_manager();
        crate::tasks::manager::reset_task_manager();

        let mut context = Context::default();

        // Register host console object, timer APIs, Worker constructor, and parallel API into JS global scope
        register_console(&mut context)?;
        register_timers(&mut context)?;
        crate::workers::register_worker_constructor(&mut context)?;
        crate::tasks::register_parallel_api(&mut context)?;

        self.context = Some(context);
        tracing::debug!(mode = %mode, "Boa JavaScript engine initialized");

        Ok(())
    }

    fn execute(&mut self, source_code: &str, path: &Path) -> Result<ExecutionResult, RuntimeError> {
        let context = self
            .context
            .as_mut()
            .ok_or_else(|| RuntimeError::EngineInitialization {
                message: "engine execute called before initialization".to_string(),
            })?;

        tracing::debug!(file = %path.display(), "evaluating JavaScript source in Boa engine");
        tracing::info!(file = %path.display(), "JavaScript execution started");

        let source = Source::from_bytes(source_code.as_bytes()).with_path(path);
        let is_mjs = path.extension().and_then(|e| e.to_str()) == Some("mjs");

        // Step 1: Initial script / module evaluation
        if is_mjs {
            match Module::parse(source, None, context) {
                Ok(module) => {
                    let _promise = module.load_link_evaluate(context);
                }
                Err(js_error) => return Err(convert_js_error(js_error, path, context)),
            }
        } else {
            match context.eval(source) {
                Ok(_val) => {}
                Err(js_error) => return Err(convert_js_error(js_error, path, context)),
            }
        }

        // Step 2: Run event loop to drain microtasks, process timers, and poll worker responses
        TIMER_QUEUE.with(|q| EventLoop::run(context, &mut q.borrow_mut(), path))?;

        tracing::info!(
            file = %path.display(),
            "JavaScript execution and event loop completed successfully"
        );

        Ok(ExecutionResult::success(None))
    }

    fn shutdown(&mut self) -> Result<(), RuntimeError> {
        tracing::debug!("shutting down Boa engine adapter");
        reset_timer_queue();
        crate::workers::reset_worker_manager();
        crate::tasks::manager::reset_task_manager();
        self.context = None;
        Ok(())
    }
}

/// Host native console functions.
fn console_log(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    if get_mode_atomic() != RuntimeMode::Quiet {
        let formatted = format_js_args(args);
        println!("{formatted}");
    }
    Ok(JsValue::undefined())
}

fn console_info(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    if get_mode_atomic() != RuntimeMode::Quiet {
        let formatted = format_js_args(args);
        println!("{formatted}");
    }
    Ok(JsValue::undefined())
}

fn console_warn(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let formatted = format_js_args(args);
    eprintln!("{formatted}");
    Ok(JsValue::undefined())
}

fn console_error(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let formatted = format_js_args(args);
    eprintln!("{formatted}");
    Ok(JsValue::undefined())
}

/// Registers the global `console` object in the Boa context.
pub(crate) fn register_console(context: &mut Context) -> Result<(), RuntimeError> {
    let console = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(console_log),
            js_string!("log"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(console_info),
            js_string!("info"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(console_warn),
            js_string!("warn"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(console_error),
            js_string!("error"),
            0,
        )
        .build();

    let global = context.global_object();
    global
        .set(js_string!("console"), console, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set console global: {e}"),
        })?;

    Ok(())
}

/// Host timer bindings for `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`, `queueMicrotask`.
fn host_set_timeout(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let delay_ms = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n.max(0.0) as u64)
        .unwrap_or(0);

    let id = TIMER_QUEUE.with(|q| {
        q.borrow_mut()
            .set_timeout(callback, Duration::from_millis(delay_ms))
    });

    Ok(JsValue::from(id.0 as f64))
}

fn host_clear_timeout(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    if let Some(id_num) = args.first().and_then(|v| v.as_number()) {
        let id = TimerId(id_num as u64);
        TIMER_QUEUE.with(|q| q.borrow_mut().cancel(id));
    }
    Ok(JsValue::undefined())
}

fn host_set_interval(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    let interval_ms = args
        .get(1)
        .and_then(|v| v.as_number())
        .map(|n| n.max(0.0) as u64)
        .unwrap_or(0);

    let id = TIMER_QUEUE.with(|q| {
        q.borrow_mut()
            .set_interval(callback, Duration::from_millis(interval_ms))
    });

    Ok(JsValue::from(id.0 as f64))
}

fn host_clear_interval(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    if let Some(id_num) = args.first().and_then(|v| v.as_number()) {
        let id = TimerId(id_num as u64);
        TIMER_QUEUE.with(|q| q.borrow_mut().cancel(id));
    }
    Ok(JsValue::undefined())
}

fn host_queue_microtask(
    _this: &JsValue,
    args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = args.first().cloned().unwrap_or(JsValue::undefined());
    // queueMicrotask schedules zero-delay timer in event loop for prompt microtask processing
    TIMER_QUEUE.with(|q| {
        q.borrow_mut()
            .set_timeout(callback, Duration::from_millis(0))
    });

    Ok(JsValue::undefined())
}

/// Registers timer host functions in the Boa context global scope.
pub(crate) fn register_timers(context: &mut Context) -> Result<(), RuntimeError> {
    let realm = context.realm();
    let fn_set_timeout = NativeFunction::from_fn_ptr(host_set_timeout).to_js_function(realm);
    let fn_clear_timeout = NativeFunction::from_fn_ptr(host_clear_timeout).to_js_function(realm);
    let fn_set_interval = NativeFunction::from_fn_ptr(host_set_interval).to_js_function(realm);
    let fn_clear_interval = NativeFunction::from_fn_ptr(host_clear_interval).to_js_function(realm);
    let fn_queue_microtask =
        NativeFunction::from_fn_ptr(host_queue_microtask).to_js_function(realm);

    let global = context.global_object();

    global
        .set(js_string!("setTimeout"), fn_set_timeout, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set setTimeout global: {e}"),
        })?;

    global
        .set(js_string!("clearTimeout"), fn_clear_timeout, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set clearTimeout global: {e}"),
        })?;

    global
        .set(js_string!("setInterval"), fn_set_interval, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set setInterval global: {e}"),
        })?;

    global
        .set(
            js_string!("clearInterval"),
            fn_clear_interval,
            false,
            context,
        )
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set clearInterval global: {e}"),
        })?;

    global
        .set(
            js_string!("queueMicrotask"),
            fn_queue_microtask,
            false,
            context,
        )
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set queueMicrotask global: {e}"),
        })?;

    Ok(())
}

/// Formats arguments passed to console host functions.
fn format_js_args(args: &[JsValue]) -> String {
    args.iter()
        .map(|arg| {
            if let Some(s) = arg.as_string() {
                s.to_std_string_escaped()
            } else {
                arg.display().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Converts a Boa [`JsError`] into a structured [`RuntimeError`].
fn convert_js_error(err: JsError, path: &Path, context: &mut Context) -> RuntimeError {
    let error_string = err.to_string();

    if error_string.contains("SyntaxError") || error_string.contains("Parsing error") {
        return RuntimeError::JavaScriptSyntax {
            path: path.to_path_buf(),
            reason: error_string,
        };
    }

    let value = err.to_opaque(context);
    let (error_type, message) = if let Some(obj) = value.as_object() {
        let name = obj
            .get(js_string!("name"), context)
            .ok()
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_else(|| "Error".to_string());

        let msg = obj
            .get(js_string!("message"), context)
            .ok()
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_else(|| value.display().to_string());

        (name, msg)
    } else {
        ("Error".to_string(), value.display().to_string())
    };

    RuntimeError::JavaScriptExecution {
        path: path.to_path_buf(),
        error_type,
        message,
        stack: None,
    }
}
