# Javryn Scheduler Architecture (V0.7)

## Overview

Javryn V0.7 introduces an **Intelligent Scheduler** responsible for coordinating explicit parallel tasks submitted via `parallel.map(...)` across independent JavaScript worker threads.

> **Crucial Boundary Note**: V0.7 schedules explicitly created parallel tasks. It does **not** perform automatic parallelization, AST dependency analysis, loop decomposition, or speculative execution (which are reserved for V0.8).

---

## Core Architecture

```text
                  Javryn Runtime
                        │
                   Parallel API (`parallel.map`)
                        │
                   TaskManager
                        │
                ┌───────▼────────┐
                │   Scheduler    │
                └───────┬────────┘
                        │
            ┌───────────┼───────────┐
            ▼           ▼           ▼
         Worker 1    Worker 2    Worker N
```

### Module Responsibilities

1. **TaskManager**: Owns task lifecycle state transitions, bounded queues, backpressure enforcement, result aggregation, and Promise resolution/rejection.
2. **Scheduler**: Owns task selection (`select_task`), worker assignment policy (`select_worker`), priority queues, load-aware placement, aging starvation prevention, and diagnostic metrics collection.
3. **WorkerManager**: Owns worker thread lifecycle, thread spawning, message posting, and thread crash detection/restart.

---

## Scheduling Policies

The scheduler abstracts placement decisions behind the `SchedulingPolicy` trait:

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_task(&self, pending_tasks: &VecDeque<ParallelTask>) -> Option<usize>;
    fn select_worker(
        &self,
        available_workers: &[WorkerId],
        worker_stats: &HashMap<WorkerId, WorkerStats>,
    ) -> Option<WorkerId>;
}
```

### Supported Policies

- **`LoadAwarePolicy` (Default)**: Tracks active running task count and historical busy execution duration for each worker thread, dynamically assigning incoming tasks to the worker with the lowest active load.
- **`PriorityFifoPolicy`**: Selects highest priority tasks first, breaking ties using queue entry index order (FIFO).

---

## Priority & Aging Starvation Prevention

Tasks are assigned an explicit `TaskPriority`:

- `TaskPriority::High`
- `TaskPriority::Normal` (Default)
- `TaskPriority::Low`

To guarantee fairness and prevent continuous streams of high-priority tasks from starving lower-priority work, the scheduler executes an **Aging Boost** pass. Tasks remaining queued beyond `500ms` automatically receive a temporary priority boost to `TaskPriority::High`, ensuring every queued task makes steady execution progress.

---

## Diagnostics & Latency Metrics

`SchedulerMetrics` integrates directly into `TaskManagerDiagnostics`:

- `scheduler_dispatches`: Total dispatches executed.
- `scheduler_completed`: Total task completions recorded.
- `queue_wait_total_ms`: Accumulated time tasks spend in queue prior to dispatch.
- `queue_wait_max_ms`: Peak single-task queue wait duration.
- `queue_wait_avg_ms`: Average queue wait latency.
- `starvation_boosts`: Counter tracking aging priority boosts.
- `worker_stats`: Map of active worker thread task counts, completion counts, and busy execution durations.

---

## Compatibility with V0.6 Guarantees

- **Deterministic Result Aggregation**: Array indexing for `parallel.map(...)` results remains strictly preserved regardless of non-deterministic completion order across workers.
- **Backpressure**: Task queue limit (`max_queued_tasks`) remains authoritative, rejecting over-capacity submissions with `ConcurrencyLimitExceeded`.
- **Cancellation**: Cancelling an operation via `parallel.cancel(...)` purges queued tasks instantly while safely waiting for running tasks to conclude.
- **Worker Failure Handling**: Failed worker threads trigger safe operation teardown and error propagation without leaking pending scheduler state.
