# Rustic Git

A Rust library for Git repository operations with a clean, type-safe API.

## Overview

Rustic Git provides a simple, ergonomic interface for common Git operations. It follows a repository-centric design where you create a `Repository` instance and call methods on it to perform Git operations.

## Features

- ✅ Repository initialization and opening
- ✅ **Enhanced file status checking** with separate staged/unstaged tracking
- ✅ **Precise Git state representation** using IndexStatus and WorktreeStatus enums
- ✅ File staging (add files, add all, add updates)
- ✅ Commit creation with hash return
- ✅ **Complete branch operations** with type-safe Branch API
- ✅ **Branch management** (create, delete, checkout, list)
- ✅ Type-safe error handling with custom GitError enum
- ✅ Universal `Hash` type for Git objects
- ✅ **Immutable collections** (Box<[T]>) for memory efficiency
- ✅ **Const enum conversions** with zero runtime cost
- ✅ Comprehensive test coverage (90+ tests)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustic-git = "0.1.0"
```

## Quick Start

```rust
use rustic_git::{Repository, Result, IndexStatus, WorktreeStatus};

fn main() -> Result<()> {
    // Initialize a new repository
    let repo = Repository::init("/path/to/repo", false)?;

    // Or open an existing repository
    let repo = Repository::open("/path/to/existing/repo")?;

    // Check repository status with enhanced API
    let status = repo.status()?;
    if !status.is_clean() {
        // Get files by staging state
        let staged_count = status.staged_files().count();
        let unstaged_count = status.unstaged_files().count();
        let untracked_count = status.untracked_entries().count();

        println!("Repository status:");
        println!("  Staged: {} files", staged_count);
        println!("  Unstaged: {} files", unstaged_count);
        println!("  Untracked: {} files", untracked_count);

        // Filter by specific status types
        let modified_files: Vec<_> = status
            .files_with_worktree_status(WorktreeStatus::Modified)
            .collect();
        println!("  Modified files: {:?}", modified_files);
    }

    // Stage files
    repo.add(&["file1.txt", "file2.txt"])?;
    // Or stage all changes
    repo.add_all()?;

    // Create a commit
    let hash = repo.commit("Add new features")?;
    println!("Created commit: {}", hash.short());

    // Branch operations
    let branches = repo.branches()?;
    println!("Current branch: {:?}", repo.current_branch()?.map(|b| b.name));

    // Create and switch to new branch
    let feature_branch = repo.checkout_new("feature/new-api", None)?;
    println!("Created and switched to: {}", feature_branch.name);

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

Get the current repository status with enhanced staged/unstaged file tracking.

```rust
let status = repo.status()?;

// Check if repository is clean
if status.is_clean() {
    println!("No changes");
} else {
    println!("Repository has changes");
}

// Get files by staging state
let staged_files: Vec<_> = status.staged_files().collect();
let unstaged_files: Vec<_> = status.unstaged_files().collect();
let untracked_files: Vec<_> = status.untracked_entries().collect();

// Filter by specific status types
let modified_in_index: Vec<_> = status
    .files_with_index_status(IndexStatus::Modified)
    .collect();
let modified_in_worktree: Vec<_> = status
    .files_with_worktree_status(WorktreeStatus::Modified)
    .collect();

// Work with all file entries directly
for entry in status.entries() {
    println!("[{}][{}] {}",
        entry.index_status.to_char(),
        entry.worktree_status.to_char(),
        entry.path.display()
    );
}
```

The `GitStatus` struct contains:
- `entries: Box<[FileEntry]>` - Immutable collection of file entries
- `is_clean()` - Returns true if no changes
- `has_changes()` - Returns true if any changes exist
- `staged_files()` - Iterator over files with index changes (staged)
- `unstaged_files()` - Iterator over files with worktree changes (unstaged)
- `untracked_entries()` - Iterator over untracked files
- `ignored_files()` - Iterator over ignored files
- `files_with_index_status(status)` - Filter by specific index status
- `files_with_worktree_status(status)` - Filter by specific worktree status

#### File Status Types

The enhanced status API uses separate enums for index (staged) and worktree (unstaged) states:

```rust
// Index (staging area) status
pub enum IndexStatus {
    Clean,      // No changes in index
    Modified,   // File modified in index
    Added,      // File added to index
    Deleted,    // File deleted in index
    Renamed,    // File renamed in index
    Copied,     // File copied in index
}

// Worktree (working directory) status
pub enum WorktreeStatus {
    Clean,      // No changes in worktree
    Modified,   // File modified in worktree
    Deleted,    // File deleted in worktree
    Untracked,  // File not tracked by git
    Ignored,    // File ignored by git
}

// File entry combining both states
pub struct FileEntry {
    pub path: PathBuf,
    pub index_status: IndexStatus,
    pub worktree_status: WorktreeStatus,
}
```

Both enums support const character conversion:
```rust
// Convert to/from git porcelain characters
let status = IndexStatus::from_char('M');  // IndexStatus::Modified
let char = status.to_char();               // 'M'

// Display formatting
println!("{}", IndexStatus::Modified);     // Prints: M
println!("{}", WorktreeStatus::Untracked); // Prints: ?
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

### Branch Operations

#### `Repository::branches() -> Result<BranchList>`

List all branches in the repository.

```rust
let branches = repo.branches()?;

// Check total count
println!("Total branches: {}", branches.len());
println!("Local branches: {}", branches.local_count());
println!("Remote branches: {}", branches.remote_count());

// Iterate over all branches
for branch in branches.iter() {
    let marker = if branch.is_current { "*" } else { " " };
    println!("  {}{} ({})", marker, branch.name, branch.commit_hash.short());
}

// Filter by type
let local_branches: Vec<_> = branches.local().collect();
let remote_branches: Vec<_> = branches.remote().collect();
```

#### `Repository::current_branch() -> Result<Option<Branch>>`

Get the currently checked out branch.

```rust
if let Some(current) = repo.current_branch()? {
    println!("On branch: {}", current.name);
    println!("Last commit: {}", current.commit_hash.short());
    if let Some(upstream) = &current.upstream {
        println!("Tracking: {}", upstream);
    }
}
```

#### `Repository::create_branch(name, start_point) -> Result<Branch>`

Create a new branch.

```rust
// Create branch from current HEAD
let branch = repo.create_branch("feature/new-api", None)?;

// Create branch from specific commit/branch
let branch = repo.create_branch("hotfix/bug-123", Some("main"))?;
let branch = repo.create_branch("release/v1.0", Some("develop"))?;
```

#### `Repository::checkout(branch) -> Result<()>`

Switch to an existing branch.

```rust
let branches = repo.branches()?;
if let Some(branch) = branches.find("develop") {
    repo.checkout(&branch)?;
    println!("Switched to: {}", branch.name);
}
```

#### `Repository::checkout_new(name, start_point) -> Result<Branch>`

Create a new branch and switch to it immediately.

```rust
// Create and checkout new branch from current HEAD
let branch = repo.checkout_new("feature/auth", None)?;

// Create and checkout from specific starting point
let branch = repo.checkout_new("feature/api", Some("develop"))?;
println!("Created and switched to: {}", branch.name);
```

#### `Repository::delete_branch(branch, force) -> Result<()>`

Delete a branch.

```rust
let branches = repo.branches()?;
if let Some(branch) = branches.find("old-feature") {
    // Safe delete (fails if unmerged)
    repo.delete_branch(&branch, false)?;

    // Force delete
    // repo.delete_branch(&branch, true)?;
}
```

#### Branch Types

The branch API uses structured types for type safety:

```rust
// Branch represents a single branch
pub struct Branch {
    pub name: String,
    pub branch_type: BranchType,
    pub is_current: bool,
    pub commit_hash: Hash,
    pub upstream: Option<String>,
}

// Branch type enumeration
pub enum BranchType {
    Local,           // Local branch
    RemoteTracking,  // Remote-tracking branch
}

// BranchList contains all branches with efficient methods
pub struct BranchList {
    // Methods:
    // - iter() -> iterator over all branches
    // - local() -> iterator over local branches
    // - remote() -> iterator over remote branches
    // - current() -> get current branch
    // - find(name) -> find branch by exact name
    // - find_by_short_name(name) -> find by short name
    // - len(), is_empty() -> collection info
}
```

#### Branch Search and Filtering

```rust
let branches = repo.branches()?;

// Find specific branches
if let Some(main) = branches.find("main") {
    println!("Found main branch: {}", main.commit_hash.short());
}

// Find by short name (useful for remote branches)
if let Some(feature) = branches.find_by_short_name("feature") {
    println!("Found feature branch: {}", feature.name);
}

// Filter by type
println!("Local branches:");
for branch in branches.local() {
    println!("  - {}", branch.name);
}

if branches.remote_count() > 0 {
    println!("Remote branches:");
    for branch in branches.remote() {
        println!("  - {}", branch.name);
    }
}

// Get current branch
if let Some(current) = branches.current() {
    println!("Currently on: {}", current.name);
}
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
use rustic_git::{Repository, IndexStatus, WorktreeStatus};
use std::fs;

fn main() -> rustic_git::Result<()> {
    // Create a new repository
    let repo = Repository::init("./my-project", false)?;

    // Create some files
    fs::write("./my-project/README.md", "# My Project")?;
    fs::create_dir_all("./my-project/src")?;
    fs::write("./my-project/src/main.rs", "fn main() { println!(\"Hello!\"); }")?;

    // Check status with enhanced API
    let status = repo.status()?;
    let untracked_count = status.untracked_entries().count();
    println!("Found {} untracked files", untracked_count);

    // Display detailed status
    for entry in status.entries() {
        println!("[{}][{}] {}",
            entry.index_status.to_char(),
            entry.worktree_status.to_char(),
            entry.path.display()
        );
    }

    // Stage all files
    repo.add_all()?;

    // Verify staging with enhanced API
    let status = repo.status()?;
    let staged_files: Vec<_> = status.staged_files().collect();
    println!("Staged {} files", staged_files.len());

    // Show specifically added files
    let added_files: Vec<_> = status
        .files_with_index_status(IndexStatus::Added)
        .collect();
    println!("Added files: {:?}", added_files);

    // Create initial commit
    let hash = repo.commit("Initial commit with project structure")?;
    println!("Created commit: {}", hash.short());

    // Branch operations workflow
    let branches = repo.branches()?;
    println!("Current branch: {:?}", repo.current_branch()?.map(|b| b.name));

    // Create a feature branch
    let feature_branch = repo.checkout_new("feature/user-auth", None)?;
    println!("Created and switched to: {}", feature_branch.name);

    // Make changes on the feature branch
    fs::write("./my-project/src/auth.rs", "pub fn authenticate() { /* TODO */ }")?;
    repo.add(&["src/auth.rs"])?;
    let feature_commit = repo.commit("Add authentication module")?;
    println!("Feature commit: {}", feature_commit.short());

    // Switch back to main and create another branch
    if let Some(main_branch) = branches.find("main") {
        repo.checkout(&main_branch)?;
        println!("Switched back to main");
    }

    let doc_branch = repo.create_branch("docs/api", None)?;
    println!("Created documentation branch: {}", doc_branch.name);

    // List all branches
    let final_branches = repo.branches()?;
    println!("\nFinal branch summary:");
    for branch in final_branches.iter() {
        let marker = if branch.is_current { "*" } else { " " };
        println!("  {}{} ({})", marker, branch.name, branch.commit_hash.short());
    }

    // Verify clean state
    let status = repo.status()?;
    assert!(status.is_clean());
    println!("Repository is clean!");

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

# Enhanced status API with staged/unstaged tracking
cargo run --example status_checking

# Staging operations (add, add_all, add_update)
cargo run --example staging_operations

# Commit workflows and Hash type usage
cargo run --example commit_workflows

# Branch operations (create, delete, checkout, list)
cargo run --example branch_operations

# Error handling patterns and recovery strategies
cargo run --example error_handling
```

### Example Files

- **`basic_usage.rs`** - Demonstrates the fundamental rustic-git workflow: initialize a repository, create files, check status, stage changes, and create commits
- **`repository_operations.rs`** - Shows repository lifecycle operations including initializing regular and bare repositories, opening existing repos, and handling errors
- **`status_checking.rs`** - Comprehensive demonstration of GitStatus and FileStatus usage with all query methods and filtering capabilities
- **`staging_operations.rs`** - Shows all staging methods (add, add_all, add_update) with before/after status comparisons
- **`commit_workflows.rs`** - Demonstrates commit operations and Hash type methods, including custom authors and hash management
- **`branch_operations.rs`** - Complete branch management demonstration: create, checkout, delete branches, and BranchList filtering
- **`error_handling.rs`** - Comprehensive error handling patterns showing GitError variants, recovery strategies, and best practices

All examples use temporary directories in `/tmp/` and include automatic cleanup for safe execution.

## Testing

Run the test suite:

```bash
cargo test
```

All tests create temporary repositories in `/tmp/` and clean up after themselves.

## Contributing

We welcome contributions! Please follow these guidelines when contributing to rustic-git:

### Code Standards

- **Rust Edition**: Use Rust edition 2024
- **Style Guide**: Follow the Rust style guide for naming conventions and formatting
- **Code Quality**: Implement best practices for code organization and maintainability
- **No Emojis**: Do not use emoji in code or commit messages

### Design Principles

- **Repository-centric API**: Static lifecycle methods (`init`, `open`) return `Repository` instances, instance methods for git operations
- **Module-based organization**: Separate files for repository.rs, error.rs, with lib.rs for re-exports only
- **Co-located unit tests**: Tests within each module (`#[cfg(test)] mod tests`) rather than separate test files
- **Early validation**: Always call `Repository::ensure_git()` before git operations to validate git availability
- **Path handling**: Use `PathBuf` for internal storage, `&Path` for method parameters and returns, `impl AsRef<Path>` for flexibility
- **Error handling**: Custom `GitError` enum with `From<io::Error>` trait for ergonomic error propagation
- **Command execution**: Use `std::process::Command` with proper error handling and stderr capture

### Development Workflow

Before submitting a pull request, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Build project
cargo build

# Run all tests
cargo test

# Run linting (no warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Verify all examples work
cargo run --example basic_usage
cargo run --example repository_operations
cargo run --example status_checking
cargo run --example staging_operations
cargo run --example commit_workflows
cargo run --example branch_operations
cargo run --example error_handling
```

### Pull Request Guidelines

1. Ensure all tests pass and examples run successfully
2. Follow conventional commit format: `type(scope): description`
3. Use types like `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
4. Keep commit messages concise and in present tense
5. Make sure your changes align with the project's design principles


## Roadmap

Future planned features:
- [ ] Commit history and log operations
- [ ] Diff operations
- [ ] Remote operations (clone, push, pull)
- [ ] Merge and rebase operations
- [ ] Tag operations
- [ ] Stash operations

## Version

Current version: 0.1.0 - Basic git workflow with branch operations (init, status, add, commit, branch)
