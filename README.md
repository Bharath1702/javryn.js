# Javryn

**A high-performance, parallel JavaScript runtime.**

> ⚠️ **Javryn V0.1 is a runtime foundation and does not execute JavaScript yet.**
> JavaScript engine integration is planned for V0.2.

## What is Javryn?

Javryn is a JavaScript runtime designed for high-performance, parallel execution across multiple CPU cores. It uses an intelligent scheduler and worker architecture to automatically distribute work.

**Current Status: V0.1 — Runtime Foundation**

V0.1 establishes the core runtime infrastructure:
- CLI argument parsing
- Script validation and metadata extraction
- Runtime lifecycle management
- Structured error handling with documented exit codes
- Cross-platform Ctrl+C handling
- Structured logging/diagnostics

## V0.1 Limitations

V0.1 does **not** include:
- JavaScript execution
- Module loading
- Async runtime
- Worker threads
- Network APIs
- Package management

These will be added in future versions (see [Roadmap](docs/ROADMAP.md)).

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/javryn/javryn.git
cd javryn

# Build in release mode
cargo build --release

# The binary is at target/release/javryn (or javryn.exe on Windows)
```

### Requirements

- Rust 1.85+ (2024 edition)
- Cargo

## Usage

```bash
# Show help
javryn --help

# Show version
javryn --version

# Run a script (V0.1: validates but does not execute)
javryn app.js

# Run with debug logging
javryn --debug app.js

# Run with verbose output
javryn --verbose app.js

# Run in quiet mode (errors only)
javryn --quiet app.js
```

### CLI Options

| Option | Description |
|--------|-------------|
| `<SCRIPT>` | Path to the JavaScript file (.js or .mjs) |
| `--debug` | Enable debug-level diagnostic logging |
| `--verbose` | Enable verbose informational output |
| `--quiet` | Suppress all non-error output |
| `--version` | Print version information |
| `--help` | Print help information |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Generic runtime failure |
| 2 | Invalid input (missing/invalid script, bad arguments) |
| 3 | Configuration failure |
| 4 | Initialization failure |
| 5 | Runtime execution failure |
| 6 | Shutdown failure |

## Architecture

```
javryn-cli          → CLI parsing, user output, exit codes
  ↓
javryn-runtime      → Lifecycle, validation, execution context
  ↓
javryn-core         → Shared types, errors, configuration
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full details.

## Development

```bash
# Check compilation
cargo check --workspace

# Run all tests
cargo test --workspace

# Run with formatting check
cargo fmt --all -- --check

# Run Clippy lints
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build release
cargo build --release
```

### Project Structure

```
javryn/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── javryn-cli/         # Binary crate (CLI entry point)
│   ├── javryn-runtime/     # Runtime lifecycle library
│   └── javryn-core/        # Shared domain types
├── examples/               # Example scripts
├── docs/                   # Architecture documentation
└── tests/                  # Integration tests
```

## Testing

```bash
# Run all tests (unit + integration)
cargo test --workspace

# Run only unit tests
cargo test --workspace --lib

# Run only integration tests
cargo test --workspace --test '*'

# Run a specific test
cargo test --workspace test_name
```

## Roadmap

| Version | Focus |
|---------|-------|
| V0.1 | Runtime Foundation ← **current** |
| V0.2 | JavaScript Engine Integration |
| V0.3 | Async Runtime |
| V0.4 | Worker Threads |
| V0.5 | Explicit Parallelism |
| V0.6 | Memory Management |
| V0.7 | Scheduler |
| V0.8 | Automatic Parallelization |
| V0.9 | Production Hardening |
| V1.0 | Production Release |

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## License

MIT — see [LICENSE](LICENSE).
