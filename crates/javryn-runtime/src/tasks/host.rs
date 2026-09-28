//! JavaScript global `parallel` API host bindings for Javryn V0.5.

use boa_engine::object::ObjectInitializer;
use boa_engine::object::builtins::{JsArray, JsPromise};
use boa_engine::{Context, JsError, JsValue, NativeFunction, js_string};
use javryn_core::RuntimeError;

use crate::workers::{JsMessage, WorkerResponse, with_worker_manager};

use super::manager::{reset_task_manager, with_task_manager};

/// Host native function for `parallel.map(items, callback)`.
fn host_parallel_map(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    tracing::info!("host_parallel_map called");
    let items_val = args.first().cloned().unwrap_or(JsValue::undefined());
    let callback_val = args.get(1).cloned().unwrap_or(JsValue::undefined());

    let items_obj = items_val.as_object().ok_or_else(|| {
        JsError::from_opaque(JsValue::from(js_string!(
            "parallel.map requires an array as the first argument"
        )))
    })?;

    let js_array = JsArray::from_object(items_obj.clone()).map_err(|_| {
        JsError::from_opaque(JsValue::from(js_string!(
            "parallel.map requires an array as the first argument"
        )))
    })?;

    // Extract callback source string representation
    let fn_source = if let Some(s) = callback_val.as_string() {
        s.to_std_string_escaped()
    } else if let Some(cb_obj) = callback_val.as_object() {
        if !cb_obj.is_callable() {
            return Err(JsError::from_opaque(JsValue::from(js_string!(
                "parallel.map requires a function or string callback as second argument"
            ))));
        }

        let global_fn = context
            .global_object()
            .get(js_string!("Function"), context)
            .map_err(|e| {
                JsError::from_opaque(JsValue::from(js_string!(format!(
                    "failed to get Function global: {e}"
                ))))
            })?
            .as_object()
            .cloned()
            .ok_or_else(|| {
                JsError::from_opaque(JsValue::from(js_string!(
                    "Function global is not an object"
                )))
            })?;

        let fn_proto = global_fn
            .get(js_string!("prototype"), context)
            .map_err(|e| {
                JsError::from_opaque(JsValue::from(js_string!(format!(
                    "failed to get Function.prototype: {e}"
                ))))
            })?
            .as_object()
            .cloned()
            .ok_or_else(|| {
                JsError::from_opaque(JsValue::from(js_string!(
                    "Function.prototype is not an object"
                )))
            })?;

        let fn_to_string = fn_proto
            .get(js_string!("toString"), context)
            .map_err(|e| {
                JsError::from_opaque(JsValue::from(js_string!(format!(
                    "failed to get Function.prototype.toString: {e}"
                ))))
            })?
            .as_object()
            .cloned()
            .ok_or_else(|| {
                JsError::from_opaque(JsValue::from(js_string!(
                    "Function.prototype.toString is not an object"
                )))
            })?;

        let fn_str_val = fn_to_string
            .call(&cb_obj.clone().into(), &[], context)
            .map_err(|e| {
                JsError::from_opaque(JsValue::from(js_string!(format!(
                    "failed to call Function.prototype.toString: {e}"
                ))))
            })?;

        let s = fn_str_val
            .as_string()
            .map(|s| s.to_std_string_escaped())
            .ok_or_else(|| {
                JsError::from_opaque(JsValue::from(js_string!(
                    "toString did not return a string"
                )))
            })?;

        if s.contains("[native code]") {
            return Err(JsError::from_opaque(JsValue::from(js_string!(
                "parallel.map callback function source cannot be serialized. Pass function source string (e.g. 'x => x * x')"
            ))));
        }
        s
    } else {
        return Err(JsError::from_opaque(JsValue::from(js_string!(
            "parallel.map requires a function or string callback as second argument"
        ))));
    };

    tracing::info!(fn_source = %fn_source, "extracted callback fn_source");

    // Serialize items to thread-safe JsMessage array
    let length = js_array.length(context).map_err(|e| {
        JsError::from_opaque(JsValue::from(js_string!(format!(
            "failed to read array length: {e}"
        ))))
    })?;

    let mut serialized_items = Vec::with_capacity(length as usize);
    for i in 0..length {
        let elem = js_array.get(i, context).map_err(|e| {
            JsError::from_opaque(JsValue::from(js_string!(format!(
                "failed to read array item {i}: {e}"
            ))))
        })?;
        let msg = JsMessage::from_js_value(&elem, context)
            .map_err(|e| JsError::from_opaque(JsValue::from(js_string!(e.to_string()))))?;
        serialized_items.push(msg);
    }

    // Create JS Promise capability
    let mut resolve_fn_opt = None;
    let mut reject_fn_opt = None;

    let promise = JsPromise::new(
        |resolving_funcs, _ctx| {
            resolve_fn_opt = Some(resolving_funcs.resolve.clone());
            reject_fn_opt = Some(resolving_funcs.reject.clone());
            Ok(JsValue::undefined())
        },
        context,
    );

    let resolve_fn = resolve_fn_opt.ok_or_else(|| {
        JsError::from_opaque(JsValue::from(js_string!(
            "failed to capture promise resolve function"
        )))
    })?;

    let reject_fn = reject_fn_opt.ok_or_else(|| {
        JsError::from_opaque(JsValue::from(js_string!(
            "failed to capture promise reject function"
        )))
    })?;

    with_task_manager(|m| {
        m.submit_map_operation(
            fn_source,
            serialized_items,
            resolve_fn.into(),
            reject_fn.into(),
            context,
        )
    })
    .map_err(|e| JsError::from_opaque(JsValue::from(js_string!(e.to_string()))))?;

    Ok(JsValue::from(promise))
}

/// Registers the global `parallel` API in the Boa context.
pub fn register_parallel_api(context: &mut Context) -> Result<(), RuntimeError> {
    reset_task_manager();

    let fn_map = NativeFunction::from_fn_ptr(host_parallel_map);
    let parallel_obj = ObjectInitializer::new(context)
        .function(fn_map, js_string!("map"), 2)
        .build();

    let global = context.global_object();
    global
        .set(js_string!("parallel"), parallel_obj, false, context)
        .map_err(|e| RuntimeError::EngineInitialization {
            message: format!("failed to set parallel global API: {e}"),
        })?;

    Ok(())
}

/// Polls responses from WorkerManager and routes task completion/failures through TaskManager.
pub fn dispatch_task_responses(context: &mut Context) -> Result<(), RuntimeError> {
    let responses = with_worker_manager(|m| m.poll_responses());

    for resp in responses {
        match resp {
            WorkerResponse::TaskCompleted { .. } | WorkerResponse::TaskFailed { .. } => {
                with_task_manager(|m| m.handle_task_response(resp, context))?;
            }
            _ => {}
        }
    }

    with_task_manager(|m| m.dispatch_pending_tasks())?;

    Ok(())
}
