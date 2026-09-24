//! Shared domain types for Javryn.
//!
//! Contains enums and structs used across multiple crates.

use std::time::Duration;

/// The diagnostic/logging mode for a runtime session.
///
/// Only one mode can be active at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeMode {
    /// Normal operation — minimal output.
    #[default]
    Normal,

    /// Debug mode — emits detailed diagnostic logging.
    Debug,

    /// Verbose mode — emits additional informational output.
    Verbose,

    /// Quiet mode — suppresses all non-error output.
    Quiet,
}

impl std::fmt::Display for RuntimeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeMode::Normal => write!(f, "normal"),
            RuntimeMode::Debug => write!(f, "debug"),
            RuntimeMode::Verbose => write!(f, "verbose"),
            RuntimeMode::Quiet => write!(f, "quiet"),
        }
    }
}

/// The result of a successful runtime execution.
///
/// In V0.1, this carries only basic diagnostic information.
/// Future versions will include execution results, metrics, etc.
#[derive(Debug, Clone)]
pub struct RuntimeResult {
    /// Total time spent in the runtime lifecycle.
    elapsed: Duration,
}

impl RuntimeResult {
    /// Creates a new `RuntimeResult` with the given elapsed duration.
    pub fn new(elapsed: Duration) -> Self {
        Self { elapsed }
    }

    /// Returns the total elapsed time for the runtime execution.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_mode_display() {
        assert_eq!(RuntimeMode::Normal.to_string(), "normal");
        assert_eq!(RuntimeMode::Debug.to_string(), "debug");
        assert_eq!(RuntimeMode::Verbose.to_string(), "verbose");
        assert_eq!(RuntimeMode::Quiet.to_string(), "quiet");
    }

    #[test]
    fn runtime_mode_default_is_normal() {
        assert_eq!(RuntimeMode::default(), RuntimeMode::Normal);
    }

    #[test]
    fn runtime_result_elapsed() {
        let result = RuntimeResult::new(Duration::from_millis(42));
        assert_eq!(result.elapsed(), Duration::from_millis(42));
    }
}
