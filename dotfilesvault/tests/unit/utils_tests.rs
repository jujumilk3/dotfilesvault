use std::path::PathBuf;
use tempfile::TempDir;

use dotfilesvault::utils::{expand_tilde, human_readable_size, is_in_home_dir, normalize_path};
use dotfilesvault::{Config, is_dotfile};

#[test]
fn test_expand_tilde_with_home_dir() {
    let home_dir = dirs::home_dir().unwrap();

    // Test with just tilde
    assert_eq!(expand_tilde("~"), home_dir);

    // Test with tilde and path
    assert_eq!(expand_tilde("~/Documents"), home_dir.join("Documents"));
    assert_eq!(
        expand_tilde("~/some/nested/path"),
        home_dir.join("some/nested/path")
    );
}

#[test]
fn test_expand_tilde_with_other_paths() {
    // Test with absolute path
    let abs_path = PathBuf::from("/absolute/path");
    assert_eq!(expand_tilde(&abs_path), abs_path);

    // Test with relative path
    let rel_path = PathBuf::from("relative/path");
    assert_eq!(expand_tilde(&rel_path), rel_path);

    // Test with path that contains tilde but doesn't start with it
    let path_with_tilde = PathBuf::from("path/with/~/tilde");
    assert_eq!(expand_tilde(&path_with_tilde), path_with_tilde);
}

#[test]
fn test_normalize_path() {
    // Create test config
    let temp_dir = TempDir::new().unwrap();
    let config = Config::new(temp_dir.path().join("vault"), temp_dir.path().join("home"));

    // Test with absolute path
    let abs_path = PathBuf::from("/absolute/path");
    assert_eq!(normalize_path(&abs_path, &config), abs_path);

    // Test with relative path
    let rel_path = PathBuf::from("relative/path");
    assert_eq!(
        normalize_path(&rel_path, &config),
        config.home_dir.join(&rel_path)
    );

    // Test with tilde path
    let tilde_path = "~/path";
    let expected = dirs::home_dir().unwrap().join("path");
    assert_eq!(normalize_path(tilde_path, &config), expected);
}

#[test]
fn test_human_readable_size() {
    // Test with bytes
    assert_eq!(human_readable_size(0), "0.00 B");
    assert_eq!(human_readable_size(500), "500.00 B");
    assert_eq!(human_readable_size(1023), "1023.00 B");

    // Test with kilobytes
    assert_eq!(human_readable_size(1024), "1.00 KB");
    assert_eq!(human_readable_size(1536), "1.50 KB");
    assert_eq!(human_readable_size(1024 * 1023), "1023.00 KB");

    // Test with megabytes
    assert_eq!(human_readable_size(1024 * 1024), "1.00 MB");
    assert_eq!(human_readable_size(1024 * 1024 * 10), "10.00 MB");

    // Test with gigabytes
    assert_eq!(human_readable_size(1024 * 1024 * 1024), "1.00 GB");

    // Test with terabytes
    assert_eq!(human_readable_size(1024 * 1024 * 1024 * 1024), "1.00 TB");
}

#[test]
fn test_is_in_home_dir() {
    // Create test config
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().join("home");
    let config = Config::new(temp_dir.path().join("vault"), home_dir.clone());

    // Test with path inside home directory
    let inside_path = home_dir.join("file.txt");
    assert!(is_in_home_dir(&inside_path, &config));

    // Test with path outside home directory
    let outside_path = PathBuf::from("/outside/home");
    assert!(!is_in_home_dir(&outside_path, &config));

    // Test with home directory itself
    assert!(is_in_home_dir(&home_dir, &config));
}

#[test]
fn test_is_dotfile() {
    // Test with dotfiles
    assert!(is_dotfile(".bashrc"));
    assert!(is_dotfile(".vimrc"));
    assert!(is_dotfile(".config"));
    assert!(is_dotfile(PathBuf::from("/home/user/.ssh")));
    assert!(is_dotfile(PathBuf::from(".git")));

    // Test with non-dotfiles
    assert!(!is_dotfile("regular.txt"));
    assert!(!is_dotfile("no_dot"));
    assert!(!is_dotfile(PathBuf::from("/home/user/documents")));
    assert!(!is_dotfile(PathBuf::from("src/main.rs")));
}
