//! Python package installation with uv through a PyPI mirror with fallback.

use crate::mirror::{load_mirrors, Registry};
use std::process::Stdio;
use tokio::process::Command;

/// Resolves the uv command
async fn resolve_uv() -> Result<&'static str, Box<dyn std::error::Error>> {
    if Command::new("uv").arg("--version").output().await.is_ok() {
        return Ok("uv");
    }
    Err("uv not found on PATH.".into())
}

/// Installs packages via the first
/// configured PyPI mirror that succeeds. uv's output streams straight to
/// the terminal; a failed mirror falls through to the next one.
pub async fn add(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_mirrors(Registry::PyPi)?;
    let uv = resolve_uv().await?;

    let mut last_status = String::from("No mirrors configured");
    for mirror in &config.mirrors {
        println!("Installing via {mirror}...");

        let status = Command::new(uv)
            .args(["add", "--index-url"])
            .arg(mirror)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        if status.success() {
            println!("Installed successfully via {mirror}.");
            return Ok(());
        }

        last_status = format!("Mirror {mirror} failed");
        eprintln!("{last_status}");
    }

    Err(format!("All mirrors failed, last error: {last_status}").into())
}
