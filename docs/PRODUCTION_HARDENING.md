# Javryn Production Hardening Architecture (V0.9)

## Overview

Javryn V0.9 establishes production readiness across reliability, resource safety, failure recovery, observability, and security boundaries.

---

## 1. Reliability & Failure Recovery

- **Worker Failure Isolation**: Worker panics (caught via `catch_unwind`) set state code `STATE_FAILED`. `TaskManager::handle_worker_failure` removes the worker handle, rejects affected operations with detailed error strings, and purges assigned queued tasks.
- **Channel Failure Handling**: Disconnected worker channels map to `RuntimeError::WorkerCommunication` or `RuntimeError::WorkerFailed` without poisoning main thread state.
- **Timed Shutdown**: `WorkerHandle::terminate_with_timeout(Duration)` monitors thread completion, returning `RuntimeError::Shutdown` if thread join times out.

---

## 2. Resource Accounting & Invariants

System state strictly satisfies:
1. $\text{running\_tasks} \le \text{max\_workers}$
2. $\text{queued\_tasks} \le \text{max\_queued\_tasks}$
3. $\text{completed\_tasks} + \text{failed\_tasks} + \text{cancelled\_tasks} + \text{pending\_tasks} = \text{submitted\_tasks}$

---

## 3. Security & Boundary Model

- **Worker Isolation**: Each worker runs an isolated `boa_engine::Context` on a dedicated OS thread (`std::thread`). No JavaScript heap memory is shared across worker boundaries.
- **Serialization Boundary**: `JsMessage` enforces a serializable value protocol (null, undefined, boolean, number, string, array, plain object). Functions, symbols, cyclic references, and host handles are rejected with `RuntimeError::WorkerSerialization`.
- **Host Resource Access**: Javryn JavaScript contexts retain standard ECMAScript host bindings (`console`, timers). Javryn is **NOT a security sandbox** against local filesystem or process access.

---

## 4. Operational Diagnostics

The `--diagnostics` CLI flag outputs runtime metrics on completion:
- Total execution time
- Active operations and running tasks
- Worker pool size and task queue bounds
- Total dispatches and completions
- Average and maximum queue wait latency (ms)
- Starvation boosts executed by scheduler
