//! Performance baseline benchmark for Javryn V0.3 Async Runtime & Event Loop.

use std::path::PathBuf;
use std::time::Instant;

use javryn_core::{RuntimeConfig, RuntimeMode};
use javryn_runtime::{Runtime, validate_script};

#[test]
fn benchmark_baseline() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let hello_path = workspace_root.join(PathBuf::from("examples/hello.js"));
    let compute_path = workspace_root.join(PathBuf::from("examples/compute.js"));
    let async_path = workspace_root.join(PathBuf::from("examples/async.js"));
    let async_await_path = workspace_root.join(PathBuf::from("examples/async-await.js"));
    let timers_path = workspace_root.join(PathBuf::from("examples/timers.js"));

    let iterations = 50;

    // Benchmark 1: Simple execution
    let start = Instant::now();
    for _ in 0..iterations {
        let config = RuntimeConfig::builder()
            .script_path(hello_path.clone())
            .mode(RuntimeMode::Quiet)
            .build()
            .unwrap();
        let script = validate_script(config.script_path()).unwrap();
        let mut runtime = Runtime::new(config).unwrap();
        let _ = runtime.run(&script).unwrap();
        runtime.shutdown().unwrap();
    }
    let hello_elapsed = start.elapsed();

    // Benchmark 2: Compute execution
    let start = Instant::now();
    for _ in 0..iterations {
        let config = RuntimeConfig::builder()
            .script_path(compute_path.clone())
            .mode(RuntimeMode::Quiet)
            .build()
            .unwrap();
        let script = validate_script(config.script_path()).unwrap();
        let mut runtime = Runtime::new(config).unwrap();
        let _ = runtime.run(&script).unwrap();
        runtime.shutdown().unwrap();
    }
    let compute_elapsed = start.elapsed();

    // Benchmark 3: Async & Promises
    let start = Instant::now();
    for _ in 0..iterations {
        let config = RuntimeConfig::builder()
            .script_path(async_path.clone())
            .mode(RuntimeMode::Quiet)
            .build()
            .unwrap();
        let script = validate_script(config.script_path()).unwrap();
        let mut runtime = Runtime::new(config).unwrap();
        let _ = runtime.run(&script).unwrap();
        runtime.shutdown().unwrap();
    }
    let async_elapsed = start.elapsed();

    // Benchmark 4: Async / Await
    let start = Instant::now();
    for _ in 0..iterations {
        let config = RuntimeConfig::builder()
            .script_path(async_await_path.clone())
            .mode(RuntimeMode::Quiet)
            .build()
            .unwrap();
        let script = validate_script(config.script_path()).unwrap();
        let mut runtime = Runtime::new(config).unwrap();
        let _ = runtime.run(&script).unwrap();
        runtime.shutdown().unwrap();
    }
    let async_await_elapsed = start.elapsed();

    // Benchmark 5: Timers & Cancellation
    let start = Instant::now();
    for _ in 0..iterations {
        let config = RuntimeConfig::builder()
            .script_path(timers_path.clone())
            .mode(RuntimeMode::Quiet)
            .build()
            .unwrap();
        let script = validate_script(config.script_path()).unwrap();
        let mut runtime = Runtime::new(config).unwrap();
        let _ = runtime.run(&script).unwrap();
        runtime.shutdown().unwrap();
    }
    let timers_elapsed = start.elapsed();

    println!("\n=== JAVRYN V0.3 PERFORMANCE BASELINE ===");
    println!("Iterations: {}", iterations);
    println!(
        "Simple Script (hello.js): {:?} per op",
        hello_elapsed / iterations
    );
    println!(
        "Compute Script (compute.js): {:?} per op",
        compute_elapsed / iterations
    );
    println!(
        "Async Promises (async.js): {:?} per op",
        async_elapsed / iterations
    );
    println!(
        "Async / Await (async-await.js): {:?} per op",
        async_await_elapsed / iterations
    );
    println!(
        "Timers & Cancellation (timers.js): {:?} per op",
        timers_elapsed / iterations
    );
    println!("=======================================\n");
}
