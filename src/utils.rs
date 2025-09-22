use std::path::Path;
use std::process::Command;

use crate::error::{GitError, Result};
use chrono::{DateTime, Utc};

/// Executes a git command and returns the stdout as a String.
/// Automatically handles error checking and provides descriptive error messages.
///
/// # Arguments
///
/// * `args` - The arguments to pass to the git command.
/// * `working_dir` - The working directory to use for the git command.
///
/// # Returns
///
/// A `Result` containing the stdout as String or a `GitError` if the command fails.
pub fn git(args: &[&str], working_dir: Option<&Path>) -> Result<String> {
    let output = git_raw(args, working_dir)?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&"<unknown>"),
            error_msg
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Executes a git command and returns the raw Output for cases needing full control.
///
/// # Arguments
///
/// * `args` - The arguments to pass to the git command.
/// * `working_dir` - The working directory to use for the git command.
///
/// # Returns
///
/// A `Result` containing the raw command output or a `GitError` if the command fails to execute.
pub fn git_raw(args: &[&str], working_dir: Option<&Path>) -> Result<std::process::Output> {
    let mut cmd = Command::new("git");
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    cmd.output().map_err(GitError::from)
}

/// Parse Unix timestamp to DateTime<Utc>
///
/// This utility function is used by both tag and stash parsing to convert
/// Unix timestamps from git command output into DateTime objects.
///
/// # Arguments
///
/// * `timestamp_str` - The Unix timestamp as a string
///
/// # Returns
///
/// A `Result` containing the parsed DateTime or a `GitError` if parsing fails.
/// If the input is empty, returns the current time as a fallback.
pub fn parse_unix_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>> {
    if timestamp_str.is_empty() {
        return Ok(Utc::now());
    }

    let timestamp: i64 = timestamp_str
        .parse()
        .map_err(|_| GitError::CommandFailed(format!("Invalid timestamp: {}", timestamp_str)))?;

    DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| GitError::CommandFailed(format!("Invalid timestamp value: {}", timestamp)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;

    #[test]
    fn test_git_raw_version_command() {
        let result = git_raw(&["--version"], None);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.status.success());
        assert!(!output.stdout.is_empty());

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        assert!(stdout_str.contains("git version"));
    }

    #[test]
    fn test_git_version_command() {
        let result = git(&["--version"], None);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("git version"));
    }

    #[test]
    fn test_git_raw_invalid_command() {
        let result = git_raw(&["invalid-command-that-does-not-exist"], None);
        assert!(result.is_ok()); // Command executes but fails

        let output = result.unwrap();
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }

    #[test]
    fn test_git_invalid_command_returns_error() {
        let result = git(&["invalid-command-that-does-not-exist"], None);
        assert!(result.is_err());

        match result.unwrap_err() {
            GitError::CommandFailed(msg) => {
                assert!(msg.contains("git invalid-command-that-does-not-exist failed"));
            }
            _ => panic!("Expected CommandFailed error"),
        }
    }

    #[test]
    fn test_git_with_working_directory() {
        let temp_dir = env::temp_dir();
        let result = git(&["--version"], Some(&temp_dir));
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("git version"));
    }

    #[test]
    fn test_git_raw_with_working_directory() {
        let temp_dir = env::temp_dir();
        let result = git_raw(&["--version"], Some(&temp_dir));
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.status.success());
    }

    #[test]
    fn test_git_with_nonexistent_working_directory() {
        let nonexistent_path = Path::new("/nonexistent/path/that/should/not/exist");
        let result = git(&["--version"], Some(nonexistent_path));
        assert!(result.is_err());

        match result.unwrap_err() {
            GitError::IoError(_) => {} // Expected
            _ => panic!("Expected IoError for nonexistent directory"),
        }
    }

    #[test]
    fn test_git_raw_with_nonexistent_working_directory() {
        let nonexistent_path = Path::new("/nonexistent/path/that/should/not/exist");
        let result = git_raw(&["--version"], Some(nonexistent_path));
        assert!(result.is_err());

        match result.unwrap_err() {
            GitError::IoError(_) => {} // Expected
            _ => panic!("Expected IoError for nonexistent directory"),
        }
    }

    #[test]
    fn test_git_empty_args() {
        let result = git(&[], None);
        assert!(result.is_err());

        match result.unwrap_err() {
            GitError::CommandFailed(msg) => {
                assert!(msg.contains("git <unknown> failed") || msg.contains("usage"));
            }
            _ => panic!("Expected CommandFailed error for empty args"),
        }
    }

    #[test]
    fn test_git_raw_empty_args() {
        let result = git_raw(&[], None);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.status.success());
    }

    #[test]
    fn test_git_help_command() {
        let result = git(&["--help"], None);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("usage:") || output.contains("Git") || output.contains("git"));
    }

    #[test]
    fn test_parse_unix_timestamp() {
        // Test valid timestamp
        let timestamp = "1642694400"; // January 20, 2022 12:00:00 UTC
        let result = parse_unix_timestamp(timestamp);
        assert!(result.is_ok());

        let datetime = result.unwrap();
        assert_eq!(datetime.timestamp(), 1642694400);

        // Test empty string (should return current time)
        let result = parse_unix_timestamp("");
        assert!(result.is_ok());

        // Test invalid timestamp
        let result = parse_unix_timestamp("invalid");
        assert!(result.is_err());

        // Test out of range timestamp
        let result = parse_unix_timestamp("999999999999999999");
        assert!(result.is_err());
    }
}
