//! Rust integration tests for Javryn V0.5 Explicit Parallel JavaScript (`parallel.map`).

use std::path::PathBuf;
use std::process::Command;

fn run_javryn_script(script_relative_path: &str) -> (bool, String, String) {
    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    exe.push("target");
    exe.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    exe.push("javryn.exe");

    if !exe.exists() {
        let mut alt_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        alt_exe.push("target");
        alt_exe.push("debug");
        alt_exe.push("javryn.exe");
        if alt_exe.exists() {
            exe = alt_exe;
        }
    }

    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(script_relative_path);

    let output = Command::new(exe)
        .arg(script_path)
        .output()
        .expect("failed to execute javryn process");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (output.status.success(), stdout, stderr)
}

#[test]
fn test_parallel_map_basic() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/basic_map.js");
    assert!(success);
    assert!(stdout.contains("RESULT:[10,20,30,40,50]"));
}

#[test]
fn test_parallel_map_empty() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/empty_map.js");
    assert!(success);
    assert!(stdout.contains("RESULT:[]"));
}

#[test]
fn test_parallel_map_ordering() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/ordering.js");
    assert!(success);
    assert!(stdout.contains("RESULT:[100,10,50,1]"));
}

#[test]
fn test_parallel_map_error_propagation() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/error_map.js");
    assert!(success);
    assert!(stdout.contains("RESULT:ERROR:Error: Task failed at element 3"));
}

#[test]
fn test_parallel_map_worker_pool_reuse() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/reuse.js");
    assert!(success);
    assert!(stdout.contains("RESULT:[2,4,9,12]"));
}

#[test]
fn test_parallel_map_concurrent_operations() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/concurrent_ops.js");
    assert!(success);
    assert!(stdout.contains("CONCURRENT_1:[2,4,6,8]"));
    assert!(stdout.contains("CONCURRENT_2:[30,60,90,120]"));
}

#[test]
fn test_parallel_map_event_loop_responsiveness() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/event_loop_responsiveness.js");
    assert!(success);
    assert!(stdout.contains("TIMER_FIRED:TRUE"));
    assert!(stdout.contains("PARALLEL_DONE:[55,55,55,55]"));
    assert!(stdout.contains("TIMER_STATUS:true"));
}

#[test]
fn test_parallel_map_serialization_matrix() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/serialization_matrix.js");
    assert!(success);
    assert!(stdout.contains("SERIALIZED_RESULT:[null,null,true,false,42,\"hello unicode 🚀\",[1,\"nested\",true],{\"key\":\"value\",\"num\":100}]"));
    assert!(stdout.contains("UNSUPPORTED_RESULT:CAUGHT_ERROR:cannot serialize Function across worker boundary"));
}

#[test]
fn test_parallel_map_large_input() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/large_input.js");
    assert!(success);
    assert!(stdout.contains("LARGE_INPUT_LEN:1000"));
    assert!(stdout.contains("LARGE_INPUT_FIRST:1"));
    assert!(stdout.contains("LARGE_INPUT_LAST:1000"));
}

#[test]
fn test_parallel_map_closure_boundary() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/closure_boundary.js");
    assert!(success);
    assert!(stdout.contains("CLOSURE_RESULT:CAUGHT_ERROR"));
}

#[test]
fn test_parallel_map_promise_methods() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/promise_methods.js");
    assert!(success);
    assert!(stdout.contains("THEN_RESULT:[10,20,30]"));
    assert!(stdout.contains("CATCH_RESULT:Error: Task failed at element 2"));
    assert!(stdout.contains("THEN_STATUS:true,CATCH_STATUS:true"));
}

#[test]
fn test_parallel_map_concurrency_isolation() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/concurrency_isolation.js");
    assert!(success);
    assert!(stdout.contains("ISOLATION_CHECK:true"));
    assert!(stdout.contains("STRESS_LEN:500,STRESS_SUM:250500"));
}

#[test]
fn test_parallel_map_backpressure_rejection() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/backpressure_test.js");
    assert!(success);
    assert!(stdout.contains("BACKPRESSURE_RESULT:CAUGHT_ERROR"));
}

#[test]
fn test_parallel_map_explicit_cancellation() {
    let (success, stdout, _stderr) = run_javryn_script("tests/javascript/parallel/explicit_cancellation.js");
    assert!(success);
    assert!(stdout.contains("EXPLICIT_CANCEL_TRIGGERED:true"));
    assert!(stdout.contains("EXPLICIT_CANCEL_RESULT:CAUGHT_ERROR"));
    assert!(stdout.contains("POST_CANCEL_RECOVERY:[20,40]"));
}




