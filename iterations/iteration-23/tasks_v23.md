# Task List - Iteration 23

**Project:** OpenCode Rust Monorepo
**Iteration:** 23
**Date:** 2026-04-15
**Phase:** Rust Conventions Compliance Implementation
**Status:** Draft

---

## P0 - Blocking Issues (Must Fix First)

### P0-001: unwrap()/expect() Elimination

**Priority:** P0 (Blocking)
**Estimated Effort:** 4 weeks
**FR Reference:** FR-001
**Status:** NOT STARTED

#### Subtasks

- [ ] **P0-001.1** Audit all `unwrap()`/`expect()` in `crates/core/src/skill.rs` (135 occurrences)
  - Categorize: ParseError, OptionUnwrap, LockError, IOError, InvalidState
  - Create error enum `SkillError` with thiserror
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.2** Audit all `unwrap()`/`expect()` in `crates/core/src/session.rs` (86 occurrences)
  - Create error enum `SessionError` with thiserror
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.3** Audit all `unwrap()`/`expect()` in `crates/core/src/project.rs` (79 occurrences)
  - Create error enum `ProjectError` with thiserror
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.4** Audit remaining `unwrap()`/`expect()` in `crates/core/` (336 occurrences across 33 files)
  - Prioritize files with >20 occurrences
  - Create appropriate error types as needed

- [ ] **P0-001.5** Audit all `unwrap()`/`expect()` in `crates/tools/src/lsp_tool.rs` (74 occurrences)
  - Create error enum `ToolError` with thiserror
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.6** Audit all `unwrap()`/`expect()` in `crates/tools/src/registry.rs` (54 occurrences)
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.7** Audit remaining `unwrap()`/`expect()` in `crates/tools/` (164 occurrences across 12 files)
  - Prioritize files with >10 occurrences

- [ ] **P0-001.8** Audit all `unwrap()`/`expect()` in `crates/agent/src/runtime.rs` (71 occurrences)
  - Use anyhow for application crate
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.9** Audit remaining `unwrap()`/`expect()` in `crates/agent/` (3 occurrences)

- [ ] **P0-001.10** Audit all `unwrap()`/`expect()` in `crates/server/` (135 occurrences across 18 files)
  - Use anyhow for application crate
  - Replace all unwrap/expect with proper error handling

- [ ] **P0-001.11** Verify zero `unwrap()`/`expect()` with command:
  ```bash
  grep -rn "unwrap()\|expect(" crates/*/src/ | grep -v "#[cfg(test)]" | wc -l
  # Target: 0
  ```

#### Top Offender Files Detail

| File | Count | Priority | Error Type to Create |
|------|-------|----------|---------------------|
| `crates/core/src/skill.rs` | 135 | P0-001.1 | `SkillError` |
| `crates/core/src/session.rs` | 86 | P0-001.2 | `SessionError` |
| `crates/core/src/project.rs` | 79 | P0-001.3 | `ProjectError` |
| `crates/tools/src/lsp_tool.rs` | 74 | P0-001.5 | `ToolError` |
| `crates/agent/src/runtime.rs` | 71 | P0-001.8 | `anyhow::Error` |
| `crates/tools/src/registry.rs` | 54 | P0-001.6 | `ToolError` |
| `crates/server/` | 135 | P0-001.10 | `anyhow::Error` |

---

## P1 - High Priority Issues

### P1-001: Repository Pattern Implementation

**Priority:** P1 (High)
**Estimated Effort:** 2 weeks
**FR Reference:** FR-002
**Dependencies:** P0-001 (needs error types)
**Status:** NOT STARTED

#### Subtasks

- [ ] **P1-001.1** Define `SessionRepository` trait in `crates/storage/src/repository.rs`
  ```rust
  #[async_trait]
  pub trait SessionRepository: Send + Sync {
      async fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError>;
      async fn find_all(&self) -> Result<Vec<Session>, StorageError>;
      async fn save(&self, session: &Session) -> Result<(), StorageError>;
      async fn delete(&self, id: &str) -> Result<(), StorageError>;
      async fn list_by_project(&self, project_id: &str) -> Result<Vec<Session>, StorageError>;
  }
  ```

- [ ] **P1-001.2** Define `ProjectRepository` trait in `crates/storage/src/repository.rs`
  ```rust
  #[async_trait]
  pub trait ProjectRepository: Send + Sync {
      async fn find_by_id(&self, id: &str) -> Result<Option<Project>, StorageError>;
      async fn find_all(&self) -> Result<Vec<Project>, StorageError>;
      async fn save(&self, project: &Project) -> Result<(), StorageError>;
      async fn delete(&self, id: &str) -> Result<(), StorageError>;
  }
  ```

- [ ] **P1-001.3** Define `StorageError` enum in `crates/storage/src/error.rs`
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum StorageError {
      #[error("database error: {0}")]
      Database(#[from] sqlx::Error),
      #[error("not found: {0}")]
      NotFound(String),
      #[error("serialization error: {0}")]
      Serialization(String),
  }
  ```

- [ ] **P1-001.4** Implement `SqliteSessionRepository` in `crates/storage/src/sqlite_repository.rs`

- [ ] **P1-001.5** Implement `SqliteProjectRepository` in `crates/storage/src/sqlite_repository.rs`

- [ ] **P1-001.6** Implement `InMemorySessionRepository` for tests in `crates/storage/src/memory_repository.rs`

- [ ] **P1-001.7** Implement `InMemoryProjectRepository` for tests

- [ ] **P1-001.8** Refactor `StorageService` to use repository traits via dependency injection

- [ ] **P1-001.9** Verify with: `grep -r "pub trait.*Repository" crates/`

---

### P1-002: Visibility Audit and Control

**Priority:** P1 (High)
**Estimated Effort:** 3 weeks
**FR Reference:** FR-003
**Status:** NOT STARTED

#### Subtasks

- [ ] **P1-002.1** Audit `crates/core/src/lib.rs` re-exports
  - Identify items that should be `pub(crate)` instead of `pub`
  - Identify items that should not be exported at all
  - Target: reduce from 140+ re-exports

- [ ] **P1-002.2** Audit `crates/core/src/skill.rs` visibility (135 `pub fn`)
  - Change to `pub(crate)` where only internal crate usage
  - Keep `pub` only for public API

- [ ] **P1-002.3** Audit `crates/core/src/session.rs` visibility (86 `pub fn`)

- [ ] **P1-002.4** Audit `crates/core/src/project.rs` visibility (79 `pub fn`)

- [ ] **P1-002.5** Audit remaining core files visibility

- [ ] **P1-002.6** Audit `crates/tools/` visibility (~150 `pub fn`)

- [ ] **P1-002.7** Audit `crates/server/` visibility (~80 `pub fn`)

- [ ] **P1-002.8** Verify with: `cargo doc --document-private-items -p opencode-core 2>&1 | grep "warning: public item"`

- [ ] **P1-002.9** Target: reduce `pub fn` in core from 501 to <50

---

### P1-003: Service Layer Refinement

**Priority:** P1 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-009
**Dependencies:** P1-001 (Repository Pattern)
**Status:** NOT STARTED

#### Subtasks

- [ ] **P1-003.1** Refactor `StorageService` to use `SessionRepository` and `ProjectRepository` traits

- [ ] **P1-003.2** Review `MdnsService` for dependency injection needs

- [ ] **P1-003.3** Review `SAP Service` for dependency injection needs

- [ ] **P1-003.4** Review `Auth Service` for dependency injection needs

- [ ] **P1-003.5** Verify service layer compiles and tests pass

---

### P1-004: Naming Conventions Audit

**Priority:** P1 (High)
**Estimated Effort:** 1 week
**FR Reference:** FR-012
**Status:** NOT STARTED

#### Subtasks

- [ ] **P1-004.1** Run naming check: `cargo clippy --all -- -D warnings 2>&1 | grep -i "naming\|snake\|pascal"`

- [ ] **P1-004.2** Fix functions with capital first letter (e.g., `GetSession` → `get_session`)

- [ ] **P1-004.3** Fix variables shadowing types (e.g., `let Session = ...` → `let session = ...`)

- [ ] **P1-004.4** Fix constants not in SCREAMING_SNAKE_CASE

- [ ] **P1-004.5** Verify: `rg "fn [A-Z]" --type rust` returns empty

- [ ] **P1-004.6** Add naming lints to CI gate

---

### P1-005: Hooks Configuration

**Priority:** P1 (High)
**Estimated Effort:** 1 day
**FR Reference:** FR-013
**Status:** NOT STARTED

#### Subtasks

- [ ] **P1-005.1** Configure `cargo fmt` post-tool hook for `.rs` files

- [ ] **P1-005.2** Configure `cargo clippy` post-tool hook for `.rs` files

- [ ] **P1-005.3** Verify hooks active: `cargo fmt --all -- --check` passes

- [ ] **P1-005.4** Verify hooks active: `cargo clippy --all -- -D warnings` passes

---

## P2 - Medium Priority Issues

### P2-001: String vs &str Optimization

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-004
**Status:** NOT STARTED

#### Subtasks

- [ ] **P2-001.1** Run: `cargo clippy --all -- -W clippy::ptr_arg`

- [ ] **P2-001.2** Fix `String` parameters where `&str` would suffice

- [ ] **P2-001.3** Use `impl Into<String>` for constructors that need ownership

- [ ] **P2-001.4** Verify all clippy::ptr_arg warnings resolved

---

### P2-002: Newtype Wrappers for Type Safety

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-005
**Status:** NOT STARTED

#### Subtasks

- [ ] **P2-002.1** Create `SessionId` newtype in `crates/core/src/types.rs`
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash)]
  pub struct SessionId(String);
  ```

- [ ] **P2-002.2** Create `ProjectId` newtype in `crates/core/src/types.rs`

- [ ] **P2-002.3** Create `UserId` newtype in `crates/core/src/types.rs`

- [ ] **P2-002.4** Create `MessageId` newtype in `crates/core/src/types.rs`

- [ ] **P2-002.5** Create `ToolId` newtype in `crates/core/src/types.rs`

- [ ] **P2-002.6** Create `SkillId` newtype in `crates/core/src/types.rs`

- [ ] **P2-002.7** Replace `String` IDs with newtypes throughout codebase

- [ ] **P2-002.8** Verify with: `rg "struct.*Id\(" crates/` returns 6+ matches

---

### P2-003: Test Coverage Target

**Priority:** P2 (Medium)
**Estimated Effort:** 2 weeks
**FR Reference:** FR-006
**Status:** NOT STARTED

#### Subtasks

- [ ] **P2-003.1** Install cargo-llvm-cov: `cargo install cargo-llvm-cov`

- [ ] **P2-003.2** Measure baseline coverage: `cargo llvm-cov --fail-under-lines 0`

- [ ] **P2-003.3** Identify low-coverage files in `crates/core/`

- [ ] **P2-003.4** Add tests for `crates/core/src/skill.rs` (target 80%+)

- [ ] **P2-003.5** Add tests for `crates/core/src/session.rs` (target 80%+)

- [ ] **P2-003.6** Add tests for `crates/core/src/project.rs` (target 80%+)

- [ ] **P2-003.7** Identify low-coverage files in `crates/tools/`

- [ ] **P2-003.8** Add tests for `crates/tools/src/lsp_tool.rs` (target 80%+)

- [ ] **P2-003.9** Add tests for `crates/tools/src/registry.rs` (target 80%+)

- [ ] **P2-003.10** Identify low-coverage files in `crates/agent/`

- [ ] **P2-003.11** Add tests for `crates/agent/src/runtime.rs` (target 80%+)

- [ ] **P2-003.12** Set CI gate: `cargo llvm-cov --fail-under-lines 80`

---

### P2-004: Unsafe Code Safety Documentation

**Priority:** P2 (Medium)
**Estimated Effort:** 1 day
**FR Reference:** FR-007
**Status:** NOT STARTED

#### Subtasks

- [ ] **P2-004.1** Audit `crates/tui/src/app.rs` (3 unsafe blocks)
  - Add `// SAFETY:` comment to each

- [ ] **P2-004.2** Audit `crates/server/src/routes/validation.rs` (2 unsafe blocks)
  - Add `// SAFETY:` comment to each

- [ ] **P2-004.3** Audit `crates/plugin/src/lib.rs` (1 unsafe block)
  - Add `// SAFETY:` comment

- [ ] **P2-004.4** Verify: `grep -rn "unsafe" crates/ | grep -v "SAFETY"` returns empty

---

### P2-005: Builder Pattern Audit

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-008
**Status:** NOT STARTED

#### Subtasks

- [ ] **P2-005.1** Audit existing builders: `rg "struct.*Builder" --type rust`

- [ ] **P2-005.2** Identify structs that need builder pattern (many optional params)

- [ ] **P2-005.3** Implement `ServerConfigBuilder` if not exists

- [ ] **P2-005.4** Implement `ClientConfigBuilder` if not exists

- [ ] **P2-005.5** Verify builders follow required pattern

---

### P2-006: Sealed Traits for Extensibility Control

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-010
**Status:** NOT STARTED

#### Subtasks

- [ ] **P2-006.1** Audit public traits: `grep -rn "pub trait" crates/*/src/ | grep -v "Sealed"`

- [ ] **P2-006.2** Identify traits that should be sealed

- [ ] **P2-006.3** Seal `Tool` trait if found public

- [ ] **P2-006.4** Seal `Agent` trait if found public

- [ ] **P2-006.5** Seal `StorageBackend` trait if found public

- [ ] **P2-006.6** Implement required pattern:
  ```rust
  mod private {
      pub trait Sealed {}
  }
  pub trait Format: private::Sealed { ... }
  ```

- [ ] **P2-006.7** Verify: `grep -rn "pub trait" crates/*/src/` shows sealed markers

---

## Verification Tasks

### Must Pass (Release Blocker)

- [ ] **VERIFY-001** `grep -r "unwrap()\|expect(" crates/*/src/*.rs` returns 0
- [ ] **VERIFY-002** `cargo clippy --all -- -D warnings` exits 0
- [ ] **VERIFY-003** `cargo fmt --all` passes
- [ ] **VERIFY-004** `cargo test` all green
- [ ] **VERIFY-005** `grep "pub trait.*Repository" crates/` shows traits defined
- [ ] **VERIFY-006** `rg 'format!.*SELECT' --type rust` returns empty (SQL injection check)
- [ ] **VERIFY-007** `rg 'sk-[a-zA-Z0-9]{20,}' --type rust` returns empty (secrets check)

### Should Pass (Quality Gate)

- [ ] **VERIFY-008** `cargo llvm-cov --fail-under-lines 80` passes
- [ ] **VERIFY-009** `grep -rn "unsafe" crates/ | grep -v "SAFETY"` returns empty
- [ ] **VERIFY-010** `rg "^    pub fn" crates/core/src/` returns <50
- [ ] **VERIFY-011** `cargo clippy -- -D warnings 2>&1 | grep -i naming` returns empty

---

## Task Dependencies

```
P0-001 (unwrap elimination)
    │
    ├──► P1-001.3 (StorageError) - needs P0 complete
    │
    └──► P1-003.1 (StorageService refactor) - depends on P1-001

P1-001 (repository traits)
    │
    └──► P1-003 (service layer) - depends on repo traits

P1-002 (visibility) - independent
P1-004 (naming) - independent
P1-005 (hooks) - independent

P2-* - all independent, can run in parallel after P0
```

---

## Priority Order (Execution Sequence)

1. **P0-001.1** - skill.rs unwrap fix (135 occurrences - CRITICAL)
2. **P0-001.2** - session.rs unwrap fix (86 occurrences - CRITICAL)
3. **P0-001.3** - project.rs unwrap fix (79 occurrences - HIGH)
4. **P0-001.4** - remaining core unwrap fix
5. **P0-001.5** - lsp_tool.rs unwrap fix (74 occurrences - HIGH)
6. **P0-001.6** - registry.rs unwrap fix (54 occurrences - MEDIUM)
7. **P0-001.7** - remaining tools unwrap fix
8. **P0-001.8** - runtime.rs unwrap fix (71 occurrences - HIGH)
9. **P0-001.9** - remaining agent unwrap fix
10. **P0-001.10** - server unwrap fix (135 occurrences)
11. **P0-001.11** - verify zero unwrap

12. **P1-001.1-3** - define repository traits (after P0-001.3)
13. **P1-001.4-7** - implement repository implementations
14. **P1-001.8** - refactor StorageService

15. **P1-002.1-9** - visibility audit
16. **P1-003.1-5** - service layer refinement
17. **P1-004.1-6** - naming conventions
18. **P1-005.1-4** - hooks configuration

19. **P2-001** - String vs &str
20. **P2-002** - newtype wrappers
21. **P2-003** - test coverage
22. **P2-004** - unsafe SAFETY
23. **P2-005** - builder pattern
24. **P2-006** - sealed traits

25. **VERIFY-001 to VERIFY-011** - verification gates

---

## Summary

| Priority | Tasks | Estimated Effort |
|----------|-------|-----------------|
| P0 | 11 tasks | 4 weeks |
| P1 | 5 issues, 30+ subtasks | 3-4 weeks |
| P2 | 6 issues, 30+ subtasks | 2-3 weeks |
| Verification | 11 gates | 1 week |

**Total Estimated Duration:** 10-13 weeks

---

*Task list generated from spec_v23.md and gap-analysis.md*
*Iteration 23 - Rust Conventions Compliance Implementation*
