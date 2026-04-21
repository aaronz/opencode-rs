# Specification: file Module (Iteration 44)

**Document Version**: 44
**Date**: 2026-04-21
**Status**: Updated from Gap Analysis
**Source PRD**: `packages/opencode/src/file/` (TypeScript)

---

## Module Overview

- **Module Name**: `file`
- **Source Path**: `packages/opencode/src/file/`
- **Rust Crate**: `crates/file/`
- **Type**: Utility
- **Purpose**: Filesystem utilities — file watching, copying, directory creation, and path normalization. Used throughout the tools layer for operations beyond basic I/O.

---

## Implementation Status Summary

| Component | Status | Gap |
|-----------|--------|-----|
| `crates/file/` crate | ❌ Missing | Must create from scratch |
| `FileService` struct | ❌ Missing | No watcher registry |
| File watching (notify) | ❌ Missing | No `watch()`/`unwatch()` |
| Path normalization | ❌ Broken | Identity function |
| File copying | ⚠️ Partial | Write exists, copy missing |
| Directory creation | ✅ Exists | Sync only, needs async |
| File existence check | ✅ Exists | Sync only |
| File deletion | ⚠️ Partial | Tool exists, not service |

**Overall Progress**: 16% (gap analysis baseline)

---

## Feature Requirements

### FR-001: File Watching (notify Integration)

**Priority**: P0
**Module**: `watch.rs`
**Status**: ❌ Not Implemented

#### Description
Watch files/directories for changes using the `notify` crate with debouncing support.

#### API
```rust
pub async fn watch(
    &self,
    path: &Path,
    debounce_ms: u64,
    callback: impl Fn(PathBuf) + Send + 'static,
) -> Result<String, FileError>
```

#### Requirements
- [ ] Register a filesystem watcher using `notify::RecommendedWatcher`
- [ ] Store watcher handle in `watch_handles: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>`
- [ ] Generate unique `watch_id` string for each watcher
- [ ] Implement debouncing via `Debouncer` struct
- [ ] Return `WatchId` string on success
- [ ] Platform backends: FSEvents (macOS), inotify (Linux), ReadDirectoryChangesW (Windows)

#### Debouncer Behavior
```rust
pub struct Debouncer {
    delay: Duration,
    pending: Arc<Mutex<HashMap<PathBuf, tokio::time::Sleep>>>,
}
```
- [ ] Multiple rapid events on same path collapse into single callback
- [ ] `delay` determines minimum time between callback invocations
- [ ] Thread-safe via `Arc<Mutex<...>>`

#### Unwatch API
```rust
pub async fn unwatch(&self, watch_id: &str) -> Result<(), FileError>
```
- [ ] Stop the watcher identified by `watch_id`
- [ ] Remove from registry
- [ ] Return `FileError::WatchNotFound` if ID not found

---

### FR-002: File Copy Operations

**Priority**: P1
**Module**: `copy.rs`
**Status**: ⚠️ Partial

#### FR-002a: Copy File with Parent Directory Creation
**Priority**: P1

```rust
pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError>
```
- [ ] Check source exists, return `FileError::NotFound` if missing
- [ ] Create parent directories via `tokio::fs::create_dir_all`
- [ ] Copy file via `tokio::fs::copy`
- [ ] Return bytes copied
- [ ] Map errors to `FileError::Io`

#### FR-002b: Copy Directory Tree Recursively
**Priority**: P1

```rust
pub async fn copy_dir(&self, from: &Path, to: &Path) -> Result<u64, FileError>
```
- [ ] Use `walkdir::WalkDir` for recursive traversal
- [ ] Preserve directory structure
- [ ] Copy all files respecting permissions
- [ ] Return total bytes copied
- [ ] Return `FileError::NotADirectory` if source is not a directory

---

### FR-003: Directory Creation

**Priority**: P2
**Module**: `lib.rs` / `copy.rs`
**Status**: ✅ Exists (sync only)

#### Current Implementation
- `AppFileSystem::ensure_dir(path: &str)` exists in `crates/core/src/filesystem.rs`

#### Requirements for Async FileService
```rust
pub async fn create_dir_all(&self, path: &Path) -> Result<(), FileError>
```
- [ ] Async version using `tokio::fs::create_dir_all`
- [ ] Recursive directory creation (like `mkdir -p`)
- [ ] Return success or `FileError::Io`

---

### FR-004: Path Normalization

**Priority**: P0
**Module**: `normalize.rs`
**Status**: ❌ Broken

#### Current State
- `AppFileSystem::normalize_path()` is an identity function (line 97-98 in `filesystem.rs`):
  ```rust
  pub fn normalize_path(p: &str) -> std::path::PathBuf {
      PathBuf::from(p)  // BUG: No actual normalization
  }
  ```

#### FR-004a: Full Normalization (canonicalize)
**Priority**: P1

```rust
pub async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FileError>
```
- [ ] Resolve symlinks
- [ ] Make absolute
- [ ] Requires path to exist
- [ ] Return `FileError::Io` on failure

#### FR-004b: Resolve Relative Path
**Priority**: P2

```rust
pub fn resolve_path(&self, base: &Path, relative: &Path) -> PathBuf
```
- [ ] Resolve `relative` against `base`
- [ ] Make absolute
- [ ] Does NOT resolve symlinks
- [ ] No filesystem access

#### FR-004c: Path Component Normalization
**Priority**: P1

```rust
pub fn normalize(&self, path: &Path) -> PathBuf
```
- [ ] Collapse `.` components (skip)
- [ ] Collapse `..` components (pop)
- [ ] Fix platform separators
- [ ] No filesystem access
- [ ] Example: `/a/b/../c/./d` → `/a/c/d`

#### FR-004d: normalize_path (Public API)
**Priority**: P0

```rust
pub fn normalize_path(&self, path: &Path) -> PathBuf
```
- [ ] Public entry point combining normalization strategies
- [ ] Platform-aware
- [ ] Produce consistent absolute paths on all platforms

---

### FR-005: File Existence Check

**Priority**: P2
**Module**: `lib.rs`
**Status**: ✅ Exists (sync only)

#### Current Implementation
- `AppFileSystem::exists(path: &str)` in `crates/core/src/filesystem.rs`

#### Requirements for Async FileService
```rust
pub async fn exists(&self, path: &Path) -> bool
```
- [ ] Fast existence check (no stat if possible)
- [ ] Return `true` if path exists (file or directory)
- [ ] Return `false` otherwise
- [ ] No error on missing path

---

### FR-006: File Deletion

**Priority**: P2
**Module**: `lib.rs`
**Status**: ⚠️ Partial

#### Current State
- `FileDeleteTool` exists in tools layer
- Not exposed via `FileService` API

#### Requirements
```rust
pub async fn remove_file(&self, path: &Path) -> Result<(), FileError>
```
- [ ] Delete a file (not directory)
- [ ] Return `FileError::NotFound` if path does not exist
- [ ] Return `FileError::NotAFile` if path is a directory

---

## Data Structures

### FileError Enum

**Priority**: P1
**Module**: `lib.rs`

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

#### Error Code Ranges (from AGENTS.md)
| Range | Category |
|-------|----------|
| 4xxx | Tool errors |

---

### FileService Struct

**Priority**: P0

```rust
pub struct FileService {
    watch_handles: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

impl FileService {
    pub fn new() -> Self
}
```

---

### Debouncer Struct

**Priority**: P1
**Module**: `watch.rs`

```rust
pub struct Debouncer {
    delay: Duration,
    pending: Arc<Mutex<HashMap<PathBuf, tokio::time::Sleep>>>,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self

    pub fn queue(&self, path: PathBuf, callback: impl FnOnce() + Send + 'static)
}
```

---

## Crate Layout

```
crates/file/
├── Cargo.toml       # notify = "6", tokio with fs/sync/rt/time, walkdir, filedescriptor
├── src/
│   ├── lib.rs       # FileService, FileError, public API exports
│   ├── watch.rs     # FileWatcher, Debouncer implementations
│   ├── copy.rs      # copy_file, copy_dir implementations
│   └── normalize.rs # canonicalize, resolve_path, normalize
└── tests/
    └── file_tests.rs
```

### Cargo.toml Dependencies

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
filedescriptor = "0.6"
anyhow = "1.0"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
```

---

## API Surface (Complete)

```rust
pub struct FileService {
    watch_handles: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

impl FileService {
    // Constructor
    pub fn new() -> Self

    // File watching (FR-001)
    pub async fn watch(
        &self,
        path: &Path,
        debounce_ms: u64,
        callback: impl Fn(PathBuf) + Send + 'static,
    ) -> Result<String, FileError>

    pub async fn unwatch(&self, watch_id: &str) -> Result<(), FileError>

    // File copying (FR-002)
    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError>
    pub async fn copy_dir(&self, from: &Path, to: &Path) -> Result<u64, FileError>

    // Directory creation (FR-003)
    pub async fn create_dir_all(&self, path: &Path) -> Result<(), FileError>

    // File operations (FR-005, FR-006)
    pub async fn exists(&self, path: &Path) -> bool
    pub async fn remove_file(&self, path: &Path) -> Result<(), FileError>

    // Path normalization (FR-004)
    pub fn normalize_path(&self, path: &Path) -> PathBuf
    pub async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FileError>
    pub fn resolve_path(&self, base: &Path, relative: &Path) -> PathBuf
    pub fn normalize(&self, path: &Path) -> PathBuf
}
```

---

## Acceptance Criteria

| ID | Criteria | Priority | Status |
|----|----------|----------|--------|
| AC-001 | `watch()` registers a filesystem watcher and returns a `WatchId` | P0 | ❌ |
| AC-002 | `unwatch()` stops the watcher and removes it from the registry | P0 | ❌ |
| AC-003 | Multiple rapid changes are debounced into single callbacks | P1 | ❌ |
| AC-004 | `copy_file()` creates parent directories and copies with correct byte count | P1 | ❌ |
| AC-005 | `copy_dir()` recursively copies an entire directory tree | P1 | ❌ |
| AC-006 | `normalize_path()` produces consistent absolute paths on all platforms | P0 | ❌ |
| AC-007 | `exists()` returns `true`/`false` without throwing errors | P2 | ⚠️ |
| AC-008 | `remove_file()` returns `NotFound` error for non-existent paths | P2 | ❌ |
| AC-009 | All operations are `Send + Sync` safe across tokio tasks | P0 | ❌ |

---

## Test Design

### Unit Tests Required

| Test | Feature | FR |
|------|---------|-----|
| `test_watch_fires_callback_on_file_change` | Watch | FR-001 |
| `test_debouncer_merges_rapid_events` | Debouncer | FR-001 |
| `test_copy_file_creates_parent_dirs` | Copy | FR-002a |
| `test_copy_dir_recursive` | Copy | FR-002b |
| `test_create_dir_all_async` | Dir creation | FR-003 |
| `test_normalize_collapse_dots` | Normalize | FR-004c |
| `test_canonicalize_resolves_symlinks` | Canonicalize | FR-004a |
| `test_resolve_path_relative` | Resolve | FR-004b |
| `test_exists_returns_bool` | Exists | FR-005 |
| `test_remove_file_not_found` | Remove | FR-006 |

### Test Implementation

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
    assert_eq!(*count.lock().unwrap(), 1);
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

| Dependency | Purpose | Status |
|------------|---------|--------|
| `notify = "6"` | Cross-platform file watching | ❌ Missing |
| `tokio` with fs, sync, rt, time | Async filesystem I/O | ⚠️ Partial |
| `walkdir` | Recursive directory traversal | ✅ Exists |
| `tracing` | Structured logging | ⚠️ Partial |
| `filedescriptor` | sendfile on Unix | ❌ Missing |
| `tempfile` (dev) | Test fixtures | ✅ Exists |
| `tokio-test` (dev) | Async test runtime | ❌ Missing |

---

## Technical Debt

| ID | Description | Est. Effort | Priority |
|----|-------------|-------------|----------|
| TD-001 | Create `crates/file/` from scratch | High | P0 |
| TD-002 | Migrate sync `AppFileSystem` utilities to async `FileService` | Medium | P1 |
| TD-003 | Add `FileError` typed error enum instead of string errors | Medium | P1 |
| TD-004 | Implement watch/unwatch with notify backend | High | P0 |
| TD-005 | Add debouncing for file change events | Medium | P1 |
| TD-006 | Implement path normalization variants | Medium | P0 |
| TD-007 | Add async copy operations with parent directory creation | Medium | P1 |
| TD-008 | Write unit tests for all FileService methods | Medium | P1 |
| TD-009 | Add integration tests for watch debounce, copy with parents, normalize | Medium | P2 |

---

## Implementation Phases

### Phase 1: Core Structure (P0)
1. Create `crates/file/` crate structure
2. Add `FileError` enum with all variants
3. Implement `FileService` struct with watcher registry
4. Add `Cargo.toml` with all dependencies

### Phase 2: Path Normalization (P0)
1. Implement `normalize()` - collapse dots, fix separators
2. Implement `resolve_path()` - relative to base
3. Implement `canonicalize()` - symlink + absolute
4. Implement `normalize_path()` - public API

### Phase 3: Async File Operations (P1)
1. Implement `exists()` async
2. Implement `create_dir_all()` async
3. Implement `remove_file()` async

### Phase 4: Copy Operations (P1)
1. Implement `copy_file()` with parent dir creation
2. Implement `copy_dir()` recursive copy

### Phase 5: File Watching (P0)
1. Implement `Debouncer` struct
2. Implement `watch()` with notify integration
3. Implement `unwatch()` for cleanup

### Phase 6: Testing (P1)
1. Write unit tests for all methods
2. Add integration tests
3. Verify `Send + Sync` safety

---

## Current Code Reference

**Existing Sync Implementation**:
- `opencode-rust/crates/core/src/filesystem.rs` - `AppFileSystem` (sync utilities)
- Contains: `is_dir`, `is_file`, `exists`, `read_json`, `write_json`, `ensure_dir`, `write_with_dirs`, `find_up`, `up`, `normalize_path` (broken), `overlaps`, `contains`

**Known Issues**:
- `normalize_path()` at line 97-99 is identity function (BUG)
- All operations are sync, not async
- No typed errors (`FileError` missing)
- No file watching capability

---

## Verification Checklist

- [ ] `crates/file/` crate created
- [ ] `Cargo.toml` has all dependencies
- [ ] `FileError` enum with 7 variants implemented
- [ ] `FileService` struct with watcher registry
- [ ] `Debouncer` struct for event debouncing
- [ ] `watch()` returns `WatchId` string
- [ ] `unwatch()` stops and removes watcher
- [ ] `copy_file()` creates parents and returns byte count
- [ ] `copy_dir()` recursively copies tree
- [ ] `create_dir_all()` is async
- [ ] `exists()` is async and returns bool
- [ ] `remove_file()` returns proper errors
- [ ] `normalize()` collapses dots and separators
- [ ] `canonicalize()` resolves symlinks
- [ ] `resolve_path()` handles relative paths
- [ ] All operations are `Send + Sync` safe
- [ ] Unit tests pass for all methods

---

*Document generated: 2026-04-21*
*Based on: PRD (packages/opencode/src/file/) + Gap Analysis (iteration-44/gap-analysis.md)*
