use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use dotfilesvault::Config;

#[test]
fn test_config_default() {
    // Default config should use home directory
    let config = Config::default();

    // Check if home directory is set correctly
    let home_dir = dirs::home_dir().expect("Failed to find home directory");
    assert_eq!(config.home_dir, home_dir);

    // Check if vault directory is set correctly
    let expected_vault_dir = home_dir.join("dotfilesvault");
    assert_eq!(config.vault_dir, expected_vault_dir);
}

#[test]
fn test_config_new() {
    // Create custom paths
    let home_dir = PathBuf::from("/custom/home");
    let vault_dir = PathBuf::from("/custom/vault");

    // Create custom config
    let config = Config::new(vault_dir.clone(), home_dir.clone());

    // Check if paths are set correctly
    assert_eq!(config.home_dir, home_dir);
    assert_eq!(config.vault_dir, vault_dir);
}

#[test]
fn test_init_vault_dir() {
    // Create temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let vault_dir = temp_dir.path().join("vault");
    let home_dir = temp_dir.path().join("home");

    // Create config with temporary directories
    let config = Config::new(vault_dir.clone(), home_dir);

    // Initialize vault directory
    config.init_vault_dir().unwrap();

    // Check if vault directory was created
    assert!(vault_dir.exists());
    assert!(vault_dir.is_dir());
}

#[test]
fn test_init_vault_dir_already_exists() {
    // Create temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let vault_dir = temp_dir.path().join("vault");
    let home_dir = temp_dir.path().join("home");

    // Create vault directory before initializing
    fs::create_dir_all(&vault_dir).unwrap();

    // Create config with temporary directories
    let config = Config::new(vault_dir.clone(), home_dir);

    // Initialize vault directory (should not fail)
    config.init_vault_dir().unwrap();

    // Check if vault directory still exists
    assert!(vault_dir.exists());
    assert!(vault_dir.is_dir());
}
