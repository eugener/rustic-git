use crate::types::Hash;
use crate::utils::git;
use crate::{Repository, Result};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

impl DiffStatus {
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'A' => Some(Self::Added),
            'M' => Some(Self::Modified),
            'D' => Some(Self::Deleted),
            'R' => Some(Self::Renamed),
            'C' => Some(Self::Copied),
            _ => None,
        }
    }

    pub const fn to_char(&self) -> char {
        match self {
            Self::Added => 'A',
            Self::Modified => 'M',
            Self::Deleted => 'D',
            Self::Renamed => 'R',
            Self::Copied => 'C',
        }
    }
}

impl fmt::Display for DiffStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
            Self::Copied => "copied",
        };
        write!(f, "{}", status_str)
    }
}

#[derive(Debug, Clone)]
pub struct DiffChunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Box<[DiffLine]>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineType {
    Context,
    Added,
    Removed,
}

impl DiffLineType {
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            ' ' => Some(Self::Context),
            '+' => Some(Self::Added),
            '-' => Some(Self::Removed),
            _ => None,
        }
    }

    pub const fn to_char(&self) -> char {
        match self {
            Self::Context => ' ',
            Self::Added => '+',
            Self::Removed => '-',
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub status: DiffStatus,
    pub chunks: Box<[DiffChunk]>,
    pub additions: usize,
    pub deletions: usize,
}

impl FileDiff {
    pub fn new(path: PathBuf, status: DiffStatus) -> Self {
        Self {
            path,
            old_path: None,
            status,
            chunks: Box::new([]),
            additions: 0,
            deletions: 0,
        }
    }

    pub fn with_old_path(mut self, old_path: PathBuf) -> Self {
        self.old_path = Some(old_path);
        self
    }

    pub fn with_chunks(mut self, chunks: Vec<DiffChunk>) -> Self {
        self.chunks = chunks.into_boxed_slice();
        self
    }

    pub fn with_stats(mut self, additions: usize, deletions: usize) -> Self {
        self.additions = additions;
        self.deletions = deletions;
        self
    }

    pub fn is_binary(&self) -> bool {
        self.chunks.is_empty() && (self.additions > 0 || self.deletions > 0)
    }
}

impl fmt::Display for FileDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.old_path {
            Some(old_path) => write!(
                f,
                "{} {} -> {}",
                self.status,
                old_path.display(),
                self.path.display()
            ),
            None => write!(f, "{} {}", self.status, self.path.display()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl DiffStats {
    pub fn new() -> Self {
        Self {
            files_changed: 0,
            insertions: 0,
            deletions: 0,
        }
    }

    pub fn add_file(&mut self, additions: usize, deletions: usize) {
        self.files_changed += 1;
        self.insertions += additions;
        self.deletions += deletions;
    }
}

impl Default for DiffStats {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DiffStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} files changed, {} insertions(+), {} deletions(-)",
            self.files_changed, self.insertions, self.deletions
        )
    }
}

#[derive(Debug, Clone)]
pub struct DiffOutput {
    pub files: Box<[FileDiff]>,
    pub stats: DiffStats,
}

impl DiffOutput {
    pub fn new(files: Vec<FileDiff>) -> Self {
        let mut stats = DiffStats::new();
        for file in &files {
            stats.add_file(file.additions, file.deletions);
        }

        Self {
            files: files.into_boxed_slice(),
            stats,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, FileDiff> {
        self.files.iter()
    }

    pub fn files_with_status(&self, status: DiffStatus) -> impl Iterator<Item = &FileDiff> {
        self.files.iter().filter(move |f| f.status == status)
    }
}

impl fmt::Display for DiffOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return writeln!(f, "No differences found");
        }

        for file in &self.files {
            writeln!(f, "{}", file)?;
        }
        writeln!(f, "{}", self.stats)
    }
}

#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub context_lines: Option<usize>,
    pub ignore_whitespace: bool,
    pub ignore_whitespace_change: bool,
    pub ignore_blank_lines: bool,
    pub paths: Option<Vec<PathBuf>>,
    pub name_only: bool,
    pub stat_only: bool,
    pub numstat: bool,
    pub cached: bool,
    pub no_index: bool,
}

impl DiffOptions {
    pub fn new() -> Self {
        Self {
            context_lines: None,
            ignore_whitespace: false,
            ignore_whitespace_change: false,
            ignore_blank_lines: false,
            paths: None,
            name_only: false,
            stat_only: false,
            numstat: false,
            cached: false,
            no_index: false,
        }
    }

    pub fn context_lines(mut self, lines: usize) -> Self {
        self.context_lines = Some(lines);
        self
    }

    pub fn ignore_whitespace(mut self) -> Self {
        self.ignore_whitespace = true;
        self
    }

    pub fn ignore_whitespace_change(mut self) -> Self {
        self.ignore_whitespace_change = true;
        self
    }

    pub fn ignore_blank_lines(mut self) -> Self {
        self.ignore_blank_lines = true;
        self
    }

    pub fn paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.paths = Some(paths);
        self
    }

    pub fn name_only(mut self) -> Self {
        self.name_only = true;
        self
    }

    pub fn stat_only(mut self) -> Self {
        self.stat_only = true;
        self
    }

    pub fn numstat(mut self) -> Self {
        self.numstat = true;
        self
    }

    pub fn cached(mut self) -> Self {
        self.cached = true;
        self
    }

    pub fn no_index(mut self) -> Self {
        self.no_index = true;
        self
    }
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository {
    /// Get diff between working directory and index (staged changes)
    ///
    /// Shows changes that are not yet staged for commit.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DiffOutput` or a `GitError`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    ///
    /// # fn main() -> rustic_git::Result<()> {
    /// let repo = Repository::open(".")?;
    /// let diff = repo.diff()?;
    /// println!("Unstaged changes: {}", diff);
    /// # Ok(())
    /// # }
    /// ```
    pub fn diff(&self) -> Result<DiffOutput> {
        self.diff_with_options(&DiffOptions::new())
    }

    /// Get diff between index and HEAD (staged changes)
    ///
    /// Shows changes that are staged for commit.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DiffOutput` or a `GitError`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::Repository;
    ///
    /// # fn main() -> rustic_git::Result<()> {
    /// let repo = Repository::open(".")?;
    /// let diff = repo.diff_staged()?;
    /// println!("Staged changes: {}", diff);
    /// # Ok(())
    /// # }
    /// ```
    pub fn diff_staged(&self) -> Result<DiffOutput> {
        self.diff_with_options(&DiffOptions::new().cached())
    }

    /// Get diff between working directory and HEAD
    ///
    /// Shows all changes (both staged and unstaged) compared to the last commit.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DiffOutput` or a `GitError`.
    pub fn diff_head(&self) -> Result<DiffOutput> {
        self.diff_commits_with_options(None, Some(&Hash::from("HEAD")), &DiffOptions::new())
    }

    /// Get diff between two commits
    ///
    /// # Arguments
    ///
    /// * `from` - The starting commit hash
    /// * `to` - The ending commit hash
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DiffOutput` or a `GitError`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, Hash};
    ///
    /// # fn main() -> rustic_git::Result<()> {
    /// let repo = Repository::open(".")?;
    /// let from = Hash::from("abc123");
    /// let to = Hash::from("def456");
    /// let diff = repo.diff_commits(&from, &to)?;
    /// println!("Changes between commits: {}", diff);
    /// # Ok(())
    /// # }
    /// ```
    pub fn diff_commits(&self, from: &Hash, to: &Hash) -> Result<DiffOutput> {
        self.diff_commits_with_options(Some(from), Some(to), &DiffOptions::new())
    }

    /// Get diff with custom options
    ///
    /// # Arguments
    ///
    /// * `options` - The diff options to use
    ///
    /// # Returns
    ///
    /// A `Result` containing the `DiffOutput` or a `GitError`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustic_git::{Repository, DiffOptions};
    ///
    /// # fn main() -> rustic_git::Result<()> {
    /// let repo = Repository::open(".")?;
    /// let options = DiffOptions::new()
    ///     .ignore_whitespace()
    ///     .context_lines(5);
    /// let diff = repo.diff_with_options(&options)?;
    /// println!("Diff with options: {}", diff);
    /// # Ok(())
    /// # }
    /// ```
    pub fn diff_with_options(&self, options: &DiffOptions) -> Result<DiffOutput> {
        self.diff_commits_with_options(None, None, options)
    }

    /// Internal method to handle all diff operations
    fn diff_commits_with_options(
        &self,
        from: Option<&Hash>,
        to: Option<&Hash>,
        options: &DiffOptions,
    ) -> Result<DiffOutput> {
        Self::ensure_git()?;

        let mut args = vec!["diff".to_string()];

        // Add options
        if let Some(lines) = options.context_lines {
            args.push(format!("-U{}", lines));
        }
        if options.ignore_whitespace {
            args.push("--ignore-all-space".to_string());
        }
        if options.ignore_whitespace_change {
            args.push("--ignore-space-change".to_string());
        }
        if options.ignore_blank_lines {
            args.push("--ignore-blank-lines".to_string());
        }
        if options.name_only {
            args.push("--name-only".to_string());
        }
        if options.stat_only {
            args.push("--stat".to_string());
        }
        if options.numstat {
            args.push("--numstat".to_string());
        }
        if options.cached {
            args.push("--cached".to_string());
        }
        if options.no_index {
            args.push("--no-index".to_string());
        }

        // Add commit range if specified
        match (from, to) {
            (Some(from_hash), Some(to_hash)) => {
                args.push(format!("{}..{}", from_hash.as_str(), to_hash.as_str()));
            }
            (None, Some(to_hash)) => {
                args.push(to_hash.as_str().to_string());
            }
            (Some(from_hash), None) => {
                args.push(format!("{}..HEAD", from_hash.as_str()));
            }
            (None, None) => {
                // Default diff behavior
            }
        }

        // Add paths if specified
        if let Some(paths) = &options.paths {
            args.push("--".to_string());
            for path in paths {
                args.push(path.to_string_lossy().to_string());
            }
        }

        let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = git(&args_str, Some(self.repo_path()))?;

        if options.name_only {
            parse_name_only_output(&output)
        } else if options.stat_only {
            parse_stat_output(&output)
        } else if options.numstat {
            parse_numstat_output(&output)
        } else {
            parse_diff_output(&output)
        }
    }
}

fn parse_name_only_output(output: &str) -> Result<DiffOutput> {
    let files: Vec<FileDiff> = output
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| FileDiff::new(PathBuf::from(line), DiffStatus::Modified))
        .collect();

    Ok(DiffOutput::new(files))
}

fn parse_stat_output(output: &str) -> Result<DiffOutput> {
    let mut files = Vec::new();
    let mut stats = DiffStats::new();

    for line in output.lines() {
        if line.contains(" | ") {
            let parts: Vec<&str> = line.split(" | ").collect();
            if parts.len() == 2 {
                let path = PathBuf::from(parts[0].trim());
                let file_diff = FileDiff::new(path, DiffStatus::Modified);
                files.push(file_diff);
            }
        } else if line.contains("files changed") || line.contains("file changed") {
            // Parse summary line like "3 files changed, 15 insertions(+), 5 deletions(-)"
            if let Some(files_part) = line.split(',').next()
                && let Some(num_str) = files_part.split_whitespace().next()
                && let Ok(num) = num_str.parse::<usize>()
            {
                stats.files_changed = num;
            }
        }
    }

    Ok(DiffOutput {
        files: files.into_boxed_slice(),
        stats,
    })
}

fn parse_numstat_output(output: &str) -> Result<DiffOutput> {
    let files: Vec<FileDiff> = output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let additions = parts[0].parse().unwrap_or(0);
                let deletions = parts[1].parse().unwrap_or(0);
                let path = PathBuf::from(parts[2]);

                let status = if additions > 0 && deletions == 0 {
                    DiffStatus::Added
                } else if additions == 0 && deletions > 0 {
                    DiffStatus::Deleted
                } else {
                    DiffStatus::Modified
                };

                Some(FileDiff::new(path, status).with_stats(additions, deletions))
            } else {
                None
            }
        })
        .collect();

    Ok(DiffOutput::new(files))
}

fn parse_diff_output(output: &str) -> Result<DiffOutput> {
    // For now, return a simplified parser
    // In a full implementation, this would parse the complete diff format
    let files: Vec<FileDiff> = output
        .lines()
        .filter(|line| line.starts_with("diff --git"))
        .filter_map(|line| {
            // Extract file paths from "diff --git a/file b/file"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let path_str = parts[3].strip_prefix("b/").unwrap_or(parts[3]);
                Some(FileDiff::new(PathBuf::from(path_str), DiffStatus::Modified))
            } else {
                None
            }
        })
        .collect();

    Ok(DiffOutput::new(files))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_diff_status_char_conversion() {
        assert_eq!(DiffStatus::from_char('A'), Some(DiffStatus::Added));
        assert_eq!(DiffStatus::from_char('M'), Some(DiffStatus::Modified));
        assert_eq!(DiffStatus::from_char('D'), Some(DiffStatus::Deleted));
        assert_eq!(DiffStatus::from_char('R'), Some(DiffStatus::Renamed));
        assert_eq!(DiffStatus::from_char('C'), Some(DiffStatus::Copied));
        assert_eq!(DiffStatus::from_char('X'), None);

        assert_eq!(DiffStatus::Added.to_char(), 'A');
        assert_eq!(DiffStatus::Modified.to_char(), 'M');
        assert_eq!(DiffStatus::Deleted.to_char(), 'D');
        assert_eq!(DiffStatus::Renamed.to_char(), 'R');
        assert_eq!(DiffStatus::Copied.to_char(), 'C');
    }

    #[test]
    fn test_diff_status_display() {
        assert_eq!(DiffStatus::Added.to_string(), "added");
        assert_eq!(DiffStatus::Modified.to_string(), "modified");
        assert_eq!(DiffStatus::Deleted.to_string(), "deleted");
        assert_eq!(DiffStatus::Renamed.to_string(), "renamed");
        assert_eq!(DiffStatus::Copied.to_string(), "copied");
    }

    #[test]
    fn test_diff_line_type_char_conversion() {
        assert_eq!(DiffLineType::from_char(' '), Some(DiffLineType::Context));
        assert_eq!(DiffLineType::from_char('+'), Some(DiffLineType::Added));
        assert_eq!(DiffLineType::from_char('-'), Some(DiffLineType::Removed));
        assert_eq!(DiffLineType::from_char('X'), None);

        assert_eq!(DiffLineType::Context.to_char(), ' ');
        assert_eq!(DiffLineType::Added.to_char(), '+');
        assert_eq!(DiffLineType::Removed.to_char(), '-');
    }

    #[test]
    fn test_file_diff_creation() {
        let path = PathBuf::from("test.txt");
        let file_diff = FileDiff::new(path.clone(), DiffStatus::Modified);

        assert_eq!(file_diff.path, path);
        assert_eq!(file_diff.status, DiffStatus::Modified);
        assert_eq!(file_diff.old_path, None);
        assert_eq!(file_diff.chunks.len(), 0);
        assert_eq!(file_diff.additions, 0);
        assert_eq!(file_diff.deletions, 0);
    }

    #[test]
    fn test_file_diff_with_old_path() {
        let old_path = PathBuf::from("old.txt");
        let new_path = PathBuf::from("new.txt");
        let file_diff =
            FileDiff::new(new_path.clone(), DiffStatus::Renamed).with_old_path(old_path.clone());

        assert_eq!(file_diff.path, new_path);
        assert_eq!(file_diff.old_path, Some(old_path));
        assert_eq!(file_diff.status, DiffStatus::Renamed);
    }

    #[test]
    fn test_file_diff_with_stats() {
        let path = PathBuf::from("test.txt");
        let file_diff = FileDiff::new(path, DiffStatus::Modified).with_stats(10, 5);

        assert_eq!(file_diff.additions, 10);
        assert_eq!(file_diff.deletions, 5);
    }

    #[test]
    fn test_diff_stats_creation() {
        let mut stats = DiffStats::new();
        assert_eq!(stats.files_changed, 0);
        assert_eq!(stats.insertions, 0);
        assert_eq!(stats.deletions, 0);

        stats.add_file(10, 5);
        assert_eq!(stats.files_changed, 1);
        assert_eq!(stats.insertions, 10);
        assert_eq!(stats.deletions, 5);

        stats.add_file(3, 2);
        assert_eq!(stats.files_changed, 2);
        assert_eq!(stats.insertions, 13);
        assert_eq!(stats.deletions, 7);
    }

    #[test]
    fn test_diff_stats_display() {
        let mut stats = DiffStats::new();
        stats.add_file(10, 5);
        stats.add_file(3, 2);

        let display = stats.to_string();
        assert!(display.contains("2 files changed"));
        assert!(display.contains("13 insertions(+)"));
        assert!(display.contains("7 deletions(-)"));
    }

    #[test]
    fn test_diff_output_creation() {
        let files = vec![
            FileDiff::new(PathBuf::from("file1.txt"), DiffStatus::Added).with_stats(5, 0),
            FileDiff::new(PathBuf::from("file2.txt"), DiffStatus::Modified).with_stats(3, 2),
        ];

        let diff_output = DiffOutput::new(files);

        assert_eq!(diff_output.len(), 2);
        assert!(!diff_output.is_empty());
        assert_eq!(diff_output.stats.files_changed, 2);
        assert_eq!(diff_output.stats.insertions, 8);
        assert_eq!(diff_output.stats.deletions, 2);
    }

    #[test]
    fn test_diff_output_empty() {
        let diff_output = DiffOutput::new(vec![]);

        assert_eq!(diff_output.len(), 0);
        assert!(diff_output.is_empty());
        assert_eq!(diff_output.stats.files_changed, 0);
    }

    #[test]
    fn test_diff_output_files_with_status() {
        let files = vec![
            FileDiff::new(PathBuf::from("added.txt"), DiffStatus::Added),
            FileDiff::new(PathBuf::from("modified.txt"), DiffStatus::Modified),
            FileDiff::new(PathBuf::from("deleted.txt"), DiffStatus::Deleted),
        ];

        let diff_output = DiffOutput::new(files);

        let added_files: Vec<_> = diff_output.files_with_status(DiffStatus::Added).collect();
        assert_eq!(added_files.len(), 1);
        assert_eq!(added_files[0].path, PathBuf::from("added.txt"));

        let modified_files: Vec<_> = diff_output
            .files_with_status(DiffStatus::Modified)
            .collect();
        assert_eq!(modified_files.len(), 1);
        assert_eq!(modified_files[0].path, PathBuf::from("modified.txt"));
    }

    #[test]
    fn test_diff_options_builder() {
        let options = DiffOptions::new()
            .context_lines(5)
            .ignore_whitespace()
            .ignore_whitespace_change()
            .ignore_blank_lines()
            .name_only()
            .stat_only()
            .numstat()
            .cached()
            .no_index();

        assert_eq!(options.context_lines, Some(5));
        assert!(options.ignore_whitespace);
        assert!(options.ignore_whitespace_change);
        assert!(options.ignore_blank_lines);
        assert!(options.name_only);
        assert!(options.stat_only);
        assert!(options.numstat);
        assert!(options.cached);
        assert!(options.no_index);
    }

    #[test]
    fn test_diff_options_with_paths() {
        let paths = vec![PathBuf::from("src/"), PathBuf::from("tests/")];
        let options = DiffOptions::new().paths(paths.clone());

        assert_eq!(options.paths, Some(paths));
    }

    #[test]
    fn test_parse_name_only_output() {
        let output = "file1.txt\nfile2.rs\nsrc/lib.rs\n";
        let result = parse_name_only_output(output).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.files[0].path, PathBuf::from("file1.txt"));
        assert_eq!(result.files[1].path, PathBuf::from("file2.rs"));
        assert_eq!(result.files[2].path, PathBuf::from("src/lib.rs"));
    }

    #[test]
    fn test_parse_numstat_output() {
        let output = "5\t0\tfile1.txt\n3\t2\tfile2.rs\n0\t10\tfile3.py\n";
        let result = parse_numstat_output(output).unwrap();

        assert_eq!(result.len(), 3);

        assert_eq!(result.files[0].path, PathBuf::from("file1.txt"));
        assert_eq!(result.files[0].status, DiffStatus::Added);
        assert_eq!(result.files[0].additions, 5);
        assert_eq!(result.files[0].deletions, 0);

        assert_eq!(result.files[1].path, PathBuf::from("file2.rs"));
        assert_eq!(result.files[1].status, DiffStatus::Modified);
        assert_eq!(result.files[1].additions, 3);
        assert_eq!(result.files[1].deletions, 2);

        assert_eq!(result.files[2].path, PathBuf::from("file3.py"));
        assert_eq!(result.files[2].status, DiffStatus::Deleted);
        assert_eq!(result.files[2].additions, 0);
        assert_eq!(result.files[2].deletions, 10);
    }

    #[test]
    fn test_repository_diff_basic() {
        let repo_path = env::temp_dir().join("rustic_git_diff_test");
        // Clean up any previous run
        if repo_path.exists() {
            std::fs::remove_dir_all(&repo_path).ok();
        }

        // Initialize repository
        let repo = Repository::init(&repo_path, false).unwrap();

        // Test diff on empty repository (should not fail)
        let result = repo.diff();
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_dir_all(&repo_path).ok();
    }
}
