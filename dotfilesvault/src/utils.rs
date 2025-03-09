use std::path::{Path, PathBuf};

use crate::Config;

/// Expand a tilde in a path to the home directory
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let path_str = path.as_ref().to_string_lossy();

    if path_str.starts_with("~/") || path_str == "~" {
        if let Some(home_dir) = dirs::home_dir() {
            if path_str == "~" {
                return home_dir;
            }

            return home_dir.join(path_str.strip_prefix("~/").unwrap());
        }
    }

    path.as_ref().to_path_buf()
}

/// Normalize a path for consistent handling
pub fn normalize_path<P: AsRef<Path>>(path: P, config: &Config) -> PathBuf {
    let path = expand_tilde(path);

    if path.is_absolute() {
        path
    } else {
        config.home_dir.join(path)
    }
}

/// Get a human-readable file size
pub fn human_readable_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Check if a path is inside the home directory
pub fn is_in_home_dir<P: AsRef<Path>>(path: P, config: &Config) -> bool {
    path.as_ref().starts_with(&config.home_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_expand_tilde() {
        let home_dir = dirs::home_dir().unwrap();

        assert_eq!(expand_tilde("~"), home_dir);
        assert_eq!(expand_tilde("~/Documents"), home_dir.join("Documents"));
        assert_eq!(
            expand_tilde("/absolute/path"),
            PathBuf::from("/absolute/path")
        );
        assert_eq!(
            expand_tilde("relative/path"),
            PathBuf::from("relative/path")
        );
    }

    #[test]
    fn test_normalize_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().join("vault"), temp_dir.path().join("home"));

        // Absolute path should remain unchanged
        let abs_path = PathBuf::from("/absolute/path");
        assert_eq!(normalize_path(&abs_path, &config), abs_path);

        // Relative path should be joined with home_dir
        let rel_path = PathBuf::from("relative/path");
        assert_eq!(
            normalize_path(&rel_path, &config),
            config.home_dir.join(&rel_path)
        );
    }

    #[test]
    fn test_human_readable_size() {
        assert_eq!(human_readable_size(500), "500.00 B");
        assert_eq!(human_readable_size(1024), "1.00 KB");
        assert_eq!(human_readable_size(1024 * 1024), "1.00 MB");
        assert_eq!(human_readable_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_is_in_home_dir() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config::new(temp_dir.path().join("vault"), temp_dir.path().join("home"));

        // Path inside home directory
        let home_path = config.home_dir.join("file.txt");
        assert!(is_in_home_dir(&home_path, &config));

        // Path outside home directory
        let outside_path = PathBuf::from("/outside/home");
        assert!(!is_in_home_dir(&outside_path, &config));
    }
}
