//! Message serialization and communication protocol for Javryn V0.4.

use std::collections::BTreeMap;
use std::path::PathBuf;

use boa_engine::object::ObjectInitializer;
use boa_engine::object::builtins::JsArray;
use boa_engine::{Context, JsValue, js_string};
use javryn_core::RuntimeError;

use super::id::{RequestId, WorkerId};

/// Thread-safe, serializable representation of a JavaScript value.
///
/// Used for message passing between isolated worker JavaScript execution contexts.
#[derive(Debug, Clone, PartialEq)]
pub enum JsMessage {
    /// JavaScript `null`
    Null,
    /// JavaScript `undefined`
    Undefined,
    /// JavaScript boolean (`true` / `false`)
    Boolean(bool),
    /// JavaScript number (`f64`)
    Number(f64),
    /// JavaScript string
    String(String),
    /// JavaScript array of serializable values
    Array(Vec<JsMessage>),
    /// JavaScript plain object mapping property names to serializable values
    Object(BTreeMap<String, JsMessage>),
}

impl JsMessage {
    /// Converts a Boa [`JsValue`] into a serializable [`JsMessage`].
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::WorkerSerialization`] if `value` contains unsupported
    /// types such as functions, symbols, native objects, or promises.
    pub fn from_js_value(value: &JsValue, context: &mut Context) -> Result<Self, RuntimeError> {
        if value.is_null() {
            Ok(JsMessage::Null)
        } else if value.is_undefined() {
            Ok(JsMessage::Undefined)
        } else if let Some(b) = value.as_boolean() {
            Ok(JsMessage::Boolean(b))
        } else if let Some(n) = value.as_number() {
            Ok(JsMessage::Number(n))
        } else if let Some(s) = value.as_string() {
            Ok(JsMessage::String(s.to_std_string_escaped()))
        } else if let Some(obj) = value.as_object() {
            if obj.is_callable() {
                return Err(RuntimeError::WorkerSerialization {
                    message: "cannot serialize Function across worker boundary".to_string(),
                });
            }

            if let Ok(js_array) = JsArray::from_object(obj.clone()) {
                let length = js_array.length(context).map_err(|e| {
                    RuntimeError::WorkerSerialization {
                        message: format!("failed to read array length: {e}"),
                    }
                })?;

                let mut list = Vec::with_capacity(length as usize);
                for i in 0..length {
                    let elem = js_array
                        .get(i, context)
                        .map_err(|e| RuntimeError::WorkerSerialization {
                            message: format!("failed to read array element index {i}: {e}"),
                        })?;
                    list.push(Self::from_js_value(&elem, context)?);
                }
                return Ok(JsMessage::Array(list));
            }

            let keys =
                obj.own_property_keys(context)
                    .map_err(|e| RuntimeError::WorkerSerialization {
                        message: format!("failed to inspect object keys: {e}"),
                    })?;

            let mut map = BTreeMap::new();
            for key in keys {
                let key_str = key.to_string();
                let prop_val = obj.get(key.clone(), context).map_err(|e| {
                    RuntimeError::WorkerSerialization {
                        message: format!("failed to read property '{key_str}': {e}"),
                    }
                })?;
                map.insert(key_str, Self::from_js_value(&prop_val, context)?);
            }
            Ok(JsMessage::Object(map))
        } else {
            Err(RuntimeError::WorkerSerialization {
                message: format!(
                    "unsupported JavaScript type for worker postMessage: {}",
                    value.display()
                ),
            })
        }
    }

    /// Converts this [`JsMessage`] back into a Boa [`JsValue`] within the specified `context`.
    pub fn to_js_value(&self, context: &mut Context) -> Result<JsValue, RuntimeError> {
        match self {
            JsMessage::Null => Ok(JsValue::null()),
            JsMessage::Undefined => Ok(JsValue::undefined()),
            JsMessage::Boolean(b) => Ok(JsValue::from(*b)),
            JsMessage::Number(n) => Ok(JsValue::from(*n)),
            JsMessage::String(s) => Ok(JsValue::from(js_string!(s.as_str()))),
            JsMessage::Array(arr) => {
                let mut js_elems = Vec::with_capacity(arr.len());
                for item in arr {
                    js_elems.push(item.to_js_value(context)?);
                }
                let js_array = JsArray::from_iter(js_elems, context);
                Ok(JsValue::from(js_array))
            }
            JsMessage::Object(map) => {
                let mut converted_props = Vec::with_capacity(map.len());
                for (key, val) in map {
                    converted_props.push((key.clone(), val.to_js_value(context)?));
                }
                let mut initializer = ObjectInitializer::new(context);
                for (key, js_val) in converted_props {
                    initializer.property(
                        js_string!(key.as_str()),
                        js_val,
                        boa_engine::property::Attribute::all(),
                    );
                }
                Ok(initializer.build().into())
            }
        }
    }
}

/// Control message sent from Main Thread / WorkerManager to a Worker thread.
#[derive(Debug)]
pub enum WorkerMessage {
    /// Request worker to load and evaluate a JavaScript file.
    Execute {
        /// Absolute or relative script file path.
        script_path: PathBuf,
    },
    /// Deliver a message payload to the worker's global `onmessage` handler.
    PostMessage {
        /// Request handle ID.
        request_id: RequestId,
        /// Message data payload.
        data: JsMessage,
    },
    /// Request worker event loop shutdown.
    Shutdown,
}

/// Response message sent from a Worker thread back to Main Thread / WorkerManager.
#[derive(Debug)]
pub enum WorkerResponse {
    /// Worker thread initialized successfully and is ready for work.
    Ready {
        /// ID of the started worker.
        worker_id: WorkerId,
    },
    /// Worker script posted a message to the main thread / parent.
    PostMessage {
        /// ID of worker emitting message.
        worker_id: WorkerId,
        /// Request handle ID.
        request_id: RequestId,
        /// Message data payload.
        data: JsMessage,
    },
    /// Worker script threw an unhandled exception or encountered a runtime error.
    Error {
        /// ID of failing worker.
        worker_id: WorkerId,
        /// Error message details.
        error: String,
    },
    /// Worker event loop and engine shut down cleanly.
    Stopped {
        /// ID of stopped worker.
        worker_id: WorkerId,
    },
}
