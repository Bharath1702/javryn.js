//! # javryn-runtime
//!
//! The Javryn runtime lifecycle engine.
//!
//! This crate owns the full runtime lifecycle:
//! 1. Script validation and metadata extraction
//! 2. Runtime initialization
//! 3. Execution (placeholder in V0.1)
//! 4. Clean shutdown
//!
//! It does **not** contain CLI logic or user-facing output formatting.

pub mod runtime;
pub mod script;

pub use runtime::Runtime;
pub use script::validate_script;
