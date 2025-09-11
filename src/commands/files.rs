//! File lifecycle operations for working with files in a Git repository.
//!
//! This module provides functionality for:
//! - Restoring files from different sources (checkout_file, restore)
//! - Unstaging files (reset_file)
//! - Removing files from repository (rm)
//! - Moving/renaming files (mv)
//! - Managing .gitignore patterns
//!
//! All operations follow Git's standard behavior and safety principles.

use crate::{Repository, Result, utils::git};
use std::path::Path;

/// Options for restore operations
#[derive(Debug, Clone, Default)]
pub struct RestoreOptions {
    /// Source to restore from (commit hash, branch name, or "HEAD")
    pub source: Option<String>,
    /// Restore staged files
    pub staged: bool,
    /// Restore working tree files
    pub worktree: bool,
}

impl RestoreOptions {
    /// Create new restore options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the source to restore from
    pub fn with_source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Enable restoring staged files
    pub fn with_staged(mut self) -> Self {
        self.staged = true;
        self
    }

    /// Enable restoring working tree files
    pub fn with_worktree(mut self) -> Self {
        self.worktree = true;
        self
    }
}

/// Options for file removal operations
#[derive(Debug, Clone, Default)]
pub struct RemoveOptions {
    /// Force removal of files
    pub force: bool,
    /// Remove files recursively (for directories)
    pub recursive: bool,
    /// Only remove from index, keep in working tree
    pub cached: bool,
    /// Don't fail if files don't match
    pub ignore_unmatch: bool,
}

impl RemoveOptions {
    /// Create new remove options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable force removal
    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Enable recursive removal
    pub fn with_recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Remove only from index, keep files in working tree
    pub fn with_cached(mut self) -> Self {
        self.cached = true;
        self
    }

    /// Don't fail if files don't match
    pub fn with_ignore_unmatch(mut self) -> Self {
        self.ignore_unmatch = true;
        self
    }
}

/// Options for move operations
#[derive(Debug, Clone, Default)]
pub struct MoveOptions {
    /// Force move even if destination exists
    pub force: bool,
    /// Show verbose output
    pub verbose: bool,
    /// Dry run - don't actually move files
    pub dry_run: bool,
}

impl MoveOptions {
    /// Create new move options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable force move
    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Enable verbose output
    pub fn with_verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Enable dry run mode
    pub fn with_dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

impl Repository {
    /// Restore file from HEAD, discarding local changes
    ///
    /// This is equivalent to `git checkout HEAD -- <file>` and will restore
    /// the file to its state in the last commit, discarding any local changes.
    ///
    /// # Arguments
    /// * `path` - Path to the file to restore
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_checkout_file");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Restore a modified file to its last committed state
    /// repo.checkout_file("modified_file.txt")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn checkout_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Repository::ensure_git()?;

        let path_str = path.as_ref().to_string_lossy();
        git(
            &["checkout", "HEAD", "--", &path_str],
            Some(self.repo_path()),
        )?;

        Ok(())
    }

    /// Restore files with advanced options
    ///
    /// This provides access to git's `restore` command with full control over
    /// source, staging area, and working tree restoration.
    ///
    /// # Arguments
    /// * `paths` - Paths to restore
    /// * `options` - Restore options
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::{Repository, RestoreOptions};
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_restore");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Restore from a specific commit
    /// let options = RestoreOptions::new()
    ///     .with_source("HEAD~1")
    ///     .with_worktree();
    /// repo.restore(&["file.txt"], options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn restore<P: AsRef<Path>>(&self, paths: &[P], options: RestoreOptions) -> Result<()> {
        Repository::ensure_git()?;

        let mut args = vec!["restore"];

        if let Some(ref source) = options.source {
            args.push("--source");
            args.push(source);
        }

        if options.staged {
            args.push("--staged");
        }

        if options.worktree {
            args.push("--worktree");
        }

        if !options.staged && !options.worktree {
            // Default to worktree if neither specified
            args.push("--worktree");
        }

        args.push("--");

        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .collect();
        let path_refs: Vec<&str> = path_strings.iter().map(String::as_str).collect();
        args.extend(path_refs);

        git(&args, Some(self.repo_path()))?;

        Ok(())
    }

    /// Unstage a specific file, removing it from the staging area
    ///
    /// This is equivalent to `git reset HEAD -- <file>` and removes the file
    /// from the staging area while keeping changes in the working directory.
    ///
    /// # Arguments
    /// * `path` - Path to the file to unstage
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_reset_file");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Unstage a previously staged file
    /// repo.reset_file("staged_file.txt")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn reset_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Repository::ensure_git()?;

        let path_str = path.as_ref().to_string_lossy();
        git(&["reset", "HEAD", "--", &path_str], Some(self.repo_path()))?;

        Ok(())
    }

    /// Remove files from the repository
    ///
    /// This removes files from both the working directory and the repository,
    /// equivalent to `git rm <files>`.
    ///
    /// # Arguments
    /// * `paths` - Paths to remove
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_rm");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Remove files from repository
    /// repo.rm(&["unwanted_file.txt", "old_dir/"])?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn rm<P: AsRef<Path>>(&self, paths: &[P]) -> Result<()> {
        self.rm_with_options(paths, RemoveOptions::new())
    }

    /// Remove files with advanced options
    ///
    /// This provides full control over file removal with options for force,
    /// recursive, cached-only removal, etc.
    ///
    /// # Arguments
    /// * `paths` - Paths to remove
    /// * `options` - Remove options
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::{Repository, RemoveOptions};
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_rm_options");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Remove from index only, keep files in working tree
    /// let options = RemoveOptions::new().with_cached();
    /// repo.rm_with_options(&["keep_local.txt"], options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn rm_with_options<P: AsRef<Path>>(
        &self,
        paths: &[P],
        options: RemoveOptions,
    ) -> Result<()> {
        Repository::ensure_git()?;

        let mut args = vec!["rm"];

        if options.force {
            args.push("--force");
        }

        if options.recursive {
            args.push("-r");
        }

        if options.cached {
            args.push("--cached");
        }

        if options.ignore_unmatch {
            args.push("--ignore-unmatch");
        }

        args.push("--");

        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .collect();
        let path_refs: Vec<&str> = path_strings.iter().map(String::as_str).collect();
        args.extend(path_refs);

        git(&args, Some(self.repo_path()))?;

        Ok(())
    }

    /// Move or rename a file or directory
    ///
    /// This is equivalent to `git mv <source> <destination>` and will move
    /// the file both in the working directory and in the repository.
    ///
    /// # Arguments
    /// * `source` - Source path
    /// * `destination` - Destination path
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_mv");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Rename a file
    /// repo.mv("old_name.txt", "new_name.txt")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn mv<P: AsRef<Path>, Q: AsRef<Path>>(&self, source: P, destination: Q) -> Result<()> {
        self.mv_with_options(source, destination, MoveOptions::new())
    }

    /// Move files with advanced options
    ///
    /// This provides full control over file moving with options for force,
    /// verbose output, and dry run mode.
    ///
    /// # Arguments
    /// * `source` - Source path
    /// * `destination` - Destination path
    /// * `options` - Move options
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::{Repository, MoveOptions};
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_mv_options");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Force move even if destination exists
    /// let options = MoveOptions::new().with_force();
    /// repo.mv_with_options("source.txt", "existing.txt", options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn mv_with_options<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        source: P,
        destination: Q,
        options: MoveOptions,
    ) -> Result<()> {
        Repository::ensure_git()?;

        let mut args = vec!["mv"];

        if options.force {
            args.push("-f");
        }

        if options.verbose {
            args.push("-v");
        }

        if options.dry_run {
            args.push("-n");
        }

        let source_str = source.as_ref().to_string_lossy();
        let dest_str = destination.as_ref().to_string_lossy();
        args.push(&source_str);
        args.push(&dest_str);

        git(&args, Some(self.repo_path()))?;

        Ok(())
    }

    /// Add patterns to .gitignore file
    ///
    /// This adds the specified patterns to the repository's .gitignore file,
    /// creating the file if it doesn't exist.
    ///
    /// # Arguments
    /// * `patterns` - Patterns to add to .gitignore
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_ignore_add");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Add patterns to .gitignore
    /// repo.ignore_add(&["*.tmp", "build/", "node_modules/"])?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn ignore_add(&self, patterns: &[&str]) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let gitignore_path = self.repo_path().join(".gitignore");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(gitignore_path)?;

        for pattern in patterns {
            writeln!(file, "{}", pattern)?;
        }

        Ok(())
    }

    /// Check if a file is ignored by .gitignore patterns
    ///
    /// This uses `git check-ignore` to determine if a file would be ignored
    /// by the current .gitignore patterns.
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// * `Ok(true)` if the file is ignored
    /// * `Ok(false)` if the file is not ignored
    /// * `Err(GitError)` if the command fails
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_ignore_check");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // Check if a file is ignored
    /// let is_ignored = repo.ignore_check("temp_file.tmp")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn ignore_check<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        Repository::ensure_git()?;

        let path_str = path.as_ref().to_string_lossy();

        match git(&["check-ignore", &path_str], Some(self.repo_path())) {
            Ok(_) => Ok(true),   // File is ignored
            Err(_) => Ok(false), // File is not ignored (check-ignore returns non-zero)
        }
    }

    /// List current ignore patterns from .gitignore
    ///
    /// This reads the .gitignore file and returns all non-empty, non-comment lines.
    ///
    /// # Returns
    /// * Vector of ignore patterns
    ///
    /// # Example
    /// ```rust,no_run
    /// use rustic_git::Repository;
    /// # use std::env;
    /// # let repo_path = env::temp_dir().join("test_ignore_list");
    /// # std::fs::create_dir_all(&repo_path).unwrap();
    /// # let repo = Repository::init(&repo_path, false).unwrap();
    ///
    /// // List all ignore patterns
    /// let patterns = repo.ignore_list()?;
    /// for pattern in patterns {
    ///     println!("Ignoring: {}", pattern);
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn ignore_list(&self) -> Result<Vec<String>> {
        use std::fs;

        let gitignore_path = self.repo_path().join(".gitignore");

        if !gitignore_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(gitignore_path)?;
        let patterns: Vec<String> = content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(String::from)
            .collect();

        Ok(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    fn create_test_repo() -> (Repository, std::path::PathBuf) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo_path = env::temp_dir().join(format!(
            "rustic_git_files_test_{}_{}",
            std::process::id(),
            timestamp
        ));

        // Ensure directory doesn't exist
        if repo_path.exists() {
            let _ = fs::remove_dir_all(&repo_path);
        }
        fs::create_dir_all(&repo_path).unwrap();

        let repo = Repository::init(&repo_path, false).unwrap();

        // Set up git user for tests
        repo.config()
            .set_user("Test User", "test@example.com")
            .unwrap();

        (repo, repo_path)
    }

    #[test]
    fn test_restore_options_builder() {
        let options = RestoreOptions::new()
            .with_source("main")
            .with_staged()
            .with_worktree();

        assert_eq!(options.source, Some("main".to_string()));
        assert!(options.staged);
        assert!(options.worktree);
    }

    #[test]
    fn test_remove_options_builder() {
        let options = RemoveOptions::new()
            .with_force()
            .with_recursive()
            .with_cached()
            .with_ignore_unmatch();

        assert!(options.force);
        assert!(options.recursive);
        assert!(options.cached);
        assert!(options.ignore_unmatch);
    }

    #[test]
    fn test_move_options_builder() {
        let options = MoveOptions::new()
            .with_force()
            .with_verbose()
            .with_dry_run();

        assert!(options.force);
        assert!(options.verbose);
        assert!(options.dry_run);
    }

    #[test]
    fn test_checkout_file() {
        let (repo, repo_path) = create_test_repo();

        // Create and commit a file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "original content").unwrap();
        repo.add(&["test.txt"]).unwrap();
        repo.commit("Add test file").unwrap();

        // Modify the file
        fs::write(&file_path, "modified content").unwrap();

        // Restore it
        repo.checkout_file("test.txt").unwrap();

        // Verify it's restored
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "original content");

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_reset_file() {
        let (repo, repo_path) = create_test_repo();

        // Create and commit a file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "content").unwrap();
        repo.add(&["test.txt"]).unwrap();
        repo.commit("Add test file").unwrap();

        // Modify and stage the file
        fs::write(&file_path, "modified content").unwrap();
        repo.add(&["test.txt"]).unwrap();

        // Reset the file (unstage)
        repo.reset_file("test.txt").unwrap();

        // Verify it's unstaged but modified in working tree
        let status = repo.status().unwrap();
        assert!(status.has_changes());

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_ignore_add_and_list() {
        let (repo, repo_path) = create_test_repo();

        // Initially no patterns
        let patterns = repo.ignore_list().unwrap();
        assert!(patterns.is_empty());

        // Add some patterns
        repo.ignore_add(&["*.tmp", "build/", "node_modules/"])
            .unwrap();

        // Verify patterns are added
        let patterns = repo.ignore_list().unwrap();
        assert_eq!(patterns.len(), 3);
        assert!(patterns.contains(&"*.tmp".to_string()));
        assert!(patterns.contains(&"build/".to_string()));
        assert!(patterns.contains(&"node_modules/".to_string()));

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_ignore_check() {
        let (repo, repo_path) = create_test_repo();

        // Add ignore pattern
        repo.ignore_add(&["*.tmp"]).unwrap();

        // Create files
        fs::write(repo_path.join("test.txt"), "content").unwrap();
        fs::write(repo_path.join("test.tmp"), "temp content").unwrap();

        // Check if files are ignored
        let txt_ignored = repo.ignore_check("test.txt").unwrap();
        let tmp_ignored = repo.ignore_check("test.tmp").unwrap();

        assert!(!txt_ignored);
        assert!(tmp_ignored);

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_mv_basic() {
        let (repo, repo_path) = create_test_repo();

        // Create and commit a file
        let original_path = repo_path.join("original.txt");
        fs::write(&original_path, "content").unwrap();
        repo.add(&["original.txt"]).unwrap();
        repo.commit("Add original file").unwrap();

        // Move the file
        repo.mv("original.txt", "renamed.txt").unwrap();

        // Verify move
        let new_path = repo_path.join("renamed.txt");
        assert!(!original_path.exists());
        assert!(new_path.exists());

        let content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(content, "content");

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_rm_basic() {
        let (repo, repo_path) = create_test_repo();

        // Create and commit a file
        let file_path = repo_path.join("to_remove.txt");
        fs::write(&file_path, "content").unwrap();
        repo.add(&["to_remove.txt"]).unwrap();
        repo.commit("Add file to remove").unwrap();

        // Remove the file
        repo.rm(&["to_remove.txt"]).unwrap();

        // Verify removal
        assert!(!file_path.exists());

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_rm_cached_only() {
        let (repo, repo_path) = create_test_repo();

        // Create and commit a file
        let file_path = repo_path.join("keep_local.txt");
        fs::write(&file_path, "content").unwrap();
        repo.add(&["keep_local.txt"]).unwrap();
        repo.commit("Add file").unwrap();

        // Remove from index only
        let options = RemoveOptions::new().with_cached();
        repo.rm_with_options(&["keep_local.txt"], options).unwrap();

        // Verify file still exists in working tree
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "content");

        fs::remove_dir_all(&repo_path).unwrap();
    }

    #[test]
    fn test_restore_with_options() {
        let (repo, repo_path) = create_test_repo();

        // Create and commit a file
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "original").unwrap();
        repo.add(&["test.txt"]).unwrap();
        repo.commit("First commit").unwrap();

        // Modify and commit again
        fs::write(&file_path, "second version").unwrap();
        repo.add(&["test.txt"]).unwrap();
        repo.commit("Second commit").unwrap();

        // Modify current working tree
        fs::write(&file_path, "current changes").unwrap();

        // Restore from first commit
        let options = RestoreOptions::new().with_source("HEAD~1").with_worktree();
        repo.restore(&["test.txt"], options).unwrap();

        // Verify restoration
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "original");

        fs::remove_dir_all(&repo_path).unwrap();
    }
}
