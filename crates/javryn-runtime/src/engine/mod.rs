//! Engine Abstraction Layer for Javryn.
//!
//! Provides a unified [`JavaScriptEngine`] trait and execution structures
//! isolating engine-specific implementation details (e.g. Boa, QuickJS, V8)
//! from `javryn-runtime` and `javryn-cli`.

pub mod boa;
#[cfg(test)]
pub mod mock;

use std::path::Path;

use javryn_core::{RuntimeError, RuntimeMode};

/// The result of executing a JavaScript program through an engine adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    /// Indicates whether execution completed successfully without unhandled exceptions.
    pub success: bool,

    /// Human-readable representation of the evaluation result, if any.
    pub return_value: Option<String>,
}

impl ExecutionResult {
    /// Creates a new successful [`ExecutionResult`].
    pub fn success(return_value: Option<String>) -> Self {
        Self {
            success: true,
            return_value,
        }
    }
}

/// Abstract contract for JavaScript runtime engine adapters.
///
/// Any engine integrated into Javryn must implement this trait.
/// This prevents engine-specific types from leaking into higher layers.
pub trait JavaScriptEngine {
    /// Initializes the engine isolate/context and registers global environment bindings.
    fn initialize(&mut self, mode: RuntimeMode) -> Result<(), RuntimeError>;

    /// Executes JavaScript source code associated with a canonical file path.
    fn execute(&mut self, source: &str, path: &Path) -> Result<ExecutionResult, RuntimeError>;

    /// Cleanly releases engine resources.
    fn shutdown(&mut self) -> Result<(), RuntimeError>;
}
