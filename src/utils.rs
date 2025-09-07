use std::path::Path;
use std::process::Command;

use crate::error::{GitError, Result};

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
