# Javryn V0.6 — Memory + Concurrency Architecture Specification

## 1. Overview
Javryn V0.6 strengthens the worker, operation, and task architecture under concurrency pressure. It establishes explicit memory ownership boundaries, task/operation lifecycle state machines, bounded task queue backpressure limits, runtime resource accounting diagnostics, worker state isolation, and failure recovery.

---

## 2. Memory Ownership Model

```text
Main Thread Runtime
   └── TaskManager
        ├── Operations (HashMap<OperationId, ParallelOperation>)
        └── Bounded Task Queue (VecDeque<ParallelTask>)

WorkerManager
   └── WorkerHandles (HashMap<WorkerId, WorkerHandle>)

Worker Thread
   ├── Isolated Boa Context
   ├── Worker EventLoop
   └── Isolated Worker Global Scope
```

### Ownership Rules
- **Main Runtime** owns `TaskManager` and `WorkerManager`.
- **TaskManager** owns active `ParallelOperation` tracking and the `pending_tasks` queue.
- **WorkerManager** owns OS thread handles (`WorkerHandle`).
- **Worker Threads** own their isolated `Context` (Boa engine), `TimerQueue`, and thread-local channels.
- **Zero Shared VM State**: VM contexts, global state, heaps, or native pointers are NEVER shared across thread boundaries.

---

## 3. Concurrency Bounds & Backpressure

- `max_workers`: Bounded thread pool size (defaulting to physical logical core count via `std::thread::available_parallelism()`).
- `max_queued_tasks`: Hard limit on total pending task items across queued operations (default: `10,000`).
- **Backpressure Behavior**: If submitting a `parallel.map()` operation would push `pending_tasks.len()` over `max_queued_tasks`, the operation is immediately rejected with a structured `ConcurrencyLimitExceeded` error and the Promise rejects.

---

## 4. State Machines

### Task Status (`TaskStatus`)
```text
Created ──► Queued ──► Running ──► Completed
                        │   ▲
                        │   │ (re-queue)
                        ├───┴─────► Failed
                        └───► Cancelled
```

### Operation Status (`OperationStatus`)
```text
Created ──► Queued ──► Running ──► Completed
                         │
                         ├──► Failed
                         └──► Cancelling ──► Cancelled
```

---

## 5. Result Assignment & Determinism
- Output arrays use direct indexed assignment (`results[input_index] = result`).
- Order is mathematically deterministic regardless of out-of-order worker thread completion timings.

---

## 6. Worker Isolation & Failure Recovery
- Reusable worker threads execute tasks in isolated Boa contexts.
- Global scope mutations made during task execution in a worker do not leak into main thread JS or affect other workers.
- If a worker thread panics or terminates unexpectedly, `handle_worker_failure` captures the event, rejects affected operations cleanly, cleans channel references, and preserves host runtime stability.
