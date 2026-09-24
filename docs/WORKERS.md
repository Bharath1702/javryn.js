# Javryn V0.4 — Independent JavaScript Workers Architecture

Javryn V0.4 introduces multi-context JavaScript execution using independent worker threads.

---

## 1. Architectural Overview

Each worker in Javryn runs on a dedicated OS thread (`std::thread`) and owns an isolated JavaScript execution engine and event loop:

```text
                         Javryn Runtime
                               │
                       Worker Manager
                               │
             ┌─────────────────┼─────────────────┐
             ▼                 ▼                 ▼
        ┌─────────┐       ┌─────────┐       ┌─────────┐
        │Worker 1 │       │Worker 2 │       │Worker N │
        ├─────────┤       ├─────────┤       ├─────────┤
        │Boa VM   │       │Boa VM   │       │Boa VM   │
        │EventLoop│       │EventLoop│       │EventLoop│
        │Timers   │       │Timers   │       │Timers   │
        │Messages │       │Messages │       │Messages │
        └────┬────┘       └────┬────┘       └────┬────┘
             │                 │                 │
             └─────────────────┼─────────────────┘
                               │
                         Message Channels
```

---

## 2. Isolation Guarantees

* **JavaScript VM Isolation**: Each worker owns a separate `boa_engine::Context`.
* **Heap Isolation**: JavaScript objects, global objects (`globalThis`), functions, and prototypes are strictly local to their worker heap. They are never shared across threads.
* **Event Loop Isolation**: Each worker runs an independent event loop driving local Promise microtasks and timer queues.
* **Failure Isolation**: An unhandled exception or panic inside Worker 1 does not corrupt or terminate Worker 2 or the main process thread.

---

## 3. Communication & Serialization (`JsMessage`)

Workers communicate with the main thread using owned Rust channels (`std::sync::mpsc`).

Data payloads are serialized into an owned, thread-safe representation:

```rust
pub enum JsMessage {
    Null,
    Undefined,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsMessage>),
    Object(BTreeMap<String, JsMessage>),
}
```

Attempts to send non-serializable objects (such as functions, symbols, or promises) return a structured `RuntimeError::WorkerSerialization`.

---

## 4. JavaScript Host API

### Main Script
```javascript
// Spawn child worker
const worker = new Worker("./workers/child.js");

// Post message to worker
worker.postMessage({ count: 42 });

// Receive message from worker
worker.onmessage = (event) => {
    console.log("main received:", event.data);
};

// Handle worker errors
worker.onerror = (event) => {
    console.error("worker error:", event.message);
};

// Terminate worker
worker.terminate();
```

### Child Worker Script (`child.js`)
```javascript
// Receive message from main thread
onmessage = (event) => {
    console.log("child received:", event.data);
    // Post response to main thread
    postMessage(event.data.count * 2);
};
```

---

## 5. Security & Resource Limits

* **Process Memory**: Workers are isolated logically at the JavaScript VM boundary within the same OS process. They are NOT process-level sandboxes.
* **Thread Cleanup**: All worker threads are owned by `WorkerHandle` and `WorkerManager`. Thread handles are joined cleanly upon termination (`terminate()`) or runtime shutdown. No detached background threads exist.
