# Changelog

All notable changes to Javryn are documented in this file.

## 1.0.0

The first public production release of Javryn.

### Features

- **JavaScript Execution** — ECMAScript evaluation powered by Boa engine v0.20, supporting ES6+ syntax, closures, classes, arrays, objects, JSON, and Math
- **Async Runtime** — Event loop with Promise microtask queue, `async`/`await`, `setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`, and `queueMicrotask`
- **Independent Workers** — OS threads (`std::thread`) with fully isolated Boa JavaScript contexts and zero shared JS memory; bi-directional `postMessage`/`onmessage` communication
- **Explicit Parallelism** — `parallel.map(array, callback)` distributing array processing across worker thread pool with deterministic result ordering
- **Intelligent Scheduler** — Load-aware worker placement, priority queuing (`High`/`Normal`/`Low`), starvation prevention (500ms aging boost), and queue latency metrics
- **Automatic Parallelization** — `--auto-parallel` CLI flag and `parallel.auto(fn)` API with static safety analysis; guaranteed sequential fallback for unsafe or uncertain code
- **Bounded Resources** — Configurable `--max-workers` (1–128), `--max-queued-tasks`, and `--shutdown-timeout` (≥100ms) with strict validation
- **Operational Diagnostics** — `--diagnostics` flag reporting execution time, active operations, queue depth, worker pool size, dispatch/completion counts, queue wait latency, and starvation boosts
- **Error Isolation** — Worker panics and crashes isolated from main thread; failed workers evicted from pool; structured error propagation via promise rejection
- **Graceful Shutdown** — Timed worker termination with configurable timeout and Ctrl+C signal handling
- **Serialization Safety** — Strict value-copy protocol (`JsMessage`) supporting null, undefined, boolean, number, string, Array, and Object; functions, symbols, and cyclic objects cleanly rejected
- **Structured Exit Codes** — Documented exit taxonomy (0–6) covering success, generic failure, invalid input, configuration failure, initialization failure, runtime failure, and shutdown failure
- **CLI** — Full command-line interface via `clap` with `--help`, `--version`, `--quiet`, `--verbose`, `--debug`, and all runtime configuration flags

### Infrastructure

- Three-crate Rust workspace architecture (`javryn-cli`, `javryn-core`, `javryn-runtime`)
- Comprehensive test suite (unit, integration, compatibility, and benchmark tests)
- Documentation suite covering architecture, parallelism, automatic parallelization, JavaScript compatibility, security model, performance, CLI reference, installation, and getting started guide
