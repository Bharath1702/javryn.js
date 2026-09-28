//! The Javryn runtime lifecycle engine.
//!
//! This module implements the deterministic runtime lifecycle:
//!
//! ```text
//! Create → Initialize → Run → Shutdown
//! ```
//!
//! In V0.1, the "Run" phase is a placeholder that validates the script
//! and produces a [`RuntimeResult`]. Future versions will plug in a
//! JavaScript engine at this point.

use std::time::Instant;

use javryn_core::{RuntimeConfig, RuntimeError, RuntimeResult, Script};

use crate::engine::JavaScriptEngine;
/// The Javryn runtime.
///
/// Owns the runtime configuration and manages the execution lifecycle.
/// This struct is designed to be constructed once, used for a single
/// execution, and then shut down.
///
/// # Lifecycle
///
/// ```text
/// Runtime::new(config)   → creates runtime
/// runtime.run(script)    → executes the lifecycle
/// runtime.shutdown()     → releases resources
/// ```
use crate::engine::boa::BoaEngineAdapter;

/// The Javryn runtime.
///
/// Owns the runtime configuration and manages the execution lifecycle.
/// This struct is constructed once, initialized, used to execute JavaScript,
/// and then shut down.
pub struct Runtime {
    /// The runtime configuration for this session.
    config: RuntimeConfig,

    /// The JavaScript execution engine adapter.
    engine: Box<dyn JavaScriptEngine>,

    /// Whether the runtime has been initialized.
    initialized: bool,

    /// Whether the runtime has been shut down.
    shut_down: bool,
}

impl Runtime {
    /// Creates a new runtime with default [`BoaEngineAdapter`].
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        Self::with_engine(config, Box::new(BoaEngineAdapter::new()))
    }

    /// Creates a new runtime with a custom [`JavaScriptEngine`] adapter implementation.
    pub fn with_engine(
        config: RuntimeConfig,
        engine: Box<dyn JavaScriptEngine>,
    ) -> Result<Self, RuntimeError> {
        tracing::debug!(mode = %config.mode(), "creating runtime");

        Ok(Self {
            config,
            engine,
            initialized: false,
            shut_down: false,
        })
    }

    /// Runs the full runtime lifecycle for the given script.
    pub fn run(&mut self, script: &Script) -> Result<RuntimeResult, RuntimeError> {
        let start = Instant::now();

        self.initialize()?;
        self.execute(script)?;

        let elapsed = start.elapsed();
        tracing::debug!(
            elapsed_ms = elapsed.as_millis(),
            "runtime lifecycle complete"
        );

        if self.config.diagnostics() {
            crate::tasks::with_task_manager(|tm| {
                let diag = tm.diagnostics();
                eprintln!("\n=== Javryn Runtime Diagnostics ===");
                eprintln!("Execution Time     : {} ms", elapsed.as_millis());
                eprintln!("Active Operations  : {}", diag.active_operations);
                eprintln!("Queued Tasks       : {}", diag.queued_tasks);
                eprintln!("Running Tasks      : {}", diag.running_tasks);
                eprintln!("Worker Pool Size   : {}", diag.pool_size);
                eprintln!("Max Task Queue     : {}", diag.max_queued_tasks);
                eprintln!(
                    "Dispatches         : {}",
                    diag.scheduler_metrics.total_dispatches
                );
                eprintln!(
                    "Completions        : {}",
                    diag.scheduler_metrics.total_completed
                );
                eprintln!(
                    "Avg Queue Wait     : {:.2} ms",
                    diag.scheduler_metrics.average_queue_wait_ms()
                );
                eprintln!(
                    "Max Queue Wait     : {:.2} ms",
                    diag.scheduler_metrics.queue_wait_max_ms
                );
                eprintln!(
                    "Starvation Boosts  : {}",
                    diag.scheduler_metrics.starvation_boosts
                );
                eprintln!("==================================\n");
            });
        }

        Ok(RuntimeResult::new(elapsed))
    }

    /// Shuts down the runtime and releases engine resources.
    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        if self.shut_down {
            tracing::debug!("runtime already shut down");
            return Ok(());
        }

        tracing::debug!("shutting down runtime engine");
        self.engine.shutdown()?;

        self.shut_down = true;
        tracing::debug!("runtime shut down complete");

        Ok(())
    }

    /// Returns a reference to the runtime configuration.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Initializes the runtime and its underlying JavaScript engine.
    fn initialize(&mut self) -> Result<(), RuntimeError> {
        if self.initialized {
            return Err(RuntimeError::Initialization {
                message: "runtime already initialized".to_string(),
            });
        }

        tracing::debug!("initializing runtime and engine");
        crate::autopar::set_auto_parallel_enabled(self.config.auto_parallel());

        // Configure task manager bounds from runtime configuration
        crate::tasks::with_task_manager(|tm| {
            let max_workers = self.config.max_workers().unwrap_or(0);
            let max_queued = self.config.max_queued_tasks().unwrap_or(0);
            tm.configure(max_workers, max_queued);
        });

        self.engine.initialize(self.config.mode())?;

        self.initialized = true;
        tracing::debug!("runtime and engine initialized");

        Ok(())
    }

    /// Reads the script source file and executes it via the JavaScript engine adapter.
    fn execute(&mut self, script: &Script) -> Result<(), RuntimeError> {
        tracing::debug!(
            script = %script.path().display(),
            size = script.metadata().file_size(),
            "reading JavaScript file source"
        );

        let source = match std::fs::read_to_string(script.path()) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                return Err(RuntimeError::ScriptUnreadable {
                    path: script.path().to_path_buf(),
                    reason: "invalid UTF-8 encoding in JavaScript source file".to_string(),
                });
            }
            Err(e) => {
                return Err(RuntimeError::ScriptUnreadable {
                    path: script.path().to_path_buf(),
                    reason: e.to_string(),
                });
            }
        };

        tracing::debug!(
            script = %script.path().display(),
            "executing script in engine"
        );

        let result = self.engine.execute(&source, script.path())?;

        tracing::debug!(
            script = %script.path().display(),
            success = result.success,
            "script execution finished"
        );

        Ok(())
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.initialized && !self.shut_down {
            // Best-effort shutdown during drop.
            tracing::warn!("runtime dropped without explicit shutdown — performing cleanup");
            let _ = self.shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::mock::MockJavaScriptEngine;
    use javryn_core::{RuntimeConfig, ScriptMetadata};
    use std::io::Write;
    use std::path::PathBuf;

    fn test_config() -> RuntimeConfig {
        RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .build()
            .unwrap()
    }

    fn test_script(dir: &tempfile::TempDir) -> Script {
        let path = dir.path().join("app.js");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(b"console.log('test');").unwrap();

        let meta = ScriptMetadata::new(20, None);
        Script::new(path, meta)
    }

    #[test]
    fn create_runtime() {
        let config = test_config();
        let runtime = Runtime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn run_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let script = test_script(&dir);
        let config = RuntimeConfig::builder()
            .script_path(script.path().to_path_buf())
            .build()
            .unwrap();

        let mut runtime = Runtime::new(config).unwrap();
        let result = runtime.run(&script);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.elapsed().as_nanos() > 0);
    }

    #[test]
    fn run_lifecycle_with_mock_engine() {
        let dir = tempfile::tempdir().unwrap();
        let script = test_script(&dir);
        let config = RuntimeConfig::builder()
            .script_path(script.path().to_path_buf())
            .build()
            .unwrap();

        let mock_engine = Box::new(MockJavaScriptEngine::new());
        let mut runtime = Runtime::with_engine(config, mock_engine).unwrap();
        let result = runtime.run(&script);
        assert!(result.is_ok());
    }

    #[test]
    fn shutdown_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let script = test_script(&dir);
        let config = RuntimeConfig::builder()
            .script_path(script.path().to_path_buf())
            .build()
            .unwrap();

        let mut runtime = Runtime::new(config).unwrap();
        runtime.run(&script).unwrap();

        assert!(runtime.shutdown().is_ok());
        assert!(runtime.shutdown().is_ok());
    }

    #[test]
    fn double_run_fails() {
        let dir = tempfile::tempdir().unwrap();
        let script = test_script(&dir);
        let config = RuntimeConfig::builder()
            .script_path(script.path().to_path_buf())
            .build()
            .unwrap();

        let mut runtime = Runtime::new(config).unwrap();
        runtime.run(&script).unwrap();

        let result = runtime.run(&script);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::Initialization { .. }
        ));
    }

    #[test]
    fn config_accessible() {
        let config = test_config();
        let runtime = Runtime::new(config).unwrap();
        assert_eq!(
            runtime.config().script_path(),
            std::path::Path::new("test.js")
        );
    }

    #[test]
    fn drop_without_shutdown_is_safe() {
        let dir = tempfile::tempdir().unwrap();
        let script = test_script(&dir);
        let config = RuntimeConfig::builder()
            .script_path(script.path().to_path_buf())
            .build()
            .unwrap();

        let mut runtime = Runtime::new(config).unwrap();
        runtime.run(&script).unwrap();
        drop(runtime);
    }
}
