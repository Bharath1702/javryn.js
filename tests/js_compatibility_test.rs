//! Integration test runner for the JS compatibility test suite (`tests/javascript/*.js`).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn javryn() -> Command {
    Command::cargo_bin("javryn").expect("binary 'javryn' should exist")
}

#[test]
fn run_javascript_compatibility_suite() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let js_test_dir = workspace_root.join("tests").join("javascript");

    let entries = fs::read_dir(&js_test_dir).expect("should read tests/javascript directory");

    let mut test_files = Vec::new();
    for entry in entries {
        let entry = entry.expect("valid dir entry");
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "js" || ext == "mjs" {
                    test_files.push(path);
                }
            }
        }
    }

    test_files.sort();
    assert!(!test_files.is_empty(), "test suite should not be empty");

    for script in test_files {
        let file_name = script.file_name().unwrap().to_string_lossy();
        println!("Running JS compatibility test: {file_name}");

        let assert = javryn()
            .arg(&script)
            .current_dir(workspace_root)
            .assert();

        assert.success();
    }
}
