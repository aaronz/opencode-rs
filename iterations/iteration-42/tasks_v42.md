# Storage Module Task List (v42)

## P0 Tasks (Critical Blockers)

### [ ] FR-055: CompactionManager Instance-Based Design
**Priority**: P0
**Module**: `crates/storage/src/compaction.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `CompactionManager::new(config: CompactionConfig)` constructor exists
- [ ] `config` field stored in struct instance
- [ ] `should_auto_compact(&self, session: &Session)` uses stored config
- [ ] `compact(&self, session: &Session)` uses stored config
- [ ] `StorageService` stores `Option<CompactionManager>` instead of static methods

**Files to Modify**:
- `crates/storage/src/compaction.rs` - Restructure struct and methods
- `crates/storage/src/service.rs` - Update to use instance
- `crates/storage/src/lib.rs` - Update exports if needed

---

## P1 Tasks (High Priority)

### [x] FR-046: Add compact_session to StorageService
**Priority**: P1
**Module**: `crates/storage/src/service.rs`
**Dependencies**: FR-055

**Acceptance Criteria**:
- [x] `compact_session(id: &str) -> Result<CompactionResult, StorageError>` method exists
- [x] Method delegates to `CompactionManager::compact()`
- [x] Returns `NotFound` error for non-existent sessions

---

### [x] FR-047: Add load_project(id) Method
**Priority**: P1
**Module**: `crates/storage/src/service.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [x] `load_project(id: &str) -> Result<Option<ProjectModel>, StorageError>` method exists
- [x] Returns `Some(ProjectModel)` if found
- [x] Returns `None` if not found
- [x] Works with UUID string format

**Tests Added**:
- `test_load_project_returns_some_for_existing_project`
- `test_load_project_returns_none_for_nonexistent_project`
- `test_load_project_handles_uuid_format_correctly`
- `test_load_project_error_handling_for_malformed_uuid`

---

### [x] FR-051: Add exists() to SessionRepository
**Priority**: P1
**Module**: `crates/storage/src/repository.rs`, `sqlite_repository.rs`, `memory_repository.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `async fn exists(&self, id: &str) -> Result<bool, StorageError>` in trait
- [ ] `SqliteSessionRepository::exists()` queries database
- [ ] `InMemorySessionRepository::exists()` checks HashMap
- [ ] Returns `true` if session exists, `false` otherwise

---

### [ ] FR-049: Add recover_session to StorageService
**Priority**: P1
**Module**: `crates/storage/src/service.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `recover_session(id: &str) -> Result<Session, StorageError>` method exists
- [ ] Method delegates to `CrashRecovery::restore()`
- [ ] Returns `NotFound` error for non-existent sessions

---

### [ ] FR-045/FR-048: Pagination Verification
**Priority**: P1
**Module**: `crates/storage/src/service.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `list_sessions(limit, offset)` works correctly
- [ ] `list_projects(limit, offset)` works correctly
- [ ] Empty results return empty vector, not error

---

## P2 Tasks (Medium Priority)

### [ ] FR-042: Add Migration Error Variant
**Priority**: P2
**Module**: `crates/storage/src/error.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `StorageError::Migration(String)` variant exists
- [ ] Error message correctly formats migration errors

---

### [ ] FR-043: Add SessionLocked Error Variant
**Priority**: P2
**Module**: `crates/storage/src/error.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `StorageError::SessionLocked(String)` variant exists
- [ ] Error message correctly formats lock errors

---

### [ ] FR-044: Add Recovery Error Variant
**Priority**: P2
**Module**: `crates/storage/src/error.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `StorageError::Recovery(String)` variant exists
- [ ] Error message correctly formats recovery errors

---

### [ ] FR-050: Add list_incomplete_sessions
**Priority**: P2
**Module**: `crates/storage/src/service.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `list_incomplete_sessions() -> Result<Vec<Uuid>, StorageError>` method exists
- [ ] Returns Vec of incomplete session IDs
- [ ] Empty list returned when no incomplete sessions exist

---

### [ ] FR-052: Implement list_by_project
**Priority**: P2
**Module**: `crates/storage/src/repository.rs`, `sqlite_repository.rs`, `memory_repository.rs`
**Dependencies**: None

**Acceptance Criteria**:
- [ ] `list_by_project(project_path: &str)` filters sessions correctly
- [ ] Only returns sessions associated with given project
- [ ] Returns empty vec for projects with no sessions (not error)

---

## Task Completion Checklist

- [ ] All P0 tasks completed
- [ ] All P1 tasks completed
- [ ] All P2 tasks completed
- [ ] `cargo build -p opencode-storage` passes
- [ ] `cargo test -p opencode-storage` passes
- [ ] `cargo clippy -p opencode-storage` passes with no warnings
