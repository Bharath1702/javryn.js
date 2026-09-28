//! Performance and benchmark integration tests for Javryn V0.5 Parallel Execution.

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

fn run_javryn_script(script_relative_path: &str) -> (bool, String, u128) {
    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    exe.push("target");
    exe.push(if cfg!(debug_assertions) { "debug" } else { "release" });
    exe.push("javryn.exe");

    if !exe.exists() {
        let mut alt_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        alt_exe.push("target");
        alt_exe.push("release");
        alt_exe.push("javryn.exe");
        if alt_exe.exists() {
            exe = alt_exe;
        }
    }

    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(script_relative_path);

    let start = Instant::now();
    let output = Command::new(exe)
        .arg(script_path)
        .output()
        .expect("failed to execute javryn process");
    let elapsed_ms = start.elapsed().as_millis();

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    (output.status.success(), stdout, elapsed_ms)
}

#[test]
fn test_benchmark_workload_a_light_arithmetic() {
    let (success, stdout, elapsed_ms) = run_javryn_script("examples/parallel-map.js");
    assert!(success);
    assert!(stdout.contains("Parallel map result: [ 1, 4, 9, 16 ]"));
    println!("\n--- BENCHMARK WORKLOAD A (LIGHT ARITHMETIC) ---");
    println!("Total Elapsed: {} ms", elapsed_ms);
}

#[test]
fn test_benchmark_workload_b_object_transformation() {
    let (success, stdout, elapsed_ms) = run_javryn_script("tests/javascript/parallel/basic_map.js");
    assert!(success);
    assert!(stdout.contains("RESULT:[10,20,30,40,50]"));
    println!("\n--- BENCHMARK WORKLOAD B (ARRAY TRANSFORMATION) ---");
    println!("Total Elapsed: {} ms", elapsed_ms);
}

#[test]
fn test_benchmark_workload_c_cpu_fibonacci() {
    let (success, stdout, elapsed_ms) = run_javryn_script("examples/parallel-cpu.js");
    assert!(success);
    assert!(stdout.contains("CPU parallel map result: [ 832040, 832040, 832040, 832040 ]"));
    println!("\n--- BENCHMARK WORKLOAD C (CPU FIBONACCI) ---");
    println!("Total Elapsed: {} ms", elapsed_ms);
}
