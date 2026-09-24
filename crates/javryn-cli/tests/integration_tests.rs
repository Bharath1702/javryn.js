//! Integration tests for the Javryn CLI.
//!
//! These tests invoke the `javryn` binary as an external process and verify
//! stdout, stderr, and exit codes.

use assert_cmd::Command;
use predicates::prelude::*;

/// Returns a `Command` for the `javryn` binary.
fn javryn() -> Command {
    Command::cargo_bin("javryn").expect("binary 'javryn' should exist")
}

// ─── Help & Version ─────────────────────────────────────────────────────────

#[test]
fn help_flag_succeeds() {
    javryn()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Javryn"))
        .stdout(predicate::str::contains("SCRIPT"));
}

#[test]
fn version_flag_succeeds() {
    javryn()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

// ─── Valid Script Execution ─────────────────────────────────────────────────

#[test]
fn valid_js_file_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("app.js");
    std::fs::write(&script, "console.log('hello');").unwrap();

    javryn().arg(&script).assert().success();
}

#[test]
fn valid_mjs_file_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("module.mjs");
    std::fs::write(&script, "export default {};").unwrap();

    javryn().arg(&script).assert().success();
}

#[test]
fn empty_js_file_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("empty.js");
    std::fs::write(&script, "").unwrap();

    javryn().arg(&script).assert().success();
}

#[test]
fn example_hello_js_succeeds() {
    // CARGO_MANIFEST_DIR points to crates/javryn-cli, go up 2 levels for workspace root.
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    javryn()
        .arg("examples/hello.js")
        .current_dir(workspace_root)
        .assert()
        .success();
}

// ─── Debug and Verbose Modes ────────────────────────────────────────────────

#[test]
fn debug_flag_produces_debug_output() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("app.js");
    std::fs::write(&script, "// debug test").unwrap();

    javryn()
        .args(["--debug"])
        .arg(&script)
        .assert()
        .success()
        .stderr(predicate::str::contains("DEBUG"));
}

#[test]
fn verbose_flag_produces_info_output() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("app.js");
    std::fs::write(&script, "// verbose test").unwrap();

    javryn()
        .args(["--verbose"])
        .arg(&script)
        .assert()
        .success()
        .stderr(predicate::str::contains("INFO"));
}

#[test]
fn quiet_flag_suppresses_output() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("app.js");
    std::fs::write(&script, "// quiet test").unwrap();

    javryn()
        .args(["--quiet"])
        .arg(&script)
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// ─── Missing Script ─────────────────────────────────────────────────────────

#[test]
fn missing_script_fails_with_exit_code_2() {
    javryn()
        .arg("nonexistent_file_12345.js")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("script not found"));
}

// ─── Directory Instead of File ──────────────────────────────────────────────

#[test]
fn directory_as_script_fails() {
    let dir = tempfile::tempdir().unwrap();
    let dir_path = dir.path().join("fakedir.js");
    std::fs::create_dir(&dir_path).unwrap();

    javryn()
        .arg(&dir_path)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("directory"));
}

// ─── Invalid Extension ──────────────────────────────────────────────────────

#[test]
fn invalid_extension_fails() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("app.py");
    std::fs::write(&script, "print('hello')").unwrap();

    javryn()
        .arg(&script)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unsupported file extension"));
}

#[test]
fn no_extension_fails() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("Makefile");
    std::fs::write(&script, "all:").unwrap();

    javryn()
        .arg(&script)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unsupported file extension"));
}

// ─── Invalid Arguments ──────────────────────────────────────────────────────

#[test]
fn no_arguments_fails() {
    javryn()
        .assert()
        .failure()
        .stderr(predicate::str::contains("SCRIPT"));
}

#[test]
fn conflicting_flags_fail() {
    javryn()
        .args(["--debug", "--verbose", "app.js"])
        .assert()
        .failure();
}

// ─── Spaces and Unicode in Filenames ────────────────────────────────────────

#[test]
fn spaces_in_filename_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("my app.js");
    std::fs::write(&script, "// spaces").unwrap();

    javryn().arg(&script).assert().success();
}

#[test]
fn unicode_filename_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("приложение.js");
    std::fs::write(&script, "// unicode").unwrap();

    javryn().arg(&script).assert().success();
}

// ─── Permission Denied (Unix only) ─────────────────────────────────────────

#[cfg(unix)]
#[test]
fn unreadable_file_fails() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("secret.js");
    {
        let mut f = std::fs::File::create(&script).unwrap();
        f.write_all(b"// secret").unwrap();
    }
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o000)).unwrap();

    javryn().arg(&script).assert().code(2).stderr(
        predicate::str::contains("permission denied").or(predicate::str::contains("cannot read")),
    );

    // Restore for cleanup.
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o644)).unwrap();
}

// ─── Exit Code Verification ─────────────────────────────────────────────────

#[test]
fn success_returns_exit_code_0() {
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("ok.js");
    std::fs::write(&script, "// ok").unwrap();

    javryn().arg(&script).assert().code(0);
}

#[test]
fn missing_file_returns_exit_code_2() {
    javryn().arg("ghost_file_99999.js").assert().code(2);
}
