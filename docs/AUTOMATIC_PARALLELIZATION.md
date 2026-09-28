# Javryn Automatic Parallelization Specification (V0.8)

## Overview

Javryn V0.8 introduces **Automatic Parallelization**, a static safety analysis and dynamic loop transformation layer that automatically parallelizes provably safe JavaScript loops and operations while guaranteeing 100% semantic correctness.

> **Primary Objective**: Preserve JavaScript semantics first. Parallelize only when safety can be proven with 100% certainty. Fall back to original sequential execution whenever safety is uncertain or side effects exist.

---

## System Architecture

```text
                    Javryn Runtime
                          │
                 JavaScript Engine (Boa)
                          │
              Automatic Parallelization Pass
                          │
              ┌───────────┴───────────┐
              │                       │
        Static Analysis         Safety Analysis
              │                       │
              └───────────┬───────────┘
                          │
              Parallelization Decision
                /                  \
               /                    \
          [ SAFE ]              [ UNSAFE ]
             │                      │
             ▼                      ▼
      Chunk Planner            Sequential JS
             │                  Fallback
             ▼                      │
        TaskManager                 │
             │                      │
         Scheduler                  │
             │                      │
       WorkerManager                │
             │                      │
      ┌──────┼──────┐               │
      ▼      ▼      ▼               │
     W1     W2     WN ──────────────┘
```

---

## Safety & Dependency Analysis

For candidate loops or expressions, the static analyzer calculates read sets ($R$) and write sets ($W$) across iterations ($i \neq j$):

1. **Read-Write Independence**: $R_i \cap W_j = \emptyset$
2. **Write-Read Independence**: $W_i \cap R_j = \emptyset$
3. **Write-Write Independence**: $W_i \cap W_j = \emptyset$ (except for disjoint indexed array assignments $output[i] \neq output[j]$).

### Supported Candidates

- Pure map callbacks: `x => x * x` or `x => x + 10`.
- Pure array transformations: `output[i] = expr` with read-only inputs.
- Opt-in `parallel.auto(fn)` blocks.

### Unsupported Candidates (Sequential Fallback)

- Accumulators: `total += x`.
- Array mutations: `arr.push()`, `arr.splice()`.
- Global state or object mutations: `obj.val = x`, `globalThis.counter++`.
- Side effects / I/O: `console.log()`, `fetch()`, timers, microtasks.
- Complex control flow: `break`, `continue`, `return`, `throw`, `async`/`await`.

---

## Chunking & Dynamic Scheduler Reuse

Large parallel loops are partitioned into dynamic chunks by the Chunk Planner:

$$\text{ChunkSize} = \max\left(25, \min\left(1000, \left\lceil \frac{N}{\text{AvailableWorkers} \times 4} \right\rceil\right)\right)$$

Chunk tasks pass directly to Javryn's V0.7 Intelligent Scheduler (`TaskManager` $\rightarrow$ `Scheduler` $\rightarrow$ `WorkerManager`).

---

## CLI & JavaScript API

- **CLI Flag**: `javryn --auto-parallel script.js`
- **JavaScript API**: `parallel.auto(() => { ... })`
