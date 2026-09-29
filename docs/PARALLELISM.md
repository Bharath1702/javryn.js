# Javryn Parallelism Guide

This document covers Javryn's parallel execution model in depth: `parallel.map`, worker architecture, the task manager, scheduler, serialization, error handling, and when parallelism helps.

---

## Why Parallelism?

Traditional JavaScript runtimes execute user code in a single thread. CPU-bound workloads that consist of independent tasks cannot take advantage of multiple CPU cores.

Javryn introduces a parallel execution system:

```text
JavaScript Program
        │
    parallel.map(array, callback)
        │
    Task Manager
        │
    Scheduler (load-aware)
        │
    Worker Pool (OS threads)
        │
    ┌───┼───┐
    W1  W2  W3  (each with isolated Boa VM)
        │
    Results (ordered by input index)
        │
    Promise resolution
```

---

## `parallel.map`

### Syntax

```javascript
const results = await parallel.map(items, callback);
```

- **`items`** — A JavaScript `Array` of serializable values
- **`callback`** — A **string** containing a JavaScript function expression (e.g., `"x => x * x"`)
- **Returns** — A `Promise` that resolves to an array of results in the same order as the input

### Basic Example

```javascript
const values = [1, 2, 3, 4];

async function main() {
    const results = await parallel.map(values, "x => x * x");
    console.log("Results:", results);
    // Results: [ 1, 4, 9, 16 ]
}

main();
```

### Why Callbacks Are Strings

Worker threads run in completely isolated JavaScript VM instances. There is no shared memory or shared function scope between the main thread and workers.

To execute a function on a worker, Javryn serializes the function as a **source string**, sends it to the worker thread, and the worker evaluates it in its own Boa context.

This means:
- The callback **cannot close over** variables from the outer scope
- The callback must be **self-contained** — all needed logic must be inside the string
- Native functions (built-in methods) **cannot be serialized** as callbacks

### Complex Callbacks

For multi-line logic, use template literals:

```javascript
const fibSource = `n => {
    function fib(x) {
        if (x <= 1) return x;
        return fib(x - 1) + fib(x - 2);
    }
    return fib(n);
}`;

async function main() {
    const results = await parallel.map([30, 30, 30, 30], fibSource);
    console.log("Results:", results);
    // Results: [ 832040, 832040, 832040, 832040 ]
}

main();
```

---

## Worker Architecture

Each parallel task executes in its own **worker thread**:

- Workers are OS threads (`std::thread`) managed by the `WorkerManager`
- Each worker has a **fully isolated Boa JavaScript engine context**
- There is **zero shared JavaScript memory** between workers
- Data is exchanged via a structured value-copy protocol (`JsMessage`)

### Worker Lifecycle

1. Task submitted to `TaskManager`
2. `Scheduler` selects optimal worker (load-aware placement)
3. Worker receives task via MPSC channel
4. Worker evaluates callback in isolated Boa context
5. Worker returns result via response channel
6. `TaskManager` assembles results by input index
7. Promise resolves when all tasks complete

---

## Task Manager

The `TaskManager` coordinates parallel operations:

- Assigns unique `OperationId` and `TaskId` identifiers
- Manages task lifecycle state machine: `Created → Queued → Running → Completed/Failed/Cancelled`
- Enforces queue backpressure (configurable via `--max-queued-tasks`)
- Reconstructs result arrays preserving input order
- Handles operation cancellation via `parallel.cancel(operationId)`

---

## Scheduler

The intelligent scheduler distributes tasks across the worker pool:

- **Load-aware placement**: selects workers with the fewest active tasks
- **Priority queuing**: tasks support `High`, `Normal`, and `Low` priority levels
- **Starvation prevention**: tasks queued longer than 500ms are automatically priority-boosted
- **Metrics collection**: tracks dispatch count, completion count, queue wait latency

---

## Serialization

### Supported Types

Worker data exchange supports these JavaScript types:

| Type | Supported |
|---|---|
| `null` | ✅ |
| `undefined` | ✅ |
| `boolean` | ✅ |
| `number` | ✅ |
| `string` | ✅ |
| `Array` | ✅ (recursive) |
| `Object` | ✅ (recursive) |

### Unsupported Types

| Type | Status |
|---|---|
| Functions | ❌ Rejected with error |
| Symbols | ❌ Rejected with error |
| Cyclic objects | ❌ Rejected with error |
| `Map`, `Set` | ❌ Not serializable |
| `SharedArrayBuffer` | ❌ Not implemented |

Unsupported types are cleanly rejected without poisoning cross-thread channels.

---

## Result Ordering

`parallel.map` **always** returns results in the same order as the input array, regardless of the order in which workers complete their tasks. Each task carries an `InputIndex` that the `TaskManager` uses to reconstruct the result array.

---

## Concurrency Limits

### `--max-workers <N>`

Controls the maximum number of worker threads in the pool.

- **Default**: number of available CPU cores
- **Range**: 1–128
- Fewer workers = less parallelism but lower resource usage
- More workers = higher parallelism but more OS thread overhead

### `--max-queued-tasks <N>`

Controls the maximum task queue depth. When the queue is full, new task submissions fail with `ConcurrencyLimitExceeded`.

- **Default**: 10000
- Prevents unbounded memory growth from very large parallel operations

---

## Cancellation

Active parallel operations can be cancelled:

```javascript
// parallel.cancel(operationId) returns a boolean
const cancelled = parallel.cancel(opId);
```

Cancelled tasks that have not yet started are removed from the queue. Tasks already running on workers are not forcefully interrupted — they complete but their results are discarded.

---

## Error Handling

Errors in parallel tasks propagate as **rejected promises**:

```javascript
async function main() {
    try {
        await parallel.map([1, 2, 3, 4], `x => {
            if (x === 3) {
                throw new Error("Task failed at element 3");
            }
            return x * 2;
        }`);
    } catch (err) {
        console.log("Caught:", err.message);
        // Caught: Error: Task failed at element 3
    }
}

main();
```

If any task in a `parallel.map` operation fails, the entire operation's promise is rejected.

### Worker Crash Isolation

If a worker thread panics or crashes:
- The failure is contained — it does not crash the main thread or other workers
- The affected worker is evicted from the pool
- The failed task's promise is rejected
- Remaining workers continue operating

---

## When Parallelism Helps

✅ **Good candidates:**
- CPU-heavy computation (Fibonacci, hashing, mathematical transformations)
- Independent data transformations (no shared state between elements)
- Batch processing where each item is processed independently

❌ **Poor candidates:**
- Tiny workloads where parallel overhead dominates execution time
- I/O-bound work (file reads, network requests)
- Tasks with shared mutable state or accumulator patterns (`total += x`)
- Tasks with dependencies between elements

### Overhead

Parallel execution has inherent overhead:
- Worker thread creation and management
- Data serialization/deserialization across thread boundaries
- Scheduler dispatch and result assembly

For very small arrays or trivial computations, this overhead can make parallel execution **slower** than sequential. Parallelism is most beneficial when the per-element work is computationally significant.

---

## Diagnostics

Use `--diagnostics` to see parallel execution metrics:

```bash
javryn --diagnostics examples/parallel-map.js
```

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

| Metric | Meaning |
|---|---|
| Execution Time | Total wall-clock time for the script |
| Active Operations | Number of in-flight `parallel.map` operations at shutdown |
| Queued Tasks | Tasks waiting in the queue at shutdown |
| Running Tasks | Tasks currently executing on workers at shutdown |
| Worker Pool Size | Number of workers in the pool |
| Max Task Queue | Configured maximum queue depth |
| Dispatches | Total tasks dispatched to workers |
| Completions | Total tasks that completed successfully |
| Avg Queue Wait | Average time a task spent in the queue before dispatch |
| Max Queue Wait | Maximum queue wait time observed |
| Starvation Boosts | Times a task was priority-boosted due to queue aging (>500ms) |
