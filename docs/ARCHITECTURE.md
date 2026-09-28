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

## Intelligent Scheduler Architecture (V0.7)

```text
                     Javryn Runtime
                           │
                 Parallel API (`parallel.map`)
                           │
                      TaskManager
                           │
                  ┌────────▼────────┐
                  │    Scheduler    │
                  └────────┬────────┘
                           │
               ┌───────────┼───────────┐
               ▼           ▼           ▼
            Worker 1    Worker 2    Worker N
```

### Subsystem Responsibilities

1. **Parallel Host API (`parallel.map`)**: Exposes host global to main context, submitting tasks and returning a native `Promise`.
2. **Task Manager (`TaskManager`)**: Owns task lifecycle state machines (`TaskStatus`, `OperationStatus`), bounded task queue (`max_queued_tasks`), result array index storage, and Promise settlement.
3. **Scheduler (`Scheduler`)**: Determines task selection (`select_task`), worker assignment (`select_worker`), priority policy (`SchedulingPolicy`), starvation prevention (aging boost), and queue latency metrics collection.
4. **Worker Manager (`WorkerManager`)**: Manages thread pool spawning, thread health, message posting, and thread recovery (`handle_worker_failure`).
5. **Isolated Workers**: Worker threads running isolated Boa engine contexts processing items in isolation and returning completed results via MPSC response channels.

## Automatic Parallelization Architecture (V0.8)

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

### Automatic Subsystem Responsibilities

1. **Static Analyzer (`StaticAnalyzer`)**: Inspects statement and expression structures, calculating iteration read/write sets ($R_i \cap W_j$) and screening out mutations or side effects.
2. **Safety Inspector**: Produces `ParallelizationDecision::Parallel` or `ParallelizationDecision::Sequential(Reason)`.
3. **Chunk Planner (`ChunkPlanner`)**: Partitions large parallel loops into dynamic chunks.
4. **Scheduler Integration**: Dispatches candidate task chunks through `TaskManager` and `Scheduler` without bypassing V0.7 worker management or diagnostics.
