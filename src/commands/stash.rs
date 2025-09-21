//! Git stash operations
//!
//! This module provides functionality for stashing, listing, applying, and managing Git stashes.
//! It supports comprehensive stash management with type-safe operations.
//!
//! # Examples
//!
//! ```rust,no_run
//! use rustic_git::{Repository, StashOptions, StashApplyOptions};
//!
//! let repo = Repository::open(".")?;
//!
//! // Save current changes to stash
//! let stash = repo.stash_save("Work in progress")?;
//! println!("Stashed: {}", stash.message);
//!
//! // List all stashes
//! let stashes = repo.stash_list()?;
//! for stash in stashes.iter() {
//!     println!("{}: {}", stash.index, stash.message);
//! }
//!
//! // Apply most recent stash
//! if let Some(latest) = stashes.latest() {
//!     repo.stash_apply(latest.index, StashApplyOptions::new())?;
//! }
//!
//! # Ok::<(), rustic_git::GitError>(())
//! ```

use crate::error::{GitError, Result};
use crate::repository::Repository;
use crate::types::Hash;
use crate::utils::{git, parse_unix_timestamp};
use chrono::{DateTime, Utc};
use std::fmt;
use std::path::PathBuf;

/// Represents a Git stash entry
#[derive(Debug, Clone, PartialEq)]
pub struct Stash {
    /// The stash index (0 is most recent)
    pub index: usize,
    /// The stash message
    pub message: String,
    /// The commit hash of the stash
    pub hash: Hash,
    /// The branch name when stash was created
    pub branch: String,
    /// When the stash was created
    pub timestamp: DateTime<Utc>,
}

impl fmt::Display for Stash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stash@{{{}}}: {}", self.index, self.message)
    }
}

/// A collection of stashes with efficient iteration and filtering methods
#[derive(Debug, Clone)]
pub struct StashList {
    stashes: Box<[Stash]>,
}

impl StashList {
    /// Create a new StashList from a vector of stashes
    pub fn new(stashes: Vec<Stash>) -> Self {
        Self {
            stashes: stashes.into_boxed_slice(),
        }
    }

    /// Get an iterator over all stashes
    pub fn iter(&self) -> impl Iterator<Item = &Stash> + '_ {
        self.stashes.iter()
    }

    /// Get the most recent stash (index 0)
    pub fn latest(&self) -> Option<&Stash> {
        self.stashes.first()
    }

    /// Get stash by index
    pub fn get(&self, index: usize) -> Option<&Stash> {
        self.stashes.iter().find(|stash| stash.index == index)
    }

    /// Find stashes whose messages contain the given substring
    pub fn find_containing<'a>(
        &'a self,
        substring: &'a str,
    ) -> impl Iterator<Item = &'a Stash> + 'a {
        self.stashes
            .iter()
            .filter(move |stash| stash.message.contains(substring))
    }

    /// Get stashes created on a specific branch
    pub fn for_branch<'a>(&'a self, branch: &'a str) -> impl Iterator<Item = &'a Stash> + 'a {
        self.stashes
            .iter()
            .filter(move |stash| stash.branch == branch)
    }

    /// Get the total number of stashes
    pub fn len(&self) -> usize {
        self.stashes.len()
    }

    /// Check if the stash list is empty
    pub fn is_empty(&self) -> bool {
        self.stashes.is_empty()
    }
}

/// Options for creating stashes
#[derive(Debug, Clone, Default)]
pub struct StashOptions {
    /// Include untracked files in the stash
    pub include_untracked: bool,
    /// Include ignored files in the stash
    pub include_all: bool,
    /// Keep staged changes in the index
    pub keep_index: bool,
    /// Create a patch-mode stash (interactive)
    pub patch: bool,
    /// Only stash staged changes
    pub staged_only: bool,
    /// Paths to specifically stash
    pub paths: Vec<PathBuf>,
}

impl StashOptions {
    /// Create new default stash options
    pub fn new() -> Self {
        Self::default()
    }

    /// Include untracked files in the stash
    pub fn with_untracked(mut self) -> Self {
        self.include_untracked = true;
        self
    }

    /// Include all files (untracked and ignored) in the stash
    pub fn with_all(mut self) -> Self {
        self.include_all = true;
        self.include_untracked = true; // --all implies --include-untracked
        self
    }

    /// Keep staged changes in the index after stashing
    pub fn with_keep_index(mut self) -> Self {
        self.keep_index = true;
        self
    }

    /// Create an interactive patch-mode stash
    pub fn with_patch(mut self) -> Self {
        self.patch = true;
        self
    }

    /// Only stash staged changes
    pub fn with_staged_only(mut self) -> Self {
        self.staged_only = true;
        self
    }

    /// Specify paths to stash
    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.paths = paths;
        self
    }
}

/// Options for applying stashes
#[derive(Debug, Clone, Default)]
pub struct StashApplyOptions {
    /// Restore staged changes to the index
    pub restore_index: bool,
    /// Suppress output messages
    pub quiet: bool,
}

impl StashApplyOptions {
    /// Create new default apply options
    pub fn new() -> Self {
        Self::default()
    }

    /// Restore staged changes to the index when applying
    pub fn with_index(mut self) -> Self {
        self.restore_index = true;
        self
    }

    /// Suppress output messages
    pub fn with_quiet(mut self) -> Self {
        self.quiet = true;
        self
    }
}

impl Repository {
    /// List all stashes in the repository
    ///
    /// Returns a `StashList` containing all stashes sorted by recency (most recent first).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// let stashes = repo.stash_list()?;
    ///
    /// println!("Found {} stashes:", stashes.len());
    /// for stash in stashes.iter() {
    ///     println!("  {}: {}", stash.index, stash.message);
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_list(&self) -> Result<StashList> {
        Self::ensure_git()?;

        let output = git(
            &["stash", "list", "--format=%gd %H %ct %gs"],
            Some(self.repo_path()),
        )?;

        if output.trim().is_empty() {
            return Ok(StashList::new(vec![]));
        }

        let mut stashes = Vec::new();

        for (index, line) in output.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(stash) = parse_stash_line(index, line) {
                stashes.push(stash);
            }
        }

        Ok(StashList::new(stashes))
    }

    /// Save current changes to a new stash with a message
    ///
    /// This is equivalent to `git stash push -m "message"`.
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message for the stash
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// let stash = repo.stash_save("Work in progress on feature X")?;
    /// println!("Created stash: {}", stash.message);
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_save(&self, message: &str) -> Result<Stash> {
        let options = StashOptions::new();
        self.stash_push(message, options)
    }

    /// Create a stash with advanced options
    ///
    /// # Arguments
    ///
    /// * `message` - A descriptive message for the stash
    /// * `options` - Stash creation options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, StashOptions};
    ///
    /// let repo = Repository::open(".")?;
    ///
    /// // Stash including untracked files
    /// let options = StashOptions::new()
    ///     .with_untracked()
    ///     .with_keep_index();
    /// let stash = repo.stash_push("WIP with untracked files", options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_push(&self, message: &str, options: StashOptions) -> Result<Stash> {
        Self::ensure_git()?;

        let mut args = vec!["stash", "push"];

        if options.include_all {
            args.push("--all");
        } else if options.include_untracked {
            args.push("--include-untracked");
        }

        if options.keep_index {
            args.push("--keep-index");
        }

        if options.patch {
            args.push("--patch");
        }

        if options.staged_only {
            args.push("--staged");
        }

        args.extend(&["-m", message]);

        // Add paths if specified
        if !options.paths.is_empty() {
            args.push("--");
            for path in &options.paths {
                if let Some(path_str) = path.to_str() {
                    args.push(path_str);
                }
            }
        }

        git(&args, Some(self.repo_path()))?;

        // Get the newly created stash (it will be at index 0)
        let stashes = self.stash_list()?;
        stashes.latest().cloned().ok_or_else(|| {
            GitError::CommandFailed(
                "Failed to create stash or retrieve stash information".to_string(),
            )
        })
    }

    /// Apply a stash without removing it from the stash list
    ///
    /// # Arguments
    ///
    /// * `index` - The stash index to apply (0 is most recent)
    /// * `options` - Apply options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, StashApplyOptions};
    ///
    /// let repo = Repository::open(".")?;
    /// let options = StashApplyOptions::new().with_index();
    /// repo.stash_apply(0, options)?; // Apply most recent stash
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_apply(&self, index: usize, options: StashApplyOptions) -> Result<()> {
        Self::ensure_git()?;

        let mut args = vec!["stash", "apply"];

        if options.restore_index {
            args.push("--index");
        }

        if options.quiet {
            args.push("--quiet");
        }

        let stash_ref = format!("stash@{{{}}}", index);
        args.push(&stash_ref);

        git(&args, Some(self.repo_path()))?;
        Ok(())
    }

    /// Apply a stash and remove it from the stash list
    ///
    /// # Arguments
    ///
    /// * `index` - The stash index to pop (0 is most recent)
    /// * `options` - Apply options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, StashApplyOptions};
    ///
    /// let repo = Repository::open(".")?;
    /// repo.stash_pop(0, StashApplyOptions::new())?; // Pop most recent stash
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_pop(&self, index: usize, options: StashApplyOptions) -> Result<()> {
        Self::ensure_git()?;

        let mut args = vec!["stash", "pop"];

        if options.restore_index {
            args.push("--index");
        }

        if options.quiet {
            args.push("--quiet");
        }

        let stash_ref = format!("stash@{{{}}}", index);
        args.push(&stash_ref);

        git(&args, Some(self.repo_path()))?;
        Ok(())
    }

    /// Show the contents of a stash
    ///
    /// # Arguments
    ///
    /// * `index` - The stash index to show (0 is most recent)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// let stash_info = repo.stash_show(0)?;
    /// println!("Stash contents:\n{}", stash_info);
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_show(&self, index: usize) -> Result<String> {
        Self::ensure_git()?;

        let output = git(
            &["stash", "show", &format!("stash@{{{}}}", index)],
            Some(self.repo_path()),
        )?;

        Ok(output)
    }

    /// Delete a specific stash
    ///
    /// # Arguments
    ///
    /// * `index` - The stash index to delete
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// repo.stash_drop(1)?; // Delete second most recent stash
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_drop(&self, index: usize) -> Result<()> {
        Self::ensure_git()?;

        git(
            &["stash", "drop", &format!("stash@{{{}}}", index)],
            Some(self.repo_path()),
        )?;

        Ok(())
    }

    /// Clear all stashes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// repo.stash_clear()?; // Remove all stashes
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn stash_clear(&self) -> Result<()> {
        Self::ensure_git()?;

        git(&["stash", "clear"], Some(self.repo_path()))?;
        Ok(())
    }
}

/// Parse a stash list line into a Stash struct
fn parse_stash_line(index: usize, line: &str) -> Result<Stash> {
    // Format: "stash@{0} hash timestamp On branch: message"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();

    if parts.len() < 4 {
        return Err(GitError::CommandFailed(format!(
            "Invalid stash list format: expected 4 parts, got {}",
            parts.len()
        )));
    }

    let hash = Hash::from(parts[1]);

    // Parse timestamp
    let timestamp = parse_unix_timestamp(parts[2]).unwrap_or_else(|_| Utc::now());

    // Extract branch name and message from parts[3] (should be "On branch: message")
    let remainder = parts[3];
    if remainder.is_empty() {
        return Err(GitError::CommandFailed(
            "Invalid stash format: missing branch and message information".to_string(),
        ));
    }

    let (branch, message) = if let Some(colon_pos) = remainder.find(':') {
        let branch_part = &remainder[..colon_pos];
        let message_part = &remainder[colon_pos + 1..].trim();

        // Extract branch name from "On branch_name" or "WIP on branch_name"
        let branch = if let Some(stripped) = branch_part.strip_prefix("On ") {
            stripped.to_string()
        } else if let Some(stripped) = branch_part.strip_prefix("WIP on ") {
            stripped.to_string()
        } else {
            "unknown".to_string()
        };

        (branch, message_part.to_string())
    } else {
        ("unknown".to_string(), remainder.to_string())
    };

    Ok(Stash {
        index,
        message,
        hash,
        branch,
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn create_test_repo() -> (Repository, std::path::PathBuf) {
        use std::thread;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let test_path = env::temp_dir().join(format!(
            "rustic_git_stash_test_{}_{}_{}",
            std::process::id(),
            timestamp,
            thread_id.replace("ThreadId(", "").replace(")", "")
        ));

        // Ensure clean state
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Configure git user for commits
        repo.config()
            .set_user("Test User", "test@example.com")
            .unwrap();

        (repo, test_path)
    }

    fn create_test_commit(
        repo: &Repository,
        test_path: &std::path::Path,
        filename: &str,
        content: &str,
    ) {
        fs::write(test_path.join(filename), content).unwrap();
        repo.add(&[filename]).unwrap();
        repo.commit(&format!("Add {}", filename)).unwrap();
    }

    #[test]
    fn test_stash_list_empty_repository() {
        let (repo, test_path) = create_test_repo();

        let stashes = repo.stash_list().unwrap();
        assert!(stashes.is_empty());
        assert_eq!(stashes.len(), 0);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_save_and_list() {
        let (repo, test_path) = create_test_repo();

        // Create initial commit
        create_test_commit(&repo, &test_path, "initial.txt", "initial content");

        // Make some changes (modify existing tracked file)
        fs::write(test_path.join("initial.txt"), "modified content").unwrap();

        // Stash the changes
        let stash = repo.stash_save("Test stash message").unwrap();
        assert_eq!(stash.message, "Test stash message");
        assert_eq!(stash.index, 0);

        // Verify stash exists in list
        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 1);
        assert!(stashes.latest().is_some());
        assert_eq!(stashes.latest().unwrap().message, "Test stash message");

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_push_with_options() {
        let (repo, test_path) = create_test_repo();

        // Create initial commit
        create_test_commit(&repo, &test_path, "initial.txt", "initial content");

        // Make some changes
        fs::write(test_path.join("initial.txt"), "modified initial").unwrap(); // Modify tracked file
        fs::write(test_path.join("tracked.txt"), "tracked content").unwrap();
        fs::write(test_path.join("untracked.txt"), "untracked content").unwrap();

        // Stage the files
        repo.add(&["tracked.txt"]).unwrap();

        // Stash with options
        let options = StashOptions::new().with_untracked().with_keep_index();
        let stash = repo.stash_push("Stash with options", options).unwrap();

        assert_eq!(stash.message, "Stash with options");

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_apply_and_pop() {
        let (repo, test_path) = create_test_repo();

        // Create initial commit
        create_test_commit(&repo, &test_path, "initial.txt", "initial content");

        // Make and stash changes (modify existing tracked file)
        fs::write(test_path.join("initial.txt"), "modified content").unwrap();
        repo.stash_save("Test stash").unwrap();

        // Verify file content is reverted after stash
        let content = fs::read_to_string(test_path.join("initial.txt")).unwrap();
        assert_eq!(content, "initial content");

        // Apply stash
        repo.stash_apply(0, StashApplyOptions::new()).unwrap();

        // Verify file content is back to modified
        let content = fs::read_to_string(test_path.join("initial.txt")).unwrap();
        assert_eq!(content, "modified content");

        // Stash should still exist
        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 1);

        // Reset working tree and pop
        fs::write(test_path.join("initial.txt"), "initial content").unwrap(); // Reset to original
        repo.stash_pop(0, StashApplyOptions::new()).unwrap();

        // File content should be modified again and stash should be gone
        let content = fs::read_to_string(test_path.join("initial.txt")).unwrap();
        assert_eq!(content, "modified content");
        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 0);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_drop_and_clear() {
        let (repo, test_path) = create_test_repo();

        // Create initial commit
        create_test_commit(&repo, &test_path, "initial.txt", "initial content");

        // Create multiple stashes by modifying the tracked file
        for i in 1..=3 {
            fs::write(test_path.join("initial.txt"), format!("content {}", i)).unwrap();
            repo.stash_save(&format!("Stash {}", i)).unwrap();
        }

        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 3);

        // Drop middle stash
        repo.stash_drop(1).unwrap();
        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 2);

        // Clear all stashes
        repo.stash_clear().unwrap();
        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 0);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_show() {
        let (repo, test_path) = create_test_repo();

        // Create initial commit
        create_test_commit(&repo, &test_path, "initial.txt", "initial content");

        // Make changes and stash (modify existing tracked file)
        fs::write(test_path.join("initial.txt"), "modified content").unwrap();
        repo.stash_save("Test stash").unwrap();

        // Show stash contents
        let show_output = repo.stash_show(0).unwrap();
        assert!(!show_output.is_empty());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_list_filtering() {
        let (repo, test_path) = create_test_repo();

        // Create initial commit
        create_test_commit(&repo, &test_path, "initial.txt", "initial content");

        // Create stashes with different messages (modify existing tracked file)
        fs::write(test_path.join("initial.txt"), "content1").unwrap();
        repo.stash_save("feature work in progress").unwrap();

        fs::write(test_path.join("initial.txt"), "content2").unwrap();
        repo.stash_save("bugfix temporary save").unwrap();

        fs::write(test_path.join("initial.txt"), "content3").unwrap();
        repo.stash_save("feature enhancement").unwrap();

        let stashes = repo.stash_list().unwrap();
        assert_eq!(stashes.len(), 3);

        // Test filtering
        let feature_stashes: Vec<_> = stashes.find_containing("feature").collect();
        assert_eq!(feature_stashes.len(), 2);

        let bugfix_stashes: Vec<_> = stashes.find_containing("bugfix").collect();
        assert_eq!(bugfix_stashes.len(), 1);

        // Test get by index
        assert!(stashes.get(0).is_some());
        assert!(stashes.get(10).is_none());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_stash_options_builder() {
        let options = StashOptions::new()
            .with_untracked()
            .with_keep_index()
            .with_paths(vec!["file1.txt".into(), "file2.txt".into()]);

        assert!(options.include_untracked);
        assert!(options.keep_index);
        assert_eq!(options.paths.len(), 2);

        let apply_options = StashApplyOptions::new().with_index().with_quiet();

        assert!(apply_options.restore_index);
        assert!(apply_options.quiet);
    }

    #[test]
    fn test_stash_display() {
        let stash = Stash {
            index: 0,
            message: "Test stash message".to_string(),
            hash: Hash::from("abc123"),
            branch: "main".to_string(),
            timestamp: Utc::now(),
        };

        let display_str = format!("{}", stash);
        assert!(display_str.contains("stash@{0}"));
        assert!(display_str.contains("Test stash message"));
    }

    #[test]
    fn test_parse_stash_line_invalid_format() {
        // Test with insufficient parts
        let invalid_line = "stash@{0} abc123"; // Only 2 parts instead of 4
        let result = parse_stash_line(0, invalid_line);

        assert!(result.is_err());
        if let Err(GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("Invalid stash list format"));
            assert!(msg.contains("expected 4 parts"));
            assert!(msg.contains("got 2"));
        } else {
            panic!("Expected CommandFailed error with specific message");
        }
    }

    #[test]
    fn test_parse_stash_line_empty_remainder() {
        // Test with empty remainder part
        let invalid_line = "stash@{0} abc123 1234567890 "; // Empty 4th part
        let result = parse_stash_line(0, invalid_line);

        assert!(result.is_err());
        if let Err(GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("missing branch and message information"));
        } else {
            panic!("Expected CommandFailed error for empty remainder");
        }
    }

    #[test]
    fn test_parse_stash_line_valid_format() {
        // Test with valid format
        let valid_line = "stash@{0} abc123def456 1234567890 On master: test message";
        let result = parse_stash_line(0, valid_line);

        assert!(result.is_ok());
        let stash = result.unwrap();
        assert_eq!(stash.index, 0);
        assert_eq!(stash.hash.as_str(), "abc123def456");
        assert_eq!(stash.branch, "master");
        assert_eq!(stash.message, "test message");
    }
}
