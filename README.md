# Javryn

**A high-performance, parallel JavaScript runtime.**

## Current Status: V0.4 — Independent JavaScript Workers

Javryn V0.4 introduces multi-context JavaScript execution with independent worker threads, isolated VM contexts, message passing (`Worker`, `postMessage`, `onmessage`), owned data serialization (`JsMessage`), and failure isolation.

```bash
javryn examples/worker-message.js
# Output:
# result: 42
```

## Features in V0.4

- **Worker Architecture**: Independent OS threads (`std::thread`) with dedicated `boa_engine::Context` per worker.
- **Message Passing**: Bi-directional message channels (`postMessage`, `onmessage`, `onerror`).
- **Data Serialization**: `JsMessage` owned type mapping numbers, strings, booleans, arrays, and objects.
- **Isolation**: Isolated JS global scope, VM heap, event loop, timer queue, and exception handling.
- **Graceful Lifecycle & Shutdown**: Deterministic thread joining (`terminate()`) without detached background threads or memory leaks.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/javryn/javryn.git
cd javryn

# Build release binary
cargo build --release

# Run a worker script
./target/release/javryn examples/worker-message.js
```

## Usage Examples

```bash
# Basic Worker Creation
javryn examples/worker-basic.js

# Worker Message Passing
javryn examples/worker-message.js

# Independent Worker CPU Computations
javryn examples/worker-cpu.js
```

### `examples/worker-message.js`
```javascript
const worker = new Worker("./examples/workers/message.js");

worker.onmessage = (event) => {
    console.log("result:", event.data);
};

worker.postMessage(21);
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic runtime failure |
| 2 | Invalid input / JavaScript syntax error |
| 3 | Configuration failure |
| 4 | Engine initialization failure |
| 5 | JavaScript execution error / unhandled exception |
| 6 | Shutdown failure |

## Development

```bash
# Check compilation
cargo check --workspace

# Run all unit, integration, & compatibility tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run Clippy lints
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Roadmap

| Version | Focus |
|---------|-------|
| V0.1 | Runtime Foundation ✅ |
| V0.2 | JavaScript Engine Integration ✅ |
| V0.3 | Async Runtime & Event Loop ✅ |
| V0.4 | Independent Workers ✅ ← **current** |
| V0.5 | Explicit Parallelism |
| V0.6 | Memory Management |
| V0.7 | Scheduler |
| V0.8 | Automatic Parallelization |
| V0.9 | Production Hardening |
| V1.0 | Production Release |

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## License

MIT — see [LICENSE](LICENSE).
