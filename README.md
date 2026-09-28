# Javryn

**A high-performance, parallel JavaScript runtime.**

## Current Status: V1.0 — Production Release

Javryn V1.0 is a production-grade, parallel JavaScript runtime built in Rust with explicit and automatic CPU multi-threading, intelligent load-aware task scheduling, bounded resource backpressure, worker thread isolation, and operational diagnostics.

```bash
javryn --diagnostics --max-workers 4 examples/parallel-map.js
```

## Features in V0.9

- **Bounded Resource Limits**: Configurable `--max-workers`, `--max-queued-tasks`, and `--shutdown-timeout` with strict boundary validation.
- **Worker Failure Isolation**: Worker panics/crashes are isolated from main thread and remaining worker pool; failed tasks trigger clean promise rejection and worker eviction.
- **Timed Graceful Shutdown**: `terminate_with_timeout` guarantees clean thread joins without deadlocking on hung workers.
- **Operational Diagnostics**: `--diagnostics` flag outputs real-time execution statistics, queue wait averages, worker pool utilization, and scheduler metrics.
- **Serialization Safety**: Rigid boundary enforcement rejecting unsupported types (functions, cyclic objects) cleanly without thread channel poisoning.

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
