use std::io;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Clone)]
pub enum GitError {
    IoError(String),
    CommandFailed(String),
}

impl From<io::Error> for GitError {
    fn from(error: io::Error) -> Self {
        GitError::IoError(error.to_string())
    }
}