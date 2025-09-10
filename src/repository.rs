use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::error::{GitError, Result};
use crate::utils::{git, git_raw};

static GIT_CHECKED: OnceLock<Result<()>> = OnceLock::new();

#[derive(Debug)]
pub struct Repository {
    repo_path: PathBuf,
}

impl Repository {
    /// Ensure that Git is available in the system PATH.
    ///
    /// This function checks if the `git` command is available in the system PATH.
    /// The result is cached, so subsequent calls are very fast.
    /// If Git is not found, it returns a `GitError::CommandFailed` with an appropriate error message.
    ///
    /// # Returns
    ///
    /// A `Result` containing either `Ok(())` if Git is available or a `GitError`.
    pub fn ensure_git() -> Result<()> {
        GIT_CHECKED
            .get_or_init(|| {
                git_raw(&["--version"], None)
                    .map_err(|_| GitError::CommandFailed("Git not found in PATH".to_string()))
                    .map(|_| ())
            })
            .clone()
    }

    /// Open an existing Git repository at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to an existing Git repository.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the opened `Repository` instance or a `GitError`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::ensure_git()?;

        let path_ref = path.as_ref();

        // Check if the path exists
        if !path_ref.exists() {
            return Err(GitError::CommandFailed(format!(
                "Path does not exist: {}",
                path_ref.display()
            )));
        }

        // Check if it's a valid git repository by running git status
        let _stdout = git(&["status", "--porcelain"], Some(path_ref)).map_err(|_| {
            GitError::CommandFailed(format!("Not a git repository: {}", path_ref.display()))
        })?;

        Ok(Self {
            repo_path: path_ref.to_path_buf(),
        })
    }

    /// Initialize a new Git repository at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the repository should be initialized.
    /// * `bare` - Whether the repository should be bare or not.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the initialized `Repository` instance or a `GitError`.
    pub fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<Self> {
        Self::ensure_git()?;

        let mut args = vec!["init"];
        if bare {
            args.push("--bare");
        }
        args.push(path.as_ref().to_str().unwrap_or(""));

        let _stdout = git(&args, None)?;

        Ok(Self {
            repo_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Get a configuration manager for this repository
    ///
    /// Returns a `RepoConfig` instance that can be used to get and set
    /// git configuration values for this repository.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::init("/tmp/test", false)?;
    /// repo.config().set_user("John Doe", "john@example.com")?;
    ///
    /// let (name, email) = repo.config().get_user()?;
    /// assert_eq!(name, "John Doe");
    /// assert_eq!(email, "john@example.com");
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn config(&self) -> crate::commands::RepoConfig<'_> {
        crate::commands::RepoConfig::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_git_init_creates_repository() {
        let test_path = "/tmp/test_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Initialize repository
        let repo = Repository::init(test_path, false).unwrap();

        // Check that .git directory was created
        assert!(Path::new(&format!("{}/.git", test_path)).exists());
        assert_eq!(repo.repo_path(), Path::new(test_path));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_git_init_bare_repository() {
        let test_path = "/tmp/test_bare_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Initialize bare repository
        let repo = Repository::init(test_path, true).unwrap();

        // Check that bare repo files were created (no .git subdirectory)
        assert!(Path::new(&format!("{}/HEAD", test_path)).exists());
        assert!(Path::new(&format!("{}/objects", test_path)).exists());
        assert!(!Path::new(&format!("{}/.git", test_path)).exists());
        assert_eq!(repo.repo_path(), Path::new(test_path));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_open_existing_repository() {
        let test_path = "/tmp/test_open_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // First create a repository
        let _created_repo = Repository::init(test_path, false).unwrap();

        // Now open the existing repository
        let opened_repo = Repository::open(test_path).unwrap();
        assert_eq!(opened_repo.repo_path(), Path::new(test_path));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_open_nonexistent_path() {
        let test_path = "/tmp/nonexistent_repo_path";

        // Ensure path doesn't exist
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Try to open non-existent repository
        let result = Repository::open(test_path);
        assert!(result.is_err());

        if let Err(GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("Path does not exist"));
        } else {
            panic!("Expected CommandFailed error");
        }
    }

    #[test]
    fn test_open_non_git_directory() {
        let test_path = "/tmp/not_a_git_repo";

        // Clean up if exists and create a regular directory
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }
        fs::create_dir(test_path).unwrap();

        // Try to open directory that's not a git repository
        let result = Repository::open(test_path);
        assert!(result.is_err());

        if let Err(GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("Not a git repository"));
        } else {
            panic!("Expected CommandFailed error");
        }

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_repo_path_method() {
        let test_path = "/tmp/test_repo_path";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Initialize repository
        let repo = Repository::init(test_path, false).unwrap();

        // Test repo_path method
        assert_eq!(repo.repo_path(), Path::new(test_path));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_repo_path_method_after_open() {
        let test_path = "/tmp/test_repo_path_open";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Initialize and then open repository
        let _created_repo = Repository::init(test_path, false).unwrap();
        let opened_repo = Repository::open(test_path).unwrap();

        // Test repo_path method on opened repository
        assert_eq!(opened_repo.repo_path(), Path::new(test_path));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_ensure_git_caching() {
        // Call ensure_git multiple times to test caching
        let result1 = Repository::ensure_git();
        let result2 = Repository::ensure_git();
        let result3 = Repository::ensure_git();

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
    }

    #[test]
    fn test_init_with_empty_string_path() {
        let result = Repository::init("", false);
        // This might succeed or fail depending on git's behavior with empty paths
        // The important thing is it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_open_with_empty_string_path() {
        let result = Repository::open("");
        assert!(result.is_err());

        match result.unwrap_err() {
            GitError::CommandFailed(msg) => {
                assert!(
                    msg.contains("Path does not exist") || msg.contains("Not a git repository")
                );
            }
            _ => panic!("Expected CommandFailed error"),
        }
    }

    #[test]
    fn test_init_with_relative_path() {
        let test_path = "relative_test_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        let result = Repository::init(test_path, false);

        if let Ok(repo) = result {
            assert_eq!(repo.repo_path(), Path::new(test_path));

            // Clean up
            fs::remove_dir_all(test_path).unwrap();
        }
    }

    #[test]
    fn test_open_with_relative_path() {
        let test_path = "relative_open_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Create the repo first
        let _created = Repository::init(test_path, false).unwrap();

        // Now open with relative path
        let result = Repository::open(test_path);
        assert!(result.is_ok());

        let repo = result.unwrap();
        assert_eq!(repo.repo_path(), Path::new(test_path));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_init_with_unicode_path() {
        let test_path = "/tmp/测试_repo_🚀";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        let result = Repository::init(test_path, false);

        if let Ok(repo) = result {
            assert_eq!(repo.repo_path(), Path::new(test_path));

            // Clean up
            fs::remove_dir_all(test_path).unwrap();
        }
    }

    #[test]
    fn test_path_with_spaces() {
        let test_path = "/tmp/test repo with spaces";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        let result = Repository::init(test_path, false);

        if let Ok(repo) = result {
            assert_eq!(repo.repo_path(), Path::new(test_path));

            // Clean up
            fs::remove_dir_all(test_path).unwrap();
        }
    }

    #[test]
    fn test_very_long_path() {
        let long_component = "a".repeat(100);
        let test_path = format!("/tmp/{}", long_component);

        // Clean up if exists
        if Path::new(&test_path).exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let result = Repository::init(&test_path, false);

        if let Ok(repo) = result {
            assert_eq!(repo.repo_path(), Path::new(&test_path));

            // Clean up
            fs::remove_dir_all(&test_path).unwrap();
        }
    }
}
