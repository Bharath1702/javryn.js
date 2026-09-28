# Javryn Installation & Setup Guide (V1.0)

## Prerequisites

* **Rust Toolchain**: Rust 1.85+ (`cargo`, `rustc`).
* **Operating System**: Windows 10/11 or Linux (x86_64).

---

## 1. Building from Source

```bash
# Clone the repository
git clone https://github.com/javryn/javryn.git
cd javryn

# Build optimized release binary
cargo build --release

# Verify executable version
./target/release/javryn --version
```

---

## 2. Running Example Workloads

```bash
# Execute basic hello world script
./target/release/javryn examples/hello.js

# Execute parallel map workload
./target/release/javryn examples/parallel-map.js

# Execute parallel map workload with real-time operational diagnostics
./target/release/javryn --diagnostics examples/parallel-map.js

# Execute automatic parallelization workload
./target/release/javryn --auto-parallel examples/parallel-cpu.js
```
