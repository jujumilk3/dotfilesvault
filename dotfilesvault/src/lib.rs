use anyhow::Result;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod backup;
pub mod history;
pub mod restore;
pub mod utils;

/// Errors that can occur in the dotfilesvault application
#[derive(Error, Debug)]
pub enum DotfilesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to find home directory")]
    NoHomeDir,

    #[error("Failed to find dotfilesvault directory")]
    NoDotfilesVaultDir,

    #[error("Dotfile not found: {0}")]
    DotfileNotFound(String),

    #[error("Version not found for dotfile: {0}")]
    VersionNotFound(String),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
}

/// Configuration for the dotfilesvault application
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the dotfilesvault directory
    pub vault_dir: PathBuf,

    /// Path to the home directory
    pub home_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().expect("Failed to find home directory");
        let vault_dir = home_dir.join("dotfilesvault");

        Self {
            vault_dir,
            home_dir,
        }
    }
}

impl Config {
    /// Create a new configuration with custom paths
    pub fn new(vault_dir: PathBuf, home_dir: PathBuf) -> Self {
        Self {
            vault_dir,
            home_dir,
        }
    }

    /// Initialize the dotfilesvault directory
    pub fn init_vault_dir(&self) -> Result<(), DotfilesError> {
        if !self.vault_dir.exists() {
            std::fs::create_dir_all(&self.vault_dir)?;
        }

        Ok(())
    }
}

/// Check if a file is a dotfile
pub fn is_dotfile<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dotfile() {
        assert!(is_dotfile(".bashrc"));
        assert!(is_dotfile("/home/user/.vimrc"));
        assert!(is_dotfile(Path::new("/home/user/.config")));

        assert!(!is_dotfile("bashrc"));
        assert!(!is_dotfile("/home/user/documents"));
        assert!(!is_dotfile(Path::new("/home/user/file.txt")));
    }
}
