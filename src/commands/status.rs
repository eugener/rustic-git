use crate::{Repository, Result};
use crate::utils::git;

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GitStatus {
    pub files: Box<[(FileStatus, String)]>,
}

impl GitStatus {

    pub fn is_clean(&self) -> bool {
        self.files.is_empty()
    }

    pub fn has_changes(&self) -> bool {
        !self.is_clean()
    }

    /// Get all files with a specific status
    pub fn files_with_status(&self, status: FileStatus) -> Vec<&String> {
        self.files
            .iter()
            .filter_map(|(s, f)| if *s == status { Some(f) } else { None })
            .collect()
    }

    /// Get all modified files
    pub fn modified_files(&self) -> Vec<&String> {
        self.files_with_status(FileStatus::Modified)
    }

    /// Get all untracked files
    pub fn untracked_files(&self) -> Vec<&String> {
        self.files_with_status(FileStatus::Untracked)
    }

    fn parse_porcelain_output(output: &str) -> Self {
        let mut files = Vec::new();

        for line in output.lines() {
            if line.len() < 3 {
                continue;
            }

            let index_status = line.chars().nth(0).unwrap_or(' ');
            let worktree_status = line.chars().nth(1).unwrap_or(' ');
            let filename = line[3..].to_string();

            let file_status = match (index_status, worktree_status) {
                ('M', _) | (_, 'M') => Some(FileStatus::Modified),
                ('A', _) => Some(FileStatus::Added),
                ('D', _) => Some(FileStatus::Deleted),
                ('R', _) => Some(FileStatus::Renamed),
                ('C', _) => Some(FileStatus::Copied),
                ('?', '?') => Some(FileStatus::Untracked),
                ('!', '!') => Some(FileStatus::Ignored),
                _ => None,
            };

            if let Some(fs) = file_status {
                files.push((fs, filename));
            }
        }

        Self {
            files: files.into_boxed_slice(),
        }
    }
}

impl Repository {
    /// Get the status of the repository.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `GitStatus` or a `GitError`.
    pub fn status(&self) -> Result<GitStatus> {
        Self::ensure_git()?;

        let stdout = git(&["status", "--porcelain"], Some(self.repo_path()))?;
        Ok(GitStatus::parse_porcelain_output(&stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_parse_porcelain_output() {
        let output = "M  modified.txt\nA  added.txt\nD  deleted.txt\n?? untracked.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.files.len(), 4);
        assert!(status.files.contains(&(FileStatus::Modified, "modified.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Added, "added.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Deleted, "deleted.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Untracked, "untracked.txt".to_string())));
        
        assert_eq!(status.modified_files(), vec![&"modified.txt".to_string()]);
        assert_eq!(status.untracked_files(), vec![&"untracked.txt".to_string()]);
        
        assert!(!status.is_clean());
        assert!(status.has_changes());
    }

    #[test]
    fn test_clean_repository_status() {
        let output = "";
        let status = GitStatus::parse_porcelain_output(output);

        assert!(status.is_clean());
        assert!(!status.has_changes());
        assert_eq!(status.files.len(), 0);
        assert!(status.modified_files().is_empty());
        assert!(status.untracked_files().is_empty());
    }

    #[test]
    fn test_repository_status() {
        let test_path = "/tmp/test_status_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Create a repository
        let repo = Repository::init(test_path, false).unwrap();

        // Get status of empty repository
        let status = repo.status().unwrap();
        assert!(status.is_clean());

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }
}