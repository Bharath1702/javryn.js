# Javryn Roadmap

## V0.1 — Runtime Foundation ✅

**Status: Current**

Establishes the core runtime infrastructure:
- Rust workspace with 3 crates (CLI, Runtime, Core)
- CLI argument parsing with clap
- Script validation and metadata extraction
- Runtime lifecycle (create → init → run → shutdown)
- Structured error hierarchy with documented exit codes
- Cross-platform Ctrl+C handling
- Structured logging via tracing
- Comprehensive test suite (unit + integration + failure)
- Full documentation

## V0.2 — JavaScript Engine Integration ✅

**Status: Current**

Integrates the JavaScript execution engine:
- Integrated `boa_engine` behind `JavaScriptEngine` trait
- Real ECMAScript execution (variables, functions, classes, closures, arrays, objects, promises, maps, sets, JSON, Math)
- Native host console bridge (`console.log`, `console.info`, `console.warn`, `console.error`)
- Diagnostic mode filtering (`--quiet`, `--verbose`, `--debug`)
- Structured error handling for syntax errors (`ExitCode 2`) and unhandled exceptions (`ExitCode 5`)
- Full JS compatibility test suite (`tests/javascript/`)
- Benchmark baseline for single-threaded JavaScript workloads (`examples/compute.js`)

## V0.3 — Async Runtime & Event Loop ✅

**Status: Current**

Establishes the single-threaded asynchronous runtime:
- Event loop coordinating script execution, microtasks, and host timers
- Microtask queue integration for Promise jobs and `queueMicrotask`
- Macro-task timers (`setTimeout`, `clearTimeout`, `setInterval`, `clearInterval`)
- Non-blocking sleep mechanism (zero CPU busy-spinning)
- Pending work detection keeping runtime alive until completion
- Full async test matrix (`tests/javascript/16_timers.js` through `24_timer_errors.js`)

## V0.4 — Independent JavaScript Workers ✅

**Status: Current**

Establishes multi-context worker architecture:
- Independent worker threads (`std::thread`) with isolated `boa_engine::Context` per worker
- Thread-safe serializable data protocol (`JsMessage`) supporting numbers, strings, booleans, arrays, objects
- Bi-directional message channels (`postMessage`, `onmessage`, `onerror`)
- Full isolation for JS globals, VM heaps, event loops, and timer queues
- Panic and unhandled exception safety isolation
- Worker thread lifecycle management (`WorkerManager`) with clean thread joins (`terminate()`)
- Integration with main event loop to poll worker messages and process events

## V0.5 — Explicit Parallelism

Enable user-controlled parallel execution:
- SharedArrayBuffer
- Atomics
- Transferable objects
- Structured clone algorithm

## V0.6 — Memory Management

Advanced memory features:
- Memory limits per worker
- Garbage collection tuning
- Memory usage reporting
- Out-of-memory handling

## V0.7 — Scheduler

Intelligent work distribution:
- Task queue with priority support
- Work stealing
- CPU affinity
- Load balancing across workers
- Scheduler configuration API

## V0.8 — Automatic Parallelization

Transparent parallelism:
- Automatic detection of parallelizable operations
- Data dependency analysis
- Parallel collection operations
- Speculative execution

## V0.9 — Production Hardening

Production readiness:
- Performance benchmarking and optimization
- Security audit
- Fuzzing
- Stress testing
- Platform-specific optimizations
- Comprehensive documentation review

## V1.0 — Production Release

Stable public release:
- Stable API guarantee
- Migration guides
- Performance baselines published
- Community documentation
- CI/CD pipeline for releases
