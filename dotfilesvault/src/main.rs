use anyhow::Result;
use clap::{Parser, Subcommand};
use log::{LevelFilter, debug, error, info};
use std::process;

use dotfilesvault::Config;
use dotfilesvault::backup::{backup_all_dotfiles, backup_specific_dotfiles};
use dotfilesvault::history::{commit_changes, get_dotfile_history};
use dotfilesvault::restore::{list_backed_up_dotfiles, restore_specific_dotfile};

/// Dotfilesvault - A tool for backing up and managing dotfiles with version history
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    /// Sets the level of verbosity
    #[clap(short, long, global = true)]
    verbose: bool,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Backup dotfiles from home directory
    Backup {
        /// Specific dotfiles to backup (defaults to all)
        #[clap(value_name = "FILES")]
        files: Vec<String>,
    },

    /// List all backed up dotfiles
    List,

    /// Show history of a specific dotfile
    History {
        /// Path to the dotfile
        #[clap(value_name = "FILE")]
        file: String,
    },

    /// Restore a dotfile from backup
    Restore {
        /// Path to the dotfile to restore
        #[clap(value_name = "FILE")]
        file: String,

        /// Specific version to restore (defaults to latest)
        #[clap(long)]
        version: Option<String>,
    },
}

fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logger
    env_logger::Builder::new()
        .filter_level(if cli.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();

    info!("Starting Dotfilesvault");

    // Create default configuration
    let config = Config::default();

    // Handle commands
    match cli.command {
        Commands::Backup { files } => {
            debug!("Running backup command");

            if files.is_empty() {
                info!("Backing up all dotfiles");
                if let Err(err) = backup_all_dotfiles(&config) {
                    error!("Failed to backup dotfiles: {}", err);
                    process::exit(1);
                }

                // Commit changes to Git repository
                if let Err(err) = commit_changes(&config, "Backup all dotfiles") {
                    error!("Failed to commit changes: {}", err);
                    process::exit(1);
                }
            } else {
                info!("Backing up specific dotfiles: {:?}", files);
                if let Err(err) = backup_specific_dotfiles(&config, &files) {
                    error!("Failed to backup specific dotfiles: {}", err);
                    process::exit(1);
                }

                // Commit changes to Git repository
                if let Err(err) =
                    commit_changes(&config, &format!("Backup specific dotfiles: {:?}", files))
                {
                    error!("Failed to commit changes: {}", err);
                    process::exit(1);
                }
            }

            info!("Backup completed successfully");
        }

        Commands::List => {
            debug!("Running list command");

            match list_backed_up_dotfiles(&config) {
                Ok(files) => {
                    if files.is_empty() {
                        println!("No dotfiles have been backed up yet.");
                    } else {
                        println!("Backed up dotfiles:");
                        for file in files {
                            println!("  {}", file.display());
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to list backed up dotfiles: {}", err);
                    process::exit(1);
                }
            }
        }

        Commands::History { file } => {
            debug!("Running history command for file: {}", file);

            match get_dotfile_history(&config, &file) {
                Ok(versions) => {
                    if versions.is_empty() {
                        println!("No history found for dotfile: {}", file);
                    } else {
                        println!("History for dotfile: {}", file);
                        for (i, version) in versions.iter().enumerate() {
                            println!(
                                "  Version {}: {} - {}",
                                i + 1,
                                version.timestamp.format("%Y-%m-%d %H:%M:%S"),
                                version.message
                            );
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to get history for dotfile: {}", err);
                    process::exit(1);
                }
            }
        }

        Commands::Restore { file, version } => {
            debug!("Running restore command for file: {}", file);

            // TODO: Implement version-specific restore
            if version.is_some() {
                error!("Version-specific restore is not yet implemented");
                process::exit(1);
            }

            if let Err(err) = restore_specific_dotfile(&config, &file) {
                error!("Failed to restore dotfile: {}", err);
                process::exit(1);
            }

            info!("Restored dotfile: {}", file);
        }
    }

    Ok(())
}
