# Javryn Installation Guide

## Prerequisites

- **Rust Toolchain**: Rust 1.85+ with `cargo` and `rustc`. Install via [rustup.rs](https://rustup.rs/).
- **Operating System**: Windows 10/11 (x86_64) or Linux (x86_64). macOS may work but is not yet verified.
- **Git**: Required for cloning the repository.

---

## Build from Source

This is currently the only supported installation method.

```bash
# 1. Clone the repository
git clone https://github.com/Bharath1702/javryn.js.git
cd javryn.js

# 2. Build the optimized release binary
cargo build --release

# 3. Verify the build
./target/release/javryn --version
# javryn 1.0.0
```

The compiled binary is located at:
- **Linux**: `target/release/javryn`
- **Windows**: `target\release\javryn.exe`

### Adding to PATH

To run `javryn` from anywhere, add it to your system `PATH`:

**Linux / macOS:**
```bash
# Copy to a directory on your PATH
sudo cp target/release/javryn /usr/local/bin/

# Or add target/release to PATH in your shell profile
export PATH="$PATH:$(pwd)/target/release"
```

**Windows (PowerShell):**
```powershell
# Add to current session
$env:PATH += ";$(Get-Location)\target\release"

# Or copy to a permanent location on your PATH
Copy-Item .\target\release\javryn.exe C:\tools\
```

---

## Prebuilt Binaries

Prebuilt release binaries are **not yet available**. This is planned for a future release.

For now, building from source is required.

---

## Development Setup

For contributing to Javryn, install additional development tools:

```bash
# Install Rust nightly formatter (optional, for latest formatting rules)
rustup component add rustfmt clippy

# Verify development toolchain
cargo fmt --version
cargo clippy --version

# Full development build cycle
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines.

---

## Running Examples

After building, verify Javryn works with the included examples:

```bash
# Hello World
./target/release/javryn examples/hello.js

# Async runtime
./target/release/javryn examples/async.js

# Parallel map
./target/release/javryn examples/parallel-map.js

# CPU-intensive parallel workload
./target/release/javryn examples/parallel-cpu.js

# Diagnostics
./target/release/javryn --diagnostics examples/parallel-map.js

# Automatic parallelization
./target/release/javryn --auto-parallel examples/auto-parallel-basic.js
```

---

## Troubleshooting

### Build fails with Rust version error

Javryn requires Rust 1.85 or later. Check your version:

```bash
rustc --version
```

Update if needed:

```bash
rustup update
```

### `boa_engine` compilation takes a long time

The first build compiles all dependencies including the Boa JavaScript engine. Subsequent builds are incremental and much faster. Release builds (`--release`) take longer than debug builds but produce a significantly faster binary.

### Permission denied on Linux

If the binary cannot be executed:

```bash
chmod +x target/release/javryn
```
