# Javryn v1.0.0 Release Notes

**A high-performance parallel JavaScript runtime built in Rust.**

Javryn v1.0.0 is the first public production release. It provides a complete JavaScript execution environment with an isolated worker architecture, explicit parallelism, and opt-in automatic parallelization.

---

## Quick Start

```bash
# Build from source (requires Rust 1.85+)
git clone https://github.com/Bharath1702/javryn.js.git
cd javryn.js
cargo build --release

# Run JavaScript
./target/release/javryn examples/hello.js
# Hello from Javryn!

# Run parallel workload
./target/release/javryn examples/parallel-map.js
# Parallel map result: [ 1, 4, 9, 16 ]

# Run with diagnostics
./target/release/javryn --diagnostics examples/parallel-map.js
```

---

## What's in V1.0

### JavaScript Execution
Run standard JavaScript (ES6+) powered by the Boa engine v0.20 — including arrow functions, classes, destructuring, Promises, `async`/`await`, and timers.

### Parallel JavaScript
`parallel.map(array, callback)` distributes independent work across isolated worker threads:

```javascript
async function main() {
    const results = await parallel.map([1, 2, 3, 4], "x => x * x");
    console.log(results); // [ 1, 4, 9, 16 ]
}
main();
```

### Automatic Parallelization
The `--auto-parallel` flag enables static analysis that identifies safe JavaScript for parallel execution. Unsafe code automatically falls back to sequential execution.

```bash
javryn --auto-parallel script.js
```

### Intelligent Scheduler
Load-aware worker placement, priority queuing, and starvation prevention distribute tasks optimally across the worker pool.

### Bounded Resources
Configure runtime limits to control resource usage:

```bash
javryn --max-workers 4 --max-queued-tasks 100 --shutdown-timeout 10000 script.js
```

### Operational Diagnostics
The `--diagnostics` flag provides execution metrics including queue latency, worker utilization, and scheduler statistics.

---

## Supported Platforms

| Platform | Status |
|---|---|
| Windows x86_64 | ✅ Verified |
| Linux x86_64 | ✅ Expected to work (Rust cross-platform) |
| macOS | ⚠️ Not yet verified |

---

## Known Limitations

- **No prebuilt binaries** — build from source is required
- **No Node.js APIs** — `fs`, `path`, `http`, `crypto`, and other Node.js built-in modules are not available
- **No browser APIs** — `DOM`, `fetch`, `WebSocket`, `localStorage` are not available
- **No multi-file module imports** — single-file `.mjs` evaluation is supported
- **Not a security sandbox** — scripts run with the full OS permissions of the host process
- **Parallel overhead** — small or trivial workloads may not benefit from parallelism

---

## Installation

See [docs/INSTALLATION.md](docs/INSTALLATION.md) for detailed setup instructions.

## Documentation

- [Getting Started](docs/GETTING_STARTED.md) — 5-minute tutorial
- [CLI Reference](docs/CLI.md) — all command-line options
- [Parallelism](docs/PARALLELISM.md) — parallel.map, workers, scheduler
- [Automatic Parallelization](docs/AUTOMATIC_PARALLELIZATION.md) — static analysis, safety rules
- [Architecture](docs/ARCHITECTURE.md) — crate structure and runtime design
- [JavaScript Compatibility](docs/JAVASCRIPT_COMPATIBILITY.md) — supported features and limitations
- [Security](docs/SECURITY.md) — security model and non-guarantees
- [Performance](docs/PERFORMANCE.md) — benchmarks and methodology

## Examples

See the [`examples/`](examples/) directory for runnable JavaScript programs demonstrating hello world, async, timers, workers, parallel map, CPU-intensive parallelism, error handling, and automatic parallelization.

## License

MIT — see [LICENSE](LICENSE).
