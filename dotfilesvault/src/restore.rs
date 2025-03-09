use anyhow::Result;
use log::{debug, info};
use std::fs;
use std::path::{Path, PathBuf};

use crate::backup::Dotfile;
use crate::{Config, DotfilesError, is_dotfile};

/// Restore a dotfile from the vault to the home directory
pub fn restore_dotfile(dotfile: &Dotfile) -> Result<(), DotfilesError> {
    // Check if the file exists in the vault
    if !dotfile.vault_path.exists() {
        return Err(DotfilesError::DotfileNotFound(
            dotfile.original_path.to_string_lossy().to_string(),
        ));
    }

    // Create parent directories if they don't exist
    if let Some(parent) = dotfile.original_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file from the vault to the original location
    fs::copy(&dotfile.vault_path, &dotfile.original_path)?;

    info!("Restored: {:?}", dotfile.original_path);

    Ok(())
}

/// Restore a specific dotfile by path
pub fn restore_specific_dotfile(config: &Config, file_path: &str) -> Result<(), DotfilesError> {
    let path = Path::new(file_path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        config.home_dir.join(path)
    };

    if !is_dotfile(&path) {
        debug!("Skipping non-dotfile: {:?}", path);
        return Ok(());
    }

    let dotfile = Dotfile::new(path, config);

    restore_dotfile(&dotfile)
}

/// List all backed up dotfiles
pub fn list_backed_up_dotfiles(config: &Config) -> Result<Vec<PathBuf>, DotfilesError> {
    if !config.vault_dir.exists() {
        return Err(DotfilesError::NoDotfilesVaultDir);
    }

    let mut backed_up_files = Vec::new();

    // Walk through the vault directory
    for entry in walkdir::WalkDir::new(&config.vault_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only include files
        if path.is_file() {
            // Get the relative path from the vault directory
            if let Ok(relative_path) = path.strip_prefix(&config.vault_dir) {
                backed_up_files.push(relative_path.to_path_buf());
            }
        }
    }

    Ok(backed_up_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_env() -> (Config, TempDir, TempDir) {
        // Create temporary directories for testing
        let home_dir = TempDir::new().unwrap();
        let vault_dir = TempDir::new().unwrap();

        // Create a test config
        let config = Config::new(
            vault_dir.path().to_path_buf(),
            home_dir.path().to_path_buf(),
        );

        // Create the vault directory
        fs::create_dir_all(&config.vault_dir).unwrap();

        // Create a test dotfile in the vault
        let vault_dotfile_path = vault_dir.path().join(".testrc");
        let mut file = File::create(&vault_dotfile_path).unwrap();
        writeln!(file, "test content").unwrap();

        (config, home_dir, vault_dir)
    }

    #[test]
    fn test_restore_dotfile() {
        let (config, home_dir, _vault_dir) = setup_test_env();

        // Create a dotfile object
        let original_path = home_dir.path().join(".testrc");
        let dotfile = Dotfile::new(original_path.clone(), &config);

        // Manually create the vault file for testing
        if let Some(parent) = dotfile.vault_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(&dotfile.vault_path).unwrap();
        writeln!(file, "test content").unwrap();

        // Restore the dotfile
        restore_dotfile(&dotfile).unwrap();

        // Check if the file was restored
        assert!(original_path.exists());

        // Check the content
        let content = fs::read_to_string(original_path).unwrap();
        assert!(content.contains("test content"));
    }

    #[test]
    fn test_list_backed_up_dotfiles() {
        let (config, _home_dir, vault_dir) = setup_test_env();

        // Create some test dotfiles in the vault
        let dotfile1 = vault_dir.path().join(".bashrc");
        let dotfile2 = vault_dir.path().join(".vimrc");

        File::create(&dotfile1).unwrap();
        File::create(&dotfile2).unwrap();

        // List backed up dotfiles
        let backed_up = list_backed_up_dotfiles(&config).unwrap();

        // Should find at least the two dotfiles we created
        assert!(backed_up.len() >= 2);

        // Check if our test dotfiles are in the list
        let has_bashrc = backed_up
            .iter()
            .any(|p| p.to_string_lossy().contains(".bashrc"));
        let has_vimrc = backed_up
            .iter()
            .any(|p| p.to_string_lossy().contains(".vimrc"));

        assert!(has_bashrc);
        assert!(has_vimrc);
    }
}
