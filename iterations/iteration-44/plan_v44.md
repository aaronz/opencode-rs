# Implementation Plan: file Module (Iteration 44)

**Version**: 44
**Date**: 2026-04-21
**Status**: Updated from Spec + Gap Analysis
**Priority**: P0 tasks must be completed first

---

## Executive Summary

The `crates/file/` crate **does not exist** and must be created from scratch. The gap analysis identified 4 P0 blockers and 4 P1 items that must be addressed before file operations can work correctly.

**Overall Progress**: 16% (gap analysis baseline)

---

## P0 Blockers (Must Fix First)

| # | Blocker | Module | Description |
|---|---------|--------|-------------|
| 1 | Create `crates/file/` crate structure | Architecture | Create crate with Cargo.toml and module layout |
| 2 | Implement `FileService` struct | Core | Service with watcher registry `Arc<Mutex<HashMap<...>>>` |
| 3 | Implement file watching | watch.rs | notify integration with debouncing |
| 4 | Fix path normalization | normalize.rs | `normalize_path()` is identity function (BUG) |

---

## Implementation Phases

### Phase 1: Core Structure (P0)

**Goal**: Create the `crates/file/` crate with basic infrastructure

**Tasks**:
1. Create `crates/file/Cargo.toml` with all dependencies
2. Create `crates/file/src/lib.rs` with module declarations
3. Implement `FileError` enum with all 7 variants
4. Implement `FileService` struct with watcher registry

**Dependencies**:
- `notify = "6"` - cross-platform file watching
- `tokio` with `fs`, `sync`, `rt`, `time` features
- `walkdir` - recursive directory traversal
- `filedescriptor` - sendfile on Unix
- `thiserror = "2.0"` - typed errors
- `tracing` - structured logging

**Deliverables**:
- `crates/file/Cargo.toml`
- `crates/file/src/lib.rs` (FileService, FileError exports)
- `crates/file/src/error.rs` (FileError enum)
- `crates/file/src/service.rs` (FileService struct)

---

### Phase 2: Path Normalization (P0)

**Goal**: Fix broken path normalization and implement all variants

**Tasks**:
1. Implement `normalize()` - collapse `.` components, handle `..`, fix separators
2. Implement `resolve_path()` - resolve relative path against base
3. Implement `canonicalize()` - resolve symlinks and make absolute
4. Implement `normalize_path()` - public API combining all strategies

**Note**: Current `normalize_path()` in `filesystem.rs:97-99` is an identity function (BUG)

**Deliverables**:
- `crates/file/src/normalize.rs`

---

### Phase 3: Async File Operations (P1)

**Goal**: Add async versions of file utility operations

**Tasks**:
1. Implement `exists()` async - fast existence check returning bool
2. Implement `create_dir_all()` async - recursive directory creation
3. Implement `remove_file()` async - file deletion with proper errors

**Deliverables**:
- Updates to `crates/file/src/lib.rs` (async methods)

---

### Phase 4: Copy Operations (P1)

**Goal**: Implement file and directory copy with proper error handling

**Tasks**:
1. Implement `copy_file()` with parent directory creation
   - Check source exists, return `FileError::NotFound` if missing
   - Create parent directories via `tokio::fs::create_dir_all`
   - Copy file via `tokio::fs::copy`
   - Return bytes copied
2. Implement `copy_dir()` recursive directory tree copy
   - Use `walkdir::WalkDir` for traversal
   - Preserve directory structure
   - Copy all files respecting permissions
   - Return total bytes copied

**Deliverables**:
- `crates/file/src/copy.rs`

---

### Phase 5: File Watching (P0)

**Goal**: Implement cross-platform file watching with debouncing

**Tasks**:
1. Implement `Debouncer` struct
   - Multiple rapid events on same path collapse into single callback
   - `delay` determines minimum time between callbacks
   - Thread-safe via `Arc<Mutex<...>>`
2. Implement `watch()` method
   - Register filesystem watcher using `notify::RecommendedWatcher`
   - Store watcher handle in `watch_handles` registry
   - Generate unique `watch_id` string
   - Return `WatchId` string on success
3. Implement `unwatch()` method
   - Stop the watcher identified by `watch_id`
   - Remove from registry
   - Return `FileError::WatchNotFound` if ID not found

**Platform backends**: FSEvents (macOS), inotify (Linux), ReadDirectoryChangesW (Windows)

**Deliverables**:
- `crates/file/src/watch.rs`

---

### Phase 6: Testing (P1)

**Goal**: Verify all operations work correctly with unit and integration tests

**Tasks**:
1. Write unit tests for all FileService methods
2. Add integration tests for watch debounce, copy with parents, normalize
3. Verify `Send + Sync` safety

**Deliverables**:
- `crates/file/tests/file_tests.rs`

---

## Technical Debt Items

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

## Crate Layout

```
crates/file/
├── Cargo.toml       # notify = "6", tokio with fs/sync/rt/time, walkdir, filedescriptor
├── src/
│   ├── lib.rs       # FileService, FileError, public API exports
│   ├── error.rs     # FileError enum
│   ├── service.rs   # FileService struct with watcher registry
│   ├── watch.rs     # FileWatcher, Debouncer implementations
│   ├── copy.rs      # copy_file, copy_dir implementations
│   └── normalize.rs # canonicalize, resolve_path, normalize
└── tests/
    └── file_tests.rs
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