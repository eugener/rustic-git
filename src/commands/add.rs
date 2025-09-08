use std::path::Path;

use crate::utils::git;
use crate::{Repository, Result};

impl Repository {
    /// Add specific files or paths to the staging area.
    ///
    /// # Arguments
    ///
    /// * `paths` - The file paths to add to the staging area
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn add<P: AsRef<Path>>(&self, paths: &[P]) -> Result<()> {
        Self::ensure_git()?;

        if paths.is_empty() {
            return Ok(());
        }

        let mut args = vec!["add"];
        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .collect();

        for path_str in &path_strings {
            args.push(path_str);
        }

        let _stdout = git(&args, Some(self.repo_path()))?;
        Ok(())
    }

    /// Add all changes to the staging area (equivalent to `git add .`).
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn add_all(&self) -> Result<()> {
        Self::ensure_git()?;
        let _stdout = git(&["add", "."], Some(self.repo_path()))?;
        Ok(())
    }

    /// Add all tracked files that have been modified (equivalent to `git add -u`).
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `GitError` if the operation fails.
    pub fn add_update(&self) -> Result<()> {
        Self::ensure_git()?;
        let _stdout = git(&["add", "-u"], Some(self.repo_path()))?;
        Ok(())
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

        Repository::init(path, false).unwrap()
    }

    fn create_test_file(repo_path: &str, filename: &str, content: &str) {
        let file_path = format!("{}/{}", repo_path, filename);
        fs::write(file_path, content).unwrap();
    }

    #[test]
    fn test_add_specific_files() {
        let test_path = "/tmp/test_add_repo";
        let repo = create_test_repo(test_path);

        // Create some test files
        create_test_file(test_path, "file1.txt", "content 1");
        create_test_file(test_path, "file2.txt", "content 2");

        // Add specific files
        let result = repo.add(&["file1.txt"]);
        assert!(result.is_ok());

        // Verify file1.txt is staged by checking status
        let status = repo.status().unwrap();
        let added_files: Vec<_> = status
            .files
            .iter()
            .filter(|(s, _)| matches!(s, crate::FileStatus::Added))
            .map(|(_, f)| f.as_str())
            .collect();

        assert!(added_files.contains(&"file1.txt"));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_add_multiple_files() {
        let test_path = "/tmp/test_add_multiple_repo";
        let repo = create_test_repo(test_path);

        // Create test files
        create_test_file(test_path, "file1.txt", "content 1");
        create_test_file(test_path, "file2.txt", "content 2");
        create_test_file(test_path, "file3.txt", "content 3");

        // Add multiple files
        let result = repo.add(&["file1.txt", "file2.txt"]);
        assert!(result.is_ok());

        // Verify files are staged
        let status = repo.status().unwrap();
        let added_files: Vec<_> = status
            .files
            .iter()
            .filter(|(s, _)| matches!(s, crate::FileStatus::Added))
            .map(|(_, f)| f.as_str())
            .collect();

        assert!(added_files.contains(&"file1.txt"));
        assert!(added_files.contains(&"file2.txt"));
        assert_eq!(added_files.len(), 2);

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_add_all() {
        let test_path = "/tmp/test_add_all_repo";
        let repo = create_test_repo(test_path);

        // Create test files
        create_test_file(test_path, "file1.txt", "content 1");
        create_test_file(test_path, "file2.txt", "content 2");
        fs::create_dir(format!("{}/subdir", test_path)).unwrap();
        create_test_file(test_path, "subdir/file3.txt", "content 3");

        // Add all files
        let result = repo.add_all();
        assert!(result.is_ok());

        // Verify all files are staged
        let status = repo.status().unwrap();
        let added_files: Vec<_> = status
            .files
            .iter()
            .filter(|(s, _)| matches!(s, crate::FileStatus::Added))
            .map(|(_, f)| f.as_str())
            .collect();

        assert!(added_files.contains(&"file1.txt"));
        assert!(added_files.contains(&"file2.txt"));
        assert!(added_files.contains(&"subdir/file3.txt"));

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_add_empty_paths() {
        let test_path = "/tmp/test_add_empty_repo";
        let repo = create_test_repo(test_path);

        // Adding empty paths should succeed without error
        let result = repo.add::<&str>(&[]);
        assert!(result.is_ok());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }

    #[test]
    fn test_add_nonexistent_file() {
        let test_path = "/tmp/test_add_nonexistent_repo";
        let repo = create_test_repo(test_path);

        // Adding non-existent file should fail
        let result = repo.add(&["nonexistent.txt"]);
        assert!(result.is_err());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }
}
