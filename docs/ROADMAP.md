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

## V0.5 — Explicit Parallel JavaScript ✅

Enable user-controlled explicit CPU-parallel execution:
- Global `parallel.map(items, callback)` returning native JavaScript `Promise`
- Decoupled `TaskManager` and `WorkerManager` architecture
- Inputs decomposed into tasks processed by reusable worker pool up to physical core bounds
- Deterministic result ordering preserved by `InputIndex` tracking
- Non-blocking main event loop processing timers and microtasks concurrently with parallel work

## V0.6 — Memory + Concurrency ✅

**Status: Current**

Advanced memory ownership, queue bounds, and concurrency features:
- Strict memory ownership boundaries with zero shared JS VM memory across threads
- Explicit `TaskStatus` and `OperationStatus` state machines preventing double-resolution or illegal transitions
- Bounded task queue backpressure limits (`max_queued_tasks`) rejecting overflow with `ConcurrencyLimitExceeded`
- Worker JS global isolation and worker crash recovery (`handle_worker_failure`)
- Real-time resource accounting and diagnostics snapshot (`active_operations`, `queued_tasks`, `running_tasks`, `pool_size`)

## V0.7 — Intelligent Scheduler ✅

**Status: Current**

Intelligent work distribution and load-aware scheduling:
- Decoupled `Scheduler` abstraction separating policy from task lifecycle (`TaskManager`) and thread management (`WorkerManager`)
- Dynamic load-aware worker placement policy (`LoadAwarePolicy`) tracking active tasks and historical execution duration
- Priority model (`TaskPriority::High`, `Normal`, `Low`) with `PriorityFifoPolicy`
- Aging starvation prevention automatically boosting tasks queued longer than 500ms
- Comprehensive latency diagnostics (`queue_wait_avg_ms`, `queue_wait_max_ms`, `starvation_boosts`) in runtime diagnostics
- Preserves V0.6 guarantees: backpressure, explicit cancellation, worker crash recovery, and deterministic indexed result ordering

## V0.8 — Automatic Parallelization (Future)

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
