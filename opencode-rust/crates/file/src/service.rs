use crate::error::FileError;
use crate::normalize::Normalizer;
use crate::watch::Debouncer;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;

pub struct FileService {
    watch_handles: Arc<TokioMutex<HashMap<String, Arc<StdMutex<Option<RecommendedWatcher>>>>>>,
    normalizer: Normalizer,
}

impl FileService {
    pub fn new() -> Self {
        Self {
            watch_handles: Arc::new(TokioMutex::new(HashMap::new())),
            normalizer: Normalizer::new(),
        }
    }

    pub async fn watch(
        &self,
        path: &Path,
        debounce_ms: u64,
        callback: impl Fn(PathBuf) + Clone + Send + Sync + 'static,
    ) -> Result<String, FileError> {
        let watch_id = uuid::Uuid::new_v4().to_string();
        let delay = Duration::from_millis(debounce_ms);
        let path_owned = path.to_path_buf();

        let callback: Arc<dyn Fn(PathBuf) + Send + Sync + 'static> = Arc::new(callback);
        let callback_clone = callback.clone();

        let debouncer = Debouncer::new(delay);
        let debouncer_clone = debouncer.clone();

        let (tx, rx) = std::sync::mpsc::channel::<PathBuf>();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                while let Ok(path) = rx.recv() {
                    let debouncer = debouncer_clone.clone();
                    let callback = callback_clone.clone();
                    let path_clone = path.clone();

                    debouncer.queue(path_clone, move || {
                        callback(path);
                    }).await;
                }
            });
        });

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        for p in event.paths {
                            if tx.send(p).is_err() {
                                return;
                            }
                        }
                    }
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| FileError::Watch(e.to_string()))?;

        let mut watcher = watcher;
        watcher
            .watch(&path_owned, RecursiveMode::Recursive)
            .map_err(|e| FileError::Watch(e.to_string()))?;

        let watcher_arc = Arc::new(StdMutex::new(Some(watcher)));
        let mut handles = self.watch_handles.lock().await;
        handles.insert(watch_id.clone(), watcher_arc);

        Ok(watch_id)
    }

    pub async fn unwatch(&self, watch_id: &str) -> Result<(), FileError> {
        let watcher_arc = {
            let mut handles = self.watch_handles.lock().await;
            handles
                .remove(watch_id)
                .ok_or_else(|| FileError::WatchNotFound(watch_id.to_string()))?
        };

        let watcher = {
            let mut guard = watcher_arc.lock().unwrap();
            guard.take()
        };
        drop(watcher);

        Ok(())
    }

    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError> {
        if !from.exists() {
            return Err(FileError::NotFound(from.to_path_buf()));
        }
        if !from.is_file() {
            return Err(FileError::NotAFile(from.to_path_buf()));
        }

        if let Some(parent) = to.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| FileError::Io {
                    context: format!(
                        "Failed to create parent directory for {}",
                        to.display()
                    ),
                    source: Arc::new(e),
                })?;
        }

        let bytes = tokio::fs::copy(from, to).await.map_err(|e| FileError::Io {
            context: format!("Failed to copy {} to {}", from.display(), to.display()),
            source: Arc::new(e),
        })?;

        Ok(bytes)
    }

    pub async fn copy_dir(&self, from: &Path, to: &Path) -> Result<u64, FileError> {
        use walkdir::WalkDir;

        if !from.exists() {
            return Err(FileError::NotFound(from.to_path_buf()));
        }
        if !from.is_dir() {
            return Err(FileError::NotADirectory(from.to_path_buf()));
        }

        let mut total_bytes = 0u64;

        for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
            let source_path = entry.path();
            let relative_path =
                source_path.strip_prefix(from).map_err(|_| FileError::Io {
                    context: String::new(),
                    source: Arc::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Failed to compute relative path",
                    )),
                })?;
            let dest_path = to.join(relative_path);

            if source_path.is_dir() {
                tokio::fs::create_dir_all(&dest_path)
                    .await
                    .map_err(|e| FileError::Io {
                        context: format!("Failed to create directory {}", dest_path.display()),
                        source: Arc::new(e),
                    })?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    tokio::fs::create_dir_all(parent)
                        .await
                        .map_err(|e| FileError::Io {
                            context: format!(
                                "Failed to create parent directory for {}",
                                dest_path.display()
                            ),
                            source: Arc::new(e),
                        })?;
                }
                let bytes = tokio::fs::copy(source_path, &dest_path)
                    .await
                    .map_err(|e| FileError::Io {
                        context: format!(
                            "Failed to copy {} to {}",
                            source_path.display(),
                            dest_path.display()
                        ),
                        source: Arc::new(e),
                    })?;
                total_bytes += bytes;
            }
        }

        Ok(total_bytes)
    }

    pub async fn create_dir_all(&self, path: &Path) -> Result<(), FileError> {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| FileError::Io {
                context: format!("Failed to create directory {}", path.display()),
                source: Arc::new(e),
            })?;
        Ok(())
    }

    pub async fn exists(&self, path: &Path) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    pub async fn remove_file(&self, path: &Path) -> Result<(), FileError> {
        if !path.exists() {
            return Err(FileError::NotFound(path.to_path_buf()));
        }
        if !path.is_file() {
            return Err(FileError::NotAFile(path.to_path_buf()));
        }
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| FileError::Io {
                context: format!("Failed to remove file {}", path.display()),
                source: Arc::new(e),
            })?;
        Ok(())
    }

    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        let normalized = self.normalizer.normalize(path);
        if normalized.is_absolute() {
            normalized
        } else {
            let cwd = std::env::current_dir().unwrap_or_default();
            self.normalizer.normalize(&cwd.join(&normalized))
        }
    }

    pub async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FileError> {
        tokio::fs::canonicalize(path)
            .await
            .map_err(|e| FileError::Io {
                context: format!("Failed to canonicalize {}", path.display()),
                source: Arc::new(e),
            })
    }

    pub fn resolve_path(&self, base: &Path, relative: &Path) -> PathBuf {
        self.normalizer.resolve_path(base, relative)
    }

    pub fn normalize(&self, path: &Path) -> PathBuf {
        self.normalizer.normalize(path)
    }
}

impl Default for FileService {
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
        let svc = FileService::new();
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.txt");
        let dst = tmp.path().join("deep").join("nested").join("dst.txt");
        tokio::fs::write(&src, "content").await.unwrap();

        let n = svc.copy_file(&src, &dst).await.unwrap();
        assert_eq!(n, 7);
        assert!(dst.exists());
    }

    #[tokio::test]
    async fn test_create_dir_all_async() {
        let svc = FileService::new();
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested").join("deeply");

        svc.create_dir_all(&path).await.unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_normalize_collapse_dots() {
        let svc = FileService::new();
        let p = svc.normalize(Path::new("/a/b/../c/./d"));
        assert_eq!(p, Path::new("/a/c/d"));
    }

    #[tokio::test]
    async fn test_exists_returns_bool() {
        let svc = FileService::new();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("exists.txt");
        tokio::fs::write(&file, "content").await.unwrap();

        assert!(svc.exists(&file).await);
        assert!(!svc.exists(&tmp.path().join("nonexistent.txt")).await);
    }

    #[tokio::test]
    async fn test_remove_file_not_found() {
        let svc = FileService::new();
        let tmp = TempDir::new().unwrap();
        let nonexistent = tmp.path().join("nonexistent.txt");
        let result = svc.remove_file(&nonexistent).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FileError::NotFound(p) => assert_eq!(p, nonexistent),
            _ => panic!("Expected FileError::NotFound"),
        }
    }
}