//! Script and script metadata types for Javryn.
//!
//! A [`Script`] represents a validated, loadable JavaScript file.
//! [`ScriptMetadata`] contains filesystem information about the script.

use std::path::PathBuf;
use std::time::SystemTime;

/// The set of file extensions accepted by Javryn.
pub const SUPPORTED_EXTENSIONS: &[&str] = &["js", "mjs"];

/// A validated script ready to be processed by the runtime.
///
/// In V0.1, the runtime does not execute JavaScript, but this type
/// establishes the contract for future engine integration.
#[derive(Debug, Clone)]
pub struct Script {
    /// The canonical, absolute path to the script file.
    path: PathBuf,

    /// Filesystem metadata about the script.
    metadata: ScriptMetadata,
}

impl Script {
    /// Creates a new `Script` from validated components.
    ///
    /// This constructor is intentionally `pub(crate)` to enforce that
    /// scripts are always created through the validation pipeline.
    pub fn new(path: PathBuf, metadata: ScriptMetadata) -> Self {
        Self { path, metadata }
    }

    /// Returns the absolute path to the script file.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the script's filesystem metadata.
    pub fn metadata(&self) -> &ScriptMetadata {
        &self.metadata
    }

    /// Returns the file name of the script (e.g., `"app.js"`).
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    /// Returns the file extension (e.g., `"js"`).
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }
}

/// Filesystem metadata about a script file.
///
/// This struct captures information that is useful for diagnostics,
/// caching, and future optimizations.
#[derive(Debug, Clone)]
pub struct ScriptMetadata {
    /// The file size in bytes.
    file_size: u64,

    /// The last modification time, if available from the filesystem.
    modified_time: Option<SystemTime>,
}

impl ScriptMetadata {
    /// Creates new script metadata.
    pub fn new(file_size: u64, modified_time: Option<SystemTime>) -> Self {
        Self {
            file_size,
            modified_time,
        }
    }

    /// Returns the file size in bytes.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Returns the last modification time, if available.
    pub fn modified_time(&self) -> Option<SystemTime> {
        self.modified_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_include_js() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"js"));
    }

    #[test]
    fn supported_extensions_include_mjs() {
        assert!(SUPPORTED_EXTENSIONS.contains(&"mjs"));
    }

    #[test]
    fn script_accessors() {
        let meta = ScriptMetadata::new(42, None);
        let script = Script::new(PathBuf::from("/test/app.js"), meta);

        assert_eq!(script.path(), std::path::Path::new("/test/app.js"));
        assert_eq!(script.file_name(), Some("app.js"));
        assert_eq!(script.extension(), Some("js"));
        assert_eq!(script.metadata().file_size(), 42);
        assert!(script.metadata().modified_time().is_none());
    }

    #[test]
    fn script_metadata_with_modified_time() {
        let now = SystemTime::now();
        let meta = ScriptMetadata::new(1024, Some(now));

        assert_eq!(meta.file_size(), 1024);
        assert!(meta.modified_time().is_some());
    }

    #[test]
    fn script_is_clone() {
        let meta = ScriptMetadata::new(0, None);
        let script = Script::new(PathBuf::from("app.js"), meta);
        let _cloned = script.clone();
    }
}
