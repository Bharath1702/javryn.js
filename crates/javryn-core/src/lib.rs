//! # javryn-core
//!
//! Core domain types, error definitions, and shared abstractions for the Javryn runtime.
//!
//! This crate is intentionally lightweight and contains no business logic.
//! It defines the vocabulary types used across `javryn-cli` and `javryn-runtime`.

pub mod config;
pub mod error;
pub mod exit_code;
pub mod script;
pub mod types;

// Re-export primary types for convenience.
pub use config::RuntimeConfig;
pub use error::RuntimeError;
pub use exit_code::ExitCode;
pub use script::{Script, ScriptMetadata};
pub use types::{RuntimeMode, RuntimeResult};
