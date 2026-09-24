# Javryn JavaScript Engine Integration

## Overview

Javryn V0.2 integrates the **Boa JavaScript engine (`boa_engine` v0.20)** behind an abstract engine adapter layer (`JavaScriptEngine`).

```text
                    ┌─────────────────┐
                    │   Javryn CLI    │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Javryn Runtime  │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Engine Adapter  │  <-- JavaScriptEngine Trait
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ JavaScript VM   │  <-- Boa Engine Context
                    └─────────────────┘
```

## Engine Selection Rationale

- **Pure Rust Implementation**: 100% Rust with zero unsafe C/C++ build steps (no CMake or C++ compiler required).
- **ECMAScript Standards Coverage**: Supports ES2024 features including classes, arrow functions, promises, closures, maps, sets, destructuring, JSON, Math, RegExp, template literals, and ES modules.
- **Thread Isolation**: `boa_engine::Context` instances are self-contained isolates. Future Javryn workers (V0.4+) can spawn an independent `Context` on each worker thread without global locks.

## Engine Abstraction Layer

The engine contract is defined in `javryn-runtime`:

```rust
pub trait JavaScriptEngine {
    fn initialize(&mut self, mode: RuntimeMode) -> Result<(), RuntimeError>;
    fn execute(&mut self, source: &str, path: &Path) -> Result<ExecutionResult, RuntimeError>;
    fn shutdown(&mut self) -> Result<(), RuntimeError>;
}
```

The concrete `BoaEngineAdapter` implements this trait. Higher layers (`javryn-cli`) do not import or depend on `boa_engine` directly.

## Native Console Bridge

Javryn registers host functions into the JavaScript global scope:

- `console.log(...args)` -> Stdout (suppressed in `--quiet` mode)
- `console.info(...args)` -> Stdout (suppressed in `--quiet` mode)
- `console.warn(...args)` -> Stderr
- `console.error(...args)` -> Stderr

## Error Model & Exit Codes

| Error Type | Trigger | Exit Code |
|------------|---------|-----------|
| `RuntimeError::JavaScriptSyntax` | Syntax/Parse error in script | `2` (`ExitCode::InvalidInput`) |
| `RuntimeError::JavaScriptExecution` | Unhandled JS exception (`throw new Error(...)`, `TypeError`, etc.) | `5` (`ExitCode::RuntimeFailure`) |
| `RuntimeError::EngineInitialization` | Engine context failure | `4` (`ExitCode::InitializationFailure`) |

## Security Boundaries

- **NO Node.js APIs**: Node.js modules (`fs`, `net`, `http`, `child_process`, `process`, `require`) are **not** present.
- **NO Native System Calls**: JavaScript code cannot invoke shell commands, access arbitrary system memory, or query environment secrets.
- **Sandboxed Scope**: Execution is confined to standard ECMAScript globals and the Javryn host `console` bridge.

## Future Worker Architecture (V0.4+)

```text
                 Javryn Runtime
                       │
                       ▼
                  Scheduler (V0.7)
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
       Worker 1     Worker 2     Worker N
          │            │            │
        JS VM        JS VM        JS VM
          │            │            │
          └────────────┼────────────┘
                       │
                       ▼
                   CPU Cores
```
