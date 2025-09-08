use crate::{Repository, Result};
use crate::utils::git;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    #[test]
    fn test_parse_porcelain_output_edge_cases() {
        // Test empty lines and malformed lines
        let output = "\n\nM  valid.txt\nXX\n  \nA  another.txt\n";
        let status = GitStatus::parse_porcelain_output(output);
        
        assert_eq!(status.files.len(), 2);
        assert!(status.files.contains(&(FileStatus::Modified, "valid.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Added, "another.txt".to_string())));
    }

    #[test]
    fn test_parse_porcelain_all_status_types() {
        let output = "M  modified.txt\nA  added.txt\nD  deleted.txt\nR  renamed.txt\nC  copied.txt\n?? untracked.txt\n!! ignored.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.files.len(), 7);
        assert!(status.files.contains(&(FileStatus::Modified, "modified.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Added, "added.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Deleted, "deleted.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Renamed, "renamed.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Copied, "copied.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Untracked, "untracked.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Ignored, "ignored.txt".to_string())));
    }

    #[test]
    fn test_parse_porcelain_worktree_modifications() {
        let output = " M worktree_modified.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.files.len(), 1);
        assert!(status.files.contains(&(FileStatus::Modified, "worktree_modified.txt".to_string())));
    }

    #[test]
    fn test_parse_porcelain_unknown_status() {
        let output = "XY unknown.txt\nZ  another_unknown.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        // Unknown statuses should be ignored
        assert_eq!(status.files.len(), 0);
    }

    #[test]
    fn test_file_status_equality() {
        assert_eq!(FileStatus::Modified, FileStatus::Modified);
        assert_ne!(FileStatus::Modified, FileStatus::Added);
        assert_eq!(FileStatus::Untracked, FileStatus::Untracked);
    }

    #[test]
    fn test_file_status_clone() {
        let status = FileStatus::Modified;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_file_status_debug() {
        let status = FileStatus::Modified;
        let debug_str = format!("{:?}", status);
        assert_eq!(debug_str, "Modified");
    }

    #[test]
    fn test_git_status_equality() {
        let files1 = vec![
            (FileStatus::Modified, "file1.txt".to_string()),
            (FileStatus::Added, "file2.txt".to_string()),
        ];
        let files2 = vec![
            (FileStatus::Modified, "file1.txt".to_string()),
            (FileStatus::Added, "file2.txt".to_string()),
        ];
        let files3 = vec![
            (FileStatus::Modified, "different.txt".to_string()),
        ];

        let status1 = GitStatus { files: files1.into_boxed_slice() };
        let status2 = GitStatus { files: files2.into_boxed_slice() };
        let status3 = GitStatus { files: files3.into_boxed_slice() };

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    #[test]
    fn test_git_status_clone() {
        let files = vec![
            (FileStatus::Modified, "file1.txt".to_string()),
        ];
        let status1 = GitStatus { files: files.into_boxed_slice() };
        let status2 = status1.clone();
        
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_git_status_debug() {
        let files = vec![
            (FileStatus::Modified, "file1.txt".to_string()),
        ];
        let status = GitStatus { files: files.into_boxed_slice() };
        let debug_str = format!("{:?}", status);
        
        assert!(debug_str.contains("GitStatus"));
        assert!(debug_str.contains("Modified"));
        assert!(debug_str.contains("file1.txt"));
    }

    #[test]
    fn test_files_with_status_multiple_same_status() {
        let output = "M  file1.txt\nM  file2.txt\nA  file3.txt\n";
        let status = GitStatus::parse_porcelain_output(output);
        
        let modified = status.files_with_status(FileStatus::Modified);
        assert_eq!(modified.len(), 2);
        assert!(modified.contains(&&"file1.txt".to_string()));
        assert!(modified.contains(&&"file2.txt".to_string()));
        
        let added = status.files_with_status(FileStatus::Added);
        assert_eq!(added.len(), 1);
        assert!(added.contains(&&"file3.txt".to_string()));
    }

    #[test]
    fn test_files_with_status_no_matches() {
        let output = "M  file1.txt\nA  file2.txt\n";
        let status = GitStatus::parse_porcelain_output(output);
        
        let deleted = status.files_with_status(FileStatus::Deleted);
        assert!(deleted.is_empty());
    }

    #[test]
    fn test_parse_porcelain_filenames_with_spaces() {
        let output = "M  file with spaces.txt\nA  another file.txt\n";
        let status = GitStatus::parse_porcelain_output(output);
        
        assert_eq!(status.files.len(), 2);
        assert!(status.files.contains(&(FileStatus::Modified, "file with spaces.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Added, "another file.txt".to_string())));
    }

    #[test]
    fn test_parse_porcelain_unicode_filenames() {
        let output = "M  测试文件.txt\nA  🚀rocket.txt\n";
        let status = GitStatus::parse_porcelain_output(output);
        
        assert_eq!(status.files.len(), 2);
        assert!(status.files.contains(&(FileStatus::Modified, "测试文件.txt".to_string())));
        assert!(status.files.contains(&(FileStatus::Added, "🚀rocket.txt".to_string())));
    }
}