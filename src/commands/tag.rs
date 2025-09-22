//! Git tag operations
//!
//! This module provides functionality for creating, listing, deleting, and managing Git tags.
//! It supports both lightweight and annotated tags with comprehensive type safety.
//!
//! # Examples
//!
//! ```rust,no_run
//! use rustic_git::{Repository, TagType, TagOptions};
//!
//! let repo = Repository::open(".")?;
//!
//! // List all tags
//! let tags = repo.tags()?;
//! for tag in tags.iter() {
//!     println!("{} -> {}", tag.name, tag.hash.short());
//! }
//!
//! // Create a lightweight tag
//! let tag = repo.create_tag("v1.0.0", None)?;
//!
//! // Create an annotated tag
//! let options = TagOptions::new()
//!     .with_message("Release version 1.0.0".to_string())
//!     .with_annotated();
//! let tag = repo.create_tag_with_options("v1.0.0-rc1", None, options)?;
//!
//! # Ok::<(), rustic_git::GitError>(())
//! ```

use crate::commands::log::Author;
use crate::error::{GitError, Result};
use crate::repository::Repository;
use crate::types::Hash;
use crate::utils::{git, parse_unix_timestamp};
use chrono::{DateTime, Utc};
use std::fmt;

/// Represents a Git tag
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// The name of the tag
    pub name: String,
    /// The commit hash this tag points to
    pub hash: Hash,
    /// The type of tag (lightweight or annotated)
    pub tag_type: TagType,
    /// The tag message (only for annotated tags)
    pub message: Option<String>,
    /// The tagger information (only for annotated tags)
    pub tagger: Option<Author>,
    /// The tag creation timestamp (only for annotated tags)
    pub timestamp: Option<DateTime<Utc>>,
}

/// Type of Git tag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    /// Lightweight tag - just a reference to a commit
    Lightweight,
    /// Annotated tag - full object with message, author, and date
    Annotated,
}

impl fmt::Display for TagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TagType::Lightweight => write!(f, "lightweight"),
            TagType::Annotated => write!(f, "annotated"),
        }
    }
}

/// A collection of tags with efficient iteration and filtering methods
#[derive(Debug, Clone)]
pub struct TagList {
    tags: Box<[Tag]>,
}

impl TagList {
    /// Create a new TagList from a vector of tags
    pub fn new(mut tags: Vec<Tag>) -> Self {
        // Sort tags by name for consistent ordering
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            tags: tags.into_boxed_slice(),
        }
    }

    /// Get an iterator over all tags
    pub fn iter(&self) -> impl Iterator<Item = &Tag> + '_ {
        self.tags.iter()
    }

    /// Get an iterator over lightweight tags only
    pub fn lightweight(&self) -> impl Iterator<Item = &Tag> + '_ {
        self.tags
            .iter()
            .filter(|tag| tag.tag_type == TagType::Lightweight)
    }

    /// Get an iterator over annotated tags only
    pub fn annotated(&self) -> impl Iterator<Item = &Tag> + '_ {
        self.tags
            .iter()
            .filter(|tag| tag.tag_type == TagType::Annotated)
    }

    /// Find a tag by exact name
    pub fn find(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|tag| tag.name == name)
    }

    /// Find tags whose names contain the given substring
    pub fn find_containing<'a>(&'a self, substring: &'a str) -> impl Iterator<Item = &'a Tag> + 'a {
        self.tags
            .iter()
            .filter(move |tag| tag.name.contains(substring))
    }

    /// Get the total number of tags
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// Check if the tag list is empty
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Get the number of lightweight tags
    pub fn lightweight_count(&self) -> usize {
        self.lightweight().count()
    }

    /// Get the number of annotated tags
    pub fn annotated_count(&self) -> usize {
        self.annotated().count()
    }

    /// Get tags that point to a specific commit
    pub fn for_commit<'a>(&'a self, hash: &'a Hash) -> impl Iterator<Item = &'a Tag> + 'a {
        self.tags.iter().filter(move |tag| &tag.hash == hash)
    }
}

/// Options for creating tags
#[derive(Debug, Clone, Default)]
pub struct TagOptions {
    /// Create an annotated tag (default: false - lightweight)
    pub annotated: bool,
    /// Force tag creation (overwrite existing tag)
    pub force: bool,
    /// Tag message (for annotated tags)
    pub message: Option<String>,
    /// Sign the tag with GPG (requires annotated)
    pub sign: bool,
}

impl TagOptions {
    /// Create new default tag options
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an annotated tag instead of lightweight
    pub fn with_annotated(mut self) -> Self {
        self.annotated = true;
        self
    }

    /// Force tag creation (overwrite existing)
    pub fn with_force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Set the tag message (implies annotated)
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self.annotated = true; // Message implies annotated tag
        self
    }

    /// Sign the tag with GPG (implies annotated)
    pub fn with_sign(mut self) -> Self {
        self.sign = true;
        self.annotated = true; // Signing implies annotated tag
        self
    }
}

impl Repository {
    /// List all tags in the repository
    ///
    /// Returns a `TagList` containing all tags sorted by name.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// let tags = repo.tags()?;
    ///
    /// println!("Found {} tags:", tags.len());
    /// for tag in tags.iter() {
    ///     println!("  {} ({}) -> {}", tag.name, tag.tag_type, tag.hash.short());
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn tags(&self) -> Result<TagList> {
        Self::ensure_git()?;

        // Use git for-each-ref to get all tag information in a single call
        // Format: refname:short objecttype objectname *objectname taggername taggeremail taggerdate:unix subject body
        let output = git(
            &[
                "for-each-ref",
                "--format=%(refname:short)|%(objecttype)|%(objectname)|%(*objectname)|%(taggername)|%(taggeremail)|%(taggerdate:unix)|%(subject)|%(body)",
                "refs/tags/",
            ],
            Some(self.repo_path()),
        )?;

        if output.trim().is_empty() {
            return Ok(TagList::new(vec![]));
        }

        let mut tags = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse tag information from for-each-ref output
            if let Ok(tag) = parse_for_each_ref_line(line) {
                tags.push(tag);
            }
        }

        Ok(TagList::new(tags))
    }

    /// Create a lightweight tag pointing to the current HEAD or specified commit
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tag to create
    /// * `target` - Optional commit hash to tag (defaults to HEAD)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    ///
    /// // Tag current HEAD
    /// let tag = repo.create_tag("v1.0.0", None)?;
    ///
    /// // Tag specific commit
    /// let commits = repo.recent_commits(1)?;
    /// if let Some(commit) = commits.iter().next() {
    ///     let tag = repo.create_tag("v0.9.0", Some(&commit.hash))?;
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn create_tag(&self, name: &str, target: Option<&Hash>) -> Result<Tag> {
        self.create_tag_with_options(name, target, TagOptions::new())
    }

    /// Create a tag with custom options
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tag to create
    /// * `target` - Optional commit hash to tag (defaults to HEAD)
    /// * `options` - Tag creation options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::{Repository, TagOptions};
    ///
    /// let repo = Repository::open(".")?;
    ///
    /// // Create annotated tag with message
    /// let options = TagOptions::new()
    ///     .with_message("Release version 1.0.0".to_string());
    /// let tag = repo.create_tag_with_options("v1.0.0", None, options)?;
    ///
    /// // Create and force overwrite existing tag
    /// let options = TagOptions::new().with_force();
    /// let tag = repo.create_tag_with_options("latest", None, options)?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn create_tag_with_options(
        &self,
        name: &str,
        target: Option<&Hash>,
        options: TagOptions,
    ) -> Result<Tag> {
        Self::ensure_git()?;

        let mut args = vec!["tag"];

        if options.annotated || options.message.is_some() {
            args.push("-a");
        }

        if options.force {
            args.push("-f");
        }

        if options.sign {
            args.push("-s");
        }

        if let Some(ref message) = options.message {
            args.push("-m");
            args.push(message);
        }

        args.push(name);

        if let Some(target_hash) = target {
            args.push(target_hash.as_str());
        }

        git(&args, Some(self.repo_path()))?;

        // Get the created tag information
        let show_output = git(&["show", "--format=fuller", name], Some(self.repo_path()))?;
        parse_tag_info(name, &show_output)
    }

    /// Delete a tag
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tag to delete
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// repo.delete_tag("v0.1.0")?;
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        Self::ensure_git()?;

        git(&["tag", "-d", name], Some(self.repo_path()))?;
        Ok(())
    }

    /// Show detailed information about a specific tag
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tag to show
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use rustic_git::Repository;
    ///
    /// let repo = Repository::open(".")?;
    /// let tag = repo.show_tag("v1.0.0")?;
    ///
    /// println!("Tag: {} ({})", tag.name, tag.tag_type);
    /// println!("Commit: {}", tag.hash.short());
    /// if let Some(message) = &tag.message {
    ///     println!("Message: {}", message);
    /// }
    /// # Ok::<(), rustic_git::GitError>(())
    /// ```
    pub fn show_tag(&self, name: &str) -> Result<Tag> {
        Self::ensure_git()?;

        let show_output = git(&["show", "--format=fuller", name], Some(self.repo_path()))?;
        parse_tag_info(name, &show_output)
    }
}

/// Parse tag information from git for-each-ref output
/// Format: refname:short|objecttype|objectname|*objectname|taggername|taggeremail|taggerdate:unix|subject|body
fn parse_for_each_ref_line(line: &str) -> Result<Tag> {
    let parts: Vec<&str> = line.split('|').collect();

    if parts.len() < 9 {
        return Err(GitError::CommandFailed(format!(
            "Invalid for-each-ref format: expected 9 parts, got {}",
            parts.len()
        )));
    }

    let name = parts[0].to_string();
    let object_type = parts[1];
    let object_name = parts[2];
    let dereferenced_object = parts[3]; // For annotated tags, this is the commit hash
    let tagger_name = parts[4];
    let tagger_email = parts[5];
    let tagger_date = parts[6];
    let subject = parts[7];
    let body = parts[8];

    // Determine tag type and commit hash
    let (tag_type, hash) = if object_type == "tag" {
        // Annotated tag - use dereferenced object (the commit it points to)
        (TagType::Annotated, Hash::from(dereferenced_object))
    } else {
        // Lightweight tag - use object name (direct commit reference)
        (TagType::Lightweight, Hash::from(object_name))
    };

    // Build tagger information for annotated tags
    let tagger =
        if tag_type == TagType::Annotated && !tagger_name.is_empty() && !tagger_email.is_empty() {
            // Parse the timestamp - if it fails, the tag metadata may be corrupted
            // Use Unix epoch as fallback to clearly indicate corrupted/invalid timestamp data
            let timestamp = parse_unix_timestamp(tagger_date).unwrap_or_else(|_| {
                // Timestamp parsing failed - this indicates malformed git metadata
                // Use Unix epoch (1970-01-01) as fallback to make data corruption obvious
                DateTime::from_timestamp(0, 0).unwrap()
            });
            Some(Author {
                name: tagger_name.to_string(),
                email: tagger_email.to_string(),
                timestamp,
            })
        } else {
            None
        };

    // Build message for annotated tags
    let message = if tag_type == TagType::Annotated && (!subject.is_empty() || !body.is_empty()) {
        let full_message = if !body.is_empty() {
            format!("{}\n\n{}", subject, body)
        } else {
            subject.to_string()
        };
        Some(full_message.trim().to_string())
    } else {
        None
    };

    // Timestamp for the tag
    let timestamp = if tag_type == TagType::Annotated {
        tagger.as_ref().map(|t| t.timestamp)
    } else {
        None
    };

    Ok(Tag {
        name,
        hash,
        tag_type,
        message,
        tagger,
        timestamp,
    })
}

/// Parse tag information from git show output (fallback method)
fn parse_tag_info(tag_name: &str, show_output: &str) -> Result<Tag> {
    let lines: Vec<&str> = show_output.lines().collect();

    // Determine if this is an annotated tag or lightweight tag
    let is_annotated = show_output.contains("tag ") && show_output.contains("Tagger:");

    if is_annotated {
        parse_annotated_tag(tag_name, &lines)
    } else {
        parse_lightweight_tag(tag_name, &lines)
    }
}

/// Parse annotated tag information
fn parse_annotated_tag(tag_name: &str, lines: &[&str]) -> Result<Tag> {
    let mut hash = None;
    let mut tagger = None;
    let mut collecting_message = false;
    let mut message_lines = Vec::new();

    for line in lines {
        if line.starts_with("commit ") {
            if let Some(hash_str) = line.split_whitespace().nth(1) {
                hash = Some(Hash::from(hash_str));
            }
        } else if let Some(stripped) = line.strip_prefix("Tagger: ") {
            tagger = parse_author_line(stripped);
        } else if line.trim().is_empty() && !collecting_message {
            collecting_message = true;
        } else if collecting_message && !line.starts_with("commit ") && !line.starts_with("Author:")
        {
            message_lines.push(line.trim());
        }
    }

    let message_text = if message_lines.is_empty() {
        None
    } else {
        Some(message_lines.join("\n").trim().to_string())
    };

    let timestamp = tagger.as_ref().map(|t| t.timestamp);

    Ok(Tag {
        name: tag_name.to_string(),
        hash: hash.ok_or_else(|| {
            GitError::CommandFailed("Could not parse tag commit hash".to_string())
        })?,
        tag_type: TagType::Annotated,
        message: message_text,
        tagger,
        timestamp,
    })
}

/// Parse lightweight tag information
fn parse_lightweight_tag(tag_name: &str, lines: &[&str]) -> Result<Tag> {
    let mut hash = None;

    for line in lines {
        if line.starts_with("commit ")
            && let Some(hash_str) = line.split_whitespace().nth(1)
        {
            hash = Some(Hash::from(hash_str));
            break;
        }
    }

    Ok(Tag {
        name: tag_name.to_string(),
        hash: hash.ok_or_else(|| {
            GitError::CommandFailed("Could not parse tag commit hash".to_string())
        })?,
        tag_type: TagType::Lightweight,
        message: None,
        tagger: None,
        timestamp: None,
    })
}

/// Parse author information from a git tagger line
/// Format: "Tagger: Name <email>" (timestamp not available in this format)
fn parse_author_line(line: &str) -> Option<Author> {
    // Parse format: "Name <email>" (no timestamp in git show --format=fuller tagger line)
    if let Some(email_start) = line.find('<')
        && let Some(email_end) = line.find('>')
    {
        let name = line[..email_start].trim().to_string();
        let email = line[email_start + 1..email_end].to_string();

        // Timestamp is not available in the tagger line from git show --format=fuller
        // We use the current time as a fallback, which matches the review feedback
        // that tagger timestamp may default
        let timestamp = Utc::now();

        return Some(Author {
            name,
            email,
            timestamp,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    fn create_test_repo() -> (Repository, std::path::PathBuf) {
        use std::thread;
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let test_path = env::temp_dir().join(format!(
            "rustic_git_tag_test_{}_{}_{}",
            std::process::id(),
            timestamp,
            thread_id.replace("ThreadId(", "").replace(")", "")
        ));

        // Ensure clean state
        if test_path.exists() {
            fs::remove_dir_all(&test_path).unwrap();
        }

        let repo = Repository::init(&test_path, false).unwrap();

        // Configure git user for commits
        repo.config()
            .set_user("Test User", "test@example.com")
            .unwrap();

        (repo, test_path)
    }

    fn create_test_commit(repo: &Repository, test_path: &std::path::Path) {
        fs::write(test_path.join("test.txt"), "test content").unwrap();
        repo.add(&["test.txt"]).unwrap();
        repo.commit("Test commit").unwrap();
    }

    #[test]
    fn test_tag_list_empty_repository() {
        let (repo, test_path) = create_test_repo();

        let tags = repo.tags().unwrap();
        assert!(tags.is_empty());
        assert_eq!(tags.len(), 0);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_create_lightweight_tag() {
        let (repo, test_path) = create_test_repo();
        create_test_commit(&repo, &test_path);

        let tag = repo.create_tag("v1.0.0", None).unwrap();
        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.tag_type, TagType::Lightweight);
        assert!(tag.message.is_none());
        assert!(tag.tagger.is_none());

        // Verify tag exists in list
        let tags = repo.tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert!(tags.find("v1.0.0").is_some());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_create_annotated_tag() {
        let (repo, test_path) = create_test_repo();
        create_test_commit(&repo, &test_path);

        let options = TagOptions::new().with_message("Release version 1.0.0".to_string());
        let tag = repo
            .create_tag_with_options("v1.0.0", None, options)
            .unwrap();

        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.tag_type, TagType::Annotated);
        assert!(tag.message.is_some());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_delete_tag() {
        let (repo, test_path) = create_test_repo();
        create_test_commit(&repo, &test_path);

        // Create a tag
        repo.create_tag("to-delete", None).unwrap();

        // Verify it exists
        let tags = repo.tags().unwrap();
        assert_eq!(tags.len(), 1);

        // Delete it
        repo.delete_tag("to-delete").unwrap();

        // Verify it's gone
        let tags = repo.tags().unwrap();
        assert_eq!(tags.len(), 0);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_tag_list_filtering() {
        let (repo, test_path) = create_test_repo();
        create_test_commit(&repo, &test_path);

        // Create multiple tags
        repo.create_tag("v1.0.0", None).unwrap();
        repo.create_tag("v1.1.0", None).unwrap();
        let options = TagOptions::new().with_message("Annotated".to_string());
        repo.create_tag_with_options("v2.0.0", None, options)
            .unwrap();

        let tags = repo.tags().unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags.lightweight_count(), 2);
        assert_eq!(tags.annotated_count(), 1);

        // Test filtering
        let v1_tags: Vec<_> = tags.find_containing("v1").collect();
        assert_eq!(v1_tags.len(), 2);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_tag_options_builder() {
        let options = TagOptions::new()
            .with_annotated()
            .with_force()
            .with_message("Test message".to_string());

        assert!(options.annotated);
        assert!(options.force);
        assert_eq!(options.message, Some("Test message".to_string()));
    }

    #[test]
    fn test_show_tag() {
        let (repo, test_path) = create_test_repo();
        create_test_commit(&repo, &test_path);

        repo.create_tag("show-test", None).unwrap();
        let tag = repo.show_tag("show-test").unwrap();

        assert_eq!(tag.name, "show-test");
        assert_eq!(tag.tag_type, TagType::Lightweight);

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_tag_force_overwrite() {
        let (repo, test_path) = create_test_repo();
        create_test_commit(&repo, &test_path);

        // Create initial tag
        repo.create_tag("overwrite-test", None).unwrap();

        // Try to create again without force (should fail)
        let result = repo.create_tag("overwrite-test", None);
        assert!(result.is_err());

        // Create with force (should succeed)
        let options = TagOptions::new().with_force();
        let result = repo.create_tag_with_options("overwrite-test", None, options);
        assert!(result.is_ok());

        // Clean up
        fs::remove_dir_all(&test_path).unwrap();
    }

    #[test]
    fn test_parse_for_each_ref_line_invalid_format() {
        // Test with insufficient parts (should have 9 parts minimum)
        let invalid_line = "tag1|commit|abc123"; // Only 3 parts instead of 9
        let result = parse_for_each_ref_line(invalid_line);

        assert!(result.is_err());

        if let Err(GitError::CommandFailed(msg)) = result {
            assert!(msg.contains("Invalid for-each-ref format"));
            assert!(msg.contains("expected 9 parts"));
            assert!(msg.contains("got 3"));
        } else {
            panic!("Expected CommandFailed error with specific message");
        }
    }

    #[test]
    fn test_parse_for_each_ref_line_with_invalid_timestamp() {
        // Test annotated tag with invalid timestamp - should still parse but use fallback timestamp
        let line_with_invalid_timestamp =
            "v1.0.0|tag|abc123|def456|John Doe|john@example.com|invalid-timestamp|Subject|Body";
        let result = parse_for_each_ref_line(line_with_invalid_timestamp);

        assert!(result.is_ok());
        let tag = result.unwrap();
        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(tag.tag_type, TagType::Annotated);
        assert!(tag.tagger.is_some());

        // The timestamp should use Unix epoch (1970-01-01) as fallback for invalid data
        let tagger = tag.tagger.unwrap();
        assert_eq!(tagger.name, "John Doe");
        assert_eq!(tagger.email, "john@example.com");

        // Verify fallback timestamp is Unix epoch (indicates data corruption)
        assert_eq!(tagger.timestamp.timestamp(), 0); // Unix epoch
        assert_eq!(
            tagger.timestamp.format("%Y-%m-%d").to_string(),
            "1970-01-01"
        );
    }
}
