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
│  Lifecycle · Engine Abstraction         │
└──────────────────┬──────────────────────┘
                   │ uses types from
                   ▼
┌─────────────────────────────────────────┐
│            javryn-core                  │
│  Types · Errors · Config · Exit Codes   │
└─────────────────────────────────────────┘
```

## Engine Abstraction Layer (V0.2)

```text
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

## Crate Responsibilities

### javryn-cli
- Argument parsing with `clap`
- Logging initialization with `tracing-subscriber`
- Ctrl+C handling with `ctrlc`
- Translating `RuntimeError` into human-readable text and exit codes

### javryn-runtime
- Script validation and file reading
- Engine abstraction layer (`JavaScriptEngine` trait & `ExecutionResult`)
- Concrete `BoaEngineAdapter` wrapping `boa_engine`
- Native `console` global object binding
- Runtime lifecycle management (`initialize`, `execute`, `shutdown`)

### javryn-core
- Shared domain types (`Script`, `ScriptMetadata`, `RuntimeConfig`, `RuntimeMode`)
- Structured `RuntimeError` definitions including `JavaScriptSyntax` and `JavaScriptExecution`
- Documented `ExitCode` mappings

## Data & Error Flow

```
User executes javryn script.js
       │
       ▼
  Parse CLI arguments
       │
       ▼
  Validate script path & extension
       │
       ▼
  Read source code (UTF-8)
       │
       ▼
  Initialize Runtime & Engine Adapter
       │
       ▼
  Evaluate JavaScript source in engine
       │
       ├─ OK  ──► Console Output / Success Result (Exit Code 0)
       ├─ SyntaxError ──► RuntimeError::JavaScriptSyntax (Exit Code 2)
       └─ Exception   ──► RuntimeError::JavaScriptExecution (Exit Code 5)
```

## Explicit Parallel Architecture (V0.5)

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

### Decoupled Subsystem Responsibilities

1. **Parallel Host API (`parallel.map`)**: JavaScript host global exposed to main context. Serializes arguments/source and submits operation to `TaskManager`.
2. **Task Manager (`TaskManager`)**: Manages `OperationId`, `TaskId`, input index mapping (`InputIndex`), task queueing, worker pool assignment, and deterministic ordered result assembly.
3. **Worker Manager (`WorkerManager`)**: Manages the reusable pool of worker threads. Executes tasks by passing serialized callback source + argument payloads (`WorkerMessage::ExecuteTask`) to available workers.
4. **Isolated Workers**: Dedicated worker threads running isolated Boa JavaScript contexts and event loops. Executed results (`WorkerResponse::TaskCompleted`) are dispatched asynchronously through the main thread event loop.
