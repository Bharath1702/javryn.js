# Javryn Security Model

## Overview

Javryn establishes thread-isolation boundaries between parallel worker environments, but **is NOT an OS-level security sandbox**.

> ⚠️ **Javryn is NOT a security sandbox against untrusted code execution.**

---

## 1. What Javryn Provides

### Worker VM Isolation

- Each worker thread runs an isolated `boa_engine::Context` on a dedicated OS thread (`std::thread`)
- JavaScript heap memory is **100% unshared** between worker threads and the main execution thread
- Failure or uncaught panic in a worker thread is isolated using `std::panic::catch_unwind`, preventing main process crashes
- Failed workers are evicted from the pool without affecting other workers

### Value Copy Serialization (`JsMessage`)

- Worker communication is enforced via a strict value-copy protocol
- Supported types: `null`, `undefined`, `boolean`, `number`, `string`, `Array`, `Object`
- Non-serializable types (functions, symbols, host native handles, cyclic object structures) are **rejected cleanly** with `RuntimeError::WorkerSerialization` without poisoning cross-thread channels

### Graceful Shutdown

- `--shutdown-timeout` guarantees that worker threads are joined or terminated within a configurable time limit
- Ctrl+C signal handling triggers orderly runtime cancellation

---

## 2. What Javryn Does NOT Provide

### No Process Isolation

Scripts executed in Javryn inherit the **full OS permissions** of the running user process. A JavaScript program can:

- Access the filesystem (if host APIs are extended to support it)
- Consume unlimited CPU and memory
- Interact with any resource the host process has access to

### No Filesystem Isolation

Native host functions operate with process-level filesystem access. Worker threads share the same OS-level permissions as the main thread.

### No Network Isolation

There is no network sandboxing. Future host API extensions for network access would operate with full process permissions.

### No CPU Resource Quotas

While `--shutdown-timeout` guarantees graceful thread termination on exit, an infinite `while(true)` loop inside a worker thread will consume CPU until runtime shutdown. There are no per-worker CPU time limits.

### No Memory Limits

There are no per-worker or per-script memory limits. A script that allocates unbounded memory will consume host process memory.

---

## 3. Guidelines for Rust Developers Extending Javryn

When adding custom native functions:

1. **Return `JsError`** rather than panicking inside native function pointers
2. **Validate inputs** — check array bounds and string lengths before allocation
3. **Do not store raw `boa_engine::JsValue` references** across thread boundaries — they are not `Send` or `Sync`
4. **Use the `JsMessage` protocol** for all cross-thread data transfer

---

## 4. Reporting Security Issues

If you discover a security vulnerability, please report it by opening a GitHub issue or contacting the maintainers directly. Do not include sensitive details in public issue descriptions.
