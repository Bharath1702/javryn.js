# Javryn

**A high-performance, parallel JavaScript runtime.**

## Current Status: V0.7 — Intelligent Scheduler

Javryn V0.7 upgrades the task dispatch architecture with a dedicated **Intelligent Scheduler**. The runtime features load-aware dynamic worker selection, task priority scheduling, aging starvation prevention, and queue latency diagnostics while preserving deterministic result ordering, cancellation, backpressure, and worker failure recovery.

```bash
javryn examples/parallel-map.js
# Output:
# Parallel map result: [ 1, 4, 9, 16 ]
```

## Features in V0.7

- **Dedicated Intelligent Scheduler**: Decoupled `Scheduler` module handling task selection (`select_task`) and worker placement (`select_worker`).
- **Dynamic Load-Aware Dispatching**: `LoadAwarePolicy` tracks worker thread active task load and busy duration to prevent idle workers during uneven workloads.
- **Priority & Aging Starvation Prevention**: `TaskPriority` support with automated aging boosts for tasks queued over 500ms.
- **Scheduler Metrics & Diagnostics**: Latency tracking (`queue_wait_avg_ms`, `queue_wait_max_ms`), dispatch counts, starvation boosts, and worker utilization metrics in `TaskManagerDiagnostics`.
- **Explicit Parallel API**: `parallel.map(items, callback)` returning a native JavaScript `Promise`.
- **Concurrency & Backpressure Control**: Configurable worker pool and task queue limits (`ConcurrencyLimitExceeded`).
- **Deterministic Result Aggregation**: Array indexing preserves result order regardless of non-deterministic thread completion order.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/javryn/javryn.git
cd javryn

# Build release binary
cargo build --release

# Run a parallel script
./target/release/javryn examples/parallel-map.js
```

## Usage Examples

```bash
# Basic Parallel Map
javryn examples/parallel-map.js

# CPU Parallel Workload
javryn examples/parallel-cpu.js

# Error Propagation
javryn examples/parallel-error.js
```

### `examples/parallel-map.js`
```javascript
const values = [1, 2, 3, 4];

async function main() {
    const results = await parallel.map(values, "x => x * x");
    console.log("Parallel map result:", results);
}

main();
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic runtime failure |
| 2 | Invalid input / JavaScript syntax error |
| 3 | Configuration failure |
| 4 | Engine initialization failure |
| 5 | JavaScript execution error / unhandled exception |
| 6 | Shutdown failure |

## Development

```bash
# Check compilation
cargo check --workspace

# Run all unit, integration, & compatibility tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run Clippy lints
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Roadmap

| Version | Focus |
|---------|-------|
| V0.1 | Runtime Foundation ✅ |
| V0.2 | JavaScript Engine Integration ✅ |
| V0.3 | Async Runtime & Event Loop ✅ |
| V0.4 | Independent Workers ✅ |
| V0.5 | Explicit Parallelism ✅ |
| V0.6 | Memory + Concurrency ✅ ← **current** |
| V0.7 | Intelligent Scheduler |
| V0.8 | Automatic Parallelization |
| V0.9 | Production Hardening |
| V1.0 | Production Release |

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## License

MIT — see [LICENSE](LICENSE).
