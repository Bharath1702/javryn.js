# Javryn Performance Baseline & Benchmarks (V1.0)

## Overview

Javryn performance is evaluated across startup latency, single-threaded execution, multi-worker parallel mapping, and automatic parallelization overhead.

---

## 1. Verified Performance Baseline (Windows / x86_64)

| Benchmark Metric | Measurement | Command / Target |
|---|---|---|
| **CLI Startup Latency** | **< 5 ms** | `javryn --version` |
| **Hello World Execution** | **~12 ms** | `javryn examples/hello.js` |
| **Parallel Map Workload** | **10 ms** | `javryn --diagnostics examples/parallel-map.js` |
| **Baseline Suite Execution** | **15.95 s** | `cargo test -p javryn-runtime --test benchmark` |

---

## 2. Diagnostics Metrics & Queue Overhead

When executed with `--diagnostics`, Javryn tracks queue wait latency and starvation boosts:

```text
=== Javryn Runtime Diagnostics ===
Execution Time     : 10 ms
Active Operations  : 0
Queued Tasks       : 0
Running Tasks      : 0
Worker Pool Size   : 16
Max Task Queue     : 10000
Dispatches         : 4
Completions        : 4
Avg Queue Wait     : 6.00 ms
Max Queue Wait     : 6 ms
Starvation Boosts  : 0
==================================
```

---

## 3. Reproducing Benchmarks

To run the baseline benchmark suite locally:

```bash
# Run benchmark target
cargo test -p javryn-runtime --test benchmark -- --nocapture

# Run release build diagnostics against parallel map
cargo build --release
./target/release/javryn --diagnostics examples/parallel-map.js
```
