use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use super::{DirectoryEntry, FileMetadata, FileResult, FileSystem, FileSystemError};

#[allow(dead_code)]
pub struct FakeFileSystem {
    files: RwLock<HashMap<PathBuf, FileContent>>,
}

struct FileContent {
    content: String,
    is_dir: bool,
    modified: chrono::DateTime<Utc>,
}

impl FakeFileSystem {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            files: RwLock::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn add_file(&self, path: PathBuf, content: String) {
        let mut files = self.files.write().unwrap();
        files.insert(
            path,
            FileContent {
                content,
                is_dir: false,
                modified: Utc::now(),
            },
        );
    }

    #[allow(dead_code)]
    pub fn add_directory(&self, path: PathBuf) {
        let mut files = self.files.write().unwrap();
        files.insert(
            path,
            FileContent {
                content: String::new(),
                is_dir: true,
                modified: Utc::now(),
            },
        );
    }
}

impl Default for FakeFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
#[allow(clippy::ptr_arg)]
impl FileSystem for FakeFileSystem {
    async fn read_file(&self, path: &Path) -> FileResult<String> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .map(|f| {
                if f.is_dir {
                    Err(FileSystemError::IsDirectory)
                } else {
                    Ok(f.content.clone())
                }
            })
            .unwrap_or(Err(FileSystemError::NotFound))
    }

    async fn write_file(&self, path: &Path, content: &str) -> FileResult<()> {
        let mut files = self.files.write().unwrap();
        files.insert(
            path.to_path_buf(),
            FileContent {
                content: content.to_string(),
                is_dir: false,
                modified: Utc::now(),
            },
        );
        Ok(())
    }

    async fn append_file(&self, path: &Path, content: &str) -> FileResult<()> {
        let mut files = self.files.write().unwrap();
        if let Some(existing) = files.get_mut(path) {
            if existing.is_dir {
                return Err(FileSystemError::IsDirectory);
            }
            existing.content.push_str(content);
            existing.modified = Utc::now();
        } else {
            files.insert(
                path.to_path_buf(),
                FileContent {
                    content: content.to_string(),
                    is_dir: false,
                    modified: Utc::now(),
                },
            );
        }
        Ok(())
    }

    async fn delete_file(&self, path: &Path) -> FileResult<()> {
        let mut files = self.files.write().unwrap();
        if let Some(f) = files.get(path) {
            if f.is_dir {
                return Err(FileSystemError::IsDirectory);
            }
        }
        files.remove(path).ok_or(FileSystemError::NotFound)?;
        Ok(())
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> FileResult<()> {
        let content = {
            let files = self.files.read().unwrap();
            if let Some(f) = files.get(from) {
                if f.is_dir {
                    return Err(FileSystemError::IsDirectory);
                }
                f.content.clone()
            } else {
                return Err(FileSystemError::NotFound);
            }
        };
        self.write_file(to, &content).await
    }

    async fn move_file(&self, from: &Path, to: &Path) -> FileResult<()> {
        self.copy_file(from, to).await?;
        self.delete_file(from).await
    }

    async fn create_directory(&self, path: &Path) -> FileResult<()> {
        let mut files = self.files.write().unwrap();
        if files.contains_key(path) {
            return Err(FileSystemError::AlreadyExists);
        }
        files.insert(
            path.to_path_buf(),
            FileContent {
                content: String::new(),
                is_dir: true,
                modified: Utc::now(),
            },
        );
        Ok(())
    }

    async fn delete_directory(&self, path: &Path, recursive: bool) -> FileResult<()> {
        let mut files = self.files.write().unwrap();
        if let Some(f) = files.get(path) {
            if !f.is_dir {
                return Err(FileSystemError::NotADirectory);
            }
        } else {
            return Err(FileSystemError::NotFound);
        }

        if !recursive {
            let has_children = files.keys().any(|p| p.starts_with(path));
            if has_children {
                return Err(FileSystemError::IoError);
            }
        }

        files.remove(path);
        Ok(())
    }

    async fn list_directory(&self, path: &Path) -> FileResult<Vec<DirectoryEntry>> {
        let files = self.files.read().unwrap();
        let entries: Vec<DirectoryEntry> = files
            .iter()
            .filter(|(p, _)| p.parent() == Some(path) || **p == *path)
            .map(|(p, f)| DirectoryEntry {
                path: p.clone(),
                is_file: !f.is_dir,
                is_dir: f.is_dir,
                size: f.content.len() as u64,
            })
            .collect();
        Ok(entries)
    }

    async fn exists(&self, path: &Path) -> bool {
        let files = self.files.read().unwrap();
        files.contains_key(path)
    }

    async fn is_file(&self, path: &Path) -> bool {
        let files = self.files.read().unwrap();
        files.get(path).map(|f| !f.is_dir).unwrap_or(false)
    }

    async fn is_dir(&self, path: &Path) -> bool {
        let files = self.files.read().unwrap();
        files.get(path).map(|f| f.is_dir).unwrap_or(false)
    }

    async fn metadata(&self, path: &Path) -> FileResult<FileMetadata> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .map(|f| FileMetadata {
                path: path.to_path_buf(),
                is_file: !f.is_dir,
                is_dir: f.is_dir,
                size: f.content.len() as u64,
                modified: Some(f.modified),
                created: Some(f.modified),
            })
            .ok_or(FileSystemError::NotFound)
    }

    async fn read_file_if_exists(&self, path: &Path) -> FileResult<Option<String>> {
        Ok(Some(self.read_file(path).await.ok()).flatten())
    }

    async fn atomic_write(&self, path: &Path, content: &str) -> FileResult<()> {
        self.write_file(path, content).await
    }
}
