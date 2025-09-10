use crate::types::Hash;
use crate::utils::git;
use crate::{Repository, Result};
use chrono::{DateTime, Utc};
use std::fmt;
use std::path::PathBuf;

/// Git log format string for parsing commit information
/// Format: hash|author_name|author_email|author_timestamp|committer_name|committer_email|committer_timestamp|parent_hashes|subject|body
const GIT_LOG_FORMAT: &str = "--pretty=format:%H|%an|%ae|%at|%cn|%ce|%ct|%P|%s|%b";

/// Date format for git date filters
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMessage {
    pub subject: String,
    pub body: Option<String>,
}

impl CommitMessage {
    pub fn new(subject: String, body: Option<String>) -> Self {
        Self { subject, body }
    }

    /// Get the full message (subject + body if present)
    pub fn full(&self) -> String {
        match &self.body {
            Some(body) => format!("{}\n\n{}", self.subject, body),
            None => self.subject.clone(),
        }
    }

    /// Check if message is empty
    pub fn is_empty(&self) -> bool {
        self.subject.is_empty()
    }
}

impl fmt::Display for CommitMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub hash: Hash,
    pub author: Author,
    pub committer: Author,
    pub message: CommitMessage,
    pub timestamp: DateTime<Utc>,
    pub parents: Box<[Hash]>,
}

impl Commit {
    /// Check if this is a merge commit (has multiple parents)
    pub fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }

    /// Check if this is a root commit (has no parents)
    pub fn is_root(&self) -> bool {
        self.parents.is_empty()
    }

    /// Get the main parent commit hash (first parent for merges)
    pub fn main_parent(&self) -> Option<&Hash> {
        self.parents.first()
    }

    /// Check if commit matches author
    pub fn is_authored_by(&self, author: &str) -> bool {
        self.author.name.contains(author) || self.author.email.contains(author)
    }

    /// Check if commit message contains text
    pub fn message_contains(&self, text: &str) -> bool {
        self.message
            .subject
            .to_lowercase()
            .contains(&text.to_lowercase())
            || self
                .message
                .body
                .as_ref()
                .is_some_and(|body| body.to_lowercase().contains(&text.to_lowercase()))
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} by {} at {}",
            self.hash.short(),
            self.message.subject,
            self.author.name,
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommitLog {
    commits: Box<[Commit]>,
}

impl CommitLog {
    /// Create a new CommitLog from a vector of commits
    pub fn new(commits: Vec<Commit>) -> Self {
        Self {
            commits: commits.into_boxed_slice(),
        }
    }

    /// Get all commits
    pub fn all(&self) -> &[Commit] {
        &self.commits
    }

    /// Get an iterator over all commits
    pub fn iter(&self) -> impl Iterator<Item = &Commit> {
        self.commits.iter()
    }

    /// Get commits by a specific author
    pub fn by_author(&self, author: &str) -> impl Iterator<Item = &Commit> {
        self.commits
            .iter()
            .filter(move |c| c.is_authored_by(author))
    }

    /// Get commits since a specific date
    pub fn since(&self, date: DateTime<Utc>) -> impl Iterator<Item = &Commit> {
        self.commits.iter().filter(move |c| c.timestamp >= date)
    }

    /// Get commits until a specific date
    pub fn until(&self, date: DateTime<Utc>) -> impl Iterator<Item = &Commit> {
        self.commits.iter().filter(move |c| c.timestamp <= date)
    }

    /// Get commits with message containing text
    pub fn with_message_containing(&self, text: &str) -> impl Iterator<Item = &Commit> {
        let text = text.to_lowercase();
        self.commits
            .iter()
            .filter(move |c| c.message_contains(&text))
    }

    /// Get only merge commits
    pub fn merges_only(&self) -> impl Iterator<Item = &Commit> {
        self.commits.iter().filter(|c| c.is_merge())
    }

    /// Get commits excluding merges
    pub fn no_merges(&self) -> impl Iterator<Item = &Commit> {
        self.commits.iter().filter(|c| !c.is_merge())
    }

    /// Find commit by full hash
    pub fn find_by_hash(&self, hash: &Hash) -> Option<&Commit> {
        self.commits.iter().find(|c| &c.hash == hash)
    }

    /// Find commit by short hash
    pub fn find_by_short_hash(&self, short: &str) -> Option<&Commit> {
        self.commits.iter().find(|c| c.hash.short() == short)
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.commits.is_empty()
    }

    /// Get the count of commits
    pub fn len(&self) -> usize {
        self.commits.len()
    }

    /// Get the first (most recent) commit
    pub fn first(&self) -> Option<&Commit> {
        self.commits.first()
    }

    /// Get the last (oldest) commit
    pub fn last(&self) -> Option<&Commit> {
        self.commits.last()
    }
}

impl fmt::Display for CommitLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for commit in &self.commits {
            writeln!(f, "{}", commit)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    pub max_count: Option<usize>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub committer: Option<String>,
    pub grep: Option<String>,
    pub paths: Vec<PathBuf>,
    pub follow_renames: bool,
    pub merges_only: bool,
    pub no_merges: bool,
}

impl LogOptions {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum number of commits to retrieve
    pub fn max_count(mut self, count: usize) -> Self {
        self.max_count = Some(count);
        self
    }

    /// Filter commits since a date
    pub fn since(mut self, date: DateTime<Utc>) -> Self {
        self.since = Some(date);
        self
    }

    /// Filter commits until a date
    pub fn until(mut self, date: DateTime<Utc>) -> Self {
        self.until = Some(date);
        self
    }

    /// Filter by author name or email
    pub fn author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    /// Filter by committer name or email
    pub fn committer(mut self, committer: String) -> Self {
        self.committer = Some(committer);
        self
    }

    /// Filter by commit message content
    pub fn grep(mut self, pattern: String) -> Self {
        self.grep = Some(pattern);
        self
    }

    /// Filter by file paths
    pub fn paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.paths = paths;
        self
    }

    /// Follow file renames
    pub fn follow_renames(mut self, follow: bool) -> Self {
        self.follow_renames = follow;
        self
    }

    /// Show only merge commits
    pub fn merges_only(mut self, only: bool) -> Self {
        self.merges_only = only;
        self
    }

    /// Exclude merge commits
    pub fn no_merges(mut self, exclude: bool) -> Self {
        self.no_merges = exclude;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDetails {
    pub commit: Commit,
    pub files_changed: Vec<PathBuf>,
    pub insertions: usize,
    pub deletions: usize,
}

impl CommitDetails {
    /// Get total changes (insertions + deletions)
    pub fn total_changes(&self) -> usize {
        self.insertions + self.deletions
    }

    /// Check if any files were changed
    pub fn has_changes(&self) -> bool {
        !self.files_changed.is_empty()
    }
}

impl fmt::Display for CommitDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.commit)?;
        writeln!(f, "Files changed: {}", self.files_changed.len())?;
        writeln!(f, "Insertions: +{}", self.insertions)?;
        writeln!(f, "Deletions: -{}", self.deletions)?;

        if !self.files_changed.is_empty() {
            writeln!(f, "\nFiles:")?;
            for file in &self.files_changed {
                writeln!(f, "  {}", file.display())?;
            }
        }

        Ok(())
    }
}

/// Parse git log output with our custom format
fn parse_log_output(output: &str) -> Result<Vec<Commit>> {
    let mut commits = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse format: hash|author_name|author_email|author_timestamp|committer_name|committer_email|committer_timestamp|parent_hashes|subject|body
        let parts: Vec<&str> = line.splitn(10, '|').collect();
        if parts.len() < 9 {
            continue; // Skip malformed lines
        }

        let hash = Hash::from(parts[0].to_string());
        let author_name = parts[1].to_string();
        let author_email = parts[2].to_string();
        let author_timestamp = parse_timestamp(parts[3])?;
        let committer_name = parts[4].to_string();
        let committer_email = parts[5].to_string();
        let committer_timestamp = parse_timestamp(parts[6])?;
        let parent_hashes = parse_parent_hashes(parts[7]);
        let subject = parts[8].to_string();
        let body = if parts.len() > 9 && !parts[9].is_empty() {
            Some(parts[9].to_string())
        } else {
            None
        };

        let author = Author {
            name: author_name,
            email: author_email,
            timestamp: author_timestamp,
        };

        let committer = Author {
            name: committer_name,
            email: committer_email,
            timestamp: committer_timestamp,
        };

        let message = CommitMessage::new(subject, body);

        let commit = Commit {
            hash,
            author,
            committer,
            message,
            timestamp: author_timestamp, // Use author timestamp for commit timestamp
            parents: parent_hashes,
        };

        commits.push(commit);
    }

    Ok(commits)
}

/// Parse Unix timestamp to DateTime<Utc>
fn parse_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>> {
    let timestamp: i64 = timestamp_str.parse().map_err(|_| {
        crate::error::GitError::CommandFailed(format!("Invalid timestamp: {}", timestamp_str))
    })?;

    DateTime::from_timestamp(timestamp, 0).ok_or_else(|| {
        crate::error::GitError::CommandFailed(format!("Invalid timestamp value: {}", timestamp))
    })
}

/// Parse parent hashes from space-separated string
fn parse_parent_hashes(parents_str: &str) -> Box<[Hash]> {
    if parents_str.is_empty() {
        return Box::new([]);
    }

    parents_str
        .split_whitespace()
        .map(|hash| Hash::from(hash.to_string()))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

impl Repository {
    /// Get commit history with default options
    pub fn log(&self) -> Result<CommitLog> {
        self.log_with_options(&LogOptions::new().max_count(100))
    }

    /// Get recent N commits
    pub fn recent_commits(&self, count: usize) -> Result<CommitLog> {
        self.log_with_options(&LogOptions::new().max_count(count))
    }

    /// Get commit history with custom options
    pub fn log_with_options(&self, options: &LogOptions) -> Result<CommitLog> {
        Self::ensure_git()?;

        // Build all formatted arguments first
        let mut args_vec: Vec<String> = vec![
            "log".to_string(),
            GIT_LOG_FORMAT.to_string(),
            "--no-show-signature".to_string(),
        ];

        // Add options to git command
        if let Some(count) = options.max_count {
            args_vec.push("-n".to_string());
            args_vec.push(count.to_string());
        }

        if let Some(since) = &options.since {
            args_vec.push(format!("--since={}", since.format(DATE_FORMAT)));
        }

        if let Some(until) = &options.until {
            args_vec.push(format!("--until={}", until.format(DATE_FORMAT)));
        }

        if let Some(author) = &options.author {
            args_vec.push(format!("--author={}", author));
        }

        if let Some(committer) = &options.committer {
            args_vec.push(format!("--committer={}", committer));
        }

        if let Some(grep) = &options.grep {
            args_vec.push(format!("--grep={}", grep));
        }

        // Add boolean flags
        if options.follow_renames {
            args_vec.push("--follow".to_string());
        }

        if options.merges_only {
            args_vec.push("--merges".to_string());
        }

        if options.no_merges {
            args_vec.push("--no-merges".to_string());
        }

        // Add path filters at the end
        if !options.paths.is_empty() {
            args_vec.push("--".to_string());
            for path in &options.paths {
                args_vec.push(path.to_string_lossy().to_string());
            }
        }

        // Convert to &str slice for git function
        let all_args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        let stdout = git(&all_args, Some(self.repo_path()))?;
        let commits = parse_log_output(&stdout)?;
        Ok(CommitLog::new(commits))
    }

    /// Get commits in a range between two commits
    pub fn log_range(&self, from: &Hash, to: &Hash) -> Result<CommitLog> {
        Self::ensure_git()?;

        let range = format!("{}..{}", from.as_str(), to.as_str());
        let args = vec!["log", GIT_LOG_FORMAT, "--no-show-signature", &range];

        let stdout = git(&args, Some(self.repo_path()))?;
        let commits = parse_log_output(&stdout)?;
        Ok(CommitLog::new(commits))
    }

    /// Get commits that affected specific paths
    pub fn log_for_paths(&self, paths: &[impl AsRef<std::path::Path>]) -> Result<CommitLog> {
        let path_bufs: Vec<PathBuf> = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
        let options = LogOptions::new().paths(path_bufs);
        self.log_with_options(&options)
    }

    /// Get detailed information about a specific commit
    pub fn show_commit(&self, hash: &Hash) -> Result<CommitDetails> {
        Self::ensure_git()?;

        // Get commit info
        let commit_args = vec![
            "log",
            GIT_LOG_FORMAT,
            "--no-show-signature",
            "-n",
            "1",
            hash.as_str(),
        ];

        let commit_output = git(&commit_args, Some(self.repo_path()))?;
        let mut commits = parse_log_output(&commit_output)?;

        if commits.is_empty() {
            return Err(crate::error::GitError::CommandFailed(format!(
                "Commit not found: {}",
                hash
            )));
        }

        let commit = commits.remove(0);

        // Get diff stats
        let stats_args = vec!["show", "--stat", "--format=", hash.as_str()];

        let stats_output = git(&stats_args, Some(self.repo_path()))?;
        let (files_changed, insertions, deletions) = parse_diff_stats(&stats_output);

        Ok(CommitDetails {
            commit,
            files_changed,
            insertions,
            deletions,
        })
    }
}

/// Parse diff stats from git show --stat output
fn parse_diff_stats(output: &str) -> (Vec<PathBuf>, usize, usize) {
    let mut files_changed = Vec::new();
    let mut total_insertions = 0;
    let mut total_deletions = 0;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse lines like: "src/main.rs | 15 +++++++++------"
        if let Some(pipe_pos) = line.find(" | ") {
            let filename = line[..pipe_pos].trim();
            files_changed.push(PathBuf::from(filename));

            // Parse insertions/deletions from the rest of the line
            let stats_part = &line[pipe_pos + 3..];
            if let Some(space_pos) = stats_part.find(' ')
                && let Ok(changes) = stats_part[..space_pos].parse::<usize>()
            {
                let symbols = &stats_part[space_pos + 1..];
                let plus_count = symbols.chars().filter(|&c| c == '+').count();
                let minus_count = symbols.chars().filter(|&c| c == '-').count();

                // Distribute changes based on +/- ratio
                let total_symbols = plus_count + minus_count;
                if total_symbols > 0 {
                    let insertions = (changes * plus_count) / total_symbols;
                    let deletions = changes - insertions;
                    total_insertions += insertions;
                    total_deletions += deletions;
                }
            }
        }
        // Parse summary line like: "2 files changed, 15 insertions(+), 8 deletions(-)"
        else if line.contains("files changed") || line.contains("file changed") {
            if let Some(insertions_pos) = line.find(" insertions(+)")
                && let Some(start) = line[..insertions_pos].rfind(' ')
                && let Ok(insertions) = line[start + 1..insertions_pos].parse::<usize>()
            {
                total_insertions = insertions;
            }
            if let Some(deletions_pos) = line.find(" deletions(-)")
                && let Some(start) = line[..deletions_pos].rfind(' ')
                && let Ok(deletions) = line[start + 1..deletions_pos].parse::<usize>()
            {
                total_deletions = deletions;
            }
        }
    }

    (files_changed, total_insertions, total_deletions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_author_display() {
        let author = Author {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            timestamp: DateTime::from_timestamp(1640995200, 0).unwrap(),
        };
        assert_eq!(format!("{}", author), "John Doe <john@example.com>");
    }

    #[test]
    fn test_commit_message_creation() {
        let msg = CommitMessage::new("Initial commit".to_string(), None);
        assert_eq!(msg.subject, "Initial commit");
        assert!(msg.body.is_none());
        assert_eq!(msg.full(), "Initial commit");

        let msg_with_body = CommitMessage::new(
            "Add feature".to_string(),
            Some("This adds a new feature\nwith multiple lines".to_string()),
        );
        assert_eq!(
            msg_with_body.full(),
            "Add feature\n\nThis adds a new feature\nwith multiple lines"
        );
    }

    #[test]
    fn test_commit_is_merge() {
        let commit = Commit {
            hash: Hash::from("abc123".to_string()),
            author: Author {
                name: "Test".to_string(),
                email: "test@example.com".to_string(),
                timestamp: DateTime::from_timestamp(1640995200, 0).unwrap(),
            },
            committer: Author {
                name: "Test".to_string(),
                email: "test@example.com".to_string(),
                timestamp: DateTime::from_timestamp(1640995200, 0).unwrap(),
            },
            message: CommitMessage::new("Test commit".to_string(), None),
            timestamp: DateTime::from_timestamp(1640995200, 0).unwrap(),
            parents: vec![
                Hash::from("parent1".to_string()),
                Hash::from("parent2".to_string()),
            ]
            .into_boxed_slice(),
        };

        assert!(commit.is_merge());
        assert!(!commit.is_root());
    }

    #[test]
    fn test_commit_log_filtering() {
        let commits = vec![
            create_test_commit(
                "abc123",
                "John Doe",
                "john@example.com",
                "Fix bug",
                1640995200,
            ),
            create_test_commit(
                "def456",
                "Jane Smith",
                "jane@example.com",
                "Add feature",
                1640995300,
            ),
            create_test_commit(
                "ghi789",
                "John Doe",
                "john@example.com",
                "Update docs",
                1640995400,
            ),
        ];

        let log = CommitLog::new(commits);

        // Test by author
        let john_commits: Vec<_> = log.by_author("John Doe").collect();
        assert_eq!(john_commits.len(), 2);

        // Test message search
        let fix_commits: Vec<_> = log.with_message_containing("fix").collect();
        assert_eq!(fix_commits.len(), 1);
        assert_eq!(fix_commits[0].message.subject, "Fix bug");
    }

    #[test]
    fn test_parse_timestamp() {
        let timestamp = parse_timestamp("1640995200").unwrap();
        assert_eq!(timestamp.timestamp(), 1640995200);
    }

    #[test]
    fn test_parse_parent_hashes() {
        let parents = parse_parent_hashes("abc123 def456 ghi789");
        assert_eq!(parents.len(), 3);
        assert_eq!(parents[0].as_str(), "abc123");
        assert_eq!(parents[1].as_str(), "def456");
        assert_eq!(parents[2].as_str(), "ghi789");

        let no_parents = parse_parent_hashes("");
        assert_eq!(no_parents.len(), 0);
    }

    #[test]
    fn test_log_options_builder() {
        let options = LogOptions::new()
            .max_count(50)
            .author("john@example.com".to_string())
            .follow_renames(true);

        assert_eq!(options.max_count, Some(50));
        assert_eq!(options.author, Some("john@example.com".to_string()));
        assert!(options.follow_renames);
    }

    #[test]
    fn test_parse_diff_stats() {
        let output = "src/main.rs | 15 +++++++++------\nREADME.md | 3 +++\n 2 files changed, 18 insertions(+), 6 deletions(-)";
        let (files, insertions, deletions) = parse_diff_stats(output);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0], PathBuf::from("src/main.rs"));
        assert_eq!(files[1], PathBuf::from("README.md"));
        assert_eq!(insertions, 18);
        assert_eq!(deletions, 6);
    }

    #[test]
    fn test_commit_details_display() {
        let commit = create_test_commit(
            "abc123",
            "John Doe",
            "john@example.com",
            "Test commit",
            1640995200,
        );
        let details = CommitDetails {
            commit,
            files_changed: vec![PathBuf::from("src/main.rs"), PathBuf::from("README.md")],
            insertions: 15,
            deletions: 8,
        };

        assert_eq!(details.total_changes(), 23);
        assert!(details.has_changes());

        let display_output = format!("{}", details);
        assert!(display_output.contains("Files changed: 2"));
        assert!(display_output.contains("Insertions: +15"));
        assert!(display_output.contains("Deletions: -8"));
    }

    // Helper function to create test commits
    fn create_test_commit(
        hash: &str,
        author_name: &str,
        author_email: &str,
        subject: &str,
        timestamp: i64,
    ) -> Commit {
        Commit {
            hash: Hash::from(hash.to_string()),
            author: Author {
                name: author_name.to_string(),
                email: author_email.to_string(),
                timestamp: DateTime::from_timestamp(timestamp, 0).unwrap(),
            },
            committer: Author {
                name: author_name.to_string(),
                email: author_email.to_string(),
                timestamp: DateTime::from_timestamp(timestamp, 0).unwrap(),
            },
            message: CommitMessage::new(subject.to_string(), None),
            timestamp: DateTime::from_timestamp(timestamp, 0).unwrap(),
            parents: Box::new([]),
        }
    }

    #[test]
    fn test_repository_log() {
        let test_path = "/tmp/test_log_repo";

        // Clean up if exists
        if Path::new(test_path).exists() {
            fs::remove_dir_all(test_path).unwrap();
        }

        // Create a repository with some commits
        let repo = Repository::init(test_path, false).unwrap();

        // Create initial commit
        std::fs::write(format!("{}/test1.txt", test_path), "content1").unwrap();
        repo.add(&["test1.txt"]).unwrap();
        let _hash1 = repo.commit("First commit").unwrap();

        // Create second commit
        std::fs::write(format!("{}/test2.txt", test_path), "content2").unwrap();
        repo.add(&["test2.txt"]).unwrap();
        let _hash2 = repo.commit("Second commit").unwrap();

        // Test log functionality
        let log = repo.log().unwrap();
        assert_eq!(log.len(), 2);

        let recent = repo.recent_commits(1).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent.first().unwrap().message.subject, "Second commit");

        // Clean up
        fs::remove_dir_all(test_path).unwrap();
    }
}
