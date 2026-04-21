use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("Path not found: {0}")]
    NotFound(PathBuf),

    #[error("Not a file: {0}")]
    NotAFile(PathBuf),

    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("IO error: {context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Watch not found: {0}")]
    WatchNotFound(String),

    #[error("Path too long: {0}")]
    PathTooLong(PathBuf),
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::Io {
            context: String::new(),
            source: err,
        }
    }
}