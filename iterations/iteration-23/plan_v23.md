# Implementation Plan - Iteration 23

**Project:** OpenCode Rust Monorepo
**Iteration:** 23
**Date:** 2026-04-15
**Phase:** Rust Conventions Compliance Implementation
**Status:** Draft

---

## Table of Contents

1. [Overview](#1-overview)
2. [P0 - Blocking Issues (Must Fix First)](#2-p0---blocking-issues-must-fix-first)
3. [P1 - High Priority Issues](#3-p1---high-priority-issues)
4. [P2 - Medium Priority Issues](#4-p2---medium-priority-issues)
5. [Implementation Phases](#5-implementation-phases)
6. [Verification Gates](#6-verification-gates)
7. [Risk Assessment](#7-risk-assessment)
8. [Dependencies](#8-dependencies)

---

## 1. Overview

### Goal

Achieve Rust Conventions Compliance per the PRD specification. Zero `unwrap()`/`expect()` in production code, proper error handling patterns, repository trait abstraction, visibility controls, and 80%+ test coverage.

### Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| `unwrap()`/`expect()` count | ~1,137+ | 0 |
| `pub fn` in core | 501 | <50 |
| Repository traits | 0 | Defined + implemented |
| Test coverage | Unknown | 80%+ |
| Newtype wrappers | 2 | 6+ |

### Rule Hierarchy

1. **Rust-specific rules** (`.opencode/rules/rust/`) take precedence over common rules
2. **Language idioms** override generic recommendations
3. **Zero tolerance** for warnings — `cargo clippy -- -D warnings` must pass
4. **Immutability by default** — prefer `let` over `let mut`

---

## 2. P0 - Blocking Issues (Must Fix First)

### P0-001: unwrap()/expect() Elimination

**Priority:** P0 (Blocking)
**Estimated Effort:** 4 weeks
**FR Reference:** FR-001

#### Current State

| Crate | unwrap()/expect() Count | Files Affected |
|-------|------------------------|----------------|
| `crates/core/` | 636 | 36 files |
| `crates/tools/` | 292 | 14 files |
| `crates/agent/` | 74 | 2 files |
| `crates/server/` | 135 | 18 files |
| **Total** | **~1,137+** | **70+ files** |

#### Top Offender Files (Priority Order)

1. `crates/core/src/skill.rs` — 135 (CRITICAL)
2. `crates/core/src/session.rs` — 86 (CRITICAL)
3. `crates/core/src/project.rs` — 79 (HIGH)
4. `crates/tools/src/lsp_tool.rs` — 74 (HIGH)
5. `crates/tools/src/registry.rs` — 54 (MEDIUM)
6. `crates/agent/src/runtime.rs` — 71 (HIGH)

#### Implementation Strategy

**Phase P0-1: Audit and Categorize (Week 1)**
```bash
# Generate categorized inventory
grep -rn "\.unwrap()" crates/core/src/ | sort -t: -k2 -n | head -100
grep -rn "\.expect(" crates/core/src/ | sort -t: -k2 -n | head -50
```

Categorize each occurrence:
- `ParseError` - parsing failures (should return `Result::Err`)
- `Optionunwrap` - missing values (should use `ok_or`/`ok_or_else`)
- `LockError` - synchronization issues (should propagate)
- `IOError` - file/network errors (should use `?` with `From` trait)
- `InvalidState` - programming errors (should `panic!` with context)

**Phase P0-2: Core Crate Refactor (Week 2)**

Error types to add in `crates/core/src/error.rs`:
```rust
// Existing: OpenCodeError with 1xxx-9xxx codes
// Add specific error variants for each module

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),  // 5001

    #[error("invalid session state: {0}")]
    InvalidState(String),  // 5002

    #[error("session serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),  // 5003
}

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("skill not found: {0}")]
    NotFound(String),  // 4001

    #[error("skill parse error: {0}")]
    Parse(String),  // 4002

    #[error("skill execution failed: {0}")]
    Execution(String),  // 4003
}
```

Refactor order:
1. `skill.rs` (135 occurrences) - split into error categories
2. `session.rs` (86 occurrences) - SessionError variants
3. `project.rs` (79 occurrences) - ProjectError variants
4. Other core files

**Phase P0-3: Tools Crate Refactor (Week 3)**

Add error types in respective crate error modules:
```rust
// crates/tools/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("tool not found: {0}")]
    NotFound(String),  // 4001

    #[error("tool registry error: {0}")]
    Registry(String),  // 4004

    #[error("tool execution failed: {0}")]
    Execution(String),  // 4003

    #[error("invalid tool arguments: {0}")]
    InvalidArgs(String),  // 7001
}
```

Refactor order:
1. `lsp_tool.rs` (74 occurrences)
2. `registry.rs` (54 occurrences)
3. Other tools files

**Phase P0-4: Server/Agent Crate Refactor (Week 4)**

For application crates (server, agent), use `anyhow` for flexibility:
```rust
// crates/server/src/error.rs
use anyhow::{Context, Result};

pub type Result<T> = std::result::Result<T, anyhow::Error>;

// Or use thiserror for more structured errors
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("route error: {0}")]
    Route(#[from] route_error::RouteError),

    #[error("validation error: {0}")]
    Validation(String),
}
```

#### Verification

```bash
# Must return 0
grep -rn "unwrap()\|expect(" crates/*/src/ | grep -v "#\[cfg(test)\]" | wc -l

# Must pass
cargo clippy --all -- -D warnings
```

---

## 3. P1 - High Priority Issues

### P1-001: Repository Pattern Implementation

**Priority:** P1 (High)
**Estimated Effort:** 2 weeks
**FR Reference:** FR-002
**Dependencies:** P0-001 (error handling first)

#### Current State

```bash
$ grep -r "pub trait.*Repository" crates/
# NO MATCHES FOUND - NOT COMPLIANT
```

#### Implementation

**Step 1: Define Repository Traits**

File: `crates/storage/src/repository.rs`

```rust
use async_trait::async_trait;
use crate::error::StorageError;
use crate::session::Session;
use crate::project::Project;

/// Repository trait for session data access
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError>;
    async fn find_all(&self) -> Result<Vec<Session>, StorageError>;
    async fn save(&self, session: &Session) -> Result<(), StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<Session>, StorageError>;
}

/// Repository trait for project data access
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Project>, StorageError>;
    async fn find_all(&self) -> Result<Vec<Project>, StorageError>;
    async fn save(&self, project: &Project) -> Result<(), StorageError>;
    async fn delete(&self, id: &str) -> Result<(), StorageError>;
}
```

**Step 2: Implement SqliteRepository**

File: `crates/storage/src/sqlite_repository.rs`

```rust
pub struct SqliteSessionRepository {
    pool: SqlitePool,
}

#[async_trait]
impl SessionRepository for SqliteSessionRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError> {
        let session = sqlx::query_as::<_, SessionRow>(
            "SELECT id, data, created_at, updated_at FROM sessions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Database)?;

        Ok(session.map(|row| row.into()))
    }
    // ... other methods
}
```

**Step 3: Create InMemoryRepository for tests**

File: `crates/storage/src/memory_repository.rs`

```rust
pub struct InMemorySessionRepository {
    sessions: RwLock<HashMap<String, Session>>,
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    // ... implementation using HashMap
}
```

**Step 4: Refactor StorageService**

```rust
pub struct StorageService {
    session_repo: Box<dyn SessionRepository>,
    project_repo: Box<dyn ProjectRepository>,
}

impl StorageService {
    pub fn new(session_repo: Box<dyn SessionRepository>, project_repo: Box<dyn ProjectRepository>) -> Self {
        Self { session_repo, project_repo }
    }

    pub async fn load_session(&self, id: &str) -> Result<Option<Session>, OpenCodeError> {
        self.session_repo.find_by_id(id).await.map_err(OpenCodeError::from)
    }
}
```

---

### P1-002: Visibility Audit and Control

**Priority:** P1 (High)
**Estimated Effort:** 3 weeks
**FR Reference:** FR-003

#### Current State

| Crate | `pub fn` Count | Target |
|-------|---------------|--------|
| `crates/core/` | 501 | <50 |
| `crates/tools/` | ~150 | <30 |
| `crates/server/` | ~80 | <20 |

#### Audit Checklist

Run to identify public items:
```bash
# List all public functions in core
rg "^    pub fn" crates/core/src/ -c | sort -t: -k2 -n -r | head -30

# Find items that should likely be pub(crate)
cargo doc --document-private-items -p opencode-core 2>&1 | grep "warning: public item"
```

#### Visibility Reduction Strategy

**Items to change to `pub(crate)`:**
- Internal helper functions used across modules
- Re-exported internal types
- Cross-crate but not public API items

**Items to keep `pub`:**
- `lib.rs` re-exports (intended public API)
- Types used by external crates
- Trait definitions for public APIs

**Order of Audit:**
1. `crates/core/src/lib.rs` - review re-exports
2. `crates/core/src/session.rs` - 86 `pub fn`
3. `crates/core/src/skill.rs` - 135 `pub fn`
4. `crates/core/src/project.rs` - 79 `pub fn`
5. Other high-count files

---

### P1-003: Service Layer Refinement

**Priority:** P1 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-009
**Dependencies:** P1-001 (Repository Pattern)

#### Current State

| Service | Status |
|---------|--------|
| `StorageService` | Exists but not abstracted |
| `MdnsService` | Needs DI review |
| `SAP Service` | Needs DI review |
| `Auth Service` | Needs DI review |

#### Implementation

After repository pattern is implemented, refactor StorageService:
```rust
// Before
pub struct StorageService {
    pool: SqlitePool,
}

// After
pub struct StorageService<R: SessionRepository, P: ProjectRepository> {
    session_repo: R,
    project_repo: P,
}

impl<R: SessionRepository, P: ProjectRepository> StorageService<R, P> {
    pub fn new(session_repo: R, project_repo: P) -> Self {
        Self { session_repo, project_repo }
    }
}
```

---

### P1-004: Naming Conventions Audit

**Priority:** P1 (High)
**Estimated Effort:** 1 week
**FR Reference:** FR-012

#### Verification

```bash
# Run clippy naming checks
cargo clippy --all -- -D warnings 2>&1 | grep -i "naming\|snake\|pascal"

# Check for non-snake_case identifiers
rg "fn [A-Z]" --type rust
```

#### Common Issues to Fix

| Issue | Pattern | Fix |
|-------|---------|-----|
| Function starting with capital | `fn GetSession` | `fn get_session` |
| Type as variable | `let Session = ...` | `let session = ...` |
| Constant not SCREAMING | `const max_size` | `const MAX_SIZE` |

---

### P1-005: Hooks Configuration

**Priority:** P1 (High)
**Estimated Effort:** 1 day
**FR Reference:** FR-013

#### Configuration

In `~/.claude/settings.json` or project hooks:

```json
{
  "hooks": {
    "postToolUse": [
      {
        "tool": "Write",
        "pattern": "**/*.rs",
        "run": ["cargo fmt"]
      },
      {
        "tool": "Write",
        "pattern": "**/*.rs",
        "run": ["cargo clippy -- -D warnings"]
      }
    ]
  }
}
```

#### Verification

```bash
cargo fmt --all -- --check
cargo clippy --all -- -D warnings
```

---

## 4. P2 - Medium Priority Issues

### P2-001: String vs &str Optimization

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-004

#### Verification

```bash
cargo clippy --all -- -W clippy::ptr_arg
```

#### Common Fixes

```rust
// Before
fn process(id: String) { ... }

// After
fn process(id: &str) { ... }

// Or for ownership needed
fn process(id: impl Into<String>) { ... }
```

---

### P2-002: Newtype Wrappers for Type Safety

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-005

#### Required Types

```rust
// crates/core/src/types.rs

/// Newtype wrapper for session identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        SessionId(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        SessionId(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        SessionId(s.to_string())
    }
}
```

#### Required Newtypes

| Type | Location | Purpose |
|------|----------|---------|
| `SessionId` | `crates/core/src/types.rs` | Session identifier |
| `ProjectId` | `crates/core/src/types.rs` | Project identifier |
| `UserId` | `crates/core/src/types.rs` | User identifier |
| `MessageId` | `crates/core/src/types.rs` | Message identifier |
| `ToolId` | `crates/core/src/types.rs` | Tool identifier |
| `SkillId` | `crates/core/src/types.rs` | Skill identifier |

---

### P2-003: Test Coverage Target

**Priority:** P2 (Medium)
**Estimated Effort:** 2 weeks
**FR Reference:** FR-006

#### Baseline Measurement

```bash
# Install cargo-llvm-cov if not present
cargo install cargo-llvm-cov

# Get baseline coverage
cargo llvm-cov --fail-under-lines 80 --lcov --output-path lcov.info
```

#### Coverage Requirements by Crate

| Crate | Current | Target | Priority Files |
|-------|---------|--------|----------------|
| `crates/core/` | ~60% | 80%+ | skill.rs, session.rs, project.rs |
| `crates/tools/` | ~50% | 80%+ | lsp_tool.rs, registry.rs |
| `crates/agent/` | ~45% | 80%+ | runtime.rs |

#### Test Naming Convention

```rust
#[tokio::test]
async fn creates_session_with_valid_id() { ... }

#[tokio::test]
async fn rejects_session_when_id_is_empty() { ... }

#[tokio::test]
async fn returns_none_when_session_not_found() { ... }
```

---

### P2-004: Unsafe Code Safety Documentation

**Priority:** P2 (Medium)
**Estimated Effort:** 1 day
**FR Reference:** FR-007

#### Current State

| File | unsafe Count | Status |
|------|-------------|--------|
| `crates/tui/src/app.rs` | 3 | Needs SAFETY |
| `crates/server/src/routes/validation.rs` | 2 | Needs SAFETY |
| `crates/plugin/src/lib.rs` | 1 | Needs SAFETY |

#### Required Pattern

```rust
// SAFETY:
// - `ptr` must be non-null
// - `ptr` must be properly aligned
// - `ptr` must point to initialized data
// - No mutable references may exist for the lifetime of the returned reference
unsafe { &*ptr }
```

---

### P2-005: Builder Pattern Audit

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-008

#### Audit Command

```bash
rg "struct.*Builder" --type rust
```

#### Candidates for Builder Pattern

- `ServerConfig` - many optional fields
- `ClientConfig` - LLM provider config
- `SessionConfig` - session initialization

---

### P2-006: Sealed Traits

**Priority:** P2 (Medium)
**Estimated Effort:** 1 week
**FR Reference:** FR-010

#### Required Pattern

```rust
mod private {
    pub trait Sealed {}
}

pub trait Format: private::Sealed {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>, DecodeError>;
}

mod formats {
    use super::private;
    use super::Format;

    #[derive(Default)]
    pub struct JsonFormat;

    impl private::Sealed for JsonFormat {}
    impl Format for JsonFormat {
        // ...
    }
}
```

#### Candidates to Seal

- `Tool` trait - only internal implementations
- `Agent` trait - only internal implementations
- `StorageBackend` trait - only internal implementations

---

## 5. Implementation Phases

### Phase 1: Error Handling (Weeks 1-4)

| Week | Tasks | Deliverables | PRDs |
|------|-------|--------------|------|
| 1 | Audit unwrap() in core | Categorized inventory spreadsheet | FR-001 |
| 2 | Refactor core error handling | thiserror-based errors in core | FR-001 |
| 3 | Refactor tools error handling | thiserror-based errors in tools | FR-001 |
| 4 | Refactor server/agent errors | thiserror/anyhow in server/agent | FR-001 |

### Phase 2: Pattern Adoption (Weeks 5-10)

| Week | Tasks | Deliverables | PRDs |
|------|-------|--------------|------|
| 5 | Define repository traits | Trait definitions | FR-002 |
| 6 | Implement repository traits | SqliteSessionRepository | FR-002 |
| 7 | Visibility audit | Reduced pub fn count | FR-003 |
| 8 | Service layer refactor | Dependency injection | FR-009 |
| 9 | Naming conventions audit | Clippy naming warnings = 0 | FR-012 |
| 10 | Hooks configuration | CI/CD hooks active | FR-013 |

### Phase 3: Polish (Weeks 11-12)

| Week | Tasks | Deliverables | PRDs |
|------|-------|--------------|------|
| 11 | Newtype wrappers, String→&str | Type safety improvements | FR-005, FR-004 |
| 12 | Coverage, unsafe SAFETY, sealed traits | 80%+ coverage, documented unsafe | FR-006, FR-007, FR-010 |

### Phase 4: Builder & Verification (Week 13)

| Week | Tasks | Deliverables | PRDs |
|------|-------|--------------|------|
| 13 | Builder pattern audit, final verification | All FRs verified | FR-008, All |

---

## 6. Verification Gates

### Must Pass (Release Blocker)

| Criteria | Verification | Command |
|----------|--------------|---------|
| Zero `unwrap()`/`expect()` | Count = 0 | `grep -r "unwrap()\|expect(" crates/*/src/*.rs` |
| Clippy passes | Exit code 0 | `cargo clippy --all -- -D warnings` |
| Format passes | Exit code 0 | `cargo fmt --all` |
| Tests pass | All green | `cargo test` |
| Repository traits defined | Code review | `grep "pub trait.*Repository" crates/` |
| SQL injection prevention | Only parameterized | `rg 'format!.*SELECT' --type rust` |
| Secrets management | No hardcoded | `rg 'sk-[a-zA-Z0-9]{20,}' --type rust` |

### Should Pass (Quality Gate)

| Criteria | Target | Command |
|----------|--------|---------|
| Test coverage | 80%+ | `cargo llvm-cov --fail-under-lines 80` |
| unsafe with SAFETY | 100% | Manual review |
| pub fn in core | <50 | `rg "^    pub fn" crates/core/src/` |
| Naming warnings | 0 | `cargo clippy -- -D warnings 2>&1 | grep -i naming` |

---

## 7. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking API changes | High | High | Version bump, changelog |
| Test failures during refactor | High | Medium | Incremental refactor, feature flags |
| Performance regression | Low | Medium | Benchmark before/after |
| Compile time increase | Medium | Low | Incremental compilation |

---

## 8. Dependencies

```
P0-001 (unwrap elimination)
    │
    ├──► P1-001 (repository traits) - needs error types defined
    │
    └──► P1-003 (service layer) - depends on repository traits

P1-002 (visibility) - independent
P1-004 (naming) - independent
P1-005 (hooks) - independent

P2-001 (String vs &str) - independent
P2-002 (newtypes) - independent
P2-003 (coverage) - independent after P0
P2-004 (unsafe SAFETY) - independent
P2-005 (builder) - independent
P2-006 (sealed traits) - independent
```

---

## Appendix: Commands Reference

```bash
# Error handling verification
grep -rn "unwrap()\|expect(" crates/ | wc -l  # Target: 0

# Clippy verification
cargo clippy --all -- -D warnings  # Target: 0 warnings

# Format verification
cargo fmt --all -- --check  # Target: pass

# Test coverage
cargo llvm-cov --fail-under-lines 80  # Target: pass

# Public API audit
cargo doc --document-private-items -p opencode-core 2>&1 | grep "warning: public item"

# Naming conventions
cargo clippy --all -- -D warnings 2>&1 | grep -i "naming\|snake\|pascal"

# Repository traits
grep -r "pub trait.*Repository" crates/

# Unsafe blocks
grep -rn "unsafe" crates/ --include="*.rs" | grep -v "SAFETY"
```

---

*Plan generated from spec_v23.md and gap-analysis.md*
*Iteration 23 - Rust Conventions Compliance Implementation*
