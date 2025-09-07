use std::io;

#[derive(Debug)]
pub enum GitError {
    IoError(io::Error),
    CommandFailed(String),
}

impl From<io::Error> for GitError {
    fn from(error: io::Error) -> Self {
        GitError::IoError(error)
    }
}