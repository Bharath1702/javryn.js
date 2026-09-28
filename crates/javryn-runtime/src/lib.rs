//! # javryn-runtime
//!
//! The Javryn runtime lifecycle engine.
//!
//! This crate owns the full runtime lifecycle:
//! 1. Script validation and metadata extraction
//! 2. Runtime initialization
//! 3. Execution (placeholder in V0.1)
//! 4. Clean shutdown
//!
//! It does **not** contain CLI logic or user-facing output formatting.

pub mod async_runtime;
pub mod engine;
pub mod runtime;
pub mod script;
pub mod tasks;
pub mod workers;

pub use async_runtime::{EventLoop, TimerId, TimerQueue};
pub use engine::boa::BoaEngineAdapter;
pub use engine::{ExecutionResult, JavaScriptEngine};
pub use runtime::Runtime;
pub use script::validate_script;
pub use tasks::TaskManager;
pub use workers::{JsMessage, WorkerHandle, WorkerId, WorkerManager};
