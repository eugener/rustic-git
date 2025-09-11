# Rustic Git

A Rust library for Git repository operations with a clean, type-safe API.

![Build](https://github.com/eugener/rustic-git/actions/workflows/ci.yml/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/rustic-git.svg)](https://crates.io/crates/rustic-git)
[![Downloads](https://img.shields.io/crates/d/rustic-git.svg)](https://crates.io/crates/rustic-git)
[![Docs.rs](https://docs.rs/rustic-git/badge.svg)](https://docs.rs/rustic-git)
![Rust Version](https://img.shields.io/badge/rustc-1.89+-blue.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![dependency status](https://deps.rs/repo/github/eugener/rustic-git/status.svg)](https://deps.rs/repo/github/eugener/rustic-git)

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
- ✅ **Commit history & log operations** with multi-level API
- ✅ **Advanced commit querying** with filtering and analysis
- ✅ **Repository configuration management** with type-safe API
- ✅ **Remote management** with full CRUD operations and network support
- ✅ **Network operations** (fetch, push, clone) with advanced options
- ✅ **File lifecycle operations** (restore, reset, remove, move, .gitignore management)
- ✅ Type-safe error handling with custom GitError enum
- ✅ Universal `Hash` type for Git objects
- ✅ **Immutable collections** (Box<[T]>) for memory efficiency
- ✅ **Const enum conversions** with zero runtime cost
- ✅ Comprehensive test coverage (128+ tests)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustic-git = "*"
```

Or use `cargo add` to automatically add the latest version:

```bash
cargo add rustic-git
```

## Quick Start

```rust
use rustic_git::{Repository, Result, IndexStatus, WorktreeStatus, LogOptions, FetchOptions, PushOptions, RestoreOptions, RemoveOptions, MoveOptions};

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

    // Configure git user for commits
    repo.config().set_user("Your Name", "your.email@example.com")?;

    // Create a commit
    let hash = repo.commit("Add new features")?;
    println!("Created commit: {}", hash.short());

    // Branch operations
    let branches = repo.branches()?;
    println!("Current branch: {:?}", repo.current_branch()?.map(|b| b.name));

    // Create and switch to new branch
    let feature_branch = repo.checkout_new("feature/new-api", None)?;
    println!("Created and switched to: {}", feature_branch.name);

    // Commit history operations
    let commits = repo.log()?;
    println!("Total commits: {}", commits.len());

    // Get recent commits
    let recent = repo.recent_commits(5)?;
    for commit in recent.iter() {
        println!("{} - {}", commit.hash.short(), commit.message.subject);
    }

    // Advanced commit queries
    let opts = LogOptions::new()
        .max_count(10)
        .grep("fix".to_string());
    let bug_fixes = repo.log_with_options(&opts)?;
    println!("Found {} bug fixes", bug_fixes.len());

    // Remote management
    repo.add_remote("origin", "https://github.com/user/repo.git")?;
    repo.add_remote("upstream", "https://github.com/original/repo.git")?;

    // List remotes
    let remotes = repo.list_remotes()?;
    for remote in remotes.iter() {
        println!("Remote: {} -> {}", remote.name, remote.fetch_url);
    }

    // Network operations with options
    let fetch_opts = FetchOptions::new().with_prune().with_tags();
    repo.fetch_with_options("origin", fetch_opts)?;

    let push_opts = PushOptions::new().with_set_upstream();
    repo.push_with_options("origin", "main", push_opts)?;

    // File lifecycle operations
    // Restore file from HEAD
    repo.checkout_file("modified_file.txt")?;
    
    // Advanced restore with options
    let restore_opts = RestoreOptions::new()
        .with_source("HEAD~1")
        .with_worktree();
    repo.restore(&["file.txt"], restore_opts)?;
    
    // Unstage files
    repo.reset_file("staged_file.txt")?;
    
    // Remove files
    repo.rm(&["unwanted_file.txt"])?;
    
    // Remove from index only (keep in working tree)
    let rm_opts = RemoveOptions::new().with_cached();
    repo.rm_with_options(&["keep_local.txt"], rm_opts)?;
    
    // Move/rename files
    repo.mv("old_name.txt", "new_name.txt")?;
    
    // .gitignore management
    repo.ignore_add(&["*.tmp", "build/", "node_modules/"])?;
    let is_ignored = repo.ignore_check("temp_file.tmp")?;
    let patterns = repo.ignore_list()?;

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

### Configuration Operations

#### `Repository::config() -> RepoConfig`

Get a configuration manager for the repository to set and get git configuration values.

```rust
// Configure git user (convenience method)
repo.config().set_user("Your Name", "your.email@example.com")?;

// Get user configuration
let (name, email) = repo.config().get_user()?;
println!("User: {} <{}>", name, email);

// Set any git configuration value
repo.config().set("core.autocrlf", "false")?;
repo.config().set("pull.rebase", "true")?;

// Get any git configuration value
let autocrlf = repo.config().get("core.autocrlf")?;
println!("autocrlf setting: {}", autocrlf);

// Remove a configuration value
repo.config().unset("user.signingkey")?;
```

#### Configuration Methods

- **`set_user(name, email)`** - Convenience method to set both user.name and user.email
- **`get_user()`** - Get user configuration as a tuple (name, email)
- **`set(key, value)`** - Set any git configuration value
- **`get(key)`** - Get any git configuration value as String
- **`unset(key)`** - Remove a git configuration value

All configuration operations are scoped to the specific repository.

### Remote Management

#### `Repository::add_remote(name, url) -> Result<()>`

Add a remote to the repository.

```rust
repo.add_remote("origin", "https://github.com/user/repo.git")?;
repo.add_remote("upstream", "git@github.com:original/repo.git")?;
```

#### `Repository::list_remotes() -> Result<RemoteList>`

List all remotes with their URLs.

```rust
let remotes = repo.list_remotes()?;
for remote in remotes.iter() {
    println!("{} -> {}", remote.name, remote.fetch_url);
    if let Some(push_url) = &remote.push_url {
        println!("  Push URL: {}", push_url);
    }
}

// Find specific remote
if let Some(origin) = remotes.find("origin") {
    println!("Origin URL: {}", origin.fetch_url);
}
```

#### `Repository::remove_remote(name) -> Result<()>`

Remove a remote from the repository.

```rust
repo.remove_remote("old-remote")?;
```

#### `Repository::rename_remote(old_name, new_name) -> Result<()>`

Rename an existing remote.

```rust
repo.rename_remote("origin", "upstream")?;
```

#### `Repository::get_remote_url(name) -> Result<String>`

Get the URL for a specific remote.

```rust
let url = repo.get_remote_url("origin")?;
println!("Origin URL: {}", url);
```

### Network Operations

#### `Repository::fetch(remote) -> Result<()>`

Fetch changes from a remote repository.

```rust
repo.fetch("origin")?;
```

#### `Repository::fetch_with_options(remote, options) -> Result<()>`

Fetch with advanced options.

```rust
let options = FetchOptions::new()
    .with_prune()      // Remove stale remote-tracking branches
    .with_tags()       // Fetch tags
    .with_all_remotes(); // Fetch from all remotes

repo.fetch_with_options("origin", options)?;
```

#### `Repository::push(remote, branch) -> Result<()>`

Push changes to a remote repository.

```rust
repo.push("origin", "main")?;
```

#### `Repository::push_with_options(remote, branch, options) -> Result<()>`

Push with advanced options.

```rust
let options = PushOptions::new()
    .with_force()        // Force push (use with caution)
    .with_tags()         // Push tags
    .with_set_upstream(); // Set upstream tracking

repo.push_with_options("origin", "feature-branch", options)?;
```

#### `Repository::clone(url, path) -> Result<Repository>`

Clone a remote repository (static method).

```rust
let repo = Repository::clone("https://github.com/user/repo.git", "./local-copy")?;
```

### File Lifecycle Operations

#### `Repository::checkout_file(path) -> Result<()>`

Restore a file from HEAD, discarding local changes.

```rust
// Restore a modified file to its last committed state
repo.checkout_file("modified_file.txt")?;
```

#### `Repository::restore(paths, options) -> Result<()>`

Restore files with advanced options using git's restore command.

```rust
// Restore from a specific commit
let options = RestoreOptions::new()
    .with_source("HEAD~1")
    .with_worktree();
repo.restore(&["file.txt"], options)?;

// Restore only staged changes
let staged_options = RestoreOptions::new().with_staged();
repo.restore(&["file.txt"], staged_options)?;

// Restore both staged and worktree
let both_options = RestoreOptions::new()
    .with_staged()
    .with_worktree();
repo.restore(&["file.txt"], both_options)?;
```

#### `Repository::reset_file(path) -> Result<()>`

Unstage a file, removing it from the staging area while keeping changes in working directory.

```rust
// Unstage a previously staged file
repo.reset_file("staged_file.txt")?;
```

#### `Repository::rm(paths) -> Result<()>`

Remove files from both working directory and repository.

```rust
// Remove files from repository
repo.rm(&["unwanted_file.txt", "old_dir/"])?;
```

#### `Repository::rm_with_options(paths, options) -> Result<()>`

Remove files with advanced options.

```rust
// Remove from index only, keep in working tree
let cached_options = RemoveOptions::new().with_cached();
repo.rm_with_options(&["keep_local.txt"], cached_options)?;

// Force remove with recursive option
let force_options = RemoveOptions::new()
    .with_force()
    .with_recursive();
repo.rm_with_options(&["problematic_dir/"], force_options)?;

// Remove with ignore-unmatch (don't fail if files don't exist)
let safe_options = RemoveOptions::new().with_ignore_unmatch();
repo.rm_with_options(&["might_not_exist.txt"], safe_options)?;
```

#### `Repository::mv(source, destination) -> Result<()>`

Move or rename files and directories.

```rust
// Rename a file
repo.mv("old_name.txt", "new_name.txt")?;

// Move to different directory
repo.mv("file.txt", "subdir/file.txt")?;
```

#### `Repository::mv_with_options(source, destination, options) -> Result<()>`

Move files with advanced options.

```rust
// Force move even if destination exists
let force_options = MoveOptions::new().with_force();
repo.mv_with_options("source.txt", "existing.txt", force_options)?;

// Dry run to see what would be moved
let dry_run_options = MoveOptions::new()
    .with_dry_run()
    .with_verbose();
repo.mv_with_options("test.txt", "preview.txt", dry_run_options)?;
```

#### `Repository::ignore_add(patterns) -> Result<()>`

Add patterns to .gitignore file.

```rust
// Add ignore patterns
repo.ignore_add(&["*.tmp", "build/", "node_modules/", ".DS_Store"])?;
```

#### `Repository::ignore_check(path) -> Result<bool>`

Check if a file is ignored by .gitignore patterns.

```rust
// Check if file is ignored
let is_ignored = repo.ignore_check("temp_file.tmp")?;
if is_ignored {
    println!("File is ignored by .gitignore");
}
```

#### `Repository::ignore_list() -> Result<Vec<String>>`

List current ignore patterns from .gitignore.

```rust
// List all ignore patterns
let patterns = repo.ignore_list()?;
for pattern in patterns {
    println!("Ignoring: {}", pattern);
}
```

#### File Lifecycle Options

The file lifecycle operations use builder patterns for advanced configuration:

```rust
// RestoreOptions for advanced restore operations
let restore_options = RestoreOptions::new()
    .with_source("main")      // Restore from specific commit/branch
    .with_staged()            // Restore staged files
    .with_worktree();         // Restore working tree files

// RemoveOptions for file removal
let remove_options = RemoveOptions::new()
    .with_force()             // Force removal
    .with_recursive()         // Remove directories recursively
    .with_cached()            // Remove from index only
    .with_ignore_unmatch();   // Don't fail if files don't match

// MoveOptions for file moves
let move_options = MoveOptions::new()
    .with_force()             // Force move even if destination exists
    .with_verbose()           // Show verbose output
    .with_dry_run();          // Dry run mode (don't actually move)
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

### Commit History Operations

#### `Repository::log() -> Result<CommitLog>`

Get all commits in the repository.

```rust
let commits = repo.log()?;
println!("Total commits: {}", commits.len());

for commit in commits.iter() {
    println!("{} - {} by {} at {}",
        commit.hash.short(),
        commit.message.subject,
        commit.author.name,
        commit.timestamp.format("%Y-%m-%d %H:%M:%S")
    );
}
```

#### `Repository::recent_commits(count) -> Result<CommitLog>`

Get the most recent N commits.

```rust
let recent = repo.recent_commits(10)?;
for commit in recent.iter() {
    println!("{} - {}", commit.hash.short(), commit.message.subject);
    if let Some(body) = &commit.message.body {
        println!("  {}", body);
    }
}
```

#### `Repository::log_with_options(options) -> Result<CommitLog>`

Advanced commit queries with filtering options.

```rust
use chrono::{Utc, Duration};

// Search commits with message containing "fix"
let bug_fixes = repo.log_with_options(&LogOptions::new()
    .max_count(20)
    .grep("fix".to_string()))?;

// Get commits by specific author
let author_commits = repo.log_with_options(&LogOptions::new()
    .author("jane@example.com".to_string()))?;

// Get commits from date range
let since = Utc::now() - Duration::days(30);
let recent_commits = repo.log_with_options(&LogOptions::new()
    .since(since)
    .no_merges(true))?;

// Get commits affecting specific paths
let file_commits = repo.log_with_options(&LogOptions::new()
    .paths(vec!["src/main.rs".into(), "docs/".into()]))?;
```

#### `Repository::log_range(from, to) -> Result<CommitLog>`

Get commits between two specific commits.

```rust
// Get all commits between two hashes
let range_commits = repo.log_range(&from_hash, &to_hash)?;
println!("Commits in range: {}", range_commits.len());
```

#### `Repository::log_for_paths(paths) -> Result<CommitLog>`

Get commits that affected specific files or directories.

```rust
// Get commits that modified specific files
let file_commits = repo.log_for_paths(&["src/main.rs", "Cargo.toml"])?;

// Get commits that affected a directory
let dir_commits = repo.log_for_paths(&["src/"])?;
```

#### `Repository::show_commit(hash) -> Result<CommitDetails>`

Get detailed information about a specific commit including file changes.

```rust
let details = repo.show_commit(&commit_hash)?;
println!("Commit: {}", details.commit.hash);
println!("Author: {} <{}>", details.commit.author.name, details.commit.author.email);
println!("Date: {}", details.commit.timestamp);
println!("Message: {}", details.commit.message.subject);

if let Some(body) = &details.commit.message.body {
    println!("Body: {}", body);
}

println!("Files changed: {}", details.files_changed.len());
for file in &details.files_changed {
    println!("  - {}", file.display());
}

println!("Changes: +{} -{}", details.insertions, details.deletions);
```

#### Commit Types and Filtering

The commit API provides rich types for working with commit data:

```rust
// Commit represents a single commit
pub struct Commit {
    pub hash: Hash,
    pub author: Author,
    pub committer: Author,
    pub message: CommitMessage,
    pub timestamp: DateTime<Utc>,
    pub parents: Box<[Hash]>,
}

// Author information with timestamp
pub struct Author {
    pub name: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

// Parsed commit message
pub struct CommitMessage {
    pub subject: String,
    pub body: Option<String>,
}

// Detailed commit information
pub struct CommitDetails {
    pub commit: Commit,
    pub files_changed: Box<[PathBuf]>,
    pub insertions: u32,
    pub deletions: u32,
}
```

#### CommitLog Filtering

`CommitLog` provides iterator-based filtering methods:

```rust
let commits = repo.log()?;

// Filter by message content
let bug_fixes: Vec<_> = commits.with_message_containing("fix").collect();
let features: Vec<_> = commits.with_message_containing("feat").collect();

// Filter by date
use chrono::{Utc, Duration};
let last_week = Utc::now() - Duration::weeks(1);
let recent: Vec<_> = commits.since(last_week).collect();

// Filter by commit type
let merge_commits: Vec<_> = commits.merges_only().collect();
let regular_commits: Vec<_> = commits.no_merges().collect();

// Search by hash
if let Some(commit) = commits.find_by_hash(&target_hash) {
    println!("Found: {}", commit.message.subject);
}

if let Some(commit) = commits.find_by_short_hash("abc1234") {
    println!("Found by short hash: {}", commit.message.subject);
}
```

#### LogOptions Builder

`LogOptions` provides a builder pattern for advanced queries:

```rust
let options = LogOptions::new()
    .max_count(50)                          // Limit number of commits
    .since(Utc::now() - Duration::days(30)) // Since date
    .until(Utc::now())                      // Until date
    .author("jane@example.com".to_string()) // Filter by author
    .committer("john@example.com".to_string()) // Filter by committer
    .grep("important".to_string())          // Search in commit messages
    .follow_renames(true)                   // Follow file renames
    .merges_only(true)                      // Only merge commits
    .no_merges(true)                        // Exclude merge commits
    .paths(vec!["src/".into()]);            // Filter by paths

let filtered_commits = repo.log_with_options(&options)?;
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

    // Configure git user for commits
    repo.config().set_user("Your Name", "your.email@example.com")?;

    // Set some additional repository settings
    repo.config().set("core.autocrlf", "false")?;
    repo.config().set("pull.rebase", "true")?;

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

    // Display final configuration
    let (user_name, user_email) = repo.config().get_user()?;
    println!("Repository configured for: {} <{}>", user_name, user_email);

    let autocrlf = repo.config().get("core.autocrlf")?;
    let rebase_setting = repo.config().get("pull.rebase")?;
    println!("Settings: autocrlf={}, pull.rebase={}", autocrlf, rebase_setting);

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

# Repository configuration management
cargo run --example config_operations

# Commit history and log operations with advanced querying
cargo run --example commit_history

# Remote management and network operations
cargo run --example remote_operations

# File lifecycle operations (restore, remove, move, .gitignore)
cargo run --example file_lifecycle_operations

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
- **`config_operations.rs`** - Repository configuration management demonstration: user setup, configuration values, and repository-scoped settings
- **`commit_history.rs`** - Comprehensive commit history & log operations showing all querying APIs, filtering, analysis, and advanced LogOptions usage
- **`remote_operations.rs`** - Complete remote management demonstration: add, remove, rename remotes, fetch/push operations with options, and network operations
- **`file_lifecycle_operations.rs`** - Comprehensive file management demonstration: restore, reset, remove, move operations, .gitignore management, and advanced file lifecycle workflows
- **`error_handling.rs`** - Comprehensive error handling patterns showing GitError variants, recovery strategies, and best practices

All examples use OS-appropriate temporary directories and include automatic cleanup for safe execution.

## Testing

Run the test suite:

```bash
cargo test
```

All tests create temporary repositories in OS-appropriate temporary directories and clean up after themselves.

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
cargo run --example config_operations
cargo run --example commit_history
cargo run --example remote_operations
cargo run --example file_lifecycle_operations
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
- [ ] Tag operations (create, list, delete, push tags)
- [ ] Stash operations (save, apply, pop, list)
- [ ] Merge and rebase operations
- [ ] Diff operations
- [ ] Repository analysis (blame, statistics, health check)

## Status

rustic-git provides a complete git workflow including repository management, status checking, staging operations, commits, branch operations, commit history analysis, remote management, network operations, and comprehensive file lifecycle management.
