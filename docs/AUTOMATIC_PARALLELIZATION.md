# Javryn Automatic Parallelization

## Overview

Javryn can automatically analyze JavaScript functions and decide whether they are safe to execute in parallel. This is an **opt-in** feature enabled via the `--auto-parallel` CLI flag or the `parallel.auto(fn)` JavaScript API.

> **Design principle:** Preserve JavaScript semantics first. Parallelize only when safety can be proven. Fall back to sequential execution whenever safety is uncertain.

---

## How to Use

### CLI Flag

```bash
javryn --auto-parallel script.js
```

### JavaScript API

```javascript
parallel.auto(() => {
    // Your code here
    // Analyzer will determine if this is safe for parallel execution
});
```

---

## How It Works

```text
JavaScript Function
        │
   Static Analyzer
        │
   Safety Decision
     /        \
  SAFE       UNSAFE
    │           │
  Parallel    Sequential
  Execution   Fallback
```

1. The **Static Analyzer** inspects the function body for side effects, mutations, and dependencies
2. If the function is **provably safe** (pure expression, no shared state), it may be parallelized
3. If the function is **unsafe or uncertain**, it is executed sequentially — exactly as if `--auto-parallel` were not used

There is **never a correctness risk**. The worst case is that the function runs sequentially.

---

## Supported Patterns (Safe → Parallel)

These patterns are recognized as safe for automatic parallelization:

```javascript
// Pure arithmetic expressions
x => x * x
x => x + 10
x => Math.sqrt(x)

// Pure transformations
x => x * 2 + 1
```

The key requirement: the function must be a **pure expression** — it reads only its argument and produces a result without side effects.

---

## Unsupported Patterns (Unsafe → Sequential Fallback)

The following patterns cause the analyzer to fall back to sequential execution:

### Shared Accumulators / Mutations

```javascript
x => { total += x; return x; }       // += mutation
x => { counter++; return x; }        // ++ mutation
```

### Array Mutations

```javascript
x => { arr.push(x); return x; }      // .push()
x => { arr.pop(); return x; }        // .pop()
x => { arr.splice(0, 1); return x; } // .splice()
```

### Side Effects / I/O

```javascript
x => { console.log(x); return x; }   // console output
x => { fetch('http://api'); }         // network I/O
```

### Global State Access

```javascript
x => { globalThis.counter++; }       // globalThis
x => { window.location = 'test'; }   // window
```

### Timers and Async

```javascript
x => { setTimeout(() => {}, 10); }   // timer
x => { async function f() {} }       // async
x => { await f(); }                  // await
```

### Complex Control Flow

```javascript
x => { try { f(); } catch(e) {} }    // try/catch
x => { throw new Error(); }          // throw
x => { break; }                      // break
x => { continue; }                   // continue
```

### Indexed Array Writes

```javascript
x => { output[0] = x; }             // indexed write
```

---

## Static Analysis Details

The `StaticAnalyzer` performs textual safety inspection of the function source. It checks for patterns that indicate:

1. **Read-Write Conflicts** — Does iteration `i` write to data that iteration `j` reads?
2. **Write-Write Conflicts** — Do multiple iterations write to the same location?
3. **Side Effects** — Does the function interact with the outside world (console, timers, network)?

If any of these patterns are detected, the function is classified as **unsafe** and runs sequentially.

### Limitations

The current static analyzer is **conservative** — it uses textual pattern matching rather than full AST-level data flow analysis. This means:

- Some theoretically safe functions may be classified as unsafe (false negatives for parallelization)
- The analyzer will **never** classify an unsafe function as safe (no false positives)
- Complex expressions that are actually pure may not be recognized as safe

This is by design: correctness is always prioritized over parallelization opportunity.

---

## Chunking

When automatic parallelization is applied to large workloads, the Chunk Planner partitions items into optimally-sized chunks:

```
ChunkSize = max(25, min(1000, ceil(N / (AvailableWorkers × 4))))
```

Chunks are dispatched through the standard Javryn scheduler for load-aware execution across the worker pool.

---

## Diagnostics

Use `--diagnostics` with `--auto-parallel` to see parallelization metrics:

```bash
javryn --auto-parallel --diagnostics script.js
```

The analyzer logs its decisions at the `info` tracing level. Use `--verbose` to see analyzer decisions:

```bash
javryn --verbose --auto-parallel script.js
```
