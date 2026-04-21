# Task清单: file Module (Iteration 44)

**Version**: 44
**Date**: 2026-04-21
**Status**: Updated from Spec + Gap Analysis
**Total Tasks**: 24
**P0 Tasks**: 10
**P1 Tasks**: 9
**P2 Tasks**: 5

---

## P0 Tasks (Must Complete First)

### Phase 1: Core Structure

- [ ] **T-001**: Create `crates/file/Cargo.toml` with all dependencies
  - notify = "6"
  - tokio with fs, sync, rt, time features
  - walkdir = "2"
  - filedescriptor = "0.6"
  - thiserror = "2.0"
  - tracing
  - anyhow
  - tempfile (dev)
  - tokio-test (dev)

- [ ] **T-002**: Create `crates/file/src/lib.rs` with module declarations and public exports

- [ ] **T-003**: Implement `FileError` enum with 7 variants
  - NotFound(PathBuf)
  - NotAFile(PathBuf)
  - NotADirectory(PathBuf)
  - Io { context: String, source: std::io::Error }
  - Watch(String)
  - WatchNotFound(String)
  - PathTooLong(PathBuf)

- [x] **T-004**: Implement `FileService` struct with watcher registry
  - `watch_handles: Arc<Mutex<HashMap<String, notify::RecommendedWatcher>>>`

### Phase 2: Path Normalization

- [ ] **T-005**: Implement `normalize()` function - collapse `.` components, handle `..`, fix separators
  - No filesystem access
  - Example: `/a/b/../c/./d` → `/a/c/d`

- [x] **T-006**: Implement `resolve_path(base, relative)` function
  - Resolve relative against base
  - Make absolute
  - Does NOT resolve symlinks

- [ ] **T-007**: Implement `canonicalize()` async function
  - Resolve symlinks
  - Make absolute
  - Requires path to exist
  - Return FileError::Io on failure

- [x] **T-008**: Implement `normalize_path()` public API ✅ Done
  - Combine normalization strategies
  - Platform-aware
  - Produce consistent absolute paths

### Phase 5: File Watching

- [ ] **T-009**: Implement `Debouncer` struct
  - `delay: Duration`
  - `pending: Arc<Mutex<HashMap<PathBuf, tokio::time::Sleep>>>`
  - `queue()` method to collapse rapid events
  - Thread-safe via Arc<Mutex<...>>

- [ ] **T-010**: Implement `watch()` method
  - Use `notify::RecommendedWatcher`
  - Generate unique watch_id string
  - Store watcher in registry
  - Support debouncing
  - Return WatchId string

- [ ] **T-011**: Implement `unwatch()` method
  - Stop watcher by watch_id
  - Remove from registry
  - Return FileError::WatchNotFound if not found

---

## P1 Tasks (High Priority)

### Phase 3: Async File Operations

- [ ] **T-012**: Implement `exists()` async function
  - Fast existence check
  - Return true if path exists, false otherwise
  - No error on missing path

- [ ] **T-013**: Implement `create_dir_all()` async function
  - Recursive directory creation (like `mkdir -p`)
  - Use `tokio::fs::create_dir_all`
  - Return success or FileError::Io

- [ ] **T-014**: Implement `remove_file()` async function
  - Delete a file (not directory)
  - Return FileError::NotFound if path does not exist
  - Return FileError::NotAFile if path is a directory

### Phase 4: Copy Operations

- [ ] **T-015**: Implement `copy_file()` async function
  - Check source exists, return FileError::NotFound if missing
  - Create parent directories via `tokio::fs::create_dir_all`
  - Copy file via `tokio::fs::copy`
  - Return bytes copied
  - Map errors to FileError::Io

- [ ] **T-016**: Implement `copy_dir()` async function
  - Use `walkdir::WalkDir` for recursive traversal
  - Preserve directory structure
  - Copy all files respecting permissions
  - Return total bytes copied
  - Return FileError::NotADirectory if source is not a directory

### Phase 6: Testing

- [ ] **T-017**: Write unit test `test_watch_fires_callback_on_file_change`
  - Verify watch() registers watcher and callback fires on file change

- [ ] **T-018**: Write unit test `test_debouncer_merges_rapid_events`
  - Verify multiple rapid events collapse to single callback

- [ ] **T-019**: Write unit test `test_copy_file_creates_parent_dirs`
  - Verify copy_file() creates parent directories and returns byte count

- [ ] **T-020**: Write unit test `test_copy_dir_recursive`
  - Verify copy_dir() recursively copies directory tree

- [ ] **T-021**: Write unit test `test_normalize_collapse_dots`
  - Verify normalize() collapses dot components

---

## P2 Tasks (Medium Priority)

### Phase 6: Testing (continued)

- [ ] **T-022**: Write unit test `test_canonicalize_resolves_symlinks`
  - Verify canonicalize() resolves symlinks and makes absolute

- [ ] **T-023**: Write unit test `test_resolve_path_relative`
  - Verify resolve_path() handles relative paths correctly

- [ ] **T-024**: Write unit test `test_create_dir_all_async`
  - Verify create_dir_all() creates nested directories

- [ ] **T-025**: Write unit test `test_exists_returns_bool`
  - Verify exists() returns bool without throwing errors

- [ ] **T-026**: Write unit test `test_remove_file_not_found`
  - Verify remove_file() returns NotFound error for non-existent paths

---

## Task Dependencies

```
T-001 → T-002 → T-003 → T-004 → T-005 → T-006 → T-007 → T-008
                                           ↓
T-009 ← T-010 ← T-011 (watching depends on service)
          ↓
T-015 → T-016 (copy depends on basic structure)
    ↓
T-012 → T-013 → T-014 (async ops depend on service)

T-017 → T-018 → T-019 → T-020 → T-021 → T-022 → T-023 → T-024 → T-025 → T-026
```

---

## Priority Order

1. **P0**: T-001 through T-011 (Core structure, normalization, watching)
2. **P1**: T-012 through T-021 (Async ops, copy, core tests)
3. **P2**: T-022 through T-026 (Remaining tests)

---

## File Locations

| Task | File |
|------|------|
| T-001 | `crates/file/Cargo.toml` |
| T-002 | `crates/file/src/lib.rs` |
| T-003 | `crates/file/src/error.rs` |
| T-004 | `crates/file/src/service.rs` |
| T-005, T-006, T-007, T-008 | `crates/file/src/normalize.rs` |
| T-009, T-010, T-011 | `crates/file/src/watch.rs` |
| T-012, T-013, T-014 | `crates/file/src/lib.rs` or `service.rs` |
| T-015, T-016 | `crates/file/src/copy.rs` |
| T-017 through T-026 | `crates/file/tests/file_tests.rs` |

---

*Task list generated: 2026-04-21*
*Based on: Spec (iteration-44/spec.md) + Gap Analysis (iteration-44/gap-analysis.md)*