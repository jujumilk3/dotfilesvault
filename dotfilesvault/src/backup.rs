use anyhow::Result;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{Config, DotfilesError, is_dotfile};

/// Represents a dotfile to be backed up
#[derive(Debug, Clone)]
pub struct Dotfile {
    /// Original path in the home directory
    pub original_path: PathBuf,

    /// Path in the vault directory
    pub vault_path: PathBuf,
}

impl Dotfile {
    /// Create a new Dotfile instance
    pub fn new(original_path: PathBuf, config: &Config) -> Self {
        // Calculate the relative path from the home directory
        let relative_path = original_path
            .strip_prefix(&config.home_dir)
            .unwrap_or(&original_path);

        // Create the vault path
        let vault_path = config.vault_dir.join(relative_path);

        Self {
            original_path,
            vault_path,
        }
    }
}

/// Find all dotfiles in the home directory
pub fn find_dotfiles(config: &Config) -> Result<Vec<Dotfile>, DotfilesError> {
    let mut dotfiles = Vec::new();

    // Walk through the home directory
    for entry in WalkDir::new(&config.home_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip the dotfilesvault directory itself
        if path.starts_with(&config.vault_dir) {
            continue;
        }

        // Check if it's a dotfile
        if is_dotfile(path) && path.is_file() {
            let dotfile = Dotfile::new(path.to_path_buf(), config);
            dotfiles.push(dotfile);
        }
    }

    Ok(dotfiles)
}

/// Backup a single dotfile
pub fn backup_dotfile(dotfile: &Dotfile) -> Result<(), DotfilesError> {
    // Create parent directories if they don't exist
    if let Some(parent) = dotfile.vault_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file
    fs::copy(&dotfile.original_path, &dotfile.vault_path)?;

    info!("Backed up: {:?}", dotfile.original_path);

    Ok(())
}

/// Backup all dotfiles
pub fn backup_all_dotfiles(config: &Config) -> Result<(), DotfilesError> {
    // Initialize the vault directory
    config.init_vault_dir()?;

    // Find all dotfiles
    let dotfiles = find_dotfiles(config)?;

    debug!("Found {} dotfiles", dotfiles.len());

    // Backup each dotfile
    for dotfile in dotfiles {
        backup_dotfile(&dotfile)?;
    }

    info!("Backup completed successfully");

    Ok(())
}

/// Backup specific dotfiles
pub fn backup_specific_dotfiles(config: &Config, files: &[String]) -> Result<(), DotfilesError> {
    // Initialize the vault directory
    config.init_vault_dir()?;

    for file_str in files {
        let path = Path::new(file_str);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            config.home_dir.join(path)
        };

        if !path.exists() {
            return Err(DotfilesError::DotfileNotFound(file_str.clone()));
        }

        if !is_dotfile(&path) {
            debug!("Skipping non-dotfile: {:?}", path);
            continue;
        }

        let dotfile = Dotfile::new(path, config);
        backup_dotfile(&dotfile)?;
    }

    info!("Backup of specific files completed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn setup_test_env() -> (Config, TempDir) {
        // Create temporary directories for testing
        let home_dir = TempDir::new().unwrap();
        let vault_dir = TempDir::new().unwrap();

        // Create a test dotfile
        let dotfile_path = home_dir.path().join(".testrc");
        File::create(&dotfile_path).unwrap();

        // Create a non-dotfile
        let regular_file_path = home_dir.path().join("regular.txt");
        File::create(&regular_file_path).unwrap();

        // Create a test config
        let config = Config::new(
            vault_dir.path().to_path_buf(),
            home_dir.path().to_path_buf(),
        );

        (config, home_dir)
    }

    #[test]
    fn test_find_dotfiles() {
        let (config, _home_dir) = setup_test_env();

        let dotfiles = find_dotfiles(&config).unwrap();

        // Should find exactly one dotfile
        assert_eq!(dotfiles.len(), 1);
        assert!(
            dotfiles[0]
                .original_path
                .to_str()
                .unwrap()
                .contains(".testrc")
        );
    }

    #[test]
    fn test_backup_dotfile() {
        let (config, _home_dir) = setup_test_env();

        // Initialize the vault directory
        config.init_vault_dir().unwrap();

        // Find the test dotfile
        let dotfiles = find_dotfiles(&config).unwrap();
        assert_eq!(dotfiles.len(), 1);

        // Backup the dotfile
        backup_dotfile(&dotfiles[0]).unwrap();

        // Check if the file was backed up
        assert!(dotfiles[0].vault_path.exists());
    }
}
