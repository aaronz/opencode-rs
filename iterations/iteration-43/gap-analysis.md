# Gap Analysis Report: file Module (Iteration-43)

**Date**: 2026-04-21
**PRD Reference**: `packages/opencode/src/file/index.ts` (No existing Rust equivalent — implement in `crates/file/`)
**Analysis Summary**: The `file` module as specified in the PRD **does not exist**. The codebase has fragmented file utilities spread across multiple locations but lacks the unified `FileService` with the specified API.

---

## 1. Gap Summary

| Gap | Severity | Module | Status |
|-----|----------|--------|--------|
| `FileService` struct not implemented | **P0** | `crates/file/` | Missing |
| `Debouncer` struct not implemented | **P0** | `crates/file/` | Missing |
| `FileError` enum not implemented | **P0** | `crates/file/` | Missing |
| `watch()` method not implemented as specified | **P0** | `crates/file/` | Partial |
| `unwatch()` method not implemented | **P0** | `crates/file/` | Missing |
| `copy_file()` method not implemented | **P1** | `crates/file/` | Missing |
| `copy_dir()` method not implemented | **P1** | `crates/file/` | Missing |
| Path normalization variants not implemented | **P1** | `crates/file/` | Partial |
| `crates/file/` crate not created | **P0** | Workspace | Missing |
| Tests for FileService not implemented | **P1** | `crates/file/` | Missing |

---

## 2. Detailed Gap Analysis

### 2.1 Module Organization

**PRD Requirement**:
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

**Current State**:
- `crates/file/` does **NOT exist**
- Fragmented implementations exist in:
  - `crates/core/src/watcher.rs` - FileWatcher (sync, no callback support)
  - `crates/core/src/filesystem.rs` - AppFileSystem (sync utilities)
  - `crates/util/src/fs.rs` - Async utilities (read, write, atomic_write, ensure_dir)
  - `crates/tools/src/file_tools.rs` - Tool implementations (FileReadTool, FileWriteTool, etc.)

**Gap**: The entire `crates/file/` module structure is missing. All functionality needs to be implemented as a cohesive unit.

---

### 2.2 FileService Implementation

**PRD Requirement**:
```rust
pub struct FileService {
    watch_handles: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>,
}

impl FileService {
    pub async fn watch(
        &self,
        path: &Path,
        debounce_ms: u64,
        callback: impl Fn(PathBuf) + Send + 'static,
    ) -> Result<String, FileError>

    pub async fn unwatch(&self, watch_id: &str) -> Result<(), FileError>

    pub async fn copy_file(&self, from: &Path, to: &Path) -> Result<u64, FileError>

    pub async fn copy_dir(&self, from: &Path, to: &Path) -> Result<u64, FileError>

    pub async fn create_dir_all(&self, path: &Path) -> Result<(), FileError>

    pub async fn exists(&self, path: &Path) -> bool

    pub async fn remove_file(&self, path: &Path) -> Result<(), FileError>

    pub fn normalize_path(&self, path: &Path) -> PathBuf
}
```

**Current State**:
| Method | Status | Location |
|--------|--------|----------|
| `watch()` | Partial | `FileWatcher::start()` in `watcher.rs` but no async callback mechanism |
| `unwatch()` | Missing | None |
| `copy_file()` | Missing | None |
| `copy_dir()` | Missing | None |
| `create_dir_all()` | Partial | `AppFileSystem::ensure_dir()` (sync), `util/fs::ensure_dir()` (async) |
| `exists()` | Partial | `AppFileSystem::exists()` (sync only) |
| `remove_file()` | Partial | `FileDeleteTool` in `file_tools.rs` (as tool, not service) |
| `normalize_path()` | Partial | `AppFileSystem::normalize_path()` (trivial implementation) |

---

### 2.3 Path Normalization Variants

**PRD Requirement**:
```rust
pub async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FileError>
pub fn resolve_path(&self, base: &Path, relative: &Path) -> PathBuf
pub fn normalize(&self, path: &Path) -> PathBuf
```

**Current State**:
| Method | Status | Issue |
|--------|--------|-------|
| `canonicalize()` | **Missing** | None |
| `resolve_path()` | **Missing** | None |
| `normalize()` | Partial | `AppFileSystem::normalize_path()` just returns `PathBuf::from(p)` without actual normalization |

---

### 2.4 Debouncer

**PRD Requirement**:
```rust
pub struct Debouncer {
    delay: Duration,
    pending: Arc<Mutex<HashMap<PathBuf, tokio::time::Sleep>>>,
}

impl Debouncer {
    pub fn queue(&self, path: PathBuf, callback: impl FnOnce());
}
```

**Current State**: **NOT IMPLEMENTED**

The existing `FileWatcher` in `watcher.rs` does not have any debouncing mechanism. It uses `notify::RecommendedWatcher` directly without debouncing rapid events.

---

### 2.5 FileError Enum

**PRD Requirement**:
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

**Current State**: **NOT IMPLEMENTED** as separate enum

The codebase uses `OpenCodeError` in `crates/core/src/error.rs` which has:
- `Io(std::io::Error)` - partially covers IO errors
- `Config(String)` - for configuration errors

But it lacks the specific `FileError` variants with structured context as specified.

---

### 2.6 Dependencies

**PRD Requirement** (`Cargo.toml`):
```toml
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

**Current State**:
- `notify = "6"` ✓ (already in `crates/core/Cargo.toml`)
- `tokio` with `fs`, `sync`, `rt`, `time` ✓ (in workspace)
- `walkdir` ✓ (in workspace)
- `thiserror` ✓ (in workspace)
- `serde`, `serde_json` ✓ (in workspace)
- `tracing` ✓ (in workspace)
- `filedescriptor` ✗ (NOT present)
- `tempfile` ✓ (in workspace)
- `tokio-test` ✓ (in workspace)

---

## 3. Priority Classification

### P0 - Blocking Issues (Must Fix)

| Issue | Description | Impact |
|-------|-------------|--------|
| Missing `crates/file/` crate | Entire module not created | Cannot implement file service pattern |
| `FileService` struct not implemented | Core API missing | All file operations cannot use unified interface |
| `watch()`/`unwatch()` not implemented | File watching API incomplete | Cannot watch files with callbacks |
| `Debouncer` not implemented | No event debouncing | Rapid file changes will flood callbacks |
| `FileError` not implemented | Error handling incomplete | Cannot properly handle file-specific errors |

### P1 - Important Issues

| Issue | Description | Impact |
|-------|-------------|--------|
| `copy_file()` not implemented | Cannot copy files with parent dir creation | Missing core functionality |
| `copy_dir()` not implemented | Cannot copy directory trees | Missing core functionality |
| Path normalization variants missing | `canonicalize()`, `resolve_path()` not implemented | Path handling incomplete |
| Tests not implemented | No unit tests for FileService | Quality risk |

### P2 - Nice to Have

| Issue | Description | Impact |
|-------|-------------|--------|
| `filedescriptor` crate not used | Could optimize copy operations | Minor performance improvement |
| `normalize_path()` is trivial | Current implementation just wraps path | Could be more robust |

---

## 4. Technical Debt

| Item | Description | Recommendation |
|------|-------------|---------------|
| Fragmented file utilities | File utilities scattered across `core/filesystem.rs`, `util/fs.rs`, `tools/file_tools.rs` | Consolidate into `crates/file/` |
| Sync `AppFileSystem` | `AppFileSystem` is sync-only, PRD requires async | Rewrite as async `FileService` |
| No debouncing in FileWatcher | Current `FileWatcher` doesn't debounce | Implement `Debouncer` |
| Generic Io errors | Using `OpenCodeError::Io` instead of `FileError` | Create dedicated `FileError` enum |
| No integration tests | Only basic unit tests exist | Add comprehensive tests |

---

## 5. Implementation Progress

| Feature | Status | Notes |
|---------|--------|-------|
| `crates/file/` crate | **0%** | Not created |
| `FileService` struct | **0%** | Not implemented |
| `watch()` method | **10%** | `FileWatcher::start()` exists but no callback mechanism |
| `unwatch()` method | **0%** | Not implemented |
| `copy_file()` method | **0%** | Not implemented |
| `copy_dir()` method | **0%** | Not implemented |
| `create_dir_all()` method | **60%** | Exists in `util/fs` but not in FileService |
| `exists()` method | **50%** | Sync version exists, async version needed |
| `remove_file()` method | **50%** | Exists as `FileDeleteTool`, needs integration |
| `normalize_path()` method | **20%** | Trivial implementation only |
| `canonicalize()` method | **0%** | Not implemented |
| `resolve_path()` method | **0%** | Not implemented |
| `normalize()` method | **20%** | Trivial implementation only |
| `Debouncer` struct | **0%** | Not implemented |
| `FileError` enum | **0%** | Not implemented |
| Unit tests | **10%** | Only existing FileWatcher tests |

---

## 6. Crate Layout Analysis

### Current Fragmented State
```
crates/
├── core/src/
│   ├── filesystem.rs    # AppFileSystem (sync, limited)
│   └── watcher.rs      # FileWatcher (no callbacks/debouncing)
├── util/src/
│   └── fs.rs           # Async utilities (read, write, atomic_write)
└── tools/src/
    └── file_tools.rs   # Tool implementations (FileReadTool, etc.)
```

### Required Structure (per PRD)
```
crates/file/
├── Cargo.toml
├── src/
│   ├── lib.rs          # FileService, FileError
│   ├── watch.rs        # FileWatcher, Debouncer
│   ├── copy.rs         # copy_file, copy_dir
│   └── normalize.rs   # canonicalize, resolve_path, normalize
└── tests/
    └── file_tests.rs
```

---

## 7. Recommendations

### Immediate Actions (P0)
1. Create `crates/file/` crate with proper dependencies
2. Implement `FileError` enum matching PRD specification
3. Implement `FileService` with all methods
4. Implement `Debouncer` for event debouncing
5. Implement `watch()`/`unwatch()` with proper callback mechanism

### Short-term Actions (P1)
1. Implement `copy_file()` and `copy_dir()`
2. Implement path normalization variants
3. Add comprehensive unit tests
4. Consolidate fragmented utilities from `core/filesystem.rs` and `util/fs.rs`

### Long-term Actions (P2)
1. Add `filedescriptor` for optimized copy operations
2. Add integration tests
3. Consider adding file metadata caching

---

## 8. Verification Checklist

Based on PRD Acceptance Criteria:

- [ ] `watch()` registers a filesystem watcher and returns a `WatchId` - **NOT IMPLEMENTED**
- [ ] `unwatch()` stops the watcher and removes it from the registry - **NOT IMPLEMENTED**
- [ ] Multiple rapid changes are debounced into single callbacks - **NOT IMPLEMENTED**
- [ ] `copy_file()` creates parent directories and copies with correct byte count - **NOT IMPLEMENTED**
- [ ] `copy_dir()` recursively copies an entire directory tree - **NOT IMPLEMENTED**
- [ ] `normalize_path()` produces consistent absolute paths on all platforms - **PARTIAL**
- [ ] `exists()` returns `true`/`false` without throwing errors - **PARTIAL** (sync only)
- [ ] `remove_file()` returns `NotFound` error for non-existent paths - **PARTIAL** (via tool, not service)
- [ ] All operations are `Send + Sync` safe across tokio tasks - **NOT VERIFIED**

---

## Conclusion

The `file` module as specified in the PRD **does not exist** in the current codebase. All specified functionality needs to be implemented from scratch in a new `crates/file/` crate. The existing file utilities are fragmented and do not provide the unified `FileService` API with async operations, debouncing, and proper error handling specified in the PRD.

**Estimated completion**: Full implementation requires creating the crate structure, implementing all methods, and adding comprehensive tests.
