//! # Javryn CLI
//!
//! The command-line entry point for the Javryn JavaScript runtime.
//!
//! This binary is responsible for:
//! - Parsing CLI arguments via `clap`
//! - Initializing structured logging via `tracing`
//! - Invoking the runtime lifecycle
//! - Converting runtime errors into human-readable messages and exit codes
//!
//! It contains **no runtime business logic**.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use tracing_subscriber::EnvFilter;

use javryn_core::{ExitCode, RuntimeConfig, RuntimeError, RuntimeMode};
use javryn_runtime::{Runtime, validate_script};

/// Javryn — A high-performance, parallel JavaScript runtime.
///
/// V0.1: Runtime Foundation (JavaScript execution not yet implemented).
#[derive(Parser, Debug)]
#[command(
    name = "javryn",
    version,
    about = "Javryn — A high-performance JavaScript runtime",
    long_about = "Javryn is a high-performance, parallel JavaScript runtime.\n\n\
                  V0.1 is the runtime foundation. JavaScript execution will be\n\
                  available in V0.2."
)]
struct Cli {
    /// Path to the JavaScript file to run.
    #[arg(value_name = "SCRIPT")]
    script: PathBuf,

    /// Enable debug-level diagnostic logging.
    #[arg(long, conflicts_with_all = &["verbose", "quiet"])]
    debug: bool,

    /// Enable verbose informational output.
    #[arg(long, conflicts_with_all = &["debug", "quiet"])]
    verbose: bool,

    /// Suppress all non-error output.
    #[arg(long, conflicts_with_all = &["debug", "verbose"])]
    quiet: bool,

    /// Enable automatic AST parallelization mode.
    #[arg(long = "auto-parallel")]
    auto_parallel: bool,
}

impl Cli {
    /// Determines the [`RuntimeMode`] from the parsed flags.
    fn runtime_mode(&self) -> RuntimeMode {
        if self.debug {
            RuntimeMode::Debug
        } else if self.verbose {
            RuntimeMode::Verbose
        } else if self.quiet {
            RuntimeMode::Quiet
        } else {
            RuntimeMode::Normal
        }
    }
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    // Initialize logging based on the selected mode.
    init_logging(cli.runtime_mode());

    // Set up Ctrl+C handler.
    let interrupted = Arc::new(AtomicBool::new(false));
    setup_ctrlc_handler(Arc::clone(&interrupted));

    // Run the main lifecycle and convert the result to an exit code.
    match run(&cli, &interrupted) {
        Ok(()) => ExitCode::Success.into(),
        Err(err) => {
            let exit_code = err.exit_code();
            print_error(&err);
            tracing::debug!(exit_code = exit_code.as_i32(), "exiting with error");
            exit_code.into()
        }
    }
}

/// The main runtime lifecycle, extracted from `main` for testability.
fn run(cli: &Cli, interrupted: &AtomicBool) -> Result<(), RuntimeError> {
    tracing::debug!("Javryn V0.1 starting");

    // Step 1: Build configuration.
    let config = RuntimeConfig::builder()
        .script_path(cli.script.clone())
        .mode(cli.runtime_mode())
        .auto_parallel(cli.auto_parallel)
        .build()?;

    tracing::debug!(mode = %config.mode(), script = %config.script_path().display(), "configuration built");

    // Step 2: Validate the script.
    let script = validate_script(config.script_path())?;
    tracing::debug!(
        file = ?script.file_name(),
        size = script.metadata().file_size(),
        "script validated"
    );

    // Step 3: Create and run the runtime.
    let mut runtime = Runtime::new(config)?;

    // Check for interruption before running.
    if interrupted.load(Ordering::Relaxed) {
        tracing::info!("interrupted before execution");
        runtime.shutdown()?;
        return Ok(());
    }

    let result = runtime.run(&script)?;

    tracing::debug!(
        elapsed_ms = result.elapsed().as_millis(),
        "runtime execution complete"
    );

    // Step 4: Shutdown.
    runtime.shutdown()?;

    tracing::debug!("Javryn V0.1 finished");
    Ok(())
}

/// Initializes the `tracing` subscriber based on the runtime mode.
fn init_logging(mode: RuntimeMode) {
    let filter = match mode {
        RuntimeMode::Debug => EnvFilter::new("debug"),
        RuntimeMode::Verbose => EnvFilter::new("info"),
        RuntimeMode::Quiet => EnvFilter::new("error"),
        RuntimeMode::Normal => EnvFilter::new("warn"),
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .with_writer(std::io::stderr)
        .init();
}

/// Installs a cross-platform Ctrl+C handler.
fn setup_ctrlc_handler(interrupted: Arc<AtomicBool>) {
    if let Err(e) = ctrlc::set_handler(move || {
        interrupted.store(true, Ordering::Relaxed);
        eprintln!("\nJavryn: interrupted (Ctrl+C)");
    }) {
        tracing::warn!("failed to set Ctrl+C handler: {e}");
    }
}

/// Prints a human-readable error message to stderr.
fn print_error(err: &RuntimeError) {
    eprintln!("\nJavryn Error: {err}\n");
    eprintln!("Exit code: {}", err.exit_code().as_i32());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_script_only() {
        let cli = Cli::try_parse_from(["javryn", "app.js"]).unwrap();
        assert_eq!(cli.script, PathBuf::from("app.js"));
        assert!(!cli.debug);
        assert!(!cli.verbose);
        assert!(!cli.quiet);
        assert_eq!(cli.runtime_mode(), RuntimeMode::Normal);
    }

    #[test]
    fn cli_parses_debug_flag() {
        let cli = Cli::try_parse_from(["javryn", "--debug", "app.js"]).unwrap();
        assert!(cli.debug);
        assert_eq!(cli.runtime_mode(), RuntimeMode::Debug);
    }

    #[test]
    fn cli_parses_verbose_flag() {
        let cli = Cli::try_parse_from(["javryn", "--verbose", "app.js"]).unwrap();
        assert!(cli.verbose);
        assert_eq!(cli.runtime_mode(), RuntimeMode::Verbose);
    }

    #[test]
    fn cli_parses_quiet_flag() {
        let cli = Cli::try_parse_from(["javryn", "--quiet", "app.js"]).unwrap();
        assert!(cli.quiet);
        assert_eq!(cli.runtime_mode(), RuntimeMode::Quiet);
    }

    #[test]
    fn cli_rejects_debug_and_verbose() {
        let result = Cli::try_parse_from(["javryn", "--debug", "--verbose", "app.js"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_rejects_debug_and_quiet() {
        let result = Cli::try_parse_from(["javryn", "--debug", "--quiet", "app.js"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_rejects_verbose_and_quiet() {
        let result = Cli::try_parse_from(["javryn", "--verbose", "--quiet", "app.js"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_rejects_missing_script() {
        let result = Cli::try_parse_from(["javryn"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_script_after_flag() {
        let cli = Cli::try_parse_from(["javryn", "--debug", "path/to/script.js"]).unwrap();
        assert_eq!(cli.script, PathBuf::from("path/to/script.js"));
        assert!(cli.debug);
    }
}
