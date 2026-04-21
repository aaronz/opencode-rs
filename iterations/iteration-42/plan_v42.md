# Storage Module Implementation Plan (v42)

## Overview

This plan addresses the gaps identified in the storage module (`opencode-storage`) relative to the PRD specification. The implementation is 95% complete; this plan focuses on the remaining 5%.

## Gap Summary

| Priority | Count | Key Items |
|----------|-------|-----------|
| P0 | 1 | CompactionManager constructor mismatch |
| P1 | 5 | compact_session, load_project(id), exists(), recover_session, pagination |
| P2 | 6 | error variants, list_incomplete_sessions, list_by_project stub |

---

## P0 Tasks (Critical Blockers)

### Task 1: FR-055 - CompactionManager Instance-Based Design

**Status**: P0 BLOCKER
**Module**: `crates/storage/src/compaction.rs`

**Problem**: PRD specifies `CompactionManager::new(config: CompactionConfig)` but implementation uses static methods with no instance configuration.

**Required Changes**:
1. Restructure `CompactionManager` struct to hold `config: CompactionConfig` field
2. Add `CompactionManager::new(config: CompactionConfig) -> Self` constructor
3. Convert static methods `should_auto_compact()` and `compact()` to instance methods
4. Update all call sites to use instance-based API

**Implementation Order**:
1. Update `CompactionManager` struct definition
2. Update constructor
3. Update `should_auto_compact(&self, session: &Session) -> bool`
4. Update `compact(&self, session: &Session)` to use stored config
5. Update `StorageService` to store `CompactionManager` instance
6. Update tests to use new instance-based API

---

## P1 Tasks (High Priority)

### Task 2: FR-046 - Add compact_session to StorageService

**Module**: `crates/storage/src/service.rs`

**Changes**:
```rust
pub async fn compact_session(&self, id: &str) -> Result<CompactionResult, StorageError>
```

**Implementation**:
1. Load session by ID
2. Delegate to `CompactionManager::compact()`
3. Save compacted session
4. Return `CompactionResult`

---

### Task 3: FR-047 - Add load_project(id) Method

**Module**: `crates/storage/src/service.rs`

**Changes**:
```rust
pub async fn load_project(&self, id: &str) -> Result<Option<ProjectModel>, StorageError>
```

**Implementation**:
1. Parse UUID string to `Uuid`
2. Delegate to repository's `load` method
3. Return `Option<ProjectModel>`

---

### Task 4: FR-051 - Add exists() to SessionRepository

**Module**: `crates/storage/src/repository.rs`, `sqlite_repository.rs`, `memory_repository.rs`

**Changes**:
```rust
// In trait
async fn exists(&self, id: &str) -> Result<bool, StorageError>;

// In SqliteSessionRepository
SELECT COUNT(*) FROM sessions WHERE id = ? → returns bool

// In InMemorySessionRepository
Check HashMap contains key
```

---

### Task 5: FR-049 - Add recover_session to StorageService

**Module**: `crates/storage/src/service.rs`

**Changes**:
```rust
pub async fn recover_session(&self, id: &str) -> Result<Session, StorageError>
```

**Implementation**:
1. Delegate to `CrashRecovery::restore()` (from core)
2. Return recovered `Session`

---

### Task 6: FR-045/FR-048 - Pagination Verification

**Module**: `crates/storage/src/service.rs`

**Status**: Already implemented per spec, needs verification

**Verification Steps**:
1. Confirm `list_sessions(limit, offset)` works correctly
2. Confirm `list_projects(limit, offset)` works correctly
3. Verify empty results return empty vector, not error

---

## P2 Tasks (Medium Priority)

### Task 7: FR-042/FR-043/FR-044 - Error Variants

**Module**: `crates/storage/src/error.rs`

**Changes**:
```rust
#[error("Migration error: {0}")]
Migration(String),

#[error("Session locked: {0}")]
SessionLocked(String),

#[error("Recovery error: {0}")]
Recovery(String),
```

---

### Task 8: FR-050 - List Incomplete Sessions

**Module**: `crates/storage/src/service.rs`

**Changes**:
```rust
pub async fn list_incomplete_sessions(&self) -> Result<Vec<Uuid>, StorageError>
```

---

### Task 9: FR-052 - Implement list_by_project

**Module**: `crates/storage/src/repository.rs`, `sqlite_repository.rs`, `memory_repository.rs`

**Problem**: Currently returns empty vec by default

**Implementation**:
1. Filter sessions by project path association
2. Return matching `SessionSummary` list

---

## Implementation Order

1. **FR-055** (P0) - CompactionManager restructuring
2. **FR-046** (P1) - compact_session
3. **FR-047** (P1) - load_project(id)
4. **FR-051** (P1) - exists() method
5. **FR-049** (P1) - recover_session
6. **FR-045/FR-048** (P1) - Pagination verification
7. **FR-042/043/044** (P2) - Error variants
8. **FR-050** (P2) - list_incomplete_sessions
9. **FR-052** (P2) - list_by_project implementation

---

## Dependencies

- FR-055 must be completed before FR-046 (compact_session depends on instance-based CompactionManager)
- All other tasks are independent

---

## Verification

After each task:
1. Run `cargo build -p opencode-storage`
2. Run `cargo test -p opencode-storage`
3. Verify with `cargo clippy -p opencode-storage`
