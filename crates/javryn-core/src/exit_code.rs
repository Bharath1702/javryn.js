//! Process exit codes for Javryn.
//!
//! # Exit Code Policy
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success — the runtime completed without error |
//! | 1 | Generic runtime failure |
//! | 2 | Invalid CLI input, script path, or file |
//! | 3 | Configuration failure |
//! | 4 | Initialization failure |
//! | 5 | Runtime execution failure |
//! | 6 | Shutdown failure |

/// Documented process exit codes for the Javryn CLI.
///
/// These codes provide structured information about why the process exited.
/// See the module-level documentation for the full policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    /// The runtime completed successfully.
    Success = 0,

    /// A generic, uncategorized runtime failure.
    GenericFailure = 1,

    /// Invalid CLI arguments, missing script, invalid file, or bad path.
    InvalidInput = 2,

    /// The runtime configuration is invalid or conflicting.
    ConfigurationFailure = 3,

    /// The runtime failed during initialization.
    InitializationFailure = 4,

    /// An error occurred during runtime execution.
    RuntimeFailure = 5,

    /// The runtime failed to shut down cleanly.
    ShutdownFailure = 6,
}

impl ExitCode {
    /// Returns the numeric exit code as used by the operating system.
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(code: ExitCode) -> Self {
        std::process::ExitCode::from(code as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_values() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::GenericFailure.as_i32(), 1);
        assert_eq!(ExitCode::InvalidInput.as_i32(), 2);
        assert_eq!(ExitCode::ConfigurationFailure.as_i32(), 3);
        assert_eq!(ExitCode::InitializationFailure.as_i32(), 4);
        assert_eq!(ExitCode::RuntimeFailure.as_i32(), 5);
        assert_eq!(ExitCode::ShutdownFailure.as_i32(), 6);
    }

    #[test]
    fn exit_code_to_process_exit_code() {
        let _: std::process::ExitCode = ExitCode::Success.into();
        let _: std::process::ExitCode = ExitCode::GenericFailure.into();
    }
}
