//! Mirror Benchmark CLI shared logic, exposed as a library so other crates
//! (e.g. the TUI) can reuse the same command handlers.

use clap::{Parser, Subcommand};
use ayeneh_core::benchmark::{benchmark_all, BenchmarkResult};
use ayeneh_core::mirror::{load_mirrors, PackageManager};
use ayeneh_core::report::Report;
use ayeneh_core::{npm, pip, scheduler};

/// Mirror Benchmark: benchmark package registry mirrors and find the fastest one.
#[derive(Parser)]
#[command(name = "ayeneh-cli", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a one-off benchmark for a package manager ("pypi" or "npm") and print results.
    Run {
        /// Which package manager to benchmark: "pypi" or "npm".
        package_manager: String,
    },
    /// Run benchmarks for pypi and npm and write a JSON report to reports/.
    Report,
    /// Continuously benchmark and save reports every hour.
    Schedule,
    /// Install Python packages via PyPI mirrors, falling back to the next mirror on failure.
    Pip {
        #[command(subcommand)]
        command: PipCommand,
    },
    /// Install npm packages via npm mirrors, falling back to the next mirror on failure.
    Npm {
        #[command(subcommand)]
        command: NpmCommand,
    },
}

#[derive(Subcommand)]
pub enum PipCommand {
    /// Install a package or a -r requirements file.
    Install {
        /// pip install arguments: package names, `-r FILE`, options.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum NpmCommand {
    /// Install one or more packages.
    Install {
        /// npm install arguments: package names, options.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Runs a single benchmark for the given package manager name and prints
/// the results to stdout.
pub async fn run_once(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pm = PackageManager::from_str(name)
        .ok_or_else(|| format!("Unknown package manager '{name}'. Use 'pypi' or 'npm'."))?;

    let config = load_mirrors(pm)?;
    println!(
        "Benchmarking {} mirrors for {} (package: {})...",
        config.mirrors.len(),
        pm.name(),
        config.package
    );

    let results = benchmark_all(pm, &config.package, &config.mirrors).await;
    print_results(&results);

    Ok(())
}

/// Prints benchmark results as a simple table to stdout.
pub fn print_results(results: &[BenchmarkResult]) {
    println!("{:<40} {:>10} {:>10}", "Mirror", "Avg(ms)", "Success");
    for r in results {
        let latency = if r.timed_out {
            "timeout".to_string()
        } else {
            r.average_latency_ms.to_string()
        };
        let success = format!("{:.0}%", r.success_rate);
        println!("{:<40} {:>10} {:>10}", r.name, latency, success);
    }

    if let Some(best) = results.iter().find(|r| !r.timed_out) {
        println!("\nFastest mirror: {}", best.name);
    } else {
        println!("\nNo mirrors were reachable.");
    }
}

/// Runs benchmarks for both pypi and npm and saves a JSON report for each.
pub async fn run_report() -> Result<(), Box<dyn std::error::Error>> {
    for pm in [PackageManager::PyPi, PackageManager::Npm] {
        let config = load_mirrors(pm)?;
        println!("Benchmarking {}...", pm.name());

        let results = benchmark_all(pm, &config.package, &config.mirrors).await;
        let report = Report::new(pm.name(), &results);
        let path = report.save()?;

        println!("Report saved to {}", path.display());
    }

    Ok(())
}

/// Executes a parsed [`Cli`] subcommand, printing results to stdout. Returns
/// `Ok(())` when a subcommand was handled and `Err(e)` on failure. Calling this
/// with `None` is a no-op, letting the caller decide what to do when no command
/// was provided.
pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Some(Commands::Run { package_manager }) => run_once(&package_manager).await,
        Some(Commands::Report) => run_report().await,
        Some(Commands::Schedule) => {
            scheduler::run().await;
            Ok(())
        }
        Some(Commands::Pip {
            command: PipCommand::Install { args },
        }) => pip::install(&args).await,
        Some(Commands::Npm {
            command: NpmCommand::Install { args },
        }) => npm::install(&args).await,
        None => Ok(()),
    }
}
