//! Mirror list loading utilities.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Errors that can occur while loading mirror lists.
#[derive(Debug)]
pub enum MirrorError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl std::fmt::Display for MirrorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirrorError::Io(e) => write!(f, "IO error: {e}"),
            MirrorError::Parse(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for MirrorError {}

impl From<std::io::Error> for MirrorError {
    fn from(e: std::io::Error) -> Self {
        MirrorError::Io(e)
    }
}

impl From<serde_json::Error> for MirrorError {
    fn from(e: serde_json::Error) -> Self {
        MirrorError::Parse(e)
    }
}

/// Supported package managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    PyPi,
    Npm,
}

impl PackageManager {
    /// Parses a package manager name from a string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pypi" | "pip" => Some(PackageManager::PyPi),
            "npm" => Some(PackageManager::Npm),
            _ => None,
        }
    }

    /// Returns the relative file path for this package manager's mirror list
    /// within the configured registry directory.
    pub fn data_file_name(&self) -> &'static str {
        match self {
            PackageManager::PyPi => "pypi.json",
            PackageManager::Npm => "npm.json",
        }
    }

    /// Returns the on-disk file path for this package manager's mirror list,
    /// resolved from the configured registry directory.
    pub fn data_file(&self) -> PathBuf {
        data_dir().join(self.data_file_name())
    }

    /// Returns the display name of the package manager.
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::PyPi => "pypi",
            PackageManager::Npm => "npm",
        }
    }
}

/// A sample package to download during benchmarking, together with the
/// list of mirror URLs to test it against.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Name of the package that will actually be downloaded from each
    /// mirror when benchmarking (e.g. `"requests"` for PyPI, `"lodash"`
    /// for npm).
    pub package: String,
    /// Mirror base URLs to benchmark.
    pub mirrors: Vec<String>,
}

/// Returns the directory containing the registry configuration files.
///
/// Resolution order:
/// 1. `MIRROR_DATA_DIR` environment variable (if set).
/// 2. `./data`.
fn data_dir() -> PathBuf {
    std::env::var("MIRROR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"))
}

#[cfg(test)]
mod tests {
    use super::data_dir;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct DataDirEnv(Option<std::ffi::OsString>);

    impl DataDirEnv {
        fn set(value: impl Into<std::ffi::OsString>) -> Self {
            let previous = std::env::var_os("MIRROR_DATA_DIR");
            let value = value.into();
            unsafe { std::env::set_var("MIRROR_DATA_DIR", value) };
            Self(previous)
        }

        fn remove() -> Self {
            let previous = std::env::var_os("MIRROR_DATA_DIR");
            unsafe { std::env::remove_var("MIRROR_DATA_DIR") };
            Self(previous)
        }
    }

    impl Drop for DataDirEnv {
        fn drop(&mut self) {
            match self.0.take() {
                Some(value) => unsafe { std::env::set_var("MIRROR_DATA_DIR", value) },
                None => unsafe { std::env::remove_var("MIRROR_DATA_DIR") },
            }
        }
    }

    #[test]
    fn data_dir_defaults_to_system_registry_directory() {
        let _lock = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = DataDirEnv::remove();

        assert_eq!(data_dir(), PathBuf::from("./data"));
    }

    #[test]
    fn data_dir_uses_mirror_data_dir_when_set() {
        let _lock = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = DataDirEnv::set("/custom/registries");

        assert_eq!(data_dir(), PathBuf::from("/custom/registries"));
    }
}

/// Loads the mirror configuration (sample package + mirror URLs) for the
/// given package manager.
pub fn load_mirrors(pm: PackageManager) -> Result<MirrorConfig, MirrorError> {
    let path = pm.data_file();
    load_mirrors_from(&path)
}

/// Loads a mirror configuration from an arbitrary JSON file path.
pub fn load_mirrors_from(path: &Path) -> Result<MirrorConfig, MirrorError> {
    let content = fs::read_to_string(path)?;
    let config: MirrorConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Appends a mirror URL to the given package manager's mirror list and
/// persists the updated configuration back to its data file.
pub fn add_mirror(pm: PackageManager, mirror: &str) -> Result<(), MirrorError> {
    let path = pm.data_file();
    let mut config = load_mirrors_from(&path)?;
    config.mirrors.push(mirror.to_string());
    let content = serde_json::to_string_pretty(&config)?;
    fs::write(&path, content)?;
    Ok(())
}
