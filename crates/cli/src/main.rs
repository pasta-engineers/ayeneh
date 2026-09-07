//! Mirror Benchmark CLI entry point.

use clap::Parser;
use ayeneh_cli::{Cli, run_command};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = if cli.command.is_none() {
        eprintln!("No command provided. Run with --help for usage.");
        std::process::exit(2);
    } else {
        run_command(cli).await
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
