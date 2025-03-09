# Dotfilesvault

A Rust-based tool for backing up and managing dotfiles with version history.

## Features

- Automatically detects and backs up dotfiles from your home directory
- Maintains a version history of your dotfiles
- Allows restoration of dotfiles from backups
- Simple CLI interface for easy management

## Installation

```bash
cargo install dotfilesvault
```

## Usage

```bash
# Backup all dotfiles
dotfilesvault backup

# List all backed up dotfiles
dotfilesvault list

# Show history of a specific dotfile
dotfilesvault history ~/.bashrc

# Restore a specific dotfile
dotfilesvault restore ~/.bashrc

# Restore a specific version of a dotfile
dotfilesvault restore ~/.bashrc --version 2023-05-15-14-30-45
```

## Development

This project follows Test-Driven Development (TDD) principles:

1. Write a failing test
2. Implement the minimal code to make the test pass
3. Refactor the code while ensuring tests still pass

To run tests:

```bash
cargo test
```

## License

MIT 