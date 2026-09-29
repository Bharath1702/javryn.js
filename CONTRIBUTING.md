# Contributing to Javryn

Thank you for your interest in contributing to Javryn! This document covers everything you need to get started.

---

## Development Prerequisites

- **Rust Toolchain**: Rust 1.85+ with `cargo`, `rustfmt`, and `clippy`
- **Git**: For version control

Install Rust via [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy
```

---

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/Bharath1702/javryn.js.git
cd javryn.js
```

### Build

```bash
# Debug build (faster compilation, slower runtime)
cargo build --workspace

# Release build (slower compilation, optimized runtime)
cargo build --workspace --release
```

### Test

```bash
# Run all workspace tests
cargo test --workspace
```

### Format

```bash
# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

### Lint

```bash
# Run Clippy with strict warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Full Quality Check

Before submitting a pull request, run the complete quality gate:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

---

## Project Structure

```text
javryn.js/
├── crates/
│   ├── javryn-cli/        # CLI entry point (argument parsing, logging, exit codes)
│   ├── javryn-core/       # Domain types, config, error hierarchy
│   └── javryn-runtime/    # Runtime engine, event loop, workers, scheduler, parallel API
├── examples/              # Runnable JavaScript examples
├── tests/                 # Integration tests
├── docs/                  # Documentation
└── Cargo.toml             # Workspace configuration
```

---

## Adding Tests

- **Unit tests**: Place in the same file as the code being tested, inside a `#[cfg(test)] mod tests` block
- **Integration tests**: Place in `tests/` directory
- **JavaScript test fixtures**: Place in `tests/javascript/`

Run a specific test:

```bash
cargo test -p javryn-runtime test_name
```

---

## Adding Examples

Place runnable JavaScript examples in the `examples/` directory. Every example should:

1. Be self-contained and executable with `javryn examples/your-example.js`
2. Have a descriptive filename
3. Include a brief comment explaining what it demonstrates

---

## Documentation

- Update `docs/` if your change affects user-facing behavior
- Update `README.md` if your change affects the quick start or feature list
- Keep inline code comments accurate if modifying implementation

---

## Pull Request Guidelines

1. **One concern per PR**: Keep pull requests focused on a single change
2. **Pass quality gate**: Ensure `cargo fmt`, `cargo clippy`, and `cargo test` all pass
3. **No broken examples**: Verify that all `examples/*.js` files still work
4. **Describe the change**: Explain what changed and why in the PR description
5. **No secrets**: Do not commit API keys, tokens, or credentials

---

## Code Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `tracing` for logging (not `println!` in library code)
- Errors should use the `RuntimeError` hierarchy from `javryn-core`
- Exit codes must follow the documented taxonomy (0–6)

---

## Questions?

Open an issue on GitHub if you have questions about the codebase or need guidance on a contribution.
