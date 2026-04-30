use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{DirectoryEntry, FileMetadata, FileResult, FileSystem, FileSystemError};

#[allow(dead_code)]
pub struct RealFileSystem {
    base_path: PathBuf,
}

impl RealFileSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    #[allow(dead_code)]
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn map_err(err: std::io::Error) -> FileSystemError {
        match err.kind() {
            std::io::ErrorKind::NotFound => FileSystemError::NotFound,
            std::io::ErrorKind::PermissionDenied => FileSystemError::PermissionDenied,
            std::io::ErrorKind::AlreadyExists => FileSystemError::AlreadyExists,
            std::io::ErrorKind::IsADirectory => FileSystemError::IsDirectory,
            std::io::ErrorKind::NotADirectory => FileSystemError::NotADirectory,
            _ => FileSystemError::IoError,
        }
    }

    fn map_metadata(meta: std::fs::Metadata) -> FileMetadata {
        FileMetadata {
            path: PathBuf::new(),
            is_file: meta.is_file(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default()
                }),
            created: meta
                .created()
                .ok()
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default()
                }),
        }
    }
}

impl Default for RealFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
#[allow(clippy::ptr_arg)]
impl FileSystem for RealFileSystem {
    async fn read_file(&self, path: &Path) -> FileResult<String> {
        let mut file = fs::File::open(path).await.map_err(Self::map_err)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(Self::map_err)?;
        Ok(contents)
    }

    async fn write_file(&self, path: &Path, content: &str) -> FileResult<()> {
        let mut file = fs::File::create(path).await.map_err(Self::map_err)?;
        file.write_all(content.as_bytes())
            .await
            .map_err(Self::map_err)?;
        file.sync_all().await.map_err(Self::map_err)?;
        Ok(())
    }

    async fn append_file(&self, path: &Path, content: &str) -> FileResult<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(Self::map_err)?;
        file.write_all(content.as_bytes())
            .await
            .map_err(Self::map_err)?;
        Ok(())
    }

    async fn delete_file(&self, path: &Path) -> FileResult<()> {
        fs::remove_file(path).await.map_err(Self::map_err)
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> FileResult<()> {
        fs::copy(from, to).await.map_err(Self::map_err)?;
        Ok(())
    }

    async fn move_file(&self, from: &Path, to: &Path) -> FileResult<()> {
        fs::rename(from, to).await.map_err(Self::map_err)
    }

    async fn create_directory(&self, path: &Path) -> FileResult<()> {
        fs::create_dir_all(path).await.map_err(Self::map_err)
    }

    async fn delete_directory(&self, path: &Path, recursive: bool) -> FileResult<()> {
        if recursive {
            fs::remove_dir_all(path).await.map_err(Self::map_err)
        } else {
            fs::remove_dir(path).await.map_err(Self::map_err)
        }
    }

    async fn list_directory(&self, path: &Path) -> FileResult<Vec<DirectoryEntry>> {
        let mut entries = Vec::new();
        let mut dir = fs::read_dir(path).await.map_err(Self::map_err)?;
        while let Some(entry) = dir.next_entry().await.map_err(Self::map_err)? {
            let meta = entry.metadata().await.map_err(Self::map_err)?;
            entries.push(DirectoryEntry {
                path: entry.path(),
                is_file: meta.is_file(),
                is_dir: meta.is_dir(),
                size: meta.len(),
            });
        }
        Ok(entries)
    }

    async fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    async fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    async fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    async fn metadata(&self, path: &Path) -> FileResult<FileMetadata> {
        let meta = fs::metadata(path).await.map_err(Self::map_err)?;
        let mut result = Self::map_metadata(meta);
        result.path = path.to_path_buf();
        Ok(result)
    }

    async fn read_file_if_exists(&self, path: &Path) -> FileResult<Option<String>> {
        if path.exists() {
            self.read_file(path).await.map(Some)
        } else {
            Ok(None)
        }
    }

    async fn atomic_write(&self, path: &Path, content: &str) -> FileResult<()> {
        let temp_path = path.with_extension("tmp");
        self.write_file(&temp_path, content).await?;
        fs::rename(&temp_path, path).await.map_err(Self::map_err)
    }
}
