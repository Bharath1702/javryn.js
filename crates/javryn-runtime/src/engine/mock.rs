//! Mock JavaScript engine adapter for testing runtime error paths and lifecycle behavior.

use std::path::Path;

use javryn_core::{RuntimeError, RuntimeMode};

use super::{ExecutionResult, JavaScriptEngine};

/// A mock engine implementation used in failure testing.
#[derive(Debug, Default)]
pub struct MockJavaScriptEngine {
    pub fail_init: bool,
    pub fail_exec: bool,
    pub fail_shutdown: bool,
    pub initialized: bool,
    pub shut_down: bool,
}

impl MockJavaScriptEngine {
    /// Creates a default mock engine.
    pub fn new() -> Self {
        Self::default()
    }
}

impl JavaScriptEngine for MockJavaScriptEngine {
    fn initialize(&mut self, _mode: RuntimeMode) -> Result<(), RuntimeError> {
        if self.fail_init {
            return Err(RuntimeError::EngineInitialization {
                message: "mock engine initialization failure".to_string(),
            });
        }
        self.initialized = true;
        Ok(())
    }

    fn execute(&mut self, _source: &str, path: &Path) -> Result<ExecutionResult, RuntimeError> {
        if self.fail_exec {
            return Err(RuntimeError::JavaScriptExecution {
                path: path.to_path_buf(),
                error_type: "MockError".to_string(),
                message: "mock execution failure".to_string(),
                stack: None,
            });
        }
        Ok(ExecutionResult::success(Some("mocked".to_string())))
    }

    fn shutdown(&mut self) -> Result<(), RuntimeError> {
        if self.fail_shutdown {
            return Err(RuntimeError::Shutdown {
                message: "mock engine shutdown failure".to_string(),
            });
        }
        self.shut_down = true;
        Ok(())
    }
}
