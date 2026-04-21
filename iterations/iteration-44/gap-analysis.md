# Gap Analysis Report: file Module (Iteration 44)

**Date**: 2026-04-21
**PRD Source**: packages/opencode/src/file/ (converted to Rust `crates/file/`)
**Analysis Scope**: Core file operations, path normalization, file watching

---

## Executive Summary

The `crates/file/` crate **does not exist**. The PRD specifies a dedicated `crates/file/` module with `FileService`, but all file-related functionality is currently scattered across:
- `crates/core/src/filesystem.rs` - `AppFileSystem` (basic utilities)
- `crates/tools/src/file_tools.rs` - File tool implementations (`FileReadTool`, `FileWriteTool`, etc.)

**Critical Finding**: 0% implementation of required `FileService` architecture.

---

## Gap Analysis

### 1. Functional Completeness

| PRD Feature | Status | Current Implementation | Gap |
|-------------|--------|------------------------|-----|
| File Watching (notify crate) | ❌ Missing | None | No `watch()`/`unwatch()` with notify |
| File Copying | ⚠️ Partial | `FileWriteTool` creates dirs, no copy | No `copy_file()`/`copy_dir()` with byte count |
| Directory Creation | ✅ Exists | `AppFileSystem::ensure_dir()` | Sync only, not async |
| Path Normalization | ❌ Broken | `normalize_path()` is identity function | No symlink resolution, no absolute paths |
| File Existence Check | ✅ Exists | `AppFileSystem::exists()` | Sync only |
| File Deletion | ✅ Exists | `FileDeleteTool` | Not exposed as `FileService::remove_file()` |

**Summary**: 2/6 features properly implemented, 1/6 partial, 3/6 missing.

---

### 2. API Completeness

| Required API | Status | Notes |
|--------------|--------|-------|
| `FileService::watch()` | ❌ Missing | No watcher registry |
| `FileService::unwatch()` | ❌ Missing | No watch handle management |
| `FileService::copy_file()` | ❌ Missing | No async copy with byte count |
| `FileService::copy_dir()` | ❌ Missing | No recursive copy |
| `FileService::create_dir_all()` | ⚠️ Partial | Only `ensure_dir()` sync version |
| `FileService::exists()` | ✅ Exists | `AppFileSystem::exists()` |
| `FileService::remove_file()` | ⚠️ Partial | `FileDeleteTool` but not exposed as service method |
| `FileService::normalize_path()` | ❌ Broken | Identity function, no resolution |
| `FileService::canonicalize()` | ❌ Missing | No symlink + absolute resolution |
| `FileService::resolve_path()` | ❌ Missing | No relative-to-base resolution |
| `FileService::normalize()` | ❌ Missing | No path component normalization |

**Summary**: 2/11 API methods properly implemented.

---

### 3. Data Model Completeness

| Required Type | Status | Current Type | Gap |
|---------------|--------|--------------|-----|
| `FileError` enum | ❌ Missing | `OpenCodeError` generic | No typed errors (NotFound, NotAFile, NotADirectory, Io, Watch, WatchNotFound, PathTooLong) |
| `FileService` struct | ❌ Missing | N/A | No watcher registry with `Arc<Mutex<HashMap<...>>>` |
| `Debouncer` struct | ❌ Missing | N/A | No debouncing for rapid file changes |

---

### 4. Crate Structure

| Required Layout | Status | Gap |
|-----------------|--------|-----|
| `crates/file/Cargo.toml` | ❌ Missing | No notify, tokio, filedescriptor dependencies |
| `crates/file/src/lib.rs` | ❌ Missing | No FileService, FileError exports |
| `crates/file/src/watch.rs` | ❌ Missing | No FileWatcher, Debouncer implementations |
| `crates/file/src/copy.rs` | ❌ Missing | No async copy implementations |
| `crates/file/src/normalize.rs` | ❌ Missing | No path normalization utilities |
| `crates/file/tests/file_tests.rs` | ❌ Missing | No integration tests |

---

### 5. Dependencies

| Required Dependency | Status | Gap |
|---------------------|--------|-----|
| `notify = "6"` | ❌ Missing | Not in any Cargo.toml |
| `tokio` with fs, sync, rt, time | ⚠️ Partial | In workspace but not for file crate |
| `walkdir` | ✅ Exists | In `file_tools.rs` |
| `tracing` | ⚠️ Partial | In workspace, not specifically for file ops |
| `tempfile` (dev) | ✅ Exists | Used in tests |
| `filedescriptor` | ❌ Missing | For sendfile on Unix |

---

## P0 Blockers (Must Fix)

| Issue | Severity | Module | Fix |
|-------|----------|--------|-----|
| `crates/file/` crate does not exist | P0 | Architecture | Create crate with proper module structure |
| No `FileService` struct | P0 | Core | Implement service with watcher registry |
| File watching not implemented | P0 | watch | Use notify crate for filesystem events |
| Path normalization is identity function | P0 | normalize | Implement proper normalization (symlinks, absolute, dots) |

---

## P1 High Priority

| Issue | Severity | Module | Fix |
|-------|----------|--------|-----|
| No `copy_file()` with parent dir creation | P1 | copy | Async copy returning byte count |
| No `copy_dir()` recursive copy | P1 | copy | Implement directory tree copy |
| No `FileError` typed error enum | P1 | error | Create error types: NotFound, NotAFile, NotADirectory, Io, Watch, etc. |
| No `Debouncer` for event debouncing | P1 | watch | Batch rapid filesystem changes |

---

## P2 Medium Priority

| Issue | Severity | Module | Fix |
|-------|----------|--------|-----|
| `create_dir_all` is sync only | P2 | core | Add async version to FileService |
| `exists` is sync only | P2 | core | Add async version to FileService |
| `remove_file` not exposed as service method | P2 | tools | Add to FileService API |
| Missing canonicalize (symlink + absolute) | P2 | normalize | Implement with std::fs::canonicalize |
| Missing resolve_path | P2 | normalize | Relative to base resolution |

---

## Technical Debt

| Item | Description | Est. Effort |
|------|-------------|-------------|
| TD-001 | Create `crates/file/` from scratch | High |
| TD-002 | Migrate sync `AppFileSystem` utilities to async `FileService` | Medium |
| TD-003 | Add proper error types instead of string errors | Medium |
| TD-004 | Implement watch/unwatch with notify backend | High |
| TD-005 | Add debouncing for file change events | Medium |
| TD-006 | Implement path normalization variants (canonicalize, resolve, normalize) | Medium |
| TD-007 | Add async copy operations with parent directory creation | Medium |
| TD-008 | Write unit tests for all FileService methods | Medium |
| TD-009 | Add integration tests for watch debounce, copy with parents, normalize | Medium |

---

## Implementation Progress

```
┌─────────────────────────────────────────────────────────┐
│ Feature Completion                                       │
├─────────────────────────────────────────────────────────┤
│ File Watching          [ ]  0%  (notify integration)    │
│ File Copying          [~] 15%  (write exists, copy miss)│
│ Directory Creation    [✓] 50%  (sync only, needs async) │
│ Path Normalization    [ ]  0%  (identity fn, no resolve) │
│ File Existence Check  [✓] 80%  (sync only)              │
│ File Deletion         [✓] 70%  (tool exists, not svc)  │
├─────────────────────────────────────────────────────────┤
│ Overall Progress      [ ]  16%                           │
└─────────────────────────────────────────────────────────┘
```

---

## Required Changes

### 1. Create `crates/file/` crate structure

```
crates/file/
├── Cargo.toml       # notify = "6", tokio with fs/sync/rt/time, walkdir, etc.
├── src/
│   ├── lib.rs       # FileService, FileError exports
│   ├── watch.rs     # FileWatcher, Debouncer
│   ├── copy.rs      # copy_file, copy_dir
│   └── normalize.rs # canonicalize, resolve_path, normalize
└── tests/
    └── file_tests.rs
```

### 2. Dependencies to add (Cargo.toml)

```toml
notify = "6"
tokio = { version = "1.45", features = ["fs", "sync", "rt", "time"] }
walkdir = "2"
filedescriptor = "0.6"
thiserror = "2.0"
tracing = "0.1"
anyhow = "1.0"
tempfile = "3"  # dev
tokio-test = "0.4"  # dev
```

### 3. FileError enum to implement

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
    Io { context: String, source: std::io::Error },
    #[error("Watch error: {0}")]
    Watch(String),
    #[error("Watch not found: {0}")]
    WatchNotFound(String),
    #[error("Path too long: {0}")]
    PathTooLong(PathBuf),
}
```

---

## Recommendation

**Phase 1**: Create `crates/file/` crate with basic structure and FileError enum
**Phase 2**: Implement path normalization (canonicalize, resolve, normalize)
**Phase 3**: Implement async copy operations (copy_file, copy_dir)
**Phase 4**: Implement file watching with notify and debouncing
**Phase 5**: Add comprehensive tests

---

*Report generated: 2026-04-21*
*Analysis performed against: packages/opencode/src/file/ (PRD)*