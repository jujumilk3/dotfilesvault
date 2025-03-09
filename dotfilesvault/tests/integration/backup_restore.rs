use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

use dotfilesvault::backup::{backup_all_dotfiles, find_dotfiles};
use dotfilesvault::history::{commit_changes, get_dotfile_history};
use dotfilesvault::restore::{list_backed_up_dotfiles, restore_specific_dotfile};
use dotfilesvault::{Config, is_dotfile};

/// Set up a test environment with dotfiles
fn setup_test_env() -> (Config, TempDir) {
    // Create temporary directories for testing
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().join("home");
    let vault_dir = temp_dir.path().join("dotfilesvault");

    // Create directories
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&vault_dir).unwrap();

    // Create test dotfiles
    let dotfiles = vec![
        ".bashrc",
        ".vimrc",
        ".gitconfig",
        ".config/app/settings.json",
    ];

    for dotfile in dotfiles {
        let path = home_dir.join(dotfile);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        // Create the file with some content
        let mut file = File::create(&path).unwrap();
        writeln!(file, "# Test content for {}", dotfile).unwrap();
    }

    // Create a non-dotfile
    let regular_file = home_dir.join("regular.txt");
    let mut file = File::create(regular_file).unwrap();
    writeln!(file, "This is not a dotfile").unwrap();

    // Create a test config
    let config = Config::new(vault_dir, home_dir);

    (config, temp_dir)
}

#[test]
fn test_full_backup_and_restore_flow() {
    // Set up test environment
    let (config, _temp_dir) = setup_test_env();

    // Step 1: Find dotfiles
    let dotfiles = find_dotfiles(&config).unwrap();

    // Verify we found the expected number of dotfiles
    assert_eq!(dotfiles.len(), 3);

    // Step 2: Backup all dotfiles
    backup_all_dotfiles(&config).unwrap();

    // Step 3: Commit changes
    let commit_id = commit_changes(&config, "Test backup").unwrap();
    assert!(!commit_id.is_empty());

    // Step 4: List backed up dotfiles
    let backed_up = list_backed_up_dotfiles(&config).unwrap();

    // We're not checking the exact number here because git might add additional files
    // Just make sure our dotfiles are included
    let has_bashrc = backed_up
        .iter()
        .any(|p| p.to_string_lossy().contains(".bashrc"));
    let has_vimrc = backed_up
        .iter()
        .any(|p| p.to_string_lossy().contains(".vimrc"));
    let has_gitconfig = backed_up
        .iter()
        .any(|p| p.to_string_lossy().contains(".gitconfig"));

    assert!(has_bashrc);
    assert!(has_vimrc);
    assert!(has_gitconfig);

    // Step 5: Modify a dotfile in the home directory
    let bashrc_path = config.home_dir.join(".bashrc");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&bashrc_path)
        .unwrap();
    writeln!(file, "# Modified content").unwrap();

    // Step 6: Restore the dotfile
    let bashrc_rel_path = ".bashrc";
    restore_specific_dotfile(&config, bashrc_rel_path).unwrap();

    // Step 7: Verify the content was restored
    let content = fs::read_to_string(&bashrc_path).unwrap();
    assert!(content.contains("# Test content for .bashrc"));

    // Step 8: Get history of the dotfile
    let history = get_dotfile_history(&config, bashrc_rel_path).unwrap();
    assert!(!history.is_empty());
}

#[test]
fn test_dotfile_detection() {
    // Set up test environment
    let (config, _temp_dir) = setup_test_env();

    // Test various paths
    assert!(is_dotfile(".bashrc"));
    assert!(is_dotfile(config.home_dir.join(".vimrc")));
    assert!(is_dotfile(PathBuf::from("/home/user/.config")));

    assert!(!is_dotfile("regular.txt"));
    assert!(!is_dotfile(config.home_dir.join("regular.txt")));
    assert!(!is_dotfile(PathBuf::from("/home/user/documents")));
}

#[test]
fn test_nested_dotfiles() {
    // Set up test environment
    let (config, _temp_dir) = setup_test_env();

    // Create a nested dotfile structure
    let nested_dir = config.home_dir.join(".config/nested");
    fs::create_dir_all(&nested_dir).unwrap();

    let nested_file = nested_dir.join(".nestedrc");
    let mut file = File::create(&nested_file).unwrap();
    writeln!(file, "# Nested dotfile content").unwrap();

    // Backup all dotfiles
    backup_all_dotfiles(&config).unwrap();

    // List backed up dotfiles
    let backed_up = list_backed_up_dotfiles(&config).unwrap();

    // Check if the nested dotfile was backed up
    let has_nested = backed_up
        .iter()
        .any(|p| p.to_string_lossy().contains(".config/nested/.nestedrc"));
    assert!(has_nested);
}
