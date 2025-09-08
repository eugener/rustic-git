# Rustic Git

A Rust library for Git repository operations with a clean, type-safe API.

## Overview

Rustic Git provides a simple, ergonomic interface for common Git operations. It follows a repository-centric design where you create a `Repository` instance and call methods on it to perform Git operations.

## Features

- ✅ Repository initialization and opening
- ✅ File status checking with detailed parsing  
- ✅ File staging (add files, add all, add updates)
- ✅ Commit creation with hash return
- ✅ Type-safe error handling
- ✅ Universal `Hash` type for Git objects
- ✅ Comprehensive test coverage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustic-git = "0.1.0"
```

## Quick Start

```rust
use rustic_git::{Repository, Result};

fn main() -> Result<()> {
    // Initialize a new repository
    let repo = Repository::init("/path/to/repo", false)?;
    
    // Or open an existing repository
    let repo = Repository::open("/path/to/existing/repo")?;
    
    // Check repository status
    let status = repo.status()?;
    if !status.is_clean() {
        println!("Modified files: {:?}", status.modified_files());
        println!("Untracked files: {:?}", status.untracked_files());
    }
    
    // Stage files
    repo.add(&["file1.txt", "file2.txt"])?;
    // Or stage all changes
    repo.add_all()?;
    
    // Create a commit
    let hash = repo.commit("Add new features")?;
    println!("Created commit: {}", hash.short());
    
    Ok(())
}
```

## API Documentation

### Repository Lifecycle

#### `Repository::init(path, bare) -> Result<Repository>`

Initialize a new Git repository.

```rust
// Initialize a regular repository
let repo = Repository::init("/path/to/repo", false)?;

// Initialize a bare repository  
let bare_repo = Repository::init("/path/to/bare-repo", true)?;
```

#### `Repository::open(path) -> Result<Repository>`

Open an existing Git repository.

```rust
let repo = Repository::open("/path/to/existing/repo")?;
```

### Status Operations

#### `Repository::status() -> Result<GitStatus>`

Get the current repository status.

```rust
let status = repo.status()?;

// Check if repository is clean
if status.is_clean() {
    println!("No changes");
} else {
    println!("Repository has changes");
}

// Get files by status
let modified = status.modified_files();
let untracked = status.untracked_files();

// Or work with all files directly
for (file_status, filename) in &status.files {
    println!("{:?}: {}", file_status, filename);
}
```

The `GitStatus` struct contains:
- `files: Box<[(FileStatus, String)]>` - All files with their status
- `is_clean()` - Returns true if no changes
- `has_changes()` - Returns true if any changes exist
- `modified_files()` - Get all modified files
- `untracked_files()` - Get all untracked files
- `files_with_status(status)` - Get files with specific status

#### File Status Types

```rust
pub enum FileStatus {
    Modified,   // File has been modified
    Added,      // File has been added to index
    Deleted,    // File has been deleted
    Renamed,    // File has been renamed
    Copied,     // File has been copied
    Untracked,  // File is not tracked by git
    Ignored,    // File is ignored by git
}
```

### Staging Operations

#### `Repository::add(paths) -> Result<()>`

Add specific files to the staging area.

```rust
// Add single file
repo.add(&["file.txt"])?;

// Add multiple files
repo.add(&["file1.txt", "file2.txt", "dir/file3.txt"])?;

// Add with Path objects
use std::path::Path;
repo.add(&[Path::new("file.txt")])?;
```

#### `Repository::add_all() -> Result<()>`

Add all changes to the staging area (equivalent to `git add .`).

```rust
repo.add_all()?;
```

#### `Repository::add_update() -> Result<()>`

Add all tracked files that have been modified (equivalent to `git add -u`).

```rust
repo.add_update()?;
```

### Commit Operations

#### `Repository::commit(message) -> Result<Hash>`

Create a commit with the given message.

```rust
let hash = repo.commit("Fix critical bug")?;
println!("Commit created: {}", hash);
println!("Short hash: {}", hash.short());
```

#### `Repository::commit_with_author(message, author) -> Result<Hash>`

Create a commit with a custom author.

```rust
let hash = repo.commit_with_author(
    "Add new feature", 
    "Jane Developer <jane@example.com>"
)?;
```

### Hash Type

The `Hash` type represents Git object hashes (commits, trees, blobs, etc.).

```rust
let hash = repo.commit("message")?;

// Get full hash as string
let full_hash: &str = hash.as_str();

// Get short hash (first 7 characters)
let short_hash: &str = hash.short();

// Display formatting
println!("Commit: {}", hash);  // Displays full hash
```

## Error Handling

All operations return `Result<T, GitError>` for proper error handling.

```rust
use rustic_git::{Repository, GitError};

match repo.commit("message") {
    Ok(hash) => println!("Success: {}", hash),
    Err(GitError::CommandFailed(msg)) => eprintln!("Git command failed: {}", msg),
    Err(GitError::IoError(msg)) => eprintln!("IO error: {}", msg),
}
```

## Complete Workflow Example

```rust
use rustic_git::{Repository, FileStatus};
use std::fs;

fn main() -> rustic_git::Result<()> {
    // Create a new repository
    let repo = Repository::init("./my-project", false)?;
    
    // Create some files
    fs::write("./my-project/README.md", "# My Project")?;
    fs::write("./my-project/src/main.rs", "fn main() { println!(\"Hello!\"); }")?;
    fs::create_dir_all("./my-project/src")?;
    
    // Check status
    let status = repo.status()?;
    println!("Found {} untracked files", status.untracked_files().len());
    
    // Stage all files
    repo.add_all()?;
    
    // Verify staging
    let status = repo.status()?;
    let added_files: Vec<_> = status.files.iter()
        .filter(|(s, _)| matches!(s, FileStatus::Added))
        .map(|(_, f)| f)
        .collect();
    println!("Staged files: {:?}", added_files);
    
    // Create initial commit
    let hash = repo.commit("Initial commit with project structure")?;
    println!("Created commit: {}", hash.short());
    
    // Verify clean state
    let status = repo.status()?;
    assert!(status.is_clean());
    println!("Repository is now clean!");
    
    Ok(())
}
```

## Examples

The `examples/` directory contains comprehensive demonstrations of library functionality:

### Running Examples

```bash
# Complete workflow from init to commit
cargo run --example basic_usage

# Repository lifecycle operations
cargo run --example repository_operations

# Status checking and file state filtering  
cargo run --example status_checking

# Staging operations (add, add_all, add_update)
cargo run --example staging_operations

# Commit workflows and Hash type usage
cargo run --example commit_workflows

# Error handling patterns and recovery strategies
cargo run --example error_handling
```

### Example Files

- **`basic_usage.rs`** - Demonstrates the fundamental rustic-git workflow: initialize a repository, create files, check status, stage changes, and create commits
- **`repository_operations.rs`** - Shows repository lifecycle operations including initializing regular and bare repositories, opening existing repos, and handling errors
- **`status_checking.rs`** - Comprehensive demonstration of GitStatus and FileStatus usage with all query methods and filtering capabilities
- **`staging_operations.rs`** - Shows all staging methods (add, add_all, add_update) with before/after status comparisons
- **`commit_workflows.rs`** - Demonstrates commit operations and Hash type methods, including custom authors and hash management
- **`error_handling.rs`** - Comprehensive error handling patterns showing GitError variants, recovery strategies, and best practices

All examples use temporary directories in `/tmp/` and include automatic cleanup for safe execution.

## Testing

Run the test suite:

```bash
cargo test
```

All tests create temporary repositories in `/tmp/` and clean up after themselves.

## Contributing

This library follows these design principles:

- **Repository-centric API**: Static lifecycle methods (`init`, `open`) return `Repository` instances
- **Type safety**: Strong typing with custom error types and structured return values  
- **Ergonomic design**: Clean, intuitive API that follows Rust conventions
- **Comprehensive testing**: All functionality thoroughly tested
- **Modular organization**: Commands organized in separate modules

## License

MIT License - see LICENSE file for details.

## Roadmap

Future planned features:
- [ ] Commit history and log operations
- [ ] Diff operations  
- [ ] Branch operations
- [ ] Remote operations (clone, push, pull)
- [ ] Merge and rebase operations
- [ ] Tag operations
- [ ] Stash operations

## Version

Current version: 0.1.0 - Basic git workflow (init, status, add, commit)