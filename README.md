# Javryn

**A high-performance parallel JavaScript runtime built in Rust.**

Run JavaScript with an isolated worker architecture, explicit parallel execution, and opt-in automatic parallelization.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Built_with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/Version-1.0.0-green.svg)](CHANGELOG.md)

---

## Why Javryn?

Traditional JavaScript runtimes provide a single primary execution context. CPU-bound workloads run sequentially, even when independent tasks could execute concurrently.

Javryn takes a different approach:

```text
JavaScript → Runtime → Scheduler → Independent JavaScript Workers → Multiple Boa Contexts
```

Each worker runs in its own OS thread with a fully isolated JavaScript VM (powered by [Boa](https://github.com/boa-dev/boa)). There is **zero shared JavaScript memory** between workers — data is exchanged via structured value-copy serialization.

**When parallelism helps:**
- CPU-heavy independent workloads (number crunching, data transformation, batch processing)
- Workloads that can be decomposed into independent tasks

**When parallelism does not help:**
- Small or trivial computations (parallel overhead dominates)
- I/O-heavy workloads (network, disk)
- Highly interdependent tasks with shared mutable state

Javryn is a tool for suitable workloads, not a guarantee that every program becomes faster.

---

## Quick Start

```bash
javryn --version
# javryn 1.0.0
```

Create `hello.js`:

```javascript
console.log("Hello from Javryn!");
```

Run it:

```bash
javryn hello.js
# Hello from Javryn!
```

---

## Installation

### Build from Source

**Prerequisites:** [Rust toolchain](https://rustup.rs/) (Rust 1.85+)

```bash
# Clone the repository
git clone https://github.com/Bharath1702/javryn.js.git
cd javryn.js

# Build optimized release binary
cargo build --release

# Verify
./target/release/javryn --version
# javryn 1.0.0
```

The `javryn` binary is at `target/release/javryn` (or `target\release\javryn.exe` on Windows). Add it to your `PATH` for convenience.

> **Note:** Prebuilt binaries are not yet available. See [docs/INSTALLATION.md](docs/INSTALLATION.md) for detailed setup instructions.

---

## Parallel JavaScript

### Explicit Parallelism with `parallel.map`

Distribute array processing across isolated worker threads:

```javascript
// parallel-map.js
const values = [1, 2, 3, 4];

async function main() {
    const results = await parallel.map(values, "x => x * x");
    console.log("Parallel map result:", results);
}

main();
```

```bash
javryn parallel-map.js
# Parallel map result: [ 1, 4, 9, 16 ]
```

### CPU-Intensive Parallel Workload

For CPU-heavy tasks, provide the function body as a string so it can be serialized to worker threads:

```javascript
// parallel-cpu.js
const inputs = [30, 30, 30, 30];

const fibSource = `n => {
    function fib(x) {
        if (x <= 1) return x;
        return fib(x - 1) + fib(x - 2);
    }
    return fib(n);
}`;

async function main() {
    const start = Date.now();
    const results = await parallel.map(inputs, fibSource);
    const elapsed = Date.now() - start;
    console.log("Results:", results);
    console.log("Elapsed (ms):", elapsed);
}

main();
```

```bash
javryn parallel-cpu.js
# Results: [ 832040, 832040, 832040, 832040 ]
```

Each element executes in its own worker thread with a dedicated Boa JavaScript engine instance.

### Error Handling

Errors in parallel tasks propagate as rejected promises:

```javascript
async function main() {
    try {
        await parallel.map([1, 2, 3], `x => {
            if (x === 2) throw new Error("failed");
            return x * 2;
        }`);
    } catch (err) {
        console.log("Caught:", err.message);
    }
}

main();
```

---

## Automatic Parallelization

Javryn can analyze JavaScript functions and automatically decide whether they are safe to parallelize:

```javascript
// auto-parallel-basic.js
parallel.auto(() => {
    const numbers = [1, 2, 3, 4, 5];
    // Static analyzer verifies this is a pure, side-effect-free block
});
```

```bash
javryn --auto-parallel auto-parallel-basic.js
```

The static analyzer inspects function bodies for:

| Analyzed For | Safe | Unsafe (Sequential Fallback) |
|---|---|---|
| Pure expressions | `x => x * x` | — |
| Mutations | — | `total += x`, `arr.push()` |
| Side effects | — | `console.log()`, `fetch()` |
| Global state | — | `globalThis.x++`, `window.x` |
| Control flow | — | `break`, `continue`, `throw` |
| Async operations | — | `async`, `await`, timers |

**Safe** → parallel execution via worker pool.
**Unsafe or uncertain** → guaranteed sequential fallback. No correctness risk.

See [docs/AUTOMATIC_PARALLELIZATION.md](docs/AUTOMATIC_PARALLELIZATION.md) for details.

---

## Core Features

| Feature | Description |
|---|---|
| **JavaScript Execution** | ECMAScript evaluation powered by [Boa](https://github.com/boa-dev/boa) v0.20 |
| **Async Runtime** | Event loop with Promises, `async`/`await`, `setTimeout`, `setInterval` |
| **Independent Workers** | OS threads with fully isolated Boa contexts and zero shared JS memory |
| **Explicit Parallelism** | `parallel.map(array, callback)` distributing work across the worker pool |
| **Automatic Parallelization** | `--auto-parallel` flag with static safety analysis and sequential fallback |
| **Intelligent Scheduler** | Load-aware worker placement, priority queuing, starvation prevention |
| **Bounded Resources** | Configurable `--max-workers`, `--max-queued-tasks`, `--shutdown-timeout` |
| **Operational Diagnostics** | `--diagnostics` flag for execution metrics, queue latency, scheduler stats |
| **Error Isolation** | Worker crashes are isolated from the main thread and other workers |
| **Graceful Shutdown** | Timed worker termination with `Ctrl+C` signal handling |

---

## Architecture

```text
                    Javryn CLI
                        │
                        ▼
                   Javryn Runtime
                        │
             ┌──────────┴──────────┐
             │                     │
       JavaScript Engine      Parallel System
             │                     │
          Boa VM              Task Manager
             │                     │
         Event Loop            Scheduler
                                   │
                           ┌───────┼───────┐
                           ▼       ▼       ▼
                          W1      W2      W3
                          │       │       │
                         Boa     Boa     Boa
```

Javryn is organized as a Rust workspace with three crates:

| Crate | Responsibility |
|---|---|
| `javryn-cli` | CLI parsing, logging, signal handling, exit codes |
| `javryn-core` | Domain types, configuration, error hierarchy |
| `javryn-runtime` | Boa engine, event loop, workers, scheduler, parallel API, auto-parallelization |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full architecture breakdown.

---

## CLI Reference

```bash
javryn <SCRIPT>                          # Run a JavaScript file
javryn --version                         # Print version
javryn --help                            # Print help
javryn --diagnostics <SCRIPT>            # Run with operational diagnostics
javryn --auto-parallel <SCRIPT>          # Enable automatic parallelization
javryn --max-workers <N> <SCRIPT>        # Limit worker pool size (1–128)
javryn --max-queued-tasks <N> <SCRIPT>   # Limit task queue depth
javryn --shutdown-timeout <MS> <SCRIPT>  # Set shutdown timeout (≥100ms, default: 5000)
javryn --quiet <SCRIPT>                  # Suppress non-error output
javryn --verbose <SCRIPT>                # Enable verbose logging
javryn --debug <SCRIPT>                  # Enable debug-level logging
```

See [docs/CLI.md](docs/CLI.md) for the full reference.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic runtime failure |
| 2 | Invalid input (bad script path, syntax error) |
| 3 | Configuration failure |
| 4 | Initialization failure |
| 5 | Runtime execution failure |
| 6 | Shutdown failure |

---

## Examples

| File | Description |
|---|---|
| [`hello.js`](examples/hello.js) | Hello World |
| [`async.js`](examples/async.js) | Promises and event loop |
| [`async-await.js`](examples/async-await.js) | `async`/`await` |
| [`timers.js`](examples/timers.js) | `setTimeout` and `clearTimeout` |
| [`worker-basic.js`](examples/worker-basic.js) | Basic Worker creation |
| [`worker-message.js`](examples/worker-message.js) | Worker `postMessage` and `onmessage` |
| [`parallel-map.js`](examples/parallel-map.js) | Explicit parallel map |
| [`parallel-cpu.js`](examples/parallel-cpu.js) | CPU-intensive parallel workload |
| [`parallel-error.js`](examples/parallel-error.js) | Parallel error propagation |
| [`auto-parallel-basic.js`](examples/auto-parallel-basic.js) | Automatic parallelization |
| [`compute.js`](examples/compute.js) | Single-threaded Fibonacci baseline |

Run any example:

```bash
javryn examples/hello.js
javryn examples/parallel-map.js
javryn --diagnostics examples/parallel-cpu.js
javryn --auto-parallel examples/auto-parallel-basic.js
```

---

## JavaScript Compatibility

Javryn uses the [Boa](https://github.com/boa-dev/boa) JavaScript engine (v0.20) and supports:

- **ES6+ syntax**: arrow functions, `let`/`const`, destructuring, template literals, classes, spread
- **Promises & async/await**: microtask queue, `.then()`, `.catch()`
- **Timers**: `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`
- **JSON**: `JSON.stringify()`, `JSON.parse()`
- **Modules**: single-file `.mjs` evaluation

**Not available:**
- Node.js core modules (`fs`, `path`, `http`, `crypto`, etc.)
- Browser APIs (`DOM`, `fetch`, `WebSocket`, `localStorage`)
- `SharedArrayBuffer` / `Atomics`
- Multi-file module imports

See [docs/JAVASCRIPT_COMPATIBILITY.md](docs/JAVASCRIPT_COMPATIBILITY.md) for the full matrix.

---

## Security

> **Javryn is NOT a security sandbox.**

Worker threads provide JavaScript VM isolation (separate Boa contexts, zero shared JS memory), but scripts inherit the full OS permissions of the host process. There is no filesystem, network, or resource isolation.

See [docs/SECURITY.md](docs/SECURITY.md) for the full security model.

---

## Documentation

| Document | Description |
|---|---|
| [Installation](docs/INSTALLATION.md) | Build from source, prerequisites |
| [Getting Started](docs/GETTING_STARTED.md) | Step-by-step tutorial for new users |
| [CLI Reference](docs/CLI.md) | All CLI options with examples |
| [Parallelism](docs/PARALLELISM.md) | `parallel.map`, workers, scheduler, serialization |
| [Automatic Parallelization](docs/AUTOMATIC_PARALLELIZATION.md) | `--auto-parallel`, static analysis, safety rules |
| [Architecture](docs/ARCHITECTURE.md) | Crate structure, runtime architecture diagrams |
| [JavaScript Compatibility](docs/JAVASCRIPT_COMPATIBILITY.md) | Supported ECMAScript features and limitations |
| [Performance](docs/PERFORMANCE.md) | Benchmarks and measurement methodology |
| [Security](docs/SECURITY.md) | Security model and explicit non-guarantees |
| [Roadmap](docs/ROADMAP.md) | Version history and future direction |

---

## Development

```bash
# Check compilation
cargo check --workspace

# Run all tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run Clippy lints
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build release binary
cargo build --workspace --release
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

---

## Roadmap

Javryn V1.0 is the first public production release. Previous milestones:

| Version | Milestone |
|---------|-----------|
| V0.1 | Runtime Foundation |
| V0.2 | JavaScript Engine Integration |
| V0.3 | Async Runtime & Event Loop |
| V0.4 | Independent Workers |
| V0.5 | Explicit Parallelism |
| V0.6 | Memory + Concurrency |
| V0.7 | Intelligent Scheduler |
| V0.8 | Automatic Parallelization |
| V0.9 | Production Hardening |
| **V1.0** | **Production Release** ← current |

See [docs/ROADMAP.md](docs/ROADMAP.md) for details and future direction.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to build, test, and submit changes.

## License

MIT — see [LICENSE](LICENSE).
