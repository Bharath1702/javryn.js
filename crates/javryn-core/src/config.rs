//! Runtime configuration for Javryn.
//!
//! [`RuntimeConfig`] holds all settings needed by the runtime.
//! It is constructed at the CLI boundary and passed into the runtime as an owned value.

use std::path::PathBuf;

use crate::RuntimeMode;

/// Configuration for a Javryn runtime instance.
///
/// This struct is designed to be extended in future versions with fields such as
/// `worker_count`, `memory_limit`, `scheduler_config`, and `engine_config`
/// without breaking the existing API, thanks to the builder pattern.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Path to the JavaScript file to process.
    script_path: PathBuf,

    /// The logging/diagnostic mode for this runtime session.
    mode: RuntimeMode,

    /// Whether automatic parallelization mode is enabled.
    auto_parallel: bool,

    /// Maximum number of worker threads allowed in worker pool.
    max_workers: Option<usize>,

    /// Maximum queued tasks limit for backpressure.
    max_queued_tasks: Option<usize>,

    /// Timeout in milliseconds for graceful worker pool shutdown.
    shutdown_timeout_ms: u64,

    /// Whether operational diagnostics summary should be emitted on completion.
    diagnostics: bool,
}

impl RuntimeConfig {
    /// Creates a new [`RuntimeConfigBuilder`] for constructing configuration.
    pub fn builder() -> RuntimeConfigBuilder {
        RuntimeConfigBuilder::default()
    }

    /// Returns the path to the script file.
    pub fn script_path(&self) -> &std::path::Path {
        &self.script_path
    }

    /// Returns the runtime mode (normal, debug, verbose, quiet).
    pub fn mode(&self) -> RuntimeMode {
        self.mode
    }

    /// Returns `true` if automatic parallelization mode is active.
    pub fn auto_parallel(&self) -> bool {
        self.auto_parallel
    }

    /// Returns configured maximum workers override if specified.
    pub fn max_workers(&self) -> Option<usize> {
        self.max_workers
    }

    /// Returns configured maximum queued tasks limit if specified.
    pub fn max_queued_tasks(&self) -> Option<usize> {
        self.max_queued_tasks
    }

    /// Returns graceful shutdown timeout in milliseconds.
    pub fn shutdown_timeout_ms(&self) -> u64 {
        self.shutdown_timeout_ms
    }

    /// Returns `true` if operational diagnostics should be emitted.
    pub fn diagnostics(&self) -> bool {
        self.diagnostics
    }

    /// Returns `true` if debug mode is enabled.
    pub fn is_debug(&self) -> bool {
        self.mode == RuntimeMode::Debug
    }

    /// Returns `true` if verbose mode is enabled.
    pub fn is_verbose(&self) -> bool {
        self.mode == RuntimeMode::Verbose
    }

    /// Returns `true` if quiet mode is enabled.
    pub fn is_quiet(&self) -> bool {
        self.mode == RuntimeMode::Quiet
    }
}

/// Builder for [`RuntimeConfig`].
///
/// Ensures all required fields are set and validates the configuration
/// before producing a `RuntimeConfig`.
#[derive(Debug)]
pub struct RuntimeConfigBuilder {
    script_path: Option<PathBuf>,
    mode: Option<RuntimeMode>,
    auto_parallel: bool,
    max_workers: Option<usize>,
    max_queued_tasks: Option<usize>,
    shutdown_timeout_ms: u64,
    diagnostics: bool,
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self {
            script_path: None,
            mode: None,
            auto_parallel: false,
            max_workers: None,
            max_queued_tasks: None,
            shutdown_timeout_ms: 5000,
            diagnostics: false,
        }
    }
}

impl RuntimeConfigBuilder {
    /// Sets the script path.
    pub fn script_path(mut self, path: PathBuf) -> Self {
        self.script_path = Some(path);
        self
    }

    /// Sets the runtime mode.
    pub fn mode(mut self, mode: RuntimeMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets whether automatic parallelization mode is active.
    pub fn auto_parallel(mut self, active: bool) -> Self {
        self.auto_parallel = active;
        self
    }

    /// Sets configured maximum workers limit.
    pub fn max_workers(mut self, workers: Option<usize>) -> Self {
        self.max_workers = workers;
        self
    }

    /// Sets maximum queued tasks limit.
    pub fn max_queued_tasks(mut self, tasks: Option<usize>) -> Self {
        self.max_queued_tasks = tasks;
        self
    }

    /// Sets shutdown timeout in milliseconds.
    pub fn shutdown_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.shutdown_timeout_ms = timeout_ms;
        self
    }

    /// Sets whether operational diagnostics are enabled.
    pub fn diagnostics(mut self, enabled: bool) -> Self {
        self.diagnostics = enabled;
        self
    }

    /// Builds the [`RuntimeConfig`], returning an error if required fields are missing or invalid.
    pub fn build(self) -> Result<RuntimeConfig, crate::RuntimeError> {
        let script_path = self.script_path.ok_or(crate::RuntimeError::Configuration {
            message: "script path is required".to_string(),
        })?;

        if let Some(w) = self.max_workers {
            if w == 0 {
                return Err(crate::RuntimeError::Configuration {
                    message: "max_workers must be greater than 0".to_string(),
                });
            }
            if w > 128 {
                return Err(crate::RuntimeError::Configuration {
                    message: "max_workers exceeds maximum threshold of 128".to_string(),
                });
            }
        }

        if let Some(0) = self.max_queued_tasks {
            return Err(crate::RuntimeError::Configuration {
                message: "max_queued_tasks must be greater than 0".to_string(),
            });
        }

        if self.shutdown_timeout_ms < 100 {
            return Err(crate::RuntimeError::Configuration {
                message: "shutdown_timeout_ms must be at least 100ms".to_string(),
            });
        }

        Ok(RuntimeConfig {
            script_path,
            mode: self.mode.unwrap_or(RuntimeMode::Normal),
            auto_parallel: self.auto_parallel,
            max_workers: self.max_workers,
            max_queued_tasks: self.max_queued_tasks,
            shutdown_timeout_ms: self.shutdown_timeout_ms,
            diagnostics: self.diagnostics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_with_defaults() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("app.js"))
            .build()
            .expect("should build with defaults");

        assert_eq!(config.script_path(), std::path::Path::new("app.js"));
        assert_eq!(config.mode(), RuntimeMode::Normal);
        assert!(!config.is_debug());
        assert!(!config.is_verbose());
        assert!(!config.is_quiet());
    }

    #[test]
    fn build_with_debug_mode() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("app.js"))
            .mode(RuntimeMode::Debug)
            .build()
            .expect("should build");

        assert!(config.is_debug());
        assert!(!config.is_verbose());
    }

    #[test]
    fn build_with_verbose_mode() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("app.js"))
            .mode(RuntimeMode::Verbose)
            .build()
            .expect("should build");

        assert!(config.is_verbose());
        assert!(!config.is_debug());
    }

    #[test]
    fn build_with_quiet_mode() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("app.js"))
            .mode(RuntimeMode::Quiet)
            .build()
            .expect("should build");

        assert!(config.is_quiet());
    }

    #[test]
    fn build_missing_script_path_fails() {
        let result = RuntimeConfig::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn config_is_clone() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("app.js"))
            .build()
            .expect("should build");
        let _cloned = config.clone();
    }

    // --- V0.9 Production Hardening: Config Validation ---

    #[test]
    fn max_workers_zero_rejected() {
        let res = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .max_workers(Some(0))
            .build();
        assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        assert!(msg.contains("max_workers must be greater than 0"));
    }

    #[test]
    fn max_workers_exceeds_128_rejected() {
        let res = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .max_workers(Some(256))
            .build();
        assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        assert!(msg.contains("exceeds maximum threshold of 128"));
    }

    #[test]
    fn max_workers_128_accepted() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .max_workers(Some(128))
            .build()
            .expect("128 workers should be accepted");
        assert_eq!(config.max_workers(), Some(128));
    }

    #[test]
    fn max_queued_tasks_zero_rejected() {
        let res = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .max_queued_tasks(Some(0))
            .build();
        assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        assert!(msg.contains("max_queued_tasks must be greater than 0"));
    }

    #[test]
    fn shutdown_timeout_below_100ms_rejected() {
        let res = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .shutdown_timeout_ms(50)
            .build();
        assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        assert!(msg.contains("shutdown_timeout_ms must be at least 100ms"));
    }

    #[test]
    fn shutdown_timeout_100ms_accepted() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .shutdown_timeout_ms(100)
            .build()
            .expect("100ms should be accepted");
        assert_eq!(config.shutdown_timeout_ms(), 100);
    }

    #[test]
    fn diagnostics_flag_propagated() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .diagnostics(true)
            .build()
            .expect("should build");
        assert!(config.diagnostics());
    }

    #[test]
    fn auto_parallel_flag_propagated() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .auto_parallel(true)
            .build()
            .expect("should build");
        assert!(config.auto_parallel());
    }

    #[test]
    fn default_shutdown_timeout_is_5000ms() {
        let config = RuntimeConfig::builder()
            .script_path(PathBuf::from("test.js"))
            .build()
            .expect("should build");
        assert_eq!(config.shutdown_timeout_ms(), 5000);
    }
}
