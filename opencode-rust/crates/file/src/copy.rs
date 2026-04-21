use crate::error::FileError;
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

pub struct Copier;

impl Copier {
    pub fn new() -> Self {
        Self
    }

    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError> {
        if !from.exists() {
            return Err(FileError::NotFound(from.to_path_buf()));
        }
        if !from.is_file() {
            return Err(FileError::NotAFile(from.to_path_buf()));
        }

        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).await.map_err(|e| FileError::Io {
                context: format!("Failed to create parent directory for {}", to.display()),
                source: e,
            })?;
        }

        let bytes = fs::copy(from, to).await.map_err(|e| FileError::Io {
            context: format!("Failed to copy {} to {}", from.display(), to.display()),
            source: e,
        })?;

        Ok(bytes)
    }

    pub async fn copy_dir(&self, from: &Path, to: &Path) -> Result<u64, FileError> {
        if !from.exists() {
            return Err(FileError::NotFound(from.to_path_buf()));
        }
        if !from.is_dir() {
            return Err(FileError::NotADirectory(from.to_path_buf()));
        }

        let mut total_bytes = 0u64;

        for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
            let source_path = entry.path();
            let relative_path = source_path.strip_prefix(from).map_err(|_| FileError::Io {
                context: String::new(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Failed to compute relative path",
                ),
            })?;
            let dest_path = to.join(relative_path);

            if source_path.is_dir() {
                fs::create_dir_all(&dest_path).await.map_err(|e| FileError::Io {
                    context: format!(
                        "Failed to create directory {}",
                        dest_path.display()
                    ),
                    source: e,
                })?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).await.map_err(|e| FileError::Io {
                        context: format!(
                            "Failed to create parent directory for {}",
                            dest_path.display()
                        ),
                        source: e,
                    })?;
                }
                let bytes = fs::copy(source_path, &dest_path).await.map_err(|e| FileError::Io {
                    context: format!(
                        "Failed to copy {} to {}",
                        source_path.display(),
                        dest_path.display()
                    ),
                    source: e,
                })?;
                total_bytes += bytes;
            }
        }

        Ok(total_bytes)
    }
}

impl Default for Copier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_copy_file_creates_parent_dirs() {
        let copier = Copier::new();
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.txt");
        let dst = tmp.path().join("deep").join("nested").join("dst.txt");
        tokio::fs::write(&src, "content").await.unwrap();

        let n = copier.copy_file(&src, &dst).await.unwrap();
        assert_eq!(n, 7);
        assert!(dst.exists());
    }

    #[tokio::test]
    async fn test_copy_dir_recursive() {
        let copier = Copier::new();
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source");
        let dst = tmp.path().join("dest");

        tokio::fs::create_dir_all(&src).await.unwrap();
        tokio::fs::write(src.join("file1.txt"), "content1").await.unwrap();
        tokio::fs::write(src.join("file2.txt"), "content2").await.unwrap();
        tokio::fs::create_dir_all(src.join("subdir")).await.unwrap();
        tokio::fs::write(src.join("subdir").join("file3.txt"), "content3")
            .await
            .unwrap();

        let n = copier.copy_dir(&src, &dst).await.unwrap();
        assert!(n > 0);
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("file2.txt").exists());
        assert!(dst.join("subdir").join("file3.txt").exists());
    }
}