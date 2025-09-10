use crate::utils::git;
use crate::{Hash, Repository, Result};

impl Repository {
    /// Create a commit with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Hash` of the new commit or a `GitError`.
    pub fn commit(&self, message: &str) -> Result<Hash> {
        Self::ensure_git()?;

        if message.trim().is_empty() {
            return Err(crate::error::GitError::CommandFailed(
                "Commit message cannot be empty".to_string(),
            ));
        }

        // Check if there are staged changes
        let status = self.status()?;
        let has_staged = status.staged_files().count() > 0;

        if !has_staged {
            return Err(crate::error::GitError::CommandFailed(
                "No changes staged for commit".to_string(),
            ));
        }

        let _stdout =
            git(&["commit", "-m", message], Some(self.repo_path())).map_err(|e| match e {
                crate::error::GitError::CommandFailed(msg) => {
                    crate::error::GitError::CommandFailed(format!(
                        "Commit failed: {}. Ensure git user.name and user.email are configured.",
                        msg
                    ))
                }
                other => other,
            })?;

        // Get the commit hash of the just-created commit
        let hash_output = git(&["rev-parse", "HEAD"], Some(self.repo_path()))?;
        let commit_hash = hash_output.trim().to_string();

        Ok(Hash(commit_hash))
    }

    /// Create a commit with the given message and author.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message
    /// * `author` - The author in format "Name <email@example.com>"
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Hash` of the new commit or a `GitError`.
    pub fn commit_with_author(&self, message: &str, author: &str) -> Result<Hash> {
        Self::ensure_git()?;

        if message.trim().is_empty() {
            return Err(crate::error::GitError::CommandFailed(
                "Commit message cannot be empty".to_string(),
            ));
        }

        if author.trim().is_empty() {
            return Err(crate::error::GitError::CommandFailed(
                "Author cannot be empty".to_string(),
            ));
        }

        // Check if there are staged changes
        let status = self.status()?;
        let has_staged = status.staged_files().count() > 0;

        if !has_staged {
            return Err(crate::error::GitError::CommandFailed(
                "No changes staged for commit".to_string(),
            ));
        }

        let _stdout = git(&["commit", "-m", message, "--author", author], Some(self.repo_path()))
            .map_err(|e| match e {
                crate::error::GitError::CommandFailed(msg) => {
                    crate::error::GitError::CommandFailed(format!(
                        "Commit with author failed: {}. Ensure git user.name and user.email are configured.", 
                        msg
                    ))
                }
                other => other,
            })?;

        // Get the commit hash of the just-created commit
        let hash_output = git(&["rev-parse", "HEAD"], Some(self.repo_path()))?;
        let commit_hash = hash_output.trim().to_string();

        Ok(Hash(commit_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn create_test_repo(path: &str) -> Repository {
        // Clean up if exists
        if Path::new(path).exists() {
            fs::remove_dir_all(path).unwrap();
        }

        let repo = Repository::init(path, false).unwrap();

        // Configure git user for this repository to enable commits
        repo.config()
            .set_user("Test User", "test@example.com")
            .unwrap();

        repo
    }

    fn create_and_stage_file(repo: &Repository, repo_path: &str, filename: &str, content: &str) {
        let file_path = format!("{}/{}", repo_path, filename);
        fs::write(file_path, content).unwrap();
        repo.add(&[filename]).unwrap();
    }

    #[test]
    fn test_commit_basic() {
        let test_path = "/tmp/test_commit_repo";
        let repo = create_test_repo(test_path);

        // Create and stage a file
        create_and_stage_file(&repo, test_path, "test.txt", "test content");

        // Commit the changes
        let result = repo.commit("Initial commit");
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.as_str().is_empty());
        assert_eq!(hash.short().len(), 7);

        // Verify repository is now clean
        let status = repo.status().unwrap();
        assert!(status.is_clean());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_commit_with_author() {
        let test_path = "/tmp/test_commit_author_repo";
        let repo = create_test_repo(test_path);

        // Create and stage a file
        create_and_stage_file(&repo, test_path, "test.txt", "test content");

        // Commit with author
        let result = repo.commit_with_author("Test commit", "Test User <test@example.com>");
        assert!(result.is_ok());

        let hash = result.unwrap();
        assert!(!hash.as_str().is_empty());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_commit_empty_message() {
        let test_path = "/tmp/test_commit_empty_msg_repo";
        let repo = create_test_repo(test_path);

        // Create and stage a file
        create_and_stage_file(&repo, test_path, "test.txt", "test content");

        // Try to commit with empty message
        let result = repo.commit("");
        assert!(result.is_err());

        if let Err(crate::error::GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("empty"));
        } else {
            panic!("Expected CommandFailed error");
        }

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_commit_no_staged_changes() {
        let test_path = "/tmp/test_commit_no_changes_repo";
        let repo = create_test_repo(test_path);

        // Try to commit without staging anything
        let result = repo.commit("Test commit");
        assert!(result.is_err());

        if let Err(crate::error::GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("No changes staged"));
        } else {
            panic!("Expected CommandFailed error");
        }

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_hash_display() {
        let hash = Hash("abc123def456".to_string());
        assert_eq!(hash.as_str(), "abc123def456");
        assert_eq!(hash.short(), "abc123d");
        assert_eq!(format!("{}", hash), "abc123def456");
    }

    #[test]
    fn test_hash_short_hash() {
        let hash = Hash("abc".to_string());
        assert_eq!(hash.short(), "abc"); // Less than 7 chars, returns full hash
    }

    #[test]
    fn test_commit_with_author_empty_author() {
        let test_path = "/tmp/test_commit_empty_author_repo";
        let repo = create_test_repo(test_path);

        // Create and stage a file
        create_and_stage_file(&repo, test_path, "test.txt", "test content");

        // Try to commit with empty author
        let result = repo.commit_with_author("Test commit", "");
        assert!(result.is_err());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_git_config_is_set_in_test_repo() {
        let test_path = "/tmp/test_git_config_repo";
        let repo = create_test_repo(test_path);

        // Verify git user configuration is set using our config API
        let (name, email) = repo.config().get_user().unwrap();
        assert_eq!(name, "Test User");
        assert_eq!(email, "test@example.com");

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }
}
