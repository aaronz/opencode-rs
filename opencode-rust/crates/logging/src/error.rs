//! Logging system errors.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Logger error: {0}")]
    Logger(String),
}

impl From<rusqlite::Error> for LogError {
    fn from(err: rusqlite::Error) -> Self {
        LogError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for LogError {
    fn from(err: serde_json::Error) -> Self {
        LogError::Serialization(err.to_string())
    }
}