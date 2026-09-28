# Javryn

**A high-performance, parallel JavaScript runtime.**

## Current Status: V0.8 — Automatic Parallelization

Javryn V0.8 introduces **Automatic Parallelization**, capable of identifying a restricted, provably-safe subset of sequential JavaScript workloads and automatically executing them in parallel across independent worker threads using the underlying V0.7 Intelligent Scheduler. Any code containing side effects, mutations, or uncertain dependencies cleanly falls back to native sequential execution.

```bash
javryn --auto-parallel examples/auto-parallel-basic.js
```

## Features in V0.8

- **Static Safety & Dependency Analysis**: `StaticAnalyzer` inspects expressions and loops to guarantee iteration independence before enabling parallel execution.
- **Sequential Fallback Guarantee**: Code containing accumulator operations (`+=`), array mutations (`push`), `console.log`, or global state mutations falls back to sequential execution without throwing errors.
- **Dynamic Chunk Sizing**: `ChunkPlanner` automatically partitions large loops into balanced task chunks based on physical core counts.
- **Explicit Opt-in API & CLI Flag**: `parallel.auto(fn)` JavaScript API and `--auto-parallel` CLI flag.
- **Dedicated Intelligent Scheduler Reuse**: Integrates directly into `TaskManager` and `Scheduler` without duplicating worker infrastructure.

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
