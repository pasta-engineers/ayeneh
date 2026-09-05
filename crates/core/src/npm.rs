//! npm package installation through a registry mirror with fallback.

use crate::mirror::{load_mirrors, PackageManager};
use std::process::Stdio;
use tokio::process::Command;

/// Installs `args` (a package name, `--save`, etc.) via the first
/// configured npm mirror that succeeds. npm's output streams straight to
/// the terminal; a failed mirror falls through to the next one.
pub async fn install(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_mirrors(PackageManager::Npm)?;

    let mut last_status = String::from("no mirrors configured");
    for mirror in &config.mirrors {
        println!("Installing via {mirror}...");

        let status = Command::new("npm")
            .arg("install")
            .arg("--registry")
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

        last_status = format!("mirror {mirror} failed (exit code {:?})", status.code());
        eprintln!("{last_status}");
    }

    Err(format!("All mirrors failed, last error: {last_status}").into())
}
