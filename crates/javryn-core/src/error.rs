//! Runtime error types for Javryn.
//!
//! All runtime failures are represented as [`RuntimeError`] variants.
//! These errors are designed to be converted into human-readable messages
//! at the CLI boundary and mapped to appropriate [`ExitCode`](crate::ExitCode) values.

use std::path::PathBuf;

/// The central error type for the Javryn runtime.
///
/// Each variant maps to a specific failure category with an associated
/// [`ExitCode`](crate::ExitCode) for process exit.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// The specified script file was not found on disk.
    #[error("script not found\n\n  File: {path}")]
    ScriptNotFound {
        /// The path that was requested but does not exist.
        path: PathBuf,
    },

    /// The specified path is a directory, not a file.
    #[error("path is a directory, not a script file\n\n  Path: {path}")]
    ScriptIsDirectory {
        /// The directory path that was provided.
        path: PathBuf,
    },

    /// The script file exists but cannot be read.
    #[error("cannot read script file\n\n  File: {path}\n  Reason: {reason}")]
    ScriptUnreadable {
        /// The path to the unreadable file.
        path: PathBuf,
        /// Human-readable reason (e.g., "permission denied").
        reason: String,
    },

    /// The script has an unsupported file extension.
    #[error(
        "unsupported file extension\n\n  File: {path}\n  Extension: {extension}\n  Supported: .js, .mjs"
    )]
    InvalidExtension {
        /// The path to the file with the wrong extension.
        path: PathBuf,
        /// The actual extension found (or "<none>" if missing).
        extension: String,
    },

    /// A generic I/O error occurred during script or filesystem operations.
    #[error("I/O error\n\n  {context}\n  Reason: {source}")]
    Io {
        /// Description of what operation was being performed.
        context: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// The runtime configuration is invalid.
    #[error("invalid configuration\n\n  {message}")]
    Configuration {
        /// Description of what is wrong with the configuration.
        message: String,
    },

    /// An error occurred during runtime initialization.
    #[error("initialization failed\n\n  {message}")]
    Initialization {
        /// Description of the initialization failure.
        message: String,
    },

    /// An error occurred during runtime execution.
    #[error("runtime error\n\n  {message}")]
    Runtime {
        /// Description of the runtime failure.
        message: String,
    },

    /// An error occurred during runtime shutdown.
    #[error("shutdown failed\n\n  {message}")]
    Shutdown {
        /// Description of the shutdown failure.
        message: String,
    },

    /// An unexpected internal error. This indicates a bug in Javryn.
    #[error("internal error\n\n  {message}\n\n  This is a bug in Javryn. Please report it.")]
    Internal {
        /// Description of the internal error.
        message: String,
    },
}

impl RuntimeError {
    /// Returns the [`ExitCode`](crate::ExitCode) associated with this error category.
    pub fn exit_code(&self) -> crate::ExitCode {
        match self {
            RuntimeError::ScriptNotFound { .. }
            | RuntimeError::ScriptIsDirectory { .. }
            | RuntimeError::ScriptUnreadable { .. }
            | RuntimeError::InvalidExtension { .. } => crate::ExitCode::InvalidInput,

            RuntimeError::Io { .. } => crate::ExitCode::InvalidInput,

            RuntimeError::Configuration { .. } => crate::ExitCode::ConfigurationFailure,

            RuntimeError::Initialization { .. } => crate::ExitCode::InitializationFailure,

            RuntimeError::Runtime { .. } => crate::ExitCode::RuntimeFailure,

            RuntimeError::Shutdown { .. } => crate::ExitCode::ShutdownFailure,

            RuntimeError::Internal { .. } => crate::ExitCode::GenericFailure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_not_found_displays_correctly() {
        let err = RuntimeError::ScriptNotFound {
            path: PathBuf::from("app.js"),
        };
        let msg = err.to_string();
        assert!(msg.contains("script not found"));
        assert!(msg.contains("app.js"));
    }

    #[test]
    fn script_is_directory_displays_correctly() {
        let err = RuntimeError::ScriptIsDirectory {
            path: PathBuf::from("src/"),
        };
        let msg = err.to_string();
        assert!(msg.contains("directory"));
        assert!(msg.contains("src/"));
    }

    #[test]
    fn script_unreadable_displays_correctly() {
        let err = RuntimeError::ScriptUnreadable {
            path: PathBuf::from("secret.js"),
            reason: "permission denied".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("cannot read"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn invalid_extension_displays_correctly() {
        let err = RuntimeError::InvalidExtension {
            path: PathBuf::from("app.py"),
            extension: "py".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unsupported file extension"));
        assert!(msg.contains(".py"));
        assert!(msg.contains(".js"));
    }

    #[test]
    fn io_error_displays_correctly() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = RuntimeError::Io {
            context: "reading script metadata".to_string(),
            source: io_err,
        };
        let msg = err.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("reading script metadata"));
    }

    #[test]
    fn exit_codes_are_correct() {
        assert_eq!(
            RuntimeError::ScriptNotFound {
                path: PathBuf::from("x")
            }
            .exit_code(),
            crate::ExitCode::InvalidInput
        );
        assert_eq!(
            RuntimeError::Configuration {
                message: String::new()
            }
            .exit_code(),
            crate::ExitCode::ConfigurationFailure
        );
        assert_eq!(
            RuntimeError::Initialization {
                message: String::new()
            }
            .exit_code(),
            crate::ExitCode::InitializationFailure
        );
        assert_eq!(
            RuntimeError::Runtime {
                message: String::new()
            }
            .exit_code(),
            crate::ExitCode::RuntimeFailure
        );
        assert_eq!(
            RuntimeError::Shutdown {
                message: String::new()
            }
            .exit_code(),
            crate::ExitCode::ShutdownFailure
        );
        assert_eq!(
            RuntimeError::Internal {
                message: String::new()
            }
            .exit_code(),
            crate::ExitCode::GenericFailure
        );
    }
}
