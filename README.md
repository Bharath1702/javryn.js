# Javryn

**A high-performance, parallel JavaScript runtime.**

## Current Status: V0.6 — Memory + Concurrency

Javryn V0.6 strengthens worker and parallel task architecture under high concurrency pressure. It establishes explicit memory ownership boundaries, task/operation state machines, bounded task queue backpressure (`ConcurrencyLimitExceeded`), worker state isolation, and runtime resource accounting.

```bash
javryn examples/parallel-map.js
# Output:
# Parallel map result: [ 1, 4, 9, 16 ]
```

## Features in V0.6

- **Explicit Parallel API**: `parallel.map(items, callback)` returning a native JavaScript `Promise`.
- **Task & Operation State Machines**: Strict state transitions (`TaskStatus` and `OperationStatus`) preventing double-resolution or illegal state changes.
- **Bounded Concurrency & Backpressure**: Configurable `max_workers` and `max_queued_tasks` queue limits preventing unbounded memory growth.
- **Worker Isolation & Recovery**: Strict cross-worker memory boundaries with zero shared JS VM state; full runtime recovery on worker panic.
- **Resource Accounting & Diagnostics**: Real-time tracking of active operations, queued tasks, running tasks, and thread pool size.
- **Non-Blocking Main Loop**: Main event loop processes timers and microtasks while parallel tasks run concurrently on worker threads.

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
