# PRD: file Module

## Module Overview

- **Module Name**: `file`
- **Source Path**: `packages/opencode/src/file/`
- **Type**: Utility
- **Rust Crate**: `crates/file/` (or `crates/tools/src/file.rs` for co-location with tools)
- **Purpose**: Filesystem utilities — file watching, copying, directory creation, and path normalization. Used throughout the tools layer for operations beyond basic I/O.

---

## Functionality

### Core Features

1. **File Watching** — Watch files/directories for changes using the `notify` crate
2. **File Copying** — Async copy with parent directory creation
3. **Directory Creation** — Create directories recursively with proper permissions
4. **Path Normalization** — Resolve relative/absolute paths, handle symlinks, normalize for platform
5. **File Existence Check** — Fast existence check without stat
6. **File Deletion** — Remove files (not directories)

---

## API Surface

### `FileService`

```rust
pub struct FileService {
    watch_handles: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

impl FileService {
    /// Watch a file or directory for changes. Returns a `WatchId` string.
    pub async fn watch(
        &self,
        path: &Path,
        debounce_ms: u64,
        callback: impl Fn(PathBuf) + Send + 'static,
    ) -> Result<String, FileError>

    /// Stop watching by ID
    pub async fn unwatch(&self, watch_id: &str) -> Result<(), FileError>

    /// Copy a file to a destination, creating parent dirs if needed
    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError>

    /// Copy a directory tree recursively
    pub async fn copy_dir(&self, from: &Path, to: &Path) -> Result<u64, FileError>

    /// Create a directory and all parents (like `mkdir -p`)
    pub async fn create_dir_all(&self, path: &Path) -> Result<(), FileError>

    /// Check if a path exists (fast, no stat)
    pub async fn exists(&self, path: &Path) -> bool

    /// Delete a file (not directory)
    pub async fn remove_file(&self, path: &Path) -> Result<(), FileError>

    /// Normalize a path: resolve symlinks, absolute form, platform separators
    pub fn normalize_path(&self, path: &Path) -> PathBuf
}
```

### Path Normalization Variants

```rust
/// Canonicalize: resolve symlinks + make absolute (requires path to exist)
pub async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FileError>

/// Resolve relative to base, make absolute (does NOT resolve symlinks)
pub fn resolve_path(&self, base: &Path, relative: &Path) -> PathBuf

/// Normalize without filesystem access (collapse `..`, `.`, fix separators)
pub fn normalize(&self, path: &Path) -> PathBuf
```

---

## Data Structures

### `FileError`

```rust
#[derive(Debug, Error)]
pub enum FileError {
    #[error("Path not found: {0}")]
    NotFound(PathBuf),

    #[error("Not a file: {0}")]
    NotAFile(PathBuf),

    #[error("Not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("IO error: {context}: {source}")]
    Io { context: String, #[source] source: std::io::Error },

    #[error("Watch error: {0}")]
    Watch(String),

    #[error("Watch not found: {0}")]
    WatchNotFound(String),

    #[error("Path too long: {0}")]
    PathTooLong(PathBuf),
}
```

### Debouncer

File watching uses debouncing to batch rapid changes:

```rust
pub struct Debouncer {
    delay: Duration,
    pending: Arc<Mutex<HashMap<PathBuf, tokio::time::Sleep>>>,
}

impl Debouncer {
    /// Queue a path for debounced callback. Multiple calls within `delay` are collapsed.
    pub fn queue(&self, path: PathBuf, callback: impl FnOnce());
}
```

---

## Crate Layout

```
crates/file/
├── Cargo.toml       # notify = "6", tokio = { features = ["fs", "sync", "rt"] }
├── src/
│   ├── lib.rs       # FileService, FileError, path utilities
│   ├── watch.rs     # FileWatcher, Debouncer implementations
│   ├── copy.rs      # File/directory copy implementations
│   └── normalize.rs # Path normalization utilities
└── tests/
    └── file_tests.rs
```

### `Cargo.toml` dependencies

```toml
[package]
name = "opencode-file"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "sync", "rt", "time"] }
notify = "6"
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
walkdir = "2"
filedescriptor = "0.6"  # for copy_file using sendfile on unix
anyhow = "1.0"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

---

## File Watch Flow

```
FileService::watch(path)
  → notify::recommended_watcher(move |res| { ... })
  → Debouncer::new(debounce_ms)
  → store watcher handle with unique watch_id
  → return watch_id string

On filesystem event:
  → Debouncer::queue(path)
  → sleep(debounce_ms)
  → if no more events: callback(path)
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `notify = "6"` | Cross-platform file watching |
| `tokio::fs` | Async filesystem I/O |
| `walkdir` | Recursive directory traversal |
| `tracing` | Structured logging |
| `tempfile` (dev) | Test fixtures |

---

## Acceptance Criteria

- [ ] `watch()` registers a filesystem watcher and returns a `WatchId`
- [ ] `unwatch()` stops the watcher and removes it from the registry
- [ ] Multiple rapid changes are debounced into single callbacks
- [ ] `copy_file()` creates parent directories and copies with correct byte count
- [ ] `copy_dir()` recursively copies an entire directory tree
- [ ] `normalize_path()` produces consistent absolute paths on all platforms
- [ ] `exists()` returns `true`/`false` without throwing errors
- [ ] `remove_file()` returns `NotFound` error for non-existent paths
- [ ] All operations are `Send + Sync` safe across tokio tasks

---

## Rust Implementation Notes

### File Watch Gotchas

- `notify` watchers must be kept alive (stored in `FileService`)
- Use `notify::RecommendedWatcher` with `std::sync::mpsc` channel
- Convert to `tokio` async channel with `tokio::sync::mpsc` via spawning
- On macOS: FSEvents backend; on Linux: inotify; on Windows: ReadDirectoryChangesW

### Copy Implementation

```rust
pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError> {
    if !from.exists().await { return Err(FileError::NotFound(from.to_path_buf())); }
    if let Some(parent) = to.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| FileError::Io {
            context: format!("create parent dir for {}", to.display()),
            source: e,
        })?;
    }
    let metadata = tokio::fs::metadata(from).await.map_err(|e| FileError::Io {
        context: format!("stat {}", from.display()),
        source: e,
    })?;
    let nbytes = metadata.len();
    tokio::fs::copy(from, to).await.map_err(|e| FileError::Io {
        context: format!("copy {} → {}", from.display(), to.display()),
        source: e,
    })?;
    Ok(nbytes)
}
```

### Path Normalization

```rust
pub fn normalize(&self, path: &Path) -> PathBuf {
    let s = path.as_os_str();
    let mut components = Vec::new();
    for part in Path::new(s) {
        match part.to_str() {
            Some(".") | None => {}
            Some("..") => { components.pop(); }
            _ => components.push(part),
        }
    }
    components.iter().collect()
}
```

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_watch_fires_callback_on_file_change() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("watched.txt");
    std::fs::write(&file, "v1").unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let tx_clone = tx.clone();
    let watch_id = svc.watch(tmp.path(), 50, move |p| {
        let _ = tx_clone.blocking_send(p);
    }).await.unwrap();

    tokio::fs::write(&file, "v2").await.unwrap();
    tokio::time::sleep(Duration::from_millis(150)).await;

    let changed = rx.recv().await.unwrap();
    assert!(changed.ends_with("watched.txt"));

    svc.unwatch(&watch_id).await.unwrap();
}

#[tokio::test]
async fn test_debouncer_merges_rapid_events() {
    let debounce = Duration::from_millis(100);
    let mut d = Debouncer::new(debounce);
    let count = Arc::new(Mutex::new(0));
    let count2 = count.clone();

    d.queue(PathBuf::from("a.txt"), move || { *count2.lock().unwrap() += 1; });
    d.queue(PathBuf::from("a.txt"), move || { *count2.clone().lock().unwrap() += 1; });

    tokio::time::sleep(debounce * 2).await;
    assert_eq!(*count.lock().unwrap(), 1); // only one callback
}

#[tokio::test]
async fn test_copy_file_creates_parent_dirs() {
    let svc = FileService::new();
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("deep").join("nested").join("dst.txt");
    std::fs::write(&src, "content").unwrap();

    let n = svc.copy_file(&src, &dst).await.unwrap();
    assert_eq!(n, 7);
    assert!(dst.exists());
}

#[test]
fn test_normalize_collapse_dots() {
    let svc = FileService::new();
    let p = svc.normalize(Path::new("/a/b/../c/./d"));
    assert_eq!(p, Path::new("/a/c/d"));
}
```

---

## Integration Tests

| TS Test | Rust Test |
|---------|-----------|
| File watching debounce logic | `test_debouncer_merges_rapid_events` |
| File copy creates parents | `test_copy_file_creates_parent_dirs` |
| Path normalization | `test_normalize_collapse_dots` |
| Watch fires on change | `test_watch_fires_callback_on_file_change` |

---

*Source: `packages/opencode/src/file/index.ts`*
*No existing Rust equivalent — implement in `crates/file/`*
