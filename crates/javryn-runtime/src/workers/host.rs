//! JavaScript global `Worker` constructor and instance method host bindings for Javryn V0.4.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use boa_engine::object::ObjectInitializer;
use boa_engine::{Context, JsError, JsObject, JsValue, NativeFunction, js_string};
use javryn_core::RuntimeError;

use super::id::WorkerId;
use super::manager::with_worker_manager;
use super::message::{JsMessage, WorkerResponse};

thread_local! {
    /// Registry mapping WorkerId to JS Worker instances in main thread realm.
    static WORKER_OBJECTS: RefCell<HashMap<WorkerId, JsObject>> = RefCell::new(HashMap::new());
}

pub fn reset_worker_objects() {
    WORKER_OBJECTS.with(|r| r.borrow_mut().clear());
}

/// Returns `true` if any JS `Worker` constructor instances are registered in the main thread.
pub fn has_active_worker_objects() -> bool {
    WORKER_OBJECTS.with(|r| !r.borrow().is_empty())
}

fn register_worker_object(id: WorkerId, obj: JsObject) {
    WORKER_OBJECTS.with(|r| r.borrow_mut().insert(id, obj));
}

/// Host constructor function: `new Worker(scriptPath)`.
fn host_worker_constructor(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let script_path_str = args
        .first()
        .and_then(|v| v.as_string())
        .map(|s| s.to_std_string_escaped())
        .ok_or_else(|| {
            JsError::from_opaque(JsValue::from(js_string!(
                "Worker constructor requires a script path string argument"
            )))
        })?;

    let path = PathBuf::from(script_path_str);
    let worker_id = with_worker_manager(|m| m.spawn_worker(Some(path))).map_err(|e| {
        JsError::from_opaque(JsValue::from(js_string!(format!(
            "Failed to spawn worker: {e}"
        ))))
    })?;

    let fn_post_message = NativeFunction::from_fn_ptr(host_worker_instance_post_message);
    let fn_terminate = NativeFunction::from_fn_ptr(host_worker_instance_terminate);

    let worker_obj = ObjectInitializer::new(context)
        .property(
            js_string!("__worker_id"),
            JsValue::from(worker_id.0 as f64),
            boa_engine::property::Attribute::all(),
        )
        .function(fn_post_message, js_string!("postMessage"), 1)
        .function(fn_terminate, js_string!("terminate"), 0)
        .build();

    register_worker_object(worker_id, worker_obj.clone());

    Ok(worker_obj.into())
}

/// Host instance method: `worker.postMessage(data)`.
fn host_worker_instance_post_message(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let worker_obj = this.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(js_string!(
            "postMessage called on non-object"
        )))
    })?;

    let id_num = worker_obj
        .get(js_string!("__worker_id"), context)
        .ok()
        .and_then(|v| v.as_number())
        .ok_or_else(|| {
            JsError::from_opaque(JsValue::from(js_string!(
                "Worker object missing __worker_id"
            )))
        })?;

    let worker_id = WorkerId(id_num as u64);
    let data_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let js_msg = JsMessage::from_js_value(&data_val, context)
        .map_err(|e| JsError::from_opaque(JsValue::from(js_string!(e.to_string()))))?;

    with_worker_manager(|m| m.post_message(worker_id, js_msg)).map_err(|e| {
        JsError::from_opaque(JsValue::from(js_string!(format!(
            "postMessage failed: {e}"
        ))))
    })?;

    Ok(JsValue::undefined())
}

/// Host instance method: `worker.terminate()`.
fn host_worker_instance_terminate(
    this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let worker_obj = this.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(js_string!("terminate called on non-object")))
    })?;

    let id_num = worker_obj
        .get(js_string!("__worker_id"), context)
        .ok()
        .and_then(|v| v.as_number())
        .ok_or_else(|| {
            JsError::from_opaque(JsValue::from(js_string!(
                "Worker object missing __worker_id"
            )))
        })?;

    let worker_id = WorkerId(id_num as u64);
    with_worker_manager(|m| m.terminate_worker(worker_id)).map_err(|e| {
        JsError::from_opaque(JsValue::from(js_string!(format!("terminate failed: {e}"))))
    })?;

    WORKER_OBJECTS.with(|r| r.borrow_mut().remove(&worker_id));

    Ok(JsValue::undefined())
}

/// Registers the global `Worker` constructor in the main thread context.
pub fn register_worker_constructor(context: &mut Context) -> Result<(), RuntimeError> {
    reset_worker_objects();

    let realm = context.realm();
    let worker_ctor = boa_engine::object::FunctionObjectBuilder::new(
        realm,
        NativeFunction::from_fn_ptr(host_worker_constructor),
    )
    .constructor(true)
    .name(js_string!("Worker"))
    .build();

    let global = context.global_object();
    global
        .set(js_string!("Worker"), worker_ctor, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set Worker global: {e}"),
        })?;

    Ok(())
}

/// Polls responses from WorkerManager and dispatches `onmessage` and `onerror` events to JS Worker instances.
pub fn dispatch_worker_responses(context: &mut Context) -> Result<(), RuntimeError> {
    let responses = with_worker_manager(|m| m.poll_responses());

    for resp in responses {
        match resp {
            WorkerResponse::PostMessage {
                worker_id,
                request_id: _,
                data,
            } => {
                let maybe_cb = WORKER_OBJECTS.with(|r| {
                    r.borrow()
                        .get(&worker_id)
                        .cloned()
                        .and_then(|obj| obj.get(js_string!("onmessage"), context).ok())
                        .and_then(|val| val.as_object().cloned())
                });

                if let Some(cb) = maybe_cb {
                    let data_js = data.to_js_value(context)?;
                    let event_obj = ObjectInitializer::new(context)
                        .property(
                            js_string!("data"),
                            data_js,
                            boa_engine::property::Attribute::all(),
                        )
                        .build();
                    let _ = cb.call(&JsValue::undefined(), &[event_obj.into()], context);
                }
            }
            WorkerResponse::Error { worker_id, error } => {
                let maybe_cb = WORKER_OBJECTS.with(|r| {
                    r.borrow()
                        .get(&worker_id)
                        .cloned()
                        .and_then(|obj| obj.get(js_string!("onerror"), context).ok())
                        .and_then(|val| val.as_object().cloned())
                });

                if let Some(cb) = maybe_cb {
                    let err_obj = ObjectInitializer::new(context)
                        .property(
                            js_string!("message"),
                            js_string!(error.as_str()),
                            boa_engine::property::Attribute::all(),
                        )
                        .build();
                    let _ = cb.call(&JsValue::undefined(), &[err_obj.into()], context);
                }

                crate::tasks::manager::with_task_manager(|m| {
                    let _ = m.handle_worker_failure(worker_id, &error, context);
                });
            }
            WorkerResponse::Stopped { worker_id } => {
                WORKER_OBJECTS.with(|r| r.borrow_mut().remove(&worker_id));
                with_worker_manager(|m| {
                    let _ = m.terminate_worker(worker_id);
                });
                crate::tasks::manager::with_task_manager(|m| {
                    let _ = m.handle_worker_failure(worker_id, "Worker terminated", context);
                });
            }
            WorkerResponse::Ready { .. } => {}
            WorkerResponse::TaskCompleted { .. } | WorkerResponse::TaskFailed { .. } => {
                crate::tasks::manager::with_task_manager(|m| {
                    m.handle_task_response(resp, context)
                })?;
            }
        }
    }

    crate::tasks::manager::with_task_manager(|m| m.dispatch_pending_tasks())?;

    Ok(())
}
