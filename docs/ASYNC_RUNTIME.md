# Javryn Async Runtime & Event Loop

## Overview

Javryn V0.3 introduces a single-threaded asynchronous runtime and event loop. It coordinates microtask execution (Promise jobs), macro-task execution (timers), and host API bindings without CPU busy-spinning.

```text
                 Javryn Runtime
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
   JavaScript Engine          Async Runtime
   (BoaEngineAdapter)          (EventLoop)
          │                         │
          │                 ┌───────┴────────┐
          │                 ▼                ▼
          │            Microtasks         Timers
          │            (Job Queue)     (Priority Queue)
          │                 │                │
          └─────────────────┴────────────────┘
                            │
                            ▼
                       Event Loop
                            │
                            ▼
                    Single-Threaded JS
```

## Architecture & Lifecycles

Javryn's event loop executes in deterministic phases:

1. **Initial Script Evaluation**: Source code is compiled and evaluated in the JavaScript engine context.
2. **Microtask Phase**: Drains all pending Promise continuations and jobs in Boa's internal job queue (`context.run_jobs()`).
3. **Macro-Task Phase (Timers)**:
   - Pops all timers from the `TimerQueue` whose `due_time <= Instant::now()`.
   - Invokes their JS callback functions (`callback.call(...)`).
   - Drains microtasks immediately after each batch of callbacks.
   - Reschedules interval timers (`setInterval`).
4. **Non-Blocking Sleep**: If no timers are ready but active timers remain, the event loop sleeps using `std::thread::sleep(deadline - Instant::now())`.
5. **Runtime Completion**: Process exits when no pending microtasks or active timers remain in the queue.

## Host Timer & Microtask APIs

| Global API | Functionality |
|------------|---------------|
| `setTimeout(callback, delayMs)` | Schedules a single-shot timer after `delayMs` |
| `clearTimeout(timerId)` | Cancels a pending timer by its opaque `TimerId` |
| `setInterval(callback, intervalMs)` | Schedules a recurring interval timer every `intervalMs` |
| `clearInterval(timerId)` | Cancels a recurring interval timer |
| `queueMicrotask(callback)` | Schedules a microtask for prompt execution |

## Event Loop Ordering Semantics

Host execution follows standard JavaScript event-loop ordering:

```javascript
console.log("A");

Promise.resolve().then(() => {
    console.log("B");
});

setTimeout(() => {
    console.log("C");
    Promise.resolve().then(() => {
        console.log("D");
    });
}, 10);

console.log("E");
```

**Output**:
```text
A
E
B
C
D
```

1. **Script Turn**: Prints `A`, enqueues Promise job `B`, schedules timer `C`, prints `E`.
2. **Microtask Turn**: Drains job queue -> Prints `B`.
3. **Timer Turn**: Waits 10ms -> Fires timer -> Prints `C`, enqueues Promise job `D`.
4. **Post-Timer Microtask Turn**: Drains job queue -> Prints `D`.
5. **Completion**: All queues empty -> Process exits cleanly.

## Future Worker Architecture (V0.4+)

```text
Future V0.4

Worker 1
 ├── JS Engine Context
 └── Event Loop

Worker 2
 ├── JS Engine Context
 └── Event Loop

Worker N
 ├── JS Engine Context
 └── Event Loop
```

V0.3 maintains a strict single-threaded execution model per runtime instance. Future worker threads (V0.4+) will instantiate dedicated isolates and event loops per worker.
