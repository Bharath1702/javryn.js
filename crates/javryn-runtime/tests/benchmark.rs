//! Performance baseline benchmark for Javryn V0.1.

use std::path::PathBuf;
use std::time::Instant;

use javryn_core::{RuntimeConfig, RuntimeMode};
use javryn_runtime::{Runtime, validate_script};

#[test]
fn benchmark_baseline() {
    let script_path = PathBuf::from("examples/hello.js");
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let full_path = workspace_root.join(script_path);

    let iterations = 1_000;

    // Benchmark 1: Validation
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = validate_script(&full_path).unwrap();
    }
    let val_elapsed = start.elapsed();

    // Benchmark 2: Full Lifecycle
    let start = Instant::now();
    for _ in 0..iterations {
        let config = RuntimeConfig::builder()
            .script_path(full_path.clone())
            .mode(RuntimeMode::Quiet)
            .build()
            .unwrap();
        let script = validate_script(config.script_path()).unwrap();
        let mut runtime = Runtime::new(config).unwrap();
        let _ = runtime.run(&script).unwrap();
        runtime.shutdown().unwrap();
    }
    let lifecycle_elapsed = start.elapsed();

    println!("\n=== JAVRYN V0.1 PERFORMANCE BASELINE ===");
    println!("Iterations: {}", iterations);
    println!("Script Validation: {:?} per op", val_elapsed / iterations);
    println!(
        "Full Runtime Lifecycle: {:?} per op",
        lifecycle_elapsed / iterations
    );
    println!("=======================================\n");
}
