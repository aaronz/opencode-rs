//! Project error types.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("No project found from: {0}")]
    NotFound(PathBuf),

    #[error("Failed to read project file: {0}")]
    ReadError(PathBuf, #[source] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(String, #[source] serde_json::Error),

    #[error("Ambiguous project: multiple roots found")]
    Ambiguous,
}

#[derive(Debug)]
pub enum WorkspaceValidationError {
    PathNotFound(String),
    PathNotAccessible(String),
    PathNotDirectory(String),
    PathNotReadable(String),
    PathPermissionDenied(String),
    PathCircularSymlink(String),
    PathTraversalDetected(String),
    PathNotAbsolute(String),
}

#[allow(dead_code)]
impl WorkspaceValidationError {
    pub(crate) fn code(&self) -> u16 {
        match self {
            Self::PathNotFound(_) => 7011,
            Self::PathNotAccessible(_) => 7012,
            Self::PathNotDirectory(_) => 7013,
            Self::PathNotReadable(_) => 7014,
            Self::PathPermissionDenied(_) => 7015,
            Self::PathCircularSymlink(_) => 7016,
            Self::PathTraversalDetected(_) => 7017,
            Self::PathNotAbsolute(_) => 7018,
        }
    }
}

impl std::fmt::Display for WorkspaceValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceValidationError::PathNotFound(p) => {
                write!(f, "Workspace path does not exist: {}", p)
            }
            WorkspaceValidationError::PathNotAccessible(p) => {
                write!(f, "Workspace path is not accessible: {}", p)
            }
            WorkspaceValidationError::PathNotDirectory(p) => {
                write!(f, "Workspace path is not a directory: {}", p)
            }
            WorkspaceValidationError::PathNotReadable(p) => {
                write!(f, "Workspace path is not readable: {}", p)
            }
            WorkspaceValidationError::PathPermissionDenied(p) => {
                write!(f, "Permission denied accessing workspace path: {}", p)
            }
            WorkspaceValidationError::PathCircularSymlink(p) => {
                write!(f, "Circular symbolic link detected: {}", p)
            }
            WorkspaceValidationError::PathTraversalDetected(p) => {
                write!(f, "Path traversal detected: {}", p)
            }
            WorkspaceValidationError::PathNotAbsolute(p) => {
                write!(f, "Absolute path required, got relative path: {}", p)
            }
        }
    }
}

impl std::error::Error for WorkspaceValidationError {}

pub type WorkspaceValidationResult = Result<PathBuf, WorkspaceValidationError>;
