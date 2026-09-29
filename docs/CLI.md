# Javryn CLI Reference

Complete reference for all Javryn command-line options.

---

## Usage

```bash
javryn [OPTIONS] <SCRIPT>
```

### Arguments

| Argument | Description |
|---|---|
| `<SCRIPT>` | Path to the JavaScript file to run. Supports `.js` and `.mjs` extensions. |

---

## Options

### `--help`, `-h`

Print help information.

```bash
javryn --help
```

### `--version`, `-V`

Print version information.

```bash
javryn --version
# javryn 1.0.0
```

---

### `--auto-parallel`

Enable automatic parallelization mode. The static analyzer inspects JavaScript functions and parallelizes provably safe workloads; unsafe or uncertain code falls back to sequential execution.

```bash
javryn --auto-parallel script.js
```

**Default:** Disabled.

---

### `--diagnostics`

Emit an operational diagnostics summary after execution completes. Reports execution time, worker pool utilization, scheduler metrics, and queue latency.

```bash
javryn --diagnostics examples/parallel-map.js
```

**Example output:**

```text
=== Javryn Runtime Diagnostics ===
Execution Time     : 10 ms
Active Operations  : 0
Queued Tasks       : 0
Running Tasks      : 0
Worker Pool Size   : 16
Max Task Queue     : 10000
Dispatches         : 4
Completions        : 4
Avg Queue Wait     : 7.00 ms
Max Queue Wait     : 7 ms
Starvation Boosts  : 0
==================================
```

**Default:** Disabled.

---

### `--max-workers <WORKERS>`

Maximum number of worker threads allowed in the worker pool.

```bash
javryn --max-workers 4 examples/parallel-cpu.js
```

**Default:** Number of available CPU cores.
**Valid range:** 1–128.
**Validation:** Values of 0 or greater than 128 are rejected with exit code 3 (Configuration failure).

---

### `--max-queued-tasks <TASKS>`

Maximum number of tasks that can be queued for execution. When the queue is full, additional task submissions are rejected with a `ConcurrencyLimitExceeded` error.

```bash
javryn --max-queued-tasks 100 examples/parallel-map.js
```

**Default:** 10000.
**Validation:** A value of 0 is rejected with exit code 3 (Configuration failure).

---

### `--shutdown-timeout <MS>`

Timeout in milliseconds for graceful worker pool shutdown. Workers that do not complete within this period are terminated.

```bash
javryn --shutdown-timeout 10000 examples/parallel-cpu.js
```

**Default:** 5000 (5 seconds).
**Valid range:** ≥ 100ms.
**Validation:** Values below 100 are rejected with exit code 3 (Configuration failure).

---

### `--quiet`

Suppress all non-error output. `console.log` and `console.info` are silenced; `console.warn` and `console.error` still print to stderr.

```bash
javryn --quiet examples/hello.js
```

Cannot be combined with `--verbose` or `--debug`.

---

### `--verbose`

Enable verbose informational logging to stderr.

```bash
javryn --verbose examples/hello.js
```

Cannot be combined with `--quiet` or `--debug`.

---

### `--debug`

Enable debug-level diagnostic logging to stderr. Produces detailed runtime lifecycle information.

```bash
javryn --debug examples/hello.js
```

Cannot be combined with `--quiet` or `--verbose`.

---

## Exit Codes

| Code | Name | Meaning |
|------|------|---------|
| 0 | `Success` | The runtime completed without error |
| 1 | `GenericFailure` | A generic, uncategorized runtime failure |
| 2 | `InvalidInput` | Invalid CLI arguments, missing script, invalid file, bad extension, or JavaScript syntax error |
| 3 | `ConfigurationFailure` | The runtime configuration is invalid or conflicting (e.g., `--max-workers 0`) |
| 4 | `InitializationFailure` | The runtime failed during initialization |
| 5 | `RuntimeFailure` | An error occurred during JavaScript execution (unhandled exception, parallel task failure) |
| 6 | `ShutdownFailure` | The runtime failed to shut down cleanly |

---

## Examples

```bash
# Run a script
javryn app.js

# Run with diagnostics
javryn --diagnostics app.js

# Run parallel workload with bounded resources
javryn --max-workers 4 --max-queued-tasks 50 app.js

# Run with automatic parallelization
javryn --auto-parallel app.js

# Run in quiet mode
javryn --quiet app.js

# Combine options
javryn --diagnostics --max-workers 8 --auto-parallel app.js
```
