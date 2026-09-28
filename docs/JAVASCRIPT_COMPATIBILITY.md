# Javryn JavaScript Compatibility Matrix (V1.0)

## Overview

Javryn embeds the [Boa JavaScript Engine](https://github.com/boa-dev/boa) (v0.20) for ECMAScript evaluation, paired with a custom Rust event loop and parallel thread worker architecture.

---

## 1. ECMAScript Language Features

| Feature | Status | Details |
|---|---|---|
| **ES6+ Syntax** | Supported | Arrow functions, destructuring, template literals, `let`/`const`, spread operator, classes. |
| **Promises & Microtasks** | Supported | Standard `Promise.resolve()`, `.then()`, `.catch()`, `async/await`. Microtask queue drained per event loop tick. |
| **Arrays & Objects** | Supported | Standard JS array methods (`.map()`, `.filter()`, `.reduce()`) and object initializers. |
| **JSON Serialization** | Supported | `JSON.stringify()` and `JSON.parse()` fully functional within scripts and worker boundaries. |
| **Modules (`.mjs`)** | Single-File Supported | ESM source evaluation supported per single file. Multi-file module import resolution is deferred to V1.1+. |

---

## 2. Host Bindings & Timers

| API | Status | Details |
|---|---|---|
| `console.log`, `console.error` | Supported | Direct stdout/stderr output formatting via native Rust host functions. |
| `setTimeout(fn, delayMs)` | Supported | Non-blocking timer scheduled on Javryn `TimerQueue`. |
| `setInterval(fn, periodMs)` | Supported | Periodic timer rescheduled automatically until cleared. |
| `clearTimeout(id)`, `clearInterval(id)` | Supported | Timer cancellation by numeric `TimerId`. |

---

## 3. Javryn Parallel APIs

| API | Status | Details |
|---|---|---|
| `parallel.map(array, fnSource)` | Supported | Distributes array item evaluation across worker thread pool. Resolves Promise with ordered result array. |
| `parallel.auto` | Supported | Controlled via `--auto-parallel` flag. Automatically analyzes sequential map operations for pure expressions. |

---

## 4. Known Compatibility Limitations

1. **Browser / Web APIs**: DOM (`document`, `window`), `fetch`, `XMLHttpRequest`, `WebSocket`, and `localStorage` are **NOT** present.
2. **Node.js Core Modules**: Node built-in modules (`fs`, `path`, `http`, `child_process`, `net`, `crypto`) are **NOT** implemented.
3. **Shared Memory**: `SharedArrayBuffer` and `Atomics` are not supported. Worker threads exchange data via rigid value copy serialization (`JsMessage`).
