use crate::utils::git;
use crate::{Repository, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

impl ResetMode {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ResetMode::Soft => "--soft",
            ResetMode::Mixed => "--mixed",
            ResetMode::Hard => "--hard",
        }
    }
}

pub fn reset<P: AsRef<Path>>(repo_path: P, mode: ResetMode, commit: &str) -> Result<()> {
    let args = vec!["reset", mode.as_str(), commit];
    git(&args, Some(repo_path.as_ref()))?;
    Ok(())
}

impl Repository {
    /// Perform a soft reset to the specified commit.
    ///
    /// Moves HEAD to the specified commit but keeps both the index and working directory unchanged.
    /// Previously staged changes remain staged.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash, reference, or "HEAD~N" to reset to
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn reset_soft(&self, commit: &str) -> Result<()> {
        Self::ensure_git()?;
        reset(self.repo_path(), ResetMode::Soft, commit)?;
        Ok(())
    }

    /// Perform a mixed reset to the specified commit (default reset behavior).
    ///
    /// Moves HEAD to the specified commit and resets the index to match, but leaves the working directory unchanged.
    /// Previously staged changes become unstaged but remain in the working directory.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash, reference, or "HEAD~N" to reset to
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn reset_mixed(&self, commit: &str) -> Result<()> {
        Self::ensure_git()?;
        reset(self.repo_path(), ResetMode::Mixed, commit)?;
        Ok(())
    }

    /// Perform a hard reset to the specified commit.
    ///
    /// Moves HEAD to the specified commit and resets both the index and working directory to match.
    /// **WARNING**: This discards all uncommitted changes permanently.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash, reference, or "HEAD~N" to reset to
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn reset_hard(&self, commit: &str) -> Result<()> {
        Self::ensure_git()?;
        reset(self.repo_path(), ResetMode::Hard, commit)?;
        Ok(())
    }

    /// Perform a reset with the specified mode.
    ///
    /// This is a flexible method that allows you to specify the reset mode explicitly.
    ///
    /// # Arguments
    ///
    /// * `commit` - The commit hash, reference, or "HEAD~N" to reset to
    /// * `mode` - The reset mode (Soft, Mixed, or Hard)
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn reset_with_mode(&self, commit: &str, mode: ResetMode) -> Result<()> {
        Self::ensure_git()?;
        reset(self.repo_path(), mode, commit)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Repository;
    use std::path::PathBuf;
    use std::{env, fs};

    fn create_test_repo(test_name: &str) -> (PathBuf, Repository) {
        let temp_dir = env::temp_dir().join(format!("rustic_git_reset_test_{}", test_name));

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
    fn test_reset_mode_as_str() {
        assert_eq!(ResetMode::Soft.as_str(), "--soft");
        assert_eq!(ResetMode::Mixed.as_str(), "--mixed");
        assert_eq!(ResetMode::Hard.as_str(), "--hard");
    }

    #[test]
    fn test_reset_soft() {
        let (temp_dir, repo) = create_test_repo("reset_soft");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset soft to first commit
        reset(&temp_dir, ResetMode::Soft, &first_commit).unwrap();

        // Check that index still has file2.txt staged
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 1);
        assert!(
            status
                .staged_files()
                .any(|f| f.path.file_name().unwrap() == "file2.txt")
        );

        // Check that file2.txt still exists in working directory
        assert!(temp_dir.join("file2.txt").exists());

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_reset_mixed() {
        let (temp_dir, repo) = create_test_repo("reset_mixed");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset mixed to first commit
        reset(&temp_dir, ResetMode::Mixed, &first_commit).unwrap();

        // Check that index is clean (no staged files)
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 0);

        // Check that file2.txt still exists in working directory as untracked
        assert!(temp_dir.join("file2.txt").exists());
        assert!(
            status
                .untracked_entries()
                .any(|f| f.path.file_name().unwrap() == "file2.txt")
        );
    }

    #[test]
    fn test_reset_hard() {
        let (temp_dir, repo) = create_test_repo("reset_hard");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset hard to first commit
        reset(&temp_dir, ResetMode::Hard, &first_commit).unwrap();

        // Check that index is clean
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 0);

        // Check that file2.txt no longer exists in working directory
        assert!(!temp_dir.join("file2.txt").exists());
        assert_eq!(status.untracked_entries().count(), 0);
    }

    #[test]
    fn test_reset_invalid_commit() {
        let (temp_dir, _repo) = create_test_repo("reset_invalid_commit");

        let result = reset(&temp_dir, ResetMode::Mixed, "invalid_commit_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_head() {
        let (temp_dir, repo) = create_test_repo("reset_head");

        // Create initial commit
        create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "Initial commit");

        // Modify file and stage it
        fs::write(temp_dir.join("file1.txt"), "modified").unwrap();
        repo.add(&["file1.txt"]).unwrap();

        // Reset to HEAD (should unstage changes)
        reset(temp_dir, ResetMode::Mixed, "HEAD").unwrap();

        // Verify file is no longer staged but working directory is modified
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 0);
        assert_eq!(status.unstaged_files().count(), 1);
    }

    // Tests for Repository methods
    #[test]
    fn test_repository_reset_soft() {
        let (temp_dir, repo) = create_test_repo("repository_reset_soft");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset soft to first commit using Repository method
        repo.reset_soft(&first_commit).unwrap();

        // Check that index still has file2.txt staged
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 1);
        assert!(
            status
                .staged_files()
                .any(|f| f.path.file_name().unwrap() == "file2.txt")
        );
    }

    #[test]
    fn test_repository_reset_mixed() {
        let (temp_dir, repo) = create_test_repo("repository_reset_mixed");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset mixed to first commit using Repository method
        repo.reset_mixed(&first_commit).unwrap();

        // Check that index is clean but file exists as untracked
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 0);
        assert!(temp_dir.join("file2.txt").exists());
        assert!(
            status
                .untracked_entries()
                .any(|f| f.path.file_name().unwrap() == "file2.txt")
        );
    }

    #[test]
    fn test_repository_reset_hard() {
        let (temp_dir, repo) = create_test_repo("repository_reset_hard");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset hard to first commit using Repository method
        repo.reset_hard(&first_commit).unwrap();

        // Check that everything is reset
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 0);
        assert!(!temp_dir.join("file2.txt").exists());
        assert_eq!(status.untracked_entries().count(), 0);

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_repository_reset_with_mode() {
        let (temp_dir, repo) = create_test_repo("repository_reset_with_mode");

        // Create initial commit
        let first_commit =
            create_file_and_commit(&repo, &temp_dir, "file1.txt", "content1", "First commit");

        // Create second commit
        let _second_commit =
            create_file_and_commit(&repo, &temp_dir, "file2.txt", "content2", "Second commit");

        // Reset using reset_with_mode
        repo.reset_with_mode(&first_commit, ResetMode::Mixed)
            .unwrap();

        // Check same behavior as reset_mixed
        let status = repo.status().unwrap();
        assert_eq!(status.staged_files().count(), 0);
        assert!(temp_dir.join("file2.txt").exists());

        // Clean up
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
