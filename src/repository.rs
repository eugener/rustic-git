use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::error::{GitError, Result};
use crate::utils::git;

static GIT_CHECKED: OnceLock<Result<()>> = OnceLock::new();

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
                git(&["--version"], None)
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
        let output = git(&["status", "--porcelain"], Some(path_ref))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CommandFailed(format!(
                "Not a git repository: {}",
                error_msg
            )));
        }

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

        let output = git(&args, None)?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(GitError::CommandFailed(format!(
                "git init failed: {}",
                error_msg
            )));
        }

        Ok(Self {
            repo_path: path.as_ref().to_path_buf(),
        })
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
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
}
