# Javryn Runtime

## Runtime Lifecycle

The Javryn runtime follows a deterministic, linear lifecycle:

```
Create Runtime
     │
     ▼
Initialize
     │
     ▼
Execute (placeholder in V0.1)
     │
     ▼
Shutdown
     │
     ▼
Return Result
```

### Create

```rust
let runtime = Runtime::new(config)?;
```

Constructs a new runtime instance with the given configuration. No initialization occurs at this stage — the runtime is inert.

### Initialize

Called internally by `runtime.run()`. Sets up the runtime environment. In V0.1, this is lightweight. In future versions, this will initialize the JavaScript engine, allocate worker contexts, and configure the scheduler.

Initialization can only happen once. Calling `run()` a second time returns an `Initialization` error.

### Execute

In V0.1, execution validates the script and logs diagnostic information. No JavaScript is executed.

In V0.2+, this will pass the validated `Script` to the JavaScript engine.

### Shutdown

```rust
runtime.shutdown()?;
```

Releases all resources. This method is idempotent — calling it multiple times is safe.

If `shutdown()` is not called explicitly, the runtime's `Drop` implementation performs a best-effort cleanup with a warning.

## Configuration

`RuntimeConfig` is built using the builder pattern:

```rust
let config = RuntimeConfig::builder()
    .script_path(PathBuf::from("app.js"))
    .mode(RuntimeMode::Debug)
    .build()?;
```

### Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `script_path` | `PathBuf` | (required) | Path to the JavaScript file |
| `mode` | `RuntimeMode` | `Normal` | Logging/diagnostic mode |

### Extensibility

Future versions will add:
- `worker_count: Option<usize>`
- `memory_limit: Option<usize>`
- `scheduler_config: Option<SchedulerConfig>`
- `engine_config: Option<EngineConfig>`

The builder pattern ensures these additions are non-breaking.

## Script Validation

When `javryn app.js` is invoked, the script goes through a validation pipeline:

1. **Path resolution** — relative paths are resolved to absolute paths
2. **Existence check** — the file must exist on disk
3. **Type check** — must be a regular file, not a directory
4. **Extension check** — must be `.js` or `.mjs`
5. **Readability check** — the file must be openable for reading

Each validation step produces a specific `RuntimeError` variant on failure.

### ScriptMetadata

The `ScriptMetadata` struct captures filesystem information:

| Field | Type | Description |
|-------|------|-------------|
| `file_size` | `u64` | Size in bytes |
| `modified_time` | `Option<SystemTime>` | Last modification time |

## Error Handling

All errors flow through `RuntimeError`, which maps to documented exit codes:

| Error Variant | Exit Code | When |
|---------------|-----------|------|
| `ScriptNotFound` | 2 | Script file doesn't exist |
| `ScriptIsDirectory` | 2 | Path points to a directory |
| `ScriptUnreadable` | 2 | Permission denied or I/O error |
| `InvalidExtension` | 2 | Unsupported file extension |
| `Io` | 2 | Generic I/O failure |
| `Configuration` | 3 | Invalid configuration |
| `Initialization` | 4 | Runtime failed to initialize |
| `Runtime` | 5 | Error during execution |
| `Shutdown` | 6 | Error during shutdown |
| `Internal` | 1 | Bug in Javryn |

At the CLI boundary, errors are converted to:
1. A human-readable message on stderr
2. The corresponding process exit code

## Shutdown

The runtime handles:
- **Normal completion** — lifecycle finishes, `shutdown()` is called
- **Ctrl+C / SIGINT** — the interrupt flag is checked before execution
- **Runtime errors** — errors propagate up, runtime is still shut down
- **Drop safety** — if `shutdown()` is forgotten, `Drop` performs cleanup

RAII is used throughout — no manual resource management is needed.
