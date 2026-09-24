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
///
/// # Future Extensibility
///
/// In V0.2+, this struct will hold a reference to the JavaScript engine
/// and manage worker contexts. The public API is designed to accommodate
/// these additions without breaking changes.
pub struct Runtime {
    /// The runtime configuration for this session.
    config: RuntimeConfig,

    /// Whether the runtime has been initialized.
    initialized: bool,

    /// Whether the runtime has been shut down.
    shut_down: bool,
}

impl Runtime {
    /// Creates a new runtime with the given configuration.
    ///
    /// This does **not** perform initialization. Call [`run`](Self::run)
    /// to start the full lifecycle.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Configuration`] if the configuration is invalid.
    pub fn new(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        tracing::debug!(mode = %config.mode(), "creating runtime");

        Ok(Self {
            config,
            initialized: false,
            shut_down: false,
        })
    }

    /// Runs the full runtime lifecycle for the given script.
    ///
    /// # Lifecycle Steps
    ///
    /// 1. Initialize the runtime
    /// 2. Execute the script (placeholder in V0.1)
    /// 3. Produce a [`RuntimeResult`]
    ///
    /// The caller is responsible for calling [`shutdown`](Self::shutdown) afterward.
    ///
    /// # Errors
    ///
    /// Returns a [`RuntimeError`] if any lifecycle step fails.
    pub fn run(&mut self, script: &Script) -> Result<RuntimeResult, RuntimeError> {
        let start = Instant::now();

        self.initialize()?;
        self.execute(script)?;

        let elapsed = start.elapsed();
        tracing::debug!(
            elapsed_ms = elapsed.as_millis(),
            "runtime lifecycle complete"
        );

        Ok(RuntimeResult::new(elapsed))
    }

    /// Shuts down the runtime and releases all resources.
    ///
    /// This method is idempotent — calling it multiple times is safe.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::Shutdown`] if resource cleanup fails.
    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        if self.shut_down {
            tracing::debug!("runtime already shut down");
            return Ok(());
        }

        tracing::debug!("shutting down runtime");

        // In V0.1, there are no resources to release.
        // Future versions will shut down the JS engine, worker pool, etc.

        self.shut_down = true;
        tracing::debug!("runtime shut down complete");

        Ok(())
    }

    /// Returns a reference to the runtime configuration.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Initializes the runtime.
    ///
    /// In V0.1, this is a lightweight operation. Future versions will
    /// initialize the JavaScript engine, allocate worker contexts, etc.
    fn initialize(&mut self) -> Result<(), RuntimeError> {
        if self.initialized {
            return Err(RuntimeError::Initialization {
                message: "runtime already initialized".to_string(),
            });
        }

        tracing::debug!("initializing runtime");

        // Future: Initialize JS engine, allocate worker pool, etc.

        self.initialized = true;
        tracing::debug!("runtime initialized");

        Ok(())
    }

    /// Executes the V0.1 placeholder runtime logic.
    ///
    /// In V0.1, this validates that the script is ready and logs
    /// diagnostic information. In V0.2+, this will execute JavaScript.
    fn execute(&self, script: &Script) -> Result<(), RuntimeError> {
        tracing::debug!(
            script = ?script.file_name(),
            size = script.metadata().file_size(),
            "executing script (V0.1 placeholder)"
        );

        // V0.1: No JavaScript execution.
        // The script has already been validated by the time we get here.
        // Future versions will pass the script to the JS engine.

        tracing::info!(
            script = ?script.file_name(),
            "Javryn V0.1: script validated successfully (JavaScript execution not yet implemented)"
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
    use javryn_core::{RuntimeConfig, ScriptMetadata};
    use std::path::PathBuf;

    fn test_config() -> RuntimeConfig {
        RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .build()
            .unwrap()
    }

    fn test_script() -> Script {
        let meta = ScriptMetadata::new(42, None);
        Script::new(PathBuf::from("/test/app.js"), meta)
    }

    #[test]
    fn create_runtime() {
        let config = test_config();
        let runtime = Runtime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn run_lifecycle() {
        let config = test_config();
        let mut runtime = Runtime::new(config).unwrap();
        let script = test_script();

        let result = runtime.run(&script);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.elapsed().as_nanos() > 0);
    }

    #[test]
    fn shutdown_is_idempotent() {
        let config = test_config();
        let mut runtime = Runtime::new(config).unwrap();
        let script = test_script();

        runtime.run(&script).unwrap();

        assert!(runtime.shutdown().is_ok());
        assert!(runtime.shutdown().is_ok()); // Second call should also succeed.
    }

    #[test]
    fn double_run_fails() {
        let config = test_config();
        let mut runtime = Runtime::new(config).unwrap();
        let script = test_script();

        runtime.run(&script).unwrap();

        // Second run should fail because already initialized.
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
        let config = test_config();
        let mut runtime = Runtime::new(config).unwrap();
        let script = test_script();
        runtime.run(&script).unwrap();
        // Intentionally drop without calling shutdown.
        drop(runtime);
    }
}
