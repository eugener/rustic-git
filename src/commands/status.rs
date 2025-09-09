use crate::utils::git;
use crate::{Repository, Result};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexStatus {
    Clean,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
}

impl IndexStatus {
    /// Convert a git porcelain index character to IndexStatus
    pub const fn from_char(c: char) -> Self {
        match c {
            'M' => Self::Modified,
            'A' => Self::Added,
            'D' => Self::Deleted,
            'R' => Self::Renamed,
            'C' => Self::Copied,
            _ => Self::Clean,
        }
    }

    /// Convert IndexStatus to its git porcelain character representation
    pub const fn to_char(&self) -> char {
        match self {
            Self::Clean => ' ',
            Self::Modified => 'M',
            Self::Added => 'A',
            Self::Deleted => 'D',
            Self::Renamed => 'R',
            Self::Copied => 'C',
        }
    }
}

impl fmt::Display for IndexStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorktreeStatus {
    Clean,
    Modified,
    Deleted,
    Untracked,
    Ignored,
}

impl WorktreeStatus {
    /// Convert a git porcelain worktree character to WorktreeStatus
    pub const fn from_char(c: char) -> Self {
        match c {
            'M' => Self::Modified,
            'D' => Self::Deleted,
            '?' => Self::Untracked,
            '!' => Self::Ignored,
            _ => Self::Clean,
        }
    }

    /// Convert WorktreeStatus to its git porcelain character representation
    pub const fn to_char(&self) -> char {
        match self {
            Self::Clean => ' ',
            Self::Modified => 'M',
            Self::Deleted => 'D',
            Self::Untracked => '?',
            Self::Ignored => '!',
        }
    }
}

impl fmt::Display for WorktreeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileEntry {
    pub path: PathBuf,
    pub index_status: IndexStatus,
    pub worktree_status: WorktreeStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GitStatus {
    pub entries: Box<[FileEntry]>,
}

impl GitStatus {
    pub fn is_clean(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn has_changes(&self) -> bool {
        !self.is_clean()
    }

    // New API methods for staged/unstaged files
    /// Get all files that have changes in the index (staged)
    pub fn staged_files(&self) -> impl Iterator<Item = &FileEntry> + '_ {
        self.entries
            .iter()
            .filter(|entry| !matches!(entry.index_status, IndexStatus::Clean))
    }

    /// Get all files that have changes in the working tree (unstaged)
    pub fn unstaged_files(&self) -> impl Iterator<Item = &FileEntry> + '_ {
        self.entries
            .iter()
            .filter(|entry| !matches!(entry.worktree_status, WorktreeStatus::Clean))
    }

    /// Get all untracked files (new API)
    pub fn untracked_entries(&self) -> impl Iterator<Item = &FileEntry> + '_ {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.worktree_status, WorktreeStatus::Untracked))
    }

    /// Get all ignored files
    pub fn ignored_files(&self) -> impl Iterator<Item = &FileEntry> + '_ {
        self.entries
            .iter()
            .filter(|entry| matches!(entry.worktree_status, WorktreeStatus::Ignored))
    }

    /// Get files with specific index status
    pub fn files_with_index_status(
        &self,
        status: IndexStatus,
    ) -> impl Iterator<Item = &FileEntry> + '_ {
        self.entries
            .iter()
            .filter(move |entry| entry.index_status == status)
    }

    /// Get files with specific worktree status  
    pub fn files_with_worktree_status(
        &self,
        status: WorktreeStatus,
    ) -> impl Iterator<Item = &FileEntry> + '_ {
        self.entries
            .iter()
            .filter(move |entry| entry.worktree_status == status)
    }

    /// Get all file entries
    pub fn entries(&self) -> &[FileEntry] {
        &self.entries
    }

    fn parse_porcelain_output(output: &str) -> Self {
        let mut entries = Vec::new();

        for line in output.lines() {
            if line.len() < 3 {
                continue;
            }

            let index_char = line.chars().nth(0).unwrap_or(' ');
            let worktree_char = line.chars().nth(1).unwrap_or(' ');
            let filename = line[3..].to_string();
            let path = PathBuf::from(&filename);

            let index_status = IndexStatus::from_char(index_char);
            let worktree_status = WorktreeStatus::from_char(worktree_char);

            // Skip entries that are completely clean
            if matches!(index_status, IndexStatus::Clean)
                && matches!(worktree_status, WorktreeStatus::Clean)
            {
                continue;
            }

            let entry = FileEntry {
                path,
                index_status,
                worktree_status,
            };

            entries.push(entry);
        }

        Self {
            entries: entries.into_boxed_slice(),
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

        assert_eq!(status.entries.len(), 4);

        // Find entries by path for testing
        let modified_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("modified.txt"))
            .unwrap();
        assert_eq!(modified_entry.index_status, IndexStatus::Modified);
        assert_eq!(modified_entry.worktree_status, WorktreeStatus::Clean);

        let added_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("added.txt"))
            .unwrap();
        assert_eq!(added_entry.index_status, IndexStatus::Added);
        assert_eq!(added_entry.worktree_status, WorktreeStatus::Clean);

        let deleted_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("deleted.txt"))
            .unwrap();
        assert_eq!(deleted_entry.index_status, IndexStatus::Deleted);
        assert_eq!(deleted_entry.worktree_status, WorktreeStatus::Clean);

        let untracked_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("untracked.txt"))
            .unwrap();
        assert_eq!(untracked_entry.index_status, IndexStatus::Clean);
        assert_eq!(untracked_entry.worktree_status, WorktreeStatus::Untracked);

        // Test new API methods
        let staged_files: Vec<_> = status.staged_files().collect();
        assert_eq!(staged_files.len(), 3); // modified, added, deleted are staged

        let untracked_files: Vec<_> = status.untracked_entries().collect();
        assert_eq!(untracked_files.len(), 1);
        assert_eq!(untracked_files[0].path.to_str(), Some("untracked.txt"));

        assert!(!status.is_clean());
        assert!(status.has_changes());
    }

    #[test]
    fn test_clean_repository_status() {
        let output = "";
        let status = GitStatus::parse_porcelain_output(output);

        assert!(status.is_clean());
        assert!(!status.has_changes());
        assert_eq!(status.entries.len(), 0);
        assert_eq!(status.staged_files().count(), 0);
        assert_eq!(status.untracked_entries().count(), 0);
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

        assert_eq!(status.entries.len(), 2);

        let valid_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("valid.txt"))
            .unwrap();
        assert_eq!(valid_entry.index_status, IndexStatus::Modified);

        let another_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("another.txt"))
            .unwrap();
        assert_eq!(another_entry.index_status, IndexStatus::Added);
    }

    #[test]
    fn test_parse_porcelain_all_status_types() {
        let output = "M  modified.txt\nA  added.txt\nD  deleted.txt\nR  renamed.txt\nC  copied.txt\n?? untracked.txt\n!! ignored.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.entries.len(), 7);

        let modified = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("modified.txt"))
            .unwrap();
        assert_eq!(modified.index_status, IndexStatus::Modified);

        let added = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("added.txt"))
            .unwrap();
        assert_eq!(added.index_status, IndexStatus::Added);

        let deleted = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("deleted.txt"))
            .unwrap();
        assert_eq!(deleted.index_status, IndexStatus::Deleted);

        let renamed = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("renamed.txt"))
            .unwrap();
        assert_eq!(renamed.index_status, IndexStatus::Renamed);

        let copied = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("copied.txt"))
            .unwrap();
        assert_eq!(copied.index_status, IndexStatus::Copied);

        let untracked = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("untracked.txt"))
            .unwrap();
        assert_eq!(untracked.worktree_status, WorktreeStatus::Untracked);

        let ignored = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("ignored.txt"))
            .unwrap();
        assert_eq!(ignored.worktree_status, WorktreeStatus::Ignored);
    }

    #[test]
    fn test_parse_porcelain_worktree_modifications() {
        let output = " M worktree_modified.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.entries.len(), 1);
        let entry = &status.entries[0];
        assert_eq!(entry.path.to_str(), Some("worktree_modified.txt"));
        assert_eq!(entry.index_status, IndexStatus::Clean);
        assert_eq!(entry.worktree_status, WorktreeStatus::Modified);
    }

    #[test]
    fn test_parse_porcelain_unknown_status() {
        let output = "XY unknown.txt\nZ  another_unknown.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        // Unknown statuses should be treated as clean/clean and ignored
        assert_eq!(status.entries.len(), 0);
    }

    #[test]
    fn test_index_status_equality() {
        assert_eq!(IndexStatus::Modified, IndexStatus::Modified);
        assert_ne!(IndexStatus::Modified, IndexStatus::Added);
        assert_eq!(IndexStatus::Clean, IndexStatus::Clean);
    }

    #[test]
    fn test_worktree_status_equality() {
        assert_eq!(WorktreeStatus::Modified, WorktreeStatus::Modified);
        assert_ne!(WorktreeStatus::Modified, WorktreeStatus::Untracked);
        assert_eq!(WorktreeStatus::Clean, WorktreeStatus::Clean);
    }

    #[test]
    fn test_index_status_char_conversion() {
        // Test from_char
        assert_eq!(IndexStatus::from_char('M'), IndexStatus::Modified);
        assert_eq!(IndexStatus::from_char('A'), IndexStatus::Added);
        assert_eq!(IndexStatus::from_char('D'), IndexStatus::Deleted);
        assert_eq!(IndexStatus::from_char('R'), IndexStatus::Renamed);
        assert_eq!(IndexStatus::from_char('C'), IndexStatus::Copied);
        assert_eq!(IndexStatus::from_char(' '), IndexStatus::Clean);
        assert_eq!(IndexStatus::from_char('X'), IndexStatus::Clean); // unknown char

        // Test to_char
        assert_eq!(IndexStatus::Modified.to_char(), 'M');
        assert_eq!(IndexStatus::Added.to_char(), 'A');
        assert_eq!(IndexStatus::Deleted.to_char(), 'D');
        assert_eq!(IndexStatus::Renamed.to_char(), 'R');
        assert_eq!(IndexStatus::Copied.to_char(), 'C');
        assert_eq!(IndexStatus::Clean.to_char(), ' ');
    }

    #[test]
    fn test_worktree_status_char_conversion() {
        // Test from_char
        assert_eq!(WorktreeStatus::from_char('M'), WorktreeStatus::Modified);
        assert_eq!(WorktreeStatus::from_char('D'), WorktreeStatus::Deleted);
        assert_eq!(WorktreeStatus::from_char('?'), WorktreeStatus::Untracked);
        assert_eq!(WorktreeStatus::from_char('!'), WorktreeStatus::Ignored);
        assert_eq!(WorktreeStatus::from_char(' '), WorktreeStatus::Clean);
        assert_eq!(WorktreeStatus::from_char('X'), WorktreeStatus::Clean); // unknown char

        // Test to_char
        assert_eq!(WorktreeStatus::Modified.to_char(), 'M');
        assert_eq!(WorktreeStatus::Deleted.to_char(), 'D');
        assert_eq!(WorktreeStatus::Untracked.to_char(), '?');
        assert_eq!(WorktreeStatus::Ignored.to_char(), '!');
        assert_eq!(WorktreeStatus::Clean.to_char(), ' ');
    }

    #[test]
    fn test_bidirectional_char_conversion() {
        // Test that from_char(to_char(x)) == x for IndexStatus
        for status in [
            IndexStatus::Clean,
            IndexStatus::Modified,
            IndexStatus::Added,
            IndexStatus::Deleted,
            IndexStatus::Renamed,
            IndexStatus::Copied,
        ] {
            assert_eq!(IndexStatus::from_char(status.to_char()), status);
        }

        // Test that from_char(to_char(x)) == x for WorktreeStatus
        for status in [
            WorktreeStatus::Clean,
            WorktreeStatus::Modified,
            WorktreeStatus::Deleted,
            WorktreeStatus::Untracked,
            WorktreeStatus::Ignored,
        ] {
            assert_eq!(WorktreeStatus::from_char(status.to_char()), status);
        }
    }

    #[test]
    fn test_status_display() {
        // Test IndexStatus Display
        assert_eq!(format!("{}", IndexStatus::Modified), "M");
        assert_eq!(format!("{}", IndexStatus::Added), "A");
        assert_eq!(format!("{}", IndexStatus::Clean), " ");

        // Test WorktreeStatus Display
        assert_eq!(format!("{}", WorktreeStatus::Modified), "M");
        assert_eq!(format!("{}", WorktreeStatus::Untracked), "?");
        assert_eq!(format!("{}", WorktreeStatus::Clean), " ");
    }

    #[test]
    fn test_file_entry_equality() {
        let entry1 = FileEntry {
            path: PathBuf::from("test.txt"),
            index_status: IndexStatus::Modified,
            worktree_status: WorktreeStatus::Clean,
        };
        let entry2 = FileEntry {
            path: PathBuf::from("test.txt"),
            index_status: IndexStatus::Modified,
            worktree_status: WorktreeStatus::Clean,
        };
        let entry3 = FileEntry {
            path: PathBuf::from("other.txt"),
            index_status: IndexStatus::Modified,
            worktree_status: WorktreeStatus::Clean,
        };

        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }

    #[test]
    fn test_git_status_equality() {
        let entries1 = vec![
            FileEntry {
                path: PathBuf::from("file1.txt"),
                index_status: IndexStatus::Modified,
                worktree_status: WorktreeStatus::Clean,
            },
            FileEntry {
                path: PathBuf::from("file2.txt"),
                index_status: IndexStatus::Added,
                worktree_status: WorktreeStatus::Clean,
            },
        ];
        let entries2 = entries1.clone();
        let entries3 = vec![FileEntry {
            path: PathBuf::from("different.txt"),
            index_status: IndexStatus::Modified,
            worktree_status: WorktreeStatus::Clean,
        }];

        let status1 = GitStatus {
            entries: entries1.into_boxed_slice(),
        };
        let status2 = GitStatus {
            entries: entries2.into_boxed_slice(),
        };
        let status3 = GitStatus {
            entries: entries3.into_boxed_slice(),
        };

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    #[test]
    fn test_git_status_clone() {
        let entries = vec![FileEntry {
            path: PathBuf::from("file1.txt"),
            index_status: IndexStatus::Modified,
            worktree_status: WorktreeStatus::Clean,
        }];
        let status1 = GitStatus {
            entries: entries.into_boxed_slice(),
        };
        let status2 = status1.clone();

        assert_eq!(status1, status2);
    }

    #[test]
    fn test_git_status_debug() {
        let entries = vec![FileEntry {
            path: PathBuf::from("file1.txt"),
            index_status: IndexStatus::Modified,
            worktree_status: WorktreeStatus::Clean,
        }];
        let status = GitStatus {
            entries: entries.into_boxed_slice(),
        };
        let debug_str = format!("{:?}", status);

        assert!(debug_str.contains("GitStatus"));
        assert!(debug_str.contains("Modified"));
        assert!(debug_str.contains("file1.txt"));
    }

    #[test]
    fn test_new_api_methods() {
        let output = "M  file1.txt\nMM file2.txt\nA  file3.txt\n D file4.txt\n?? file5.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        // Test staged files (index changes)
        let staged: Vec<_> = status.staged_files().collect();
        assert_eq!(staged.len(), 3); // M, MM, A (not D since it has clean index status)

        // Test unstaged files (worktree changes)
        let unstaged: Vec<_> = status.unstaged_files().collect();
        assert_eq!(unstaged.len(), 3); // MM, D, ??

        // Test untracked files
        let untracked: Vec<_> = status.untracked_entries().collect();
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path.to_str(), Some("file5.txt"));

        // Test index status filtering
        let modified_in_index: Vec<_> = status
            .files_with_index_status(IndexStatus::Modified)
            .collect();
        assert_eq!(modified_in_index.len(), 2); // file1.txt, file2.txt

        // Test worktree status filtering
        let modified_in_worktree: Vec<_> = status
            .files_with_worktree_status(WorktreeStatus::Modified)
            .collect();
        assert_eq!(modified_in_worktree.len(), 1); // file2.txt
    }

    #[test]
    fn test_parse_porcelain_filenames_with_spaces() {
        let output = "M  file with spaces.txt\nA  another file.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.entries.len(), 2);

        let spaced_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("file with spaces.txt"))
            .unwrap();
        assert_eq!(spaced_entry.index_status, IndexStatus::Modified);

        let another_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("another file.txt"))
            .unwrap();
        assert_eq!(another_entry.index_status, IndexStatus::Added);
    }

    #[test]
    fn test_parse_porcelain_unicode_filenames() {
        let output = "M  测试文件.txt\nA  🚀rocket.txt\n";
        let status = GitStatus::parse_porcelain_output(output);

        assert_eq!(status.entries.len(), 2);

        let chinese_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("测试文件.txt"))
            .unwrap();
        assert_eq!(chinese_entry.index_status, IndexStatus::Modified);

        let rocket_entry = status
            .entries
            .iter()
            .find(|e| e.path.to_str() == Some("🚀rocket.txt"))
            .unwrap();
        assert_eq!(rocket_entry.index_status, IndexStatus::Added);
    }
}
