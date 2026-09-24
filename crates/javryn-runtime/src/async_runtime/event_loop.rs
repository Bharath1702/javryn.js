//! Single-threaded event loop for the Javryn async runtime.

use std::path::Path;
use std::time::Instant;

use boa_engine::{Context, JsError, JsValue};
use javryn_core::RuntimeError;

use super::timer::TimerQueue;

/// The single-threaded event loop engine for Javryn.
///
/// Coordinates execution of microtasks (Promise jobs) and macro-tasks (timers).
pub struct EventLoop;

impl EventLoop {
    /// Drives the event loop until all Promise microtasks and pending timers complete.
    ///
    /// # Lifecycle Phases per Iteration
    ///
    /// 1. Drain Promise job queue / microtasks via `context.run_jobs()`.
    /// 2. Check timer queue:
    ///    - Fire due timers (`due_time <= Instant::now()`) and call their JS callbacks.
    ///    - Drain microtasks again after executing timer callbacks.
    ///    - Reschedule interval timers if active.
    /// 3. If pending timers remain, sleep non-blocking until the next timer's `due_time`.
    /// 4. Repeat until no pending timers exist and no Promise jobs remain.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::JavaScriptExecution`] if a Promise job or timer callback
    /// throws an unhandled JavaScript exception.
    pub fn run(
        context: &mut Context,
        timer_queue: &mut TimerQueue,
        path: &Path,
    ) -> Result<(), RuntimeError> {
        tracing::debug!("starting event loop execution");

        loop {
            // Phase 1: Drain all pending Promise microtasks/job queue and dispatch worker responses
            context.run_jobs();
            crate::workers::dispatch_worker_responses(context)?;

            // Phase 2: Fire ready timers
            let now = Instant::now();

            while let Some(entry) = timer_queue.pop_ready(now) {
                tracing::debug!(timer_id = entry.id.0, "firing ready timer callback");

                if let Some(obj) = entry.callback.as_object() {
                    obj.call(&JsValue::undefined(), &[], context)
                        .map_err(|err| convert_event_loop_error(err, path, context))?;
                }

                // Drain microtasks & worker messages enqueued during timer callback
                context.run_jobs();
                crate::workers::dispatch_worker_responses(context)?;

                // Reschedule if interval timer
                timer_queue.reschedule_interval(entry);
            }

            // Phase 3: Check completion condition
            let has_timers = timer_queue.has_pending();
            let has_workers = crate::workers::with_worker_manager(|m| !m.is_empty());

            if !has_timers && !has_workers {
                tracing::debug!("event loop completed: no pending timers or active workers remain");
                break;
            }

            // Phase 4: Non-blocking sleep until next deadline or brief poll for worker responses
            let sleep_duration = if let Some(deadline) = timer_queue.next_deadline() {
                let now = Instant::now();
                if deadline > now {
                    deadline
                        .duration_since(now)
                        .min(std::time::Duration::from_millis(10))
                } else {
                    std::time::Duration::from_millis(0)
                }
            } else if has_workers {
                std::time::Duration::from_millis(10) // Poll interval for worker responses when no timers active
            } else {
                std::time::Duration::from_millis(0)
            };

            if sleep_duration > std::time::Duration::from_millis(0) {
                tracing::debug!(sleep_ms = sleep_duration.as_millis(), "event loop sleeping");
                std::thread::sleep(sleep_duration);
            }
        }

        Ok(())
    }
}

/// Converts a [`JsError`] encountered during event loop execution into a [`RuntimeError`].
fn convert_event_loop_error(err: JsError, path: &Path, context: &mut Context) -> RuntimeError {
    let value = err.to_opaque(context);
    let (error_type, message) = if let Some(obj) = value.as_object() {
        let name = obj
            .get(boa_engine::js_string!("name"), context)
            .ok()
            .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
            .unwrap_or_else(|| "Error".to_string());

        let msg = obj
            .get(boa_engine::js_string!("message"), context)
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
