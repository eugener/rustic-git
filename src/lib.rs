use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub struct Repository {
    repo_path: PathBuf,
}

#[derive(Debug)]
pub enum GitError {
    IoError(io::Error),
    CommandFailed(String),
}

impl From<io::Error> for GitError {
    fn from(error: io::Error) -> Self {
        GitError::IoError(error)
    }
}

impl Repository {
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
    pub fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<Self, GitError> {
        let mut cmd = Command::new("git");
        cmd.arg("init");

        if bare {
            cmd.arg("--bare");
        }

        cmd.arg(path.as_ref());

        let output = cmd.output()?;

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
}
