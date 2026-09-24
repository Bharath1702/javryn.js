//! Script validation and metadata extraction.
//!
//! This module validates that a user-provided path refers to a readable
//! JavaScript file and extracts [`ScriptMetadata`] from the filesystem.

use std::path::Path;

use javryn_core::{RuntimeError, Script, ScriptMetadata};

/// Validates a script path and produces a [`Script`] with metadata.
///
/// # Validation Steps
///
/// 1. **Existence** — the path must point to an existing filesystem entry.
/// 2. **Type** — the entry must be a regular file (not a directory or symlink to a directory).
/// 3. **Extension** — the file must have a supported extension (`.js` or `.mjs`).
/// 4. **Readability** — the file must be readable (checked by attempting to open it).
///
/// # Errors
///
/// Returns a [`RuntimeError`] variant describing the specific validation failure.
pub fn validate_script(path: &Path) -> Result<Script, RuntimeError> {
    tracing::debug!(path = %path.display(), "validating script");

    // Step 1: Resolve the path to an absolute path for consistent handling.
    let absolute_path = resolve_path(path)?;
    tracing::debug!(absolute_path = %absolute_path.display(), "resolved absolute path");

    // Step 2: Check existence via metadata (also follows symlinks).
    let fs_metadata = match std::fs::metadata(&absolute_path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(RuntimeError::ScriptNotFound {
                path: path.to_path_buf(),
            });
        }
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            return Err(RuntimeError::ScriptUnreadable {
                path: path.to_path_buf(),
                reason: "permission denied".to_string(),
            });
        }
        Err(e) => {
            return Err(RuntimeError::Io {
                context: format!("accessing '{}'", path.display()),
                source: e,
            });
        }
    };

    // Step 3: Must be a regular file.
    if fs_metadata.is_dir() {
        return Err(RuntimeError::ScriptIsDirectory {
            path: path.to_path_buf(),
        });
    }

    // Step 4: Validate file extension.
    validate_extension(&absolute_path)?;

    // Step 5: Check readability by attempting to open the file.
    check_readable(&absolute_path, path)?;

    // Step 6: Extract metadata.
    let modified_time = fs_metadata.modified().ok();
    let script_metadata = ScriptMetadata::new(fs_metadata.len(), modified_time);

    tracing::debug!(
        file_size = fs_metadata.len(),
        "script validated successfully"
    );

    Ok(Script::new(absolute_path, script_metadata))
}

/// Resolves a potentially relative path to an absolute path.
fn resolve_path(path: &Path) -> Result<std::path::PathBuf, RuntimeError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|e| RuntimeError::Io {
                context: "determining current working directory".to_string(),
                source: e,
            })
    }
}

/// Validates that the file has a supported extension.
fn validate_extension(path: &Path) -> Result<(), RuntimeError> {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if javryn_core::script::SUPPORTED_EXTENSIONS.contains(&ext) => Ok(()),
        Some(ext) => Err(RuntimeError::InvalidExtension {
            path: path.to_path_buf(),
            extension: ext.to_string(),
        }),
        None => Err(RuntimeError::InvalidExtension {
            path: path.to_path_buf(),
            extension: "<none>".to_string(),
        }),
    }
}

/// Checks that a file can be opened for reading.
fn check_readable(absolute_path: &Path, original_path: &Path) -> Result<(), RuntimeError> {
    match std::fs::File::open(absolute_path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            Err(RuntimeError::ScriptUnreadable {
                path: original_path.to_path_buf(),
                reason: "permission denied".to_string(),
            })
        }
        Err(e) => Err(RuntimeError::ScriptUnreadable {
            path: original_path.to_path_buf(),
            reason: e.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper to create a temporary JS file for testing.
    fn create_temp_js(dir: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.path().join(name);
        let mut file = std::fs::File::create(&path).expect("create temp file");
        file.write_all(content.as_bytes()).expect("write temp file");
        path
    }

    #[test]
    fn valid_js_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "app.js", "console.log('hello');");
        let script = validate_script(&path).expect("should succeed");
        assert_eq!(script.extension(), Some("js"));
        assert!(script.metadata().file_size() > 0);
    }

    #[test]
    fn valid_mjs_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "module.mjs", "export default {};");
        let script = validate_script(&path).expect("should succeed");
        assert_eq!(script.extension(), Some("mjs"));
    }

    #[test]
    fn empty_js_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "empty.js", "");
        let script = validate_script(&path).expect("empty files are valid");
        assert_eq!(script.metadata().file_size(), 0);
    }

    #[test]
    fn missing_file() {
        let result = validate_script(Path::new("definitely_does_not_exist_12345.js"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, RuntimeError::ScriptNotFound { .. }),
            "expected ScriptNotFound, got: {err:?}"
        );
    }

    #[test]
    fn directory_instead_of_file() {
        let dir = tempfile::tempdir().unwrap();
        // Create a directory with a .js name to trigger the directory check
        let dir_path = dir.path().join("not_a_file.js");
        std::fs::create_dir(&dir_path).unwrap();

        let result = validate_script(&dir_path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::ScriptIsDirectory { .. }
        ));
    }

    #[test]
    fn invalid_extension_py() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "app.py", "print('hello')");
        let result = validate_script(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, RuntimeError::InvalidExtension { .. }),
            "expected InvalidExtension, got: {err:?}"
        );
    }

    #[test]
    fn invalid_extension_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "Makefile", "all:");
        let result = validate_script(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::InvalidExtension { .. }
        ));
    }

    #[test]
    fn spaces_in_filename() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "my app.js", "// spaced");
        let script = validate_script(&path).expect("spaces in filename should work");
        assert_eq!(script.file_name(), Some("my app.js"));
    }

    #[test]
    fn unicode_filename() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "приложение.js", "// unicode");
        let script = validate_script(&path).expect("unicode filename should work");
        assert!(script.path().exists());
    }

    #[test]
    fn relative_path_resolves() {
        // This test uses a file relative to the temp dir.
        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "relative.js", "// rel");

        // Validate using the absolute path (relative path resolution
        // depends on cwd which we don't want to change in tests).
        let script = validate_script(&path).expect("should resolve");
        assert!(script.path().is_absolute());
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_file() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = create_temp_js(&dir, "secret.js", "// secret");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        let result = validate_script(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RuntimeError::ScriptUnreadable { .. }
        ));

        // Restore permissions for cleanup.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
}
