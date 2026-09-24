# Javryn Architecture

## Overview

Javryn uses a **3-crate workspace** architecture that separates concerns into distinct layers:

```
┌─────────────────────────────────────────┐
│              javryn-cli                  │
│  CLI parsing · User output · Exit codes │
└──────────────────┬──────────────────────┘
                   │ invokes
                   ▼
┌─────────────────────────────────────────┐
│           javryn-runtime                │
│  Lifecycle · Validation · Execution     │
└──────────────────┬──────────────────────┘
                   │ uses types from
                   ▼
┌─────────────────────────────────────────┐
│            javryn-core                  │
│  Types · Errors · Config · Exit Codes   │
└─────────────────────────────────────────┘
```

## Why Three Crates?

### javryn-cli

**Purpose**: The user-facing entry point.

This crate is responsible for:
- Parsing command-line arguments (via `clap`)
- Initializing structured logging (via `tracing-subscriber`)
- Setting up signal handlers (Ctrl+C)
- Invoking the runtime lifecycle
- Converting `RuntimeError` into human-readable messages and process exit codes

**Why it's separate**: CLI concerns (argument parsing, output formatting, exit codes) should never leak into the runtime engine. This boundary ensures the runtime can be embedded in other contexts (e.g., as a library, in a test harness) without bringing along CLI dependencies.

### javryn-runtime

**Purpose**: The runtime lifecycle engine.

This crate owns:
- Script validation and metadata extraction
- Runtime initialization and shutdown
- The execution lifecycle (placeholder in V0.1, JS engine in V0.2+)

**Why it's separate**: The runtime lifecycle is the core business logic. Keeping it independent of both the CLI layer above and the type definitions below ensures that:
1. The runtime can be tested without invoking the CLI
2. Future engine integrations (QuickJS, Boa, V8) are isolated here
3. The runtime API remains stable even as the CLI evolves

### javryn-core

**Purpose**: Shared vocabulary types.

Contains:
- `RuntimeConfig` and `RuntimeConfigBuilder`
- `RuntimeError` (the error hierarchy)
- `ExitCode` (documented exit code policy)
- `Script` and `ScriptMetadata`
- `RuntimeMode` and `RuntimeResult`

**Why it's separate**: Both `javryn-cli` and `javryn-runtime` need these types. Putting them in a shared crate prevents circular dependencies and ensures a single source of truth for domain concepts.

## Dependency Graph

```
javryn-cli ──────► javryn-runtime ──────► javryn-core
    │                                         ▲
    └─────────────────────────────────────────┘
```

External dependencies:
- `clap` — only in `javryn-cli`
- `tracing-subscriber` — only in `javryn-cli`
- `tracing` — in `javryn-runtime` (and re-used via `javryn-cli`)
- `thiserror` — only in `javryn-core`
- `ctrlc` — only in `javryn-cli`

## Data Flow

```
User invokes CLI
       │
       ▼
  Parse arguments (clap)
       │
       ▼
  Build RuntimeConfig
       │
       ▼
  Validate script (filesystem checks)
       │
       ▼
  Create Runtime
       │
       ▼
  Initialize → Run → Shutdown
       │
       ▼
  Return ExitCode to OS
```

## Error Flow

```
RuntimeError (from javryn-core)
       │
       ├─ ScriptNotFound
       ├─ ScriptIsDirectory
       ├─ ScriptUnreadable
       ├─ InvalidExtension
       ├─ Io
       ├─ Configuration
       ├─ Initialization
       ├─ Runtime
       ├─ Shutdown
       └─ Internal
       │
       ▼
  error.exit_code() → ExitCode
       │
       ▼
  print_error() → stderr
       │
       ▼
  process::ExitCode → OS
```

## Future Architecture (V0.2+)

The current architecture is designed to accommodate:

```
                    JAVRYN

              JavaScript Program
                       │
                       ▼
                JS Engine Layer       ← V0.2
                       │
                       ▼
              Runtime Abstraction     ← V0.1 (current)
                       │
            ┌──────────┴──────────┐
            │                     │
      Async Runtime          Task Runtime    ← V0.3–V0.4
                                  │
                                  ▼
                             Scheduler       ← V0.7
                                  │
                         ┌────────┼────────┐
                         ▼        ▼        ▼
                      Worker   Worker   Worker  ← V0.4–V0.5
```

The V0.1 foundation provides the "Runtime Abstraction" layer that all future components will build upon.
