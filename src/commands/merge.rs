//! Git merge operations
//!
//! This module provides functionality for merging branches and handling merge conflicts.
//! It supports different merge strategies and fast-forward modes with comprehensive type safety.
//!
//! # Examples
//!
//! ```rust,no_run
//! use rustic_git::{Repository, MergeOptions, MergeStatus, FastForwardMode};
//!
//! let repo = Repository::open(".")?;
//!
//! // Simple merge
//! let status = repo.merge("feature-branch")?;
//! match status {
//!     MergeStatus::Success(hash) => println!("Merge commit: {}", hash),
//!     MergeStatus::FastForward(hash) => println!("Fast-forwarded to: {}", hash),
//!     MergeStatus::UpToDate => println!("Already up to date"),
//!     MergeStatus::Conflicts(files) => {
//!         println!("Conflicts in files: {:?}", files);
//!         // Resolve conflicts manually, then commit
//!     }
//! }
//!
//! // Merge with options
//! let options = MergeOptions::new()
//!     .with_fast_forward(FastForwardMode::Never)
//!     .with_message("Merge feature branch".to_string());
//! let status = repo.merge_with_options("feature-branch", options)?;
//!
//! # Ok::<(), rustic_git::GitError>(())
//! ```

use crate::error::{GitError, Result};
use crate::repository::Repository;
use crate::types::Hash;
use crate::utils::{git, git_raw};
use std::path::{Path, PathBuf};

/// The result of a merge operation
#[derive(Debug, Clone, PartialEq)]
pub enum MergeStatus {
    /// Merge completed successfully with a new merge commit
    Success(Hash),
    /// Fast-forward merge completed (no merge commit created)
    FastForward(Hash),
    /// Already up to date, no changes needed
    UpToDate,
    /// Merge has conflicts that need manual resolution
    Conflicts(Vec<PathBuf>),
}

/// Fast-forward merge behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastForwardMode {
    /// Allow fast-forward when possible (default)
    Auto,
    /// Only fast-forward, fail if merge commit would be needed
    Only,
    /// Never fast-forward, always create merge commit
    Never,
}

impl FastForwardMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            FastForwardMode::Auto => "",
            FastForwardMode::Only => "--ff-only",
            FastForwardMode::Never => "--no-ff",
        }
    }
}

/// Merge strategy options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Default recursive strategy
    Recursive,
    /// Ours strategy (favor our changes)
    Ours,
    /// Theirs strategy (favor their changes)
    Theirs,
}

impl MergeStrategy {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MergeStrategy::Recursive => "recursive",
            MergeStrategy::Ours => "ours",
            MergeStrategy::Theirs => "theirs",
        }
    }
}

/// Options for merge operations
#[derive(Debug, Clone)]
pub struct MergeOptions {
    fast_forward: FastForwardMode,
    strategy: Option<MergeStrategy>,
    commit_message: Option<String>,
    no_commit: bool,
}

impl MergeOptions {
    /// Create new MergeOptions with default settings
    pub fn new() -> Self {
        Self {
            fast_forward: FastForwardMode::Auto,
            strategy: None,
            commit_message: None,
            no_commit: false,
        }
    }

    /// Set the fast-forward mode
    pub fn with_fast_forward(mut self, mode: FastForwardMode) -> Self {
        self.fast_forward = mode;
        self
    }

    /// Set the merge strategy
    pub fn with_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Set a custom commit message for the merge
    pub fn with_message(mut self, message: String) -> Self {
        self.commit_message = Some(message);
        self
    }

    /// Perform merge but don't automatically commit
    pub fn with_no_commit(mut self) -> Self {
        self.no_commit = true;
        self
    }
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Perform a merge operation
pub fn merge<P: AsRef<Path>>(
    repo_path: P,
    branch: &str,
    options: &MergeOptions,
) -> Result<MergeStatus> {
    let mut args = vec!["merge"];

    // Add fast-forward option if not auto
    let ff_option = options.fast_forward.as_str();
    if !ff_option.is_empty() {
        args.push(ff_option);
    }

    // Add strategy if specified
    if let Some(strategy) = options.strategy {
        args.push("-s");
        args.push(strategy.as_str());
    }

    // Add no-commit option if specified
    if options.no_commit {
        args.push("--no-commit");
    }

    // Add custom commit message if specified
    if let Some(ref message) = options.commit_message {
        args.push("-m");
        args.push(message);
    }

    // Add the branch to merge
    args.push(branch);

    let output = git_raw(&args, Some(repo_path.as_ref()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Parse the output to determine merge status
        if stdout.contains("Already up to date") || stdout.contains("Already up-to-date") {
            Ok(MergeStatus::UpToDate)
        } else if stdout.contains("Fast-forward") {
            // Extract the hash from fast-forward output
            if let Some(hash_line) = stdout.lines().find(|line| line.contains(".."))
                && let Some(hash_part) = hash_line.split("..").nth(1)
                && let Some(hash_str) = hash_part.split_whitespace().next()
            {
                let hash = Hash::from(hash_str);
                return Ok(MergeStatus::FastForward(hash));
            }
            // Fallback: get current HEAD
            let head_output = git(&["rev-parse", "HEAD"], Some(repo_path.as_ref()))?;
            let hash = Hash::from(head_output.trim());
            Ok(MergeStatus::FastForward(hash))
        } else {
            // Regular merge success - get the merge commit hash
            let head_output = git(&["rev-parse", "HEAD"], Some(repo_path.as_ref()))?;
            let hash = Hash::from(head_output.trim());
            Ok(MergeStatus::Success(hash))
        }
    } else if stderr.contains("CONFLICT")
        || stderr.contains("Automatic merge failed")
        || stdout.contains("CONFLICT")
        || stdout.contains("Automatic merge failed")
    {
        // Merge has conflicts
        let conflicts = extract_conflicted_files(repo_path.as_ref())?;
        Ok(MergeStatus::Conflicts(conflicts))
    } else {
        // Other error
        Err(GitError::CommandFailed(format!(
            "git {} failed: stdout='{}' stderr='{}'",
            args.join(" "),
            stdout,
            stderr
        )))
    }
}

/// Extract list of files with conflicts
fn extract_conflicted_files<P: AsRef<Path>>(repo_path: P) -> Result<Vec<PathBuf>> {
    let output = git(
        &["diff", "--name-only", "--diff-filter=U"],
        Some(repo_path.as_ref()),
    )?;

    let conflicts: Vec<PathBuf> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| PathBuf::from(line.trim()))
        .collect();

    Ok(conflicts)
}

/// Check if a merge is currently in progress
pub fn merge_in_progress<P: AsRef<Path>>(repo_path: P) -> Result<bool> {
    let git_dir = repo_path.as_ref().join(".git");
    let merge_head = git_dir.join("MERGE_HEAD");
    Ok(merge_head.exists())
}

/// Abort an in-progress merge
pub fn abort_merge<P: AsRef<Path>>(repo_path: P) -> Result<()> {
    git(&["merge", "--abort"], Some(repo_path.as_ref()))?;
    Ok(())
}

impl Repository {
    /// Merge the specified branch into the current branch.
    ///
    /// Performs a merge using default options (allow fast-forward, no custom message).
    ///
    /// # Arguments
    ///
    /// * `branch` - The name of the branch to merge into the current branch
    ///
    /// # Returns
    ///
    /// A `Result` containing the `MergeStatus` which indicates the outcome of the merge.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, MergeStatus};
    ///
    /// let repo = Repository::open(".")?;
    /// match repo.merge("feature-branch")? {
    ///     MergeStatus::Success(hash) => println!("Merge commit: {}", hash),
    ///     MergeStatus::FastForward(hash) => println!("Fast-forwarded to: {}", hash),
    ///     MergeStatus::UpToDate => println!("Already up to date"),
    ///     MergeStatus::Conflicts(files) => println!("Conflicts in: {:?}", files),
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn merge(&self, branch: &str) -> Result<MergeStatus> {
        Self::ensure_git()?;
        merge(self.repo_path(), branch, &MergeOptions::new())
    }

    /// Merge the specified branch with custom options.
    ///
    /// Provides full control over merge behavior including fast-forward mode,
    /// merge strategy, and commit message.
    ///
    /// # Arguments
    ///
    /// * `branch` - The name of the branch to merge into the current branch
    /// * `options` - Merge options controlling the merge behavior
    ///
    /// # Returns
    ///
    /// A `Result` containing the `MergeStatus` which indicates the outcome of the merge.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, MergeOptions, FastForwardMode};
    ///
    /// let repo = Repository::open(".")?;
    /// let options = MergeOptions::new()
    ///     .with_fast_forward(FastForwardMode::Never)
    ///     .with_message("Merge feature into main".to_string());
    ///
    /// let status = repo.merge_with_options("feature-branch", options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn merge_with_options(&self, branch: &str, options: MergeOptions) -> Result<MergeStatus> {
        Self::ensure_git()?;
        merge(self.repo_path(), branch, &options)
    }

    /// Check if a merge is currently in progress.
    ///
    /// Returns `true` if there is an ongoing merge that needs to be completed or aborted.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating whether a merge is in progress.
    pub fn merge_in_progress(&self) -> Result<bool> {
        Self::ensure_git()?;
        merge_in_progress(self.repo_path())
    }

    /// Abort an in-progress merge.
    ///
    /// Cancels the current merge operation and restores the repository to the state
    /// before the merge was started. This is useful when merge conflicts occur and
    /// you want to cancel the merge instead of resolving conflicts.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure of the abort operation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// if repo.merge_in_progress()? {
    ///     repo.abort_merge()?;
    ///     println!("Merge aborted");
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn abort_merge(&self) -> Result<()> {
        Self::ensure_git()?;
        abort_merge(self.repo_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Repository;
    use std::path::PathBuf;
    use std::{env, fs};

    fn create_test_repo(test_name: &str) -> (PathBuf, Repository) {
        let temp_dir = env::temp_dir().join(format!("rustic_git_merge_test_{}", test_name));

        // Clean up if exists
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }

        let repo = Repository::init(&temp_dir, false).unwrap();

        // Configure git user for testing
        repo.config()
            .set_user("Test User", "test@example.com")
            .unwrap();

        (temp_dir, repo)
    }

    fn create_file_and_commit(
        repo: &Repository,
        temp_dir: &Path,
        filename: &str,
        content: &str,
        message: &str,
    ) -> String {
        let file_path = temp_dir.join(filename);
        fs::write(&file_path, content).unwrap();
        repo.add(&[filename]).unwrap();
        repo.commit(message).unwrap().to_string()
    }

    #[test]
    fn test_fast_forward_mode_as_str() {
        assert_eq!(FastForwardMode::Auto.as_str(), "");
        assert_eq!(FastForwardMode::Only.as_str(), "--ff-only");
        assert_eq!(FastForwardMode::Never.as_str(), "--no-ff");
    }

    #[test]
    fn test_merge_strategy_as_str() {
        assert_eq!(MergeStrategy::Recursive.as_str(), "recursive");
        assert_eq!(MergeStrategy::Ours.as_str(), "ours");
        assert_eq!(MergeStrategy::Theirs.as_str(), "theirs");
    }

    #[test]
    fn test_merge_options_builder() {
        let options = MergeOptions::new()
            .with_fast_forward(FastForwardMode::Never)
            .with_strategy(MergeStrategy::Ours)
            .with_message("Custom merge message".to_string())
            .with_no_commit();

        assert_eq!(options.fast_forward, FastForwardMode::Never);
        assert_eq!(options.strategy, Some(MergeStrategy::Ours));
        assert_eq!(
            options.commit_message,
            Some("Custom merge message".to_string())
        );
        assert!(options.no_commit);
    }

    #[test]
    fn test_merge_fast_forward() {
        let (temp_dir, repo) = create_test_repo("merge_fast_forward");

        // Create initial commit on master
        create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "Initial commit");

        // Create and switch to feature branch
        repo.checkout_new("feature", None).unwrap();

        // Add commit to feature branch
        create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Feature commit");

        // Switch back to master
        let branches = repo.branches().unwrap();
        let master_branch = branches.find("master").unwrap();
        repo.checkout(master_branch).unwrap();

        // Merge feature branch (should fast-forward)
        let status = repo.merge("feature").unwrap();

        match status {
            MergeStatus::FastForward(_) => {
                // Verify file2.txt exists
                assert!(temp_dir.join("file2.txt").exists());
            }
            _ => panic!("Expected fast-forward merge, got: {:?}", status),
        }

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_merge_no_fast_forward() {
        let (temp_dir, repo) = create_test_repo("merge_no_ff");

        // Create initial commit on master
        create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "Initial commit");

        // Create and switch to feature branch
        repo.checkout_new("feature", None).unwrap();

        // Add commit to feature branch
        create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Feature commit");

        // Switch back to master
        let branches = repo.branches().unwrap();
        let master_branch = branches.find("master").unwrap();
        repo.checkout(master_branch).unwrap();

        // Merge feature branch with no fast-forward
        let options = MergeOptions::new().with_fast_forward(FastForwardMode::Never);
        let status = repo.merge_with_options("feature", options).unwrap();

        match status {
            MergeStatus::Success(_) => {
                // Verify merge commit was created and both files exist
                assert!(temp_dir.join("file1.txt").exists());
                assert!(temp_dir.join("file2.txt").exists());
            }
            _ => panic!("Expected merge commit, got: {:?}", status),
        }

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_merge_up_to_date() {
        let (temp_dir, repo) = create_test_repo("merge_up_to_date");

        // Create initial commit
        create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "Initial commit");

        // Create feature branch but don't add commits
        repo.checkout_new("feature", None).unwrap();
        let branches = repo.branches().unwrap();
        let master_branch = branches.find("master").unwrap();
        repo.checkout(master_branch).unwrap();

        // Try to merge feature (should be up to date)
        let status = repo.merge("feature").unwrap();

        assert_eq!(status, MergeStatus::UpToDate);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_merge_in_progress_false() {
        let (temp_dir, repo) = create_test_repo("merge_in_progress_false");

        // Create initial commit
        create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "Initial commit");

        // Check merge in progress (should be false)
        assert!(!repo.merge_in_progress().unwrap());

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_merge_conflicts() {
        let (temp_dir, repo) = create_test_repo("merge_conflicts");

        // Create initial commit
        create_file_and_commit(
            &repo,
            &temp_dir,
            "file1.txt",
            "line1\nline2\nline3",
            "Initial commit",
        );

        // Create and switch to feature branch
        repo.checkout_new("feature", None).unwrap();

        // Modify file in feature branch
        create_file_and_commit(
            &repo,
            &temp_dir,
            "file1.txt",
            "line1\nfeature_line\nline3",
            "Feature changes",
        );

        // Switch back to master and modify same file
        let branches = repo.branches().unwrap();
        let master_branch = branches.find("master").unwrap();
        repo.checkout(master_branch).unwrap();
        create_file_and_commit(
            &repo,
            &temp_dir,
            "file1.txt",
            "line1\nmaster_line\nline3",
            "Master changes",
        );

        // Try to merge feature branch (should have conflicts)
        let status = repo.merge("feature").unwrap();

        match status {
            MergeStatus::Conflicts(files) => {
                assert!(!files.is_empty());
                assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.txt"));

                // Verify merge is in progress
                assert!(repo.merge_in_progress().unwrap());

                // Abort the merge
                repo.abort_merge().unwrap();

                // Verify merge is no longer in progress
                assert!(!repo.merge_in_progress().unwrap());
            }
            _ => panic!("Expected conflicts, got: {:?}", status),
        }

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_merge_with_custom_message() {
        let (temp_dir, repo) = create_test_repo("merge_custom_message");

        // Create initial commit on master
        create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "Initial commit");

        // Create and switch to feature branch
        repo.checkout_new("feature", None).unwrap();

        // Add commit to feature branch
        create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Feature commit");

        // Switch back to master
        let branches = repo.branches().unwrap();
        let master_branch = branches.find("master").unwrap();
        repo.checkout(master_branch).unwrap();

        // Merge with custom message and no fast-forward
        let options = MergeOptions::new()
            .with_fast_forward(FastForwardMode::Never)
            .with_message("Custom merge commit message".to_string());

        let status = repo.merge_with_options("feature", options).unwrap();

        match status {
            MergeStatus::Success(_) => {
                // Get the latest commit message
                let commits = repo.recent_commits(1).unwrap();
                let latest_commit = commits.iter().next().unwrap();
                assert!(
                    latest_commit
                        .message
                        .subject
                        .contains("Custom merge commit message")
                );
            }
            _ => panic!("Expected successful merge, got: {:?}", status),
        }

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
