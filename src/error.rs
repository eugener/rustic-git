use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Clone)]
pub enum GitError {
    IoError(String),
    CommandFailed(String),
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::IoError(msg) => write!(f, "IO error: {}", msg),
            GitError::CommandFailed(msg) => write!(f, "Git command failed: {}", msg),
        }
    }
}

impl std::error::Error for GitError {}

impl From<io::Error> for GitError {
    fn from(error: io::Error) -> Self {
        GitError::IoError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_git_error_io_error_variant() {
        let error = GitError::IoError("test io error".to_string());
        match error {
            GitError::IoError(msg) => assert_eq!(msg, "test io error"),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_git_error_command_failed_variant() {
        let error = GitError::CommandFailed("test command failed".to_string());
        match error {
            GitError::CommandFailed(msg) => assert_eq!(msg, "test command failed"),
            _ => panic!("Expected CommandFailed variant"),
        }
    }

    #[test]
    fn test_git_error_clone() {
        let error1 = GitError::IoError("test error".to_string());
        let error2 = error1.clone();

        match (error1, error2) {
            (GitError::IoError(msg1), GitError::IoError(msg2)) => assert_eq!(msg1, msg2),
            _ => panic!("Clone failed or wrong variant"),
        }
    }

    #[test]
    fn test_git_error_debug() {
        let io_error = GitError::IoError("io test".to_string());
        let cmd_error = GitError::CommandFailed("cmd test".to_string());

        let io_debug = format!("{:?}", io_error);
        let cmd_debug = format!("{:?}", cmd_error);

        assert!(io_debug.contains("IoError"));
        assert!(io_debug.contains("io test"));
        assert!(cmd_debug.contains("CommandFailed"));
        assert!(cmd_debug.contains("cmd test"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let git_err: GitError = io_err.into();

        match git_err {
            GitError::IoError(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected IoError variant from io::Error conversion"),
        }
    }

    #[test]
    fn test_from_io_error_different_kinds() {
        let permission_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let git_err: GitError = permission_err.into();

        match git_err {
            GitError::IoError(msg) => assert!(msg.contains("access denied")),
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        let result = test_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_result_type_alias_error() {
        fn test_function() -> Result<String> {
            Err(GitError::CommandFailed("test error".to_string()))
        }

        let result = test_function();
        assert!(result.is_err());
        match result.unwrap_err() {
            GitError::CommandFailed(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected CommandFailed variant"),
        }
    }
}
