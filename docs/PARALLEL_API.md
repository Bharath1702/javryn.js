# Javryn V0.5 — Explicit Parallel JavaScript API Specification

## 1. Overview

Javryn V0.5 introduces the first explicit CPU-parallel execution primitive: `parallel.map()`.

It allows JavaScript applications to explicitly decompose data arrays into independent tasks and execute them concurrently across a reusable pool of worker threads running isolated Boa JavaScript contexts.

---

## 2. Global API Signature

```typescript
parallel.map<T, R>(
    items: T[],
    callback: string | ((item: T) => R)
): Promise<R[]>
```

### Arguments

1. `items` (`Array`): An array of serializable JavaScript values (`null`, `undefined`, `boolean`, `number`, `string`, `Array`, `Object`).
2. `callback` (`Function` | `String`): The function source or callback to execute for each array element inside worker contexts.

### Return Value

Returns a native JavaScript `Promise` that resolves to an ordered array containing the results of each callback invocation corresponding to the input index.

---

## 3. Usage Examples

### Basic Parallel Computation

```javascript
const values = [1, 2, 3, 4];

async function main() {
    const results = await parallel.map(values, "x => x * x");
    console.log(results); // [1, 4, 9, 16]
}

main();
```

### CPU-Intensive Workload

```javascript
const inputs = [30, 30, 30, 30];

const fibSource = `n => {
    function fib(x) {
        if (x <= 1) return x;
        return fib(x - 1) + fib(x - 2);
    }
    return fib(n);
}`;

async function main() {
    const results = await parallel.map(inputs, fibSource);
    console.log(results); // [832040, 832040, 832040, 832040]
}

main();
```

---

## 4. Architecture & Isolation Rules

```text
                         JAVRYN
                           │
              ┌────────────┴────────────┐
              │                         │
       Main JavaScript             Parallel API
          Runtime                       │
              │                    Task Manager
              │                         │
              │                    Worker Manager
              │                         │
              │              ┌──────────┼──────────┐
              │              ▼          ▼          ▼
              │           Worker 1  Worker 2  Worker N
              │              │          │          │
              │             Boa        Boa        Boa
              │              │          │          │
              │           EventLoop  EventLoop  EventLoop
              │
              └──────────── Worker Responses
                              │
                              ▼
                         Main Event Loop
                              │
                              ▼
                         Promise Result
```

1. **No Shared JavaScript Memory**: Workers do not share heap objects, Boa contexts, or variable scopes.
2. **Explicit Parallelism Only**: Standard JavaScript operations (`Array.prototype.map`) remain single-threaded. Parallelism occurs only via `parallel.map()`. V0.7 schedules explicitly created tasks; it does not automatically parallelize sequential loops.
3. **Reusable Worker Pool & Intelligent Scheduler**: Workers are created up to configured limits and tasks are scheduled dynamically via `Scheduler` with load-aware placement and starvation prevention.
4. **Deterministic Result Ordering**: Results are collected by `input_index`, preserving input array order regardless of non-deterministic thread completion timing.
5. **Non-Blocking Event Loop**: Main runtime event loop continues processing timers and microtasks while worker threads compute tasks.

---

## 5. Error & Failure Handling

- If any task callback throws an unhandled exception or fails compilation, the returned `Promise` rejects with a structured error identifying the task failure.
- Remaining queued tasks for the operation are cancelled immediately.
- Worker threads remain healthy and return to the pool for subsequent operations.
