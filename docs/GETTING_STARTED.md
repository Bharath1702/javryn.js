# Getting Started with Javryn

This guide walks you through installing Javryn, running your first JavaScript program, and exploring parallelism — all in about 5 minutes.

---

## 1. Install Javryn

Javryn is built from source using the Rust toolchain.

**Install Rust** (if you don't have it):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit [https://rustup.rs](https://rustup.rs/) for Windows/macOS installers.

**Clone and build Javryn:**

```bash
git clone https://github.com/Bharath1702/javryn.js.git
cd javryn.js
cargo build --release
```

The binary is at `target/release/javryn` (or `target\release\javryn.exe` on Windows).

---

## 2. Verify Installation

```bash
./target/release/javryn --version
```

Expected output:

```text
javryn 1.0.0
```

---

## 3. Your First Program

Create a file called `hello.js`:

```javascript
console.log("Hello from Javryn!");
```

Run it:

```bash
./target/release/javryn hello.js
```

Expected output:

```text
Hello from Javryn!
```

Javryn runs JavaScript. That's the foundation.

---

## 4. Running JavaScript

Javryn supports standard JavaScript features including variables, functions, classes, closures, arrays, objects, JSON, and `Math`:

```javascript
// basics.js
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
console.log("Doubled:", JSON.stringify(doubled));

function greet(name) {
    return `Hello, ${name}!`;
}
console.log(greet("World"));
```

```bash
./target/release/javryn basics.js
```

---

## 5. Async JavaScript

Javryn has a built-in event loop supporting Promises, `async`/`await`, and timers:

```javascript
// async-example.js
console.log("start");

Promise.resolve().then(() => {
    console.log("microtask");
});

setTimeout(() => {
    console.log("timer");
}, 50);

console.log("end");
```

```bash
./target/release/javryn async-example.js
```

Expected output:

```text
start
end
microtask
timer
```

The event loop processes microtasks before timers, matching standard JavaScript behavior.

---

## 6. Using Workers

Workers run JavaScript in separate OS threads with fully isolated Boa engine contexts:

```javascript
// main.js
const worker = new Worker("./worker-script.js");

worker.onmessage = (event) => {
    console.log("result:", event.data);
    worker.terminate();
};

worker.postMessage(21);
```

```javascript
// worker-script.js
onmessage = (event) => {
    const result = event.data * 2;
    postMessage(result);
};
```

Workers exchange data via `postMessage` / `onmessage`. Data is copied (not shared) between threads.

---

## 7. Explicit Parallelism

`parallel.map` distributes array processing across worker threads:

```javascript
// parallel-example.js
const values = [1, 2, 3, 4];

async function main() {
    const results = await parallel.map(values, "x => x * x");
    console.log("Results:", results);
}

main();
```

```bash
./target/release/javryn parallel-example.js
```

Expected output:

```text
Results: [ 1, 4, 9, 16 ]
```

**Key points:**
- The callback is passed as a **string** (e.g., `"x => x * x"`) so it can be serialized to worker threads
- Each element is processed in its own isolated Boa context
- Results are returned in the original input order
- `parallel.map` returns a `Promise` — use `await` or `.then()`

For CPU-intensive work, this is where parallelism shines:

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
    console.log("Fibonacci results:", results);
}

main();
```

---

## 8. Automatic Parallelization

Javryn can analyze JavaScript functions to determine if they're safe to parallelize:

```javascript
// auto-example.js
parallel.auto(() => {
    const numbers = [1, 2, 3, 4, 5];
    // Analyzer determines this is pure and safe
});
```

Run with the `--auto-parallel` flag:

```bash
./target/release/javryn --auto-parallel auto-example.js
```

The static analyzer checks for side effects, mutations, and dependencies. If the function is **provably safe**, it may be parallelized. If **uncertain or unsafe**, it falls back to sequential execution. There is never a correctness risk.

---

## 9. Diagnostics

See runtime execution metrics with `--diagnostics`:

```bash
./target/release/javryn --diagnostics examples/parallel-map.js
```

Example output:

```text
Parallel map result: [ 1, 4, 9, 16 ]

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

---

## 10. Where to Go Next

| Topic | Document |
|---|---|
| Full CLI reference | [docs/CLI.md](CLI.md) |
| Parallelism deep-dive | [docs/PARALLELISM.md](PARALLELISM.md) |
| Automatic parallelization | [docs/AUTOMATIC_PARALLELIZATION.md](AUTOMATIC_PARALLELIZATION.md) |
| Runtime architecture | [docs/ARCHITECTURE.md](ARCHITECTURE.md) |
| JavaScript compatibility | [docs/JAVASCRIPT_COMPATIBILITY.md](JAVASCRIPT_COMPATIBILITY.md) |
| Security model | [docs/SECURITY.md](SECURITY.md) |
| Performance benchmarks | [docs/PERFORMANCE.md](PERFORMANCE.md) |
| Contributing | [CONTRIBUTING.md](../CONTRIBUTING.md) |
