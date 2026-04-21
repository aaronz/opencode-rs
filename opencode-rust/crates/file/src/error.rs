use std::path::PathBuf;
use std::sync::Arc;
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
        source: Arc<std::io::Error>,
    },

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Watch not found: {0}")]
    WatchNotFound(String),

    #[error("Path too long: {0}")]
    PathTooLong(PathBuf),
}

impl FileError {
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        FileError::Io {
            context: context.into(),
            source: Arc::new(source),
        }
    }
}

impl Clone for FileError {
    fn clone(&self) -> Self {
        match self {
            FileError::NotFound(p) => FileError::NotFound(p.clone()),
            FileError::NotAFile(p) => FileError::NotAFile(p.clone()),
            FileError::NotADirectory(p) => FileError::NotADirectory(p.clone()),
            FileError::Io { context, source } => FileError::Io {
                context: context.clone(),
                source: source.clone(),
            },
            FileError::Watch(s) => FileError::Watch(s.clone()),
            FileError::WatchNotFound(s) => FileError::WatchNotFound(s.clone()),
            FileError::PathTooLong(p) => FileError::PathTooLong(p.clone()),
        }
    }
}

impl From<std::io::Error> for FileError {
    fn from(err: std::io::Error) -> Self {
        FileError::Io {
            context: String::new(),
            source: Arc::new(err),
        }
    }
}
