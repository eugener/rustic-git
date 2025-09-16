use std::path::Path;

use crate::utils::git;
use crate::{Repository, Result};

/// Represents a Git remote with its URLs
#[derive(Debug, Clone, PartialEq)]
pub struct Remote {
    /// The name of the remote (e.g., "origin")
    pub name: String,
    /// The fetch URL for the remote
    pub fetch_url: String,
    /// The push URL for the remote (if different from fetch URL)
    pub push_url: Option<String>,
}

impl Remote {
    /// Create a new Remote instance
    pub fn new(name: String, fetch_url: String, push_url: Option<String>) -> Self {
        Self {
            name,
            fetch_url,
            push_url,
        }
    }

    /// Get the effective push URL (returns push_url if set, otherwise fetch_url)
    pub fn push_url(&self) -> &str {
        self.push_url.as_ref().unwrap_or(&self.fetch_url)
    }
}

/// A collection of remotes in a repository
#[derive(Debug)]
pub struct RemoteList {
    remotes: Vec<Remote>,
}

impl RemoteList {
    /// Create a new empty RemoteList
    pub fn new(remotes: Vec<Remote>) -> Self {
        Self { remotes }
    }

    /// Find a remote by name
    pub fn find(&self, name: &str) -> Option<&Remote> {
        self.remotes.iter().find(|r| r.name == name)
    }

    /// Get an iterator over all remotes
    pub fn iter(&self) -> impl Iterator<Item = &Remote> {
        self.remotes.iter()
    }

    /// Get the number of remotes
    pub fn len(&self) -> usize {
        self.remotes.len()
    }

    /// Check if the remote list is empty
    pub fn is_empty(&self) -> bool {
        self.remotes.is_empty()
    }
}

/// Options for fetch operations
#[derive(Default, Debug)]
pub struct FetchOptions {
    /// Prune remote-tracking branches that no longer exist on the remote
    pub prune: bool,
    /// Fetch tags from the remote
    pub tags: bool,
    /// Fetch from all remotes instead of just one
    pub all_remotes: bool,
}

impl FetchOptions {
    /// Create new FetchOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable pruning of remote-tracking branches
    pub fn with_prune(mut self) -> Self {
        self.prune = true;
        self
    }

    /// Enable fetching tags
    pub fn with_tags(mut self) -> Self {
        self.tags = true;
        self
    }

    /// Enable fetching from all remotes
    pub fn with_all_remotes(mut self) -> Self {
        self.all_remotes = true;
        self
    }
}

/// Options for push operations
#[derive(Default, Debug)]
pub struct PushOptions {
    /// Force push (overwrites remote changes)
    pub force: bool,
    /// Push tags along with commits
    pub tags: bool,
    /// Set upstream tracking for the branch
    pub set_upstream: bool,
}

impl PushOptions {
    /// Create new PushOptions with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable force push
    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Enable pushing tags
    pub fn with_tags(mut self) -> Self {
        self.tags = true;
        self
    }

    /// Set upstream tracking for the branch
    pub fn with_set_upstream(mut self) -> Self {
        self.set_upstream = true;
        self
    }
}

impl Repository {
    /// Add a new remote to the repository
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the remote (e.g., "origin")
    /// * `url` - The URL for the remote repository
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("remote_add_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.add_remote("origin", "https://github.com/user/repo.git")?;
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn add_remote(&self, name: &str, url: &str) -> Result<()> {
        Self::ensure_git()?;
        git(&["remote", "add", name, url], Some(self.repo_path()))?;
        Ok(())
    }

    /// Remove a remote from the repository
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the remote to remove
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("remote_remove_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.add_remote("origin", "https://github.com/user/repo.git")?;
    /// repo.remove_remote("origin")?;
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn remove_remote(&self, name: &str) -> Result<()> {
        Self::ensure_git()?;
        git(&["remote", "remove", name], Some(self.repo_path()))?;
        Ok(())
    }

    /// Rename a remote
    ///
    /// # Arguments
    ///
    /// * `old_name` - The current name of the remote
    /// * `new_name` - The new name for the remote
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("remote_rename_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.add_remote("origin", "https://github.com/user/repo.git")?;
    /// repo.rename_remote("origin", "upstream")?;
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn rename_remote(&self, old_name: &str, new_name: &str) -> Result<()> {
        Self::ensure_git()?;
        git(
            &["remote", "rename", old_name, new_name],
            Some(self.repo_path()),
        )?;
        Ok(())
    }

    /// Get the URL for a specific remote
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the remote
    ///
    /// # Returns
    ///
    /// The fetch URL for the remote
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("remote_url_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// let url = "https://github.com/user/repo.git";
    /// repo.add_remote("origin", url)?;
    /// let fetched_url = repo.get_remote_url("origin")?;
    /// assert_eq!(fetched_url, url);
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn get_remote_url(&self, name: &str) -> Result<String> {
        Self::ensure_git()?;
        let output = git(&["remote", "get-url", name], Some(self.repo_path()))?;
        Ok(output.trim().to_string())
    }

    /// List all remotes in the repository
    ///
    /// # Returns
    ///
    /// A `RemoteList` containing all remotes with their URLs
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    /// use std::{env, fs};
    ///
    /// let test_path = env::temp_dir().join("remote_list_test");
    /// if test_path.exists() {
    ///     fs::remove_dir_all(&test_path).unwrap();
    /// }
    ///
    /// let repo = Repository::init(&test_path, false)?;
    /// repo.add_remote("origin", "https://github.com/user/repo.git")?;
    /// repo.add_remote("upstream", "https://github.com/original/repo.git")?;
    ///
    /// let remotes = repo.list_remotes()?;
    /// assert_eq!(remotes.len(), 2);
    /// assert!(remotes.find("origin").is_some());
    /// assert!(remotes.find("upstream").is_some());
    ///
    /// // Clean up
    /// fs::remove_dir_all(&test_path).unwrap();
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn list_remotes(&self) -> Result<RemoteList> {
        Self::ensure_git()?;

        // Get remote names
        let names_output = git(&["remote"], Some(self.repo_path()))?;
        if names_output.trim().is_empty() {
            return Ok(RemoteList::new(Vec::new()));
        }

        let mut remotes = Vec::new();

        for name in names_output.lines() {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            // Get fetch URL
            let fetch_url = match git(&["remote", "get-url", name], Some(self.repo_path())) {
                Ok(url) => url.trim().to_string(),
                Err(_) => continue, // Skip this remote if we can't get its URL
            };

            // Try to get push URL (might be different from fetch URL)
            let push_url = git(
                &["remote", "get-url", "--push", name],
                Some(self.repo_path()),
            )
            .ok()
            .map(|url| url.trim().to_string())
            .filter(|url| url != &fetch_url); // Only store if different

            remotes.push(Remote::new(name.to_string(), fetch_url, push_url));
        }

        Ok(RemoteList::new(remotes))
    }

    /// Fetch changes from a remote repository
    ///
    /// # Arguments
    ///
    /// * `remote` - The name of the remote to fetch from
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// repo.fetch("origin")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn fetch(&self, remote: &str) -> Result<()> {
        self.fetch_with_options(remote, FetchOptions::default())
    }

    /// Fetch changes from a remote repository with custom options
    ///
    /// # Arguments
    ///
    /// * `remote` - The name of the remote to fetch from
    /// * `options` - Fetch options to customize the operation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, FetchOptions};
    ///
    /// let repo = Repository::open(".")?;
    /// let options = FetchOptions::new().with_prune().with_tags();
    /// repo.fetch_with_options("origin", options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn fetch_with_options(&self, remote: &str, options: FetchOptions) -> Result<()> {
        Self::ensure_git()?;

        let mut args = vec!["fetch"];

        if options.prune {
            args.push("--prune");
        }

        if options.tags {
            args.push("--tags");
        }

        if options.all_remotes {
            args.push("--all");
        } else {
            args.push(remote);
        }

        git(&args, Some(self.repo_path()))?;
        Ok(())
    }

    /// Push changes to a remote repository
    ///
    /// # Arguments
    ///
    /// * `remote` - The name of the remote to push to
    /// * `branch` - The name of the branch to push
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// repo.push("origin", "main")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn push(&self, remote: &str, branch: &str) -> Result<()> {
        self.push_with_options(remote, branch, PushOptions::default())
    }

    /// Push changes to a remote repository with custom options
    ///
    /// # Arguments
    ///
    /// * `remote` - The name of the remote to push to
    /// * `branch` - The name of the branch to push
    /// * `options` - Push options to customize the operation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, PushOptions};
    ///
    /// let repo = Repository::open(".")?;
    /// let options = PushOptions::new().with_set_upstream();
    /// repo.push_with_options("origin", "main", options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn push_with_options(
        &self,
        remote: &str,
        branch: &str,
        options: PushOptions,
    ) -> Result<()> {
        Self::ensure_git()?;

        let mut args = vec!["push"];

        if options.force {
            args.push("--force");
        }

        if options.set_upstream {
            args.push("--set-upstream");
        }

        args.push(remote);
        args.push(branch);

        if options.tags {
            args.push("--tags");
        }

        git(&args, Some(self.repo_path()))?;
        Ok(())
    }

    /// Clone a remote repository to a local path
    ///
    /// # Arguments
    ///
    /// * `url` - The URL of the remote repository to clone
    /// * `path` - The local path where the repository should be cloned
    ///
    /// # Returns
    ///
    /// A `Repository` instance pointing to the cloned repository
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::clone("https://github.com/user/repo.git", "./local-repo")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn clone<P: AsRef<Path>>(url: &str, path: P) -> Result<Repository> {
        Self::ensure_git()?;

        let path_ref = path.as_ref();
        git(&["clone", url, &path_ref.to_string_lossy()], None)?;

        Repository::open(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn create_test_repo(path: &std::path::Path) -> Repository {
        // Clean up if exists
        if path.exists() {
            fs::remove_dir_all(path).unwrap();
        }

        Repository::init(path, false).unwrap()
    }

    #[test]
    fn test_remote_new() {
        let remote = Remote::new(
            "origin".to_string(),
            "https://github.com/user/repo.git".to_string(),
            Some("git@github.com:user/repo.git".to_string()),
        );

        assert_eq!(remote.name, "origin");
        assert_eq!(remote.fetch_url, "https://github.com/user/repo.git");
        assert_eq!(remote.push_url(), "git@github.com:user/repo.git");
    }

    #[test]
    fn test_remote_push_url_fallback() {
        let remote = Remote::new(
            "origin".to_string(),
            "https://github.com/user/repo.git".to_string(),
            None,
        );

        assert_eq!(remote.push_url(), "https://github.com/user/repo.git");
    }

    #[test]
    fn test_remote_list_operations() {
        let remotes = vec![
            Remote::new("origin".to_string(), "url1".to_string(), None),
            Remote::new("upstream".to_string(), "url2".to_string(), None),
        ];

        let list = RemoteList::new(remotes);

        assert_eq!(list.len(), 2);
        assert!(!list.is_empty());
        assert!(list.find("origin").is_some());
        assert!(list.find("nonexistent").is_none());
        assert_eq!(list.iter().count(), 2);
    }

    #[test]
    fn test_fetch_options_builder() {
        let options = FetchOptions::new()
            .with_prune()
            .with_tags()
            .with_all_remotes();

        assert!(options.prune);
        assert!(options.tags);
        assert!(options.all_remotes);
    }

    #[test]
    fn test_push_options_builder() {
        let options = PushOptions::new()
            .with_force()
            .with_tags()
            .with_set_upstream();

        assert!(options.force);
        assert!(options.tags);
        assert!(options.set_upstream);
    }

    #[test]
    fn test_add_remove_remote() {
        let test_path = env::temp_dir().join("test_add_remove_remote");
        let repo = create_test_repo(&test_path);

        // Add a remote
        repo.add_remote("origin", "https://github.com/user/repo.git")
            .unwrap();

        // Verify it was added
        let remotes = repo.list_remotes().unwrap();
        assert_eq!(remotes.len(), 1);
        assert!(remotes.find("origin").is_some());

        // Remove the remote
        repo.remove_remote("origin").unwrap();

        // Verify it was removed
        let remotes = repo.list_remotes().unwrap();
        assert_eq!(remotes.len(), 0);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_rename_remote() {
        let test_path = env::temp_dir().join("test_rename_remote");
        let repo = create_test_repo(&test_path);

        // Add a remote
        repo.add_remote("origin", "https://github.com/user/repo.git")
            .unwrap();

        // Rename it
        repo.rename_remote("origin", "upstream").unwrap();

        // Verify the rename
        let remotes = repo.list_remotes().unwrap();
        assert_eq!(remotes.len(), 1);
        assert!(remotes.find("upstream").is_some());
        assert!(remotes.find("origin").is_none());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_get_remote_url() {
        let test_path = env::temp_dir().join("test_get_remote_url");
        let repo = create_test_repo(&test_path);

        let url = "https://github.com/user/repo.git";
        repo.add_remote("origin", url).unwrap();

        let fetched_url = repo.get_remote_url("origin").unwrap();
        assert_eq!(fetched_url, url);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_list_multiple_remotes() {
        let test_path = env::temp_dir().join("test_list_multiple_remotes");
        let repo = create_test_repo(&test_path);

        // Add multiple remotes
        repo.add_remote("origin", "https://github.com/user/repo.git")
            .unwrap();
        repo.add_remote("upstream", "https://github.com/original/repo.git")
            .unwrap();

        let remotes = repo.list_remotes().unwrap();
        assert_eq!(remotes.len(), 2);

        let origin = remotes.find("origin").unwrap();
        assert_eq!(origin.fetch_url, "https://github.com/user/repo.git");

        let upstream = remotes.find("upstream").unwrap();
        assert_eq!(upstream.fetch_url, "https://github.com/original/repo.git");

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_list_remotes_empty() {
        let test_path = env::temp_dir().join("test_list_remotes_empty");
        let repo = create_test_repo(&test_path);

        let remotes = repo.list_remotes().unwrap();
        assert_eq!(remotes.len(), 0);
        assert!(remotes.is_empty());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_remove_nonexistent_remote() {
        let test_path = env::temp_dir().join("test_remove_nonexistent_remote");
        let repo = create_test_repo(&test_path);

        // Try to remove a non-existent remote
        let result = repo.remove_remote("nonexistent");
        assert!(result.is_err());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }
}
