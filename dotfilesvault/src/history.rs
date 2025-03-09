use anyhow::Result;
use chrono::{DateTime, Local, TimeZone};
use git2::{Repository, Signature};
use log::{debug, info};
use std::fs;
use std::path::Path;

use crate::backup::Dotfile;
use crate::{Config, DotfilesError};

/// Represents a version of a dotfile
#[derive(Debug, Clone)]
pub struct DotfileVersion {
    /// The commit ID
    pub commit_id: String,

    /// The timestamp of the version
    pub timestamp: DateTime<Local>,

    /// The message associated with the version
    pub message: String,
}

/// Initialize a Git repository in the vault directory
pub fn init_git_repo(config: &Config) -> Result<Repository, DotfilesError> {
    let repo_path = &config.vault_dir;

    // Check if the repository already exists
    if repo_path.join(".git").exists() {
        return Repository::open(repo_path).map_err(DotfilesError::Git);
    }

    // Initialize a new repository
    let repo = Repository::init(repo_path)?;

    // Create a .gitignore file
    let gitignore_path = repo_path.join(".gitignore");
    fs::write(gitignore_path, "# Ignore temporary files\n*.tmp\n*.bak\n")?;

    info!("Initialized Git repository in {:?}", repo_path);

    Ok(repo)
}

/// Commit changes to the Git repository
pub fn commit_changes(config: &Config, message: &str) -> Result<String, DotfilesError> {
    let repo = init_git_repo(config)?;

    // Create the signature
    let signature = Signature::now("Dotfilesvault", "dotfilesvault@example.com")?;

    // Add all files to the index
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    // Create the tree
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get the parent commit, if any
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit()?),
        Err(_) => None,
    };

    let parents = match parent_commit {
        Some(ref commit) => vec![commit],
        None => vec![],
    };

    // Create the commit
    let commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        parents.as_slice(),
    )?;

    info!("Committed changes with ID: {}", commit_id);

    Ok(commit_id.to_string())
}

/// Get the history of a specific dotfile
pub fn get_dotfile_history(
    config: &Config,
    dotfile_path: &str,
) -> Result<Vec<DotfileVersion>, DotfilesError> {
    let path = Path::new(dotfile_path);
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        config.home_dir.join(path)
    };

    let dotfile = Dotfile::new(path, config);

    // Get the relative path from the vault directory
    let relative_path = match dotfile.vault_path.strip_prefix(&config.vault_dir) {
        Ok(rel_path) => rel_path.to_path_buf(),
        Err(_) => return Err(DotfilesError::DotfileNotFound(dotfile_path.to_string())),
    };

    // Check if the file exists in the vault
    if !dotfile.vault_path.exists() {
        return Err(DotfilesError::DotfileNotFound(dotfile_path.to_string()));
    }

    // Open the repository
    let repo = match Repository::open(&config.vault_dir) {
        Ok(repo) => repo,
        Err(_) => return Err(DotfilesError::NoDotfilesVaultDir),
    };

    // Get the revwalk
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut versions = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Check if this commit modified the file
        let tree = commit.tree()?;

        if tree.get_path(&relative_path).is_ok() {
            // This commit affected the file
            let timestamp = Local
                .timestamp_opt(commit.time().seconds(), 0)
                .single()
                .unwrap_or_else(|| Local::now());

            versions.push(DotfileVersion {
                commit_id: oid.to_string(),
                timestamp,
                message: commit.message().unwrap_or("").to_string(),
            });
        }
    }

    debug!("Found {} versions for {:?}", versions.len(), dotfile_path);

    Ok(versions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_env() -> (Config, TempDir) {
        // Create temporary directories for testing
        let temp_dir = TempDir::new().unwrap();
        let vault_dir = temp_dir.path().join("dotfilesvault");
        let home_dir = temp_dir.path().join("home");

        // Create directories
        fs::create_dir_all(&vault_dir).unwrap();
        fs::create_dir_all(&home_dir).unwrap();

        // Create a test config
        let config = Config::new(vault_dir, home_dir);

        (config, temp_dir)
    }

    #[test]
    fn test_init_git_repo() {
        let (config, _temp_dir) = setup_test_env();

        // Initialize the Git repository
        let repo = init_git_repo(&config).unwrap();

        // Check if it's a valid repository
        assert!(repo.is_empty().unwrap());
        assert!(config.vault_dir.join(".git").exists());
    }

    #[test]
    fn test_commit_changes() {
        let (config, _temp_dir) = setup_test_env();

        // Initialize the Git repository
        init_git_repo(&config).unwrap();

        // Create a test file
        let test_file = config.vault_dir.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "test content").unwrap();

        // Commit the changes
        let commit_id = commit_changes(&config, "Test commit").unwrap();

        // Check if the commit ID is valid
        assert!(!commit_id.is_empty());

        // Open the repository and check the commit
        let repo = Repository::open(&config.vault_dir).unwrap();
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();

        assert_eq!(commit.message().unwrap(), "Test commit");
    }
}
