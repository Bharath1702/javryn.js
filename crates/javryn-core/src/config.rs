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
#[derive(Debug, Default)]
pub struct RuntimeConfigBuilder {
    script_path: Option<PathBuf>,
    mode: Option<RuntimeMode>,
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

    /// Builds the [`RuntimeConfig`], returning an error if required fields are missing.
    pub fn build(self) -> Result<RuntimeConfig, crate::RuntimeError> {
        let script_path = self.script_path.ok_or(crate::RuntimeError::Configuration {
            message: "script path is required".to_string(),
        })?;

        Ok(RuntimeConfig {
            script_path,
            mode: self.mode.unwrap_or(RuntimeMode::Normal),
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
}
