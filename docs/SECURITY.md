# Javryn Security & Boundary Architecture (V1.0)

## Overview

Javryn establishes a strict thread-isolation boundary between parallel worker environments, but **is NOT an OS-level sandbox**.

---

## 1. Security & Boundary Guarantees

### Worker VM Isolation
* Each worker thread runs an isolated `boa_engine::Context` on a dedicated OS thread (`std::thread`).
* JavaScript heap memory is **100% unshared** between worker threads and the main execution thread.
* Failure or uncaught panic in a worker thread is isolated using `std::panic::catch_unwind`, preventing main process crashes.

### Value Copy Serialization (`JsMessage`)
* Worker communication is enforced via a strict value-copy protocol.
* Supported types: `null`, `undefined`, `boolean`, `number`, `string`, `Array`, `Object`.
* Non-serializable types (functions, symbols, host native handles, cyclic object structures) are rejected with `RuntimeError::WorkerSerialization` without poisoning cross-thread channels.

---

## 2. Explicit Security Non-Guarantees (What Javryn Is NOT)

> [!WARNING]
> **Javryn is NOT a security sandbox against untrusted code execution.**

* **Process Permissions**: Scripts executed in Javryn inherit the full OS permissions of the running user process.
* **No File System Isolation**: Native host functions or future host extensions operate with process-level file system access.
* **No Resource Quotas for Malicious CPU Loops**: While `--shutdown-timeout` guarantees graceful thread join termination on exit, an infinite `while(true)` loop inside a worker thread will consume CPU until runtime shutdown.

---

## 3. Host API Safety Guidelines for Rust Developers

When extending Javryn with custom Rust native functions:
1. Always return `JsError` rather than panicking inside native function pointers.
2. Validate array bounds and string lengths before allocation.
3. Do not store raw `boa_engine::JsValue` references across thread boundaries.
