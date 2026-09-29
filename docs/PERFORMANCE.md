# Javryn Performance

## Overview

This document presents actual performance measurements from Javryn V1.0. All benchmarks were run on a release build (`cargo build --release`).

> **Important:** These measurements are from a specific environment. Your results will vary based on CPU, OS, and workload characteristics.

---

## 1. Test Environment

| Property | Value |
|---|---|
| **Javryn Version** | 1.0.0 |
| **Build Mode** | Release (`--release`) |
| **OS** | Windows (x86_64) |
| **Boa Engine** | v0.20 |

---

## 2. Measured Performance

| Benchmark | Measurement | Command |
|---|---|---|
| **CLI Startup + Version** | < 5 ms | `javryn --version` |
| **Hello World** | ~12 ms | `javryn examples/hello.js` |
| **Parallel Map (4 items)** | ~10 ms | `javryn examples/parallel-map.js` |
| **CPU Parallel (fib(30) × 4)** | ~3800 ms | `javryn examples/parallel-cpu.js` |

---

## 3. Diagnostics Output

When executed with `--diagnostics`, Javryn reports execution metrics:

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
Avg Queue Wait     : 7.00 ms
Max Queue Wait     : 7 ms
Starvation Boosts  : 0
==================================
```

---

## 4. Understanding Parallel Overhead

Parallel execution introduces overhead:

- **Worker thread creation**: each worker spawns an OS thread with its own Boa VM context
- **Serialization**: data is value-copied across thread boundaries via `JsMessage`
- **Scheduler dispatch**: task queuing, priority evaluation, and worker selection
- **Result assembly**: results are collected and reordered by input index

For **trivial workloads** (e.g., `x => x * x` on 4 elements), this overhead may equal or exceed the computation time. The diagnostics above show ~7ms average queue wait for a ~10ms total execution — the overhead is significant relative to the work.

For **CPU-heavy workloads** (e.g., `fib(30)` taking ~1 second per element), the overhead becomes negligible compared to the computation, and parallelism provides real speedup.

### When to Expect Speedup

- Per-element work is **computationally significant** (tens of milliseconds or more)
- Elements are **independent** (no shared state)
- Available CPU cores > 1

### When Overhead Dominates

- Per-element work is **trivial** (microseconds)
- Array size is **very small** (1–4 elements with simple operations)
- System has **limited cores**

---

## 5. Worker Reuse

Workers are pooled and reused across `parallel.map` operations. The first parallel operation incurs worker startup cost; subsequent operations reuse existing workers.

---

## 6. Reproducing Benchmarks

```bash
# Build release binary
cargo build --workspace --release

# Run hello world
./target/release/javryn examples/hello.js

# Run parallel map with diagnostics
./target/release/javryn --diagnostics examples/parallel-map.js

# Run CPU-intensive parallel workload
./target/release/javryn examples/parallel-cpu.js

# Run benchmark test suite
cargo test -p javryn-runtime --test benchmark -- --nocapture
```
