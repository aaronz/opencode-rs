mod fake;
mod traits;

pub use fake::FakeFileSystem;
pub use traits::RealFileSystem;

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    pub created: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub is_file: bool,
    pub is_dir: bool,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSystemError {
    NotFound,
    PermissionDenied,
    AlreadyExists,
    IsDirectory,
    NotADirectory,
    InvalidPath,
    IoError,
}

impl std::fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSystemError::NotFound => write!(f, "file not found"),
            FileSystemError::PermissionDenied => write!(f, "permission denied"),
            FileSystemError::AlreadyExists => write!(f, "file already exists"),
            FileSystemError::IsDirectory => write!(f, "path is a directory"),
            FileSystemError::NotADirectory => write!(f, "path is not a directory"),
            FileSystemError::InvalidPath => write!(f, "invalid path"),
            FileSystemError::IoError => write!(f, "I/O error"),
        }
    }
}

pub type FileResult<T> = Result<T, FileSystemError>;

#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait FileSystem: Send + Sync {
    async fn read_file(&self, path: &Path) -> FileResult<String>;
    async fn write_file(&self, path: &Path, content: &str) -> FileResult<()>;
    async fn append_file(&self, path: &Path, content: &str) -> FileResult<()>;
    async fn delete_file(&self, path: &Path) -> FileResult<()>;
    async fn copy_file(&self, from: &Path, to: &Path) -> FileResult<()>;
    async fn move_file(&self, from: &Path, to: &Path) -> FileResult<()>;

    async fn create_directory(&self, path: &Path) -> FileResult<()>;
    async fn delete_directory(&self, path: &Path, recursive: bool) -> FileResult<()>;
    async fn list_directory(&self, path: &Path) -> FileResult<Vec<DirectoryEntry>>;

    async fn exists(&self, path: &Path) -> bool;
    async fn is_file(&self, path: &Path) -> bool;
    async fn is_dir(&self, path: &Path) -> bool;
    async fn metadata(&self, path: &Path) -> FileResult<FileMetadata>;

    async fn read_file_if_exists(&self, path: &Path) -> FileResult<Option<String>>;
    async fn atomic_write(&self, path: &Path, content: &str) -> FileResult<()>;
}
