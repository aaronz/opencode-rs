# Specification Document - Iteration 23

**Project:** OpenCode Rust Monorepo
**Iteration:** 23
**Date:** 2026-04-15
**Phase:** Rust Conventions Compliance Implementation

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Gap Analysis Summary](#2-gap-analysis-summary)
3. [Feature Requirements (FR)](#3-feature-requirements-fr)
4. [P0 - Blocking Issues](#4-p0---blocking-issues)
5. [P1 - High Priority Issues](#5-p1---high-priority-issues)
6. [P2 - Medium Priority Issues](#6-p2---medium-priority-issues)
7. [Technical Debt](#7-technical-debt)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Acceptance Criteria](#9-acceptance-criteria)

---

## 1. Executive Summary

### Overall Status

| Category | Compliance | Gap Severity |
|----------|------------|--------------|
| Error Handling | ❌ Critical Gap | P0 |
| Visibility Audit | ⚠️ Needs Review | P1 |
| Pattern Adoption | ⚠️ Partial | P1 |
| Test Coverage | ⚠️ Unknown | P2 |
| Module Organization | ✅ Compliant | - |
| Security | ✅ Compliant | - |

### PRD Reference

This specification addresses the **Code Refactor — Rust Conventions Compliance** PRD which defines:
- Coding style enforcement (`coding-style.md`)
- Rust-specific patterns adoption (`patterns.md`)
- Testing standards compliance (`testing.md`)
- Hooks configuration (`hooks.md`)
- Security practices verification (`security.md`)

### Key Metrics

| Metric | Current State | Target |
|--------|---------------|--------|
| `unwrap()`/`expect()` count | ~1,137+ | 0 |
| `pub fn` in core crate | 501 | Audit required |
| Repository traits | 0 defined | Required |
| Test coverage | Unknown | 80%+ |
| Newtype wrappers | 2 found | Required |

---

## 2. Gap Analysis Summary

### Gap Severity Overview

| Priority | Count | Description |
|----------|-------|-------------|
| **P0** | 1 | `unwrap()`/`expect()` in production code (~1,137 occurrences) |
| **P1** | 3 | Missing repository traits, visibility audit needed, service layer incomplete |
| **P2** | 4 | String vs &str, newtype wrappers, test coverage, unsafe SAFETY comments |

### Compliance Scorecard

| Category | Status | Notes |
|----------|--------|-------|
| Error Handling | ❌ Critical Gap | P0 - Must fix |
| Visibility Audit | ⚠️ Needs Review | P1 - 501 pub fn needs audit |
| Pattern Adoption | ⚠️ Partial | P1 - Repository traits missing |
| Test Coverage | ⚠️ Unknown | P2 - Need measurement |
| Module Organization | ✅ Compliant | Domain-based |
| Security | ✅ Compliant | No hardcoded secrets |

---

## 3. Feature Requirements (FR)

| FR-ID | Feature | Priority | Status |
|-------|---------|----------|--------|
| FR-001 | Error Handling - unwrap() Elimination | P0 | Not Compliant |
| FR-002 | Repository Pattern Implementation | P1 | Not Compliant |
| FR-003 | Visibility Audit and Control | P1 | Needs Review |
| FR-004 | Ownership and Borrowing Compliance | P2 | ~60% Compliant |
| FR-005 | Newtype Pattern for Type Safety | P2 | Not Compliant |
| FR-006 | Test Coverage Target | P2 | Unknown |
| FR-007 | Unsafe Code Safety Documentation | P2 | Needs Audit |
| FR-008 | Builder Pattern Adoption | P2 | Not Audited |
| FR-009 | Service Layer Pattern | P1 | Partial |
| FR-010 | Sealed Traits for Extensibility Control | P2 | Not Compliant |
| FR-011 | Enum State Machines | P2 | Compliant |
| FR-012 | Naming Conventions Enforcement | P1 | Needs Audit |
| FR-013 | Hooks Configuration | P1 | Needs Setup |
| FR-014 | SQL Injection Prevention | P1 | Compliant |
| FR-015 | Secrets Management | P1 | Compliant |

---

### FR-001: Error Handling Standardization - unwrap() Elimination

**Priority:** P0 (Blocking)
**Status:** Not Compliant

#### Requirement
Replace all `unwrap()` and `expect()` calls in production code with proper `Result` handling using `thiserror` for library crates and `anyhow` for application crates.

#### Current State
| Crate | unwrap()/expect() Count | Files Affected |
|-------|------------------------|----------------|
| `crates/core/` | 636 | 36 files |
| `crates/tools/` | 292 | 14 files |
| `crates/agent/` | 74 | 2 files |
| `crates/server/` | 135 | 18 files |
| **Total** | **~1,137+** | **70+ files** |

#### Top Offender Files
| File | Count | Impact |
|------|-------|--------|
| `crates/core/src/skill.rs` | 135 | Critical |
| `crates/core/src/session.rs` | 86 | Critical |
| `crates/core/src/project.rs` | 79 | High |
| `crates/tools/src/lsp_tool.rs` | 74 | High |
| `crates/tools/src/registry.rs` | 54 | Medium |
| `crates/agent/src/runtime.rs` | 71 | High |

#### Required Pattern
```rust
// Library crates MUST use thiserror
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config format: {0}")]
    Parse(String),
}

// Application crates MAY use anyhow
fn load_config() -> anyhow::Result<Config> {
    let content = std::fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

#### Verification
```bash
# Must pass with zero warnings
cargo clippy --all -- -D warnings

# Count unwrap occurrences
grep -rn "unwrap()\|expect(" opencode-rust/crates/ | wc -l
```

---

### FR-002: Repository Pattern Implementation

**Priority:** P1 (High)
**Status:** Not Compliant

#### Requirement
All data access MUST be encapsulated behind traits following the repository pattern.

#### Current State
```bash
$ grep -r "pub trait.*Repository" opencode-rust/crates/
# NO MATCHES FOUND - NOT COMPLIANT
```

#### Required Pattern
```rust
// Repository trait definition
pub trait SessionRepository: Send + Sync {
    fn find_by_id(&self, id: &str) -> Result<Option<Session>, StorageError>;
    fn find_all(&self) -> Result<Vec<Session>, StorageError>;
    fn save(&self, session: &Session) -> Result<(), StorageError>;
    fn delete(&self, id: &str) -> Result<(), StorageError>;
    fn list_by_project(&self, project_id: &str) -> Result<Vec<Session>, StorageError>;
}

// Error type for storage layer
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}
```

#### Implementation Requirements
1. Define `SessionRepository` trait in `crates/storage/`
2. Define `ProjectRepository` trait in `crates/storage/`
3. Implement `SqliteSessionRepository` for production
4. Implement `SqliteProjectRepository` for production
5. Create `InMemoryRepository` for tests
6. Refactor `StorageService` to use repository traits

---

### FR-003: Visibility Audit and Control

**Priority:** P1 (High)
**Status:** Needs Review

#### Requirement
- Default to private visibility
- Use `pub(crate)` for internal crate sharing
- Only mark `pub` what is part of the public API
- Re-export public API from `lib.rs` only

#### Current State
| Crate | `pub fn` Count | Assessment |
|-------|---------------|------------|
| `crates/core/` | 501 | High - needs audit |
| `crates/tools/` | ~150 | Medium |
| `crates/server/` | ~80 | Medium |

#### Audit Checklist
- [ ] Audit each `pub fn` to determine visibility scope
- [ ] Reduce to `pub(crate)` where not part of public API
- [ ] Verify `lib.rs` re-exports only intended public API
- [ ] Run `cargo doc --document-private-items` to identify leaks

#### Visibility Categories
| Visibility | When to Use |
|------------|-------------|
| `private` (default) | Implementation details |
| `pub(crate)` | Internal API shared within crate |
| `pub` | Public API, part of crate's interface |

---

### FR-004: Ownership and Borrowing Compliance

**Priority:** P2 (Medium)
**Status:** ~60% Compliant

#### Requirement
- Prefer `&str` over `String` when ownership isn't needed
- Use `Into<String>` for constructors that need ownership
- Prefer immutable borrow over mutation

#### Current Issues
```rust
// Problem: String parameter when &str would work
pub async fn load_session(&self, id: String) -> Result<Option<Session>>

// Should be:
pub async fn load_session(&self, id: &str) -> Result<Option<Session>>
```

#### Verification
```bash
cargo clippy --all -- -W clippy::ptr_arg
```

---

### FR-005: Newtype Pattern for Type Safety

**Priority:** P2 (Medium)
**Status:** Not Compliant

#### Requirement
Prevent argument mix-ups with distinct wrapper types.

#### Current State
```bash
$ grep -r "struct.*Id(" opencode-rust/crates/
# Only 2 matches found - NOT COMPLIANT
```

#### Required Types
```rust
// Newtype wrappers for type safety
struct SessionId(String);
struct UserId(u64);
struct ProjectId(u64);
struct MessageId(u64);

impl SessionId {
    pub fn new(id: impl Into<String>) -> Self {
        SessionId(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

---

### FR-006: Test Coverage Target

**Priority:** P2 (Medium)
**Status:** Unknown

#### Requirement
- 80%+ line coverage target
- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Use `mockall` for mocking dependencies

#### Coverage Infrastructure
| Metric | Status |
|--------|--------|
| `#[cfg(test)]` modules | 214 files ✅ |
| `cargo-llvm-cov` integration | Unknown |
| Coverage gate in CI | Unknown |

#### Verification
```bash
# Measure coverage
cargo llvm-cov --fail-under-lines 80

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

---

### FR-007: Unsafe Code Safety Documentation

**Priority:** P2 (Medium)
**Status:** Needs Audit

#### Requirement
- Minimize `unsafe` blocks
- Every `unsafe` block requires `// SAFETY:` comment
- Never use `unsafe` to bypass borrow checker

#### Current State
| File | unsafe Count | Has SAFETY? |
|------|-------------|------------|
| `crates/tui/src/app.rs` | 3 | Unknown |
| `crates/server/src/routes/validation.rs` | 2 | Unknown |
| `crates/plugin/src/lib.rs` | 1 | Unknown |

#### Required Pattern
```rust
// SAFETY: `ptr` is non-null, aligned, points to initialized Widget,
// and no mutable references exist for its lifetime.
unsafe { &*ptr }
```

---

### FR-008: Builder Pattern Adoption

**Priority:** P2 (Medium)
**Status:** Not Audited

#### Requirement
Use builder pattern for structs with many optional parameters.

#### Required Pattern
```rust
impl ServerConfig {
    pub fn builder(host: impl Into<String>, port: u16) -> ServerConfigBuilder {
        ServerConfigBuilder {
            host: host.into(),
            port,
            max_connections: 100,
        }
    }
}
```

---

### FR-009: Service Layer Pattern

**Priority:** P1 (Medium)
**Status:** Partial

#### Current State
| Service | Status | Notes |
|---------|--------|-------|
| `StorageService` | ✅ Exists | But not abstracted behind trait |
| `MdnsService` | ✅ Exists | |
| `SAP Service` | ✅ Exists | |
| `Auth Service` | ✅ Exists | |

#### Requirement
Service layer should use dependency injection with trait abstractions.

---

### FR-010: Sealed Traits for Extensibility Control

**Priority:** P2 (Medium)
**Status:** Not Compliant

#### Requirement
Use sealed traits to control trait extensibility and prevent external implementations.

#### Required Pattern
```rust
mod private {
    pub trait Sealed {}
}

pub trait Format: private::Sealed {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>, DecodeError>;
}

// Implement for internal types only
mod formats {
    use super::private;
    use super::Format;

    #[derive(Default)]
    pub struct JsonFormat;

    impl private::Sealed for JsonFormat {}
    impl Format for JsonFormat {
        fn encode(&self, data: &[u8]) -> Vec<u8> { /* ... */ }
        fn decode(&self, data: &[u8]) -> Result<Vec<u8>, DecodeError> { /* ... */ }
    }
}
```

#### Audit
```bash
# Check for unsealed public traits that should be sealed
grep -rn "pub trait" opencode-rust/crates/*/src/ | grep -v "Sealed"
```

---

### FR-011: Enum State Machines

**Priority:** P2 (Medium)
**Status:** Compliant ✅

#### Requirement
Model states as enums to make illegal states unrepresentable.

#### Current Implementation
```rust
// crates/core/src/session_state.rs - compliant
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { reason: String, retries: u32 },
}
```

#### Verification
- All state machine enums use exhaustive matching
- No wildcard `_` patterns for business-critical enums
- State transitions are validated

---

### FR-012: Naming Conventions Enforcement

**Priority:** P1 (High)
**Status:** Needs Audit

#### Requirement
Enforce Rust naming conventions across all crates.

| Element | Convention | Example |
|---------|------------|---------|
| Functions/variables | `snake_case` | `get_session`, `session_id` |
| Types/traits/enums | `PascalCase` | `Session`, `ToolRegistry` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_TOKEN_BUDGET` |
| Lifetime parameters | `'a`, `'de` (short) | `'input` (complex only) |

#### Verification
```bash
# Run clippy naming checks
cargo clippy --all -- -D warnings 2>&1 | grep -i "naming\|snake\|pascal"

# Check for non-snake_case identifiers
rg "fn [A-Z]" --type rust
```

---

### FR-013: Hooks Configuration

**Priority:** P1 (High)
**Status:** Needs Setup

#### Requirement
Configure post-tool-use hooks for automated enforcement.

| Hook | Trigger | Purpose |
|------|---------|---------|
| `cargo fmt` | After editing `.rs` files | Auto-format |
| `cargo clippy` | After editing `.rs` files | Lint checks |
| `cargo check` | After editing `.rs` files | Fast compilation verify |

#### Configuration Location
Configure in `~/.claude/settings.json` or project-specific hooks.

#### Verification
```bash
# Verify hooks are active
cargo fmt --all -- --check  # Should pass
cargo clippy --all -- -D warnings  # Should pass
```

---

### FR-014: SQL Injection Prevention

**Priority:** P1 (High)
**Status:** Compliant ✅

#### Requirement
Use parameterized queries exclusively for all database operations.

#### Current Implementation
```rust
// crates/storage/src/service.rs - compliant
sqlx::query("SELECT * FROM sessions WHERE id = $1")
    .bind(&id)
    .fetch_one(&pool)
    .await?;
```

#### Verification
```bash
# Ensure no string interpolation in SQL
rg 'format!.*SELECT|format!.*INSERT|format!.*UPDATE|format!.*DELETE' --type rust
```

---

### FR-015: Secrets Management

**Priority:** P1 (High)
**Status:** Compliant ✅

#### Requirement
- NEVER hardcode secrets in source code
- Use environment variables for all credentials
- Validate secrets exist at startup

#### Required Pattern
```rust
// FORBIDDEN:
const API_KEY: &str = "sk-abc123...";

// REQUIRED:
fn load_api_key() -> anyhow::Result<String> {
    std::env::var("PAYMENT_API_KEY")
        .context("PAYMENT_API_KEY must be set")
}
```

#### Verification
```bash
# Check for hardcoded secrets
rg 'sk-[a-zA-Z0-9]{20,}' --type rust
rg 'password\s*=\s*["\'][^"\']+["\']' --type rust
rg 'api_key\s*=\s*["\'][^"\']+["\']' --type rust
```

---

## 4. P0 - Blocking Issues

### P0-001: unwrap()/expect() Elimination

**Issue:** ~1,137+ occurrences of `unwrap()`/`expect()` in production code across all crates.

**Risk:** Runtime panics possible on malformed input, network failures, parsing errors.

**Fix Strategy:**
1. Phase 1: Audit and categorize all unwrap() usages
2. Phase 2: Replace with proper Result handling using thiserror/anyhow
3. Phase 3: Add context with `.with_context(|| ...)` where needed
4. Phase 4: Verify with `cargo clippy -- -D warnings`

**Progress Tracking:**
```bash
# Before refactoring
grep -rn "unwrap()\|expect(" opencode-rust/crates/ | wc -l
# Expected: ~1137

# After refactoring (target)
grep -rn "unwrap()\|expect(" opencode-rust/crates/ | wc -l
# Expected: 0
```

---

## 5. P1 - High Priority Issues

### P1-001: Repository Trait Abstraction

**Issue:** No repository traits defined. Data access directly in `StorageService`.

**Fix:**
1. Define `SessionRepository` trait
2. Define `ProjectRepository` trait
3. Implement `SqliteSessionRepository`
4. Implement `InMemoryRepository` for tests
5. Refactor `StorageService` to use traits

### P1-002: Visibility Scope Audit

**Issue:** 501 `pub fn` in core crate needs audit to determine proper visibility.

**Fix:**
1. Review each `pub fn` for necessity
2. Change to `pub(crate)` for internal APIs
3. Ensure `lib.rs` only exports intended public API

### P1-003: Service Layer Refinement

**Issue:** `StorageService` exists but doesn't use repository trait abstraction.

**Fix:**
1. Extract repository traits from StorageService
2. Use dependency injection for testability

---

## 6. P2 - Medium Priority Issues

| Issue | Description | Fix Effort |
|-------|-------------|------------|
| String vs &str | Many `String` parameters that could be `&str` | Low |
| Newtype wrappers | Missing `SessionId`, `UserId`, `ProjectId` types | Low |
| Test coverage | Need to measure and reach 80% | Medium |
| unsafe SAFETY | 6 blocks need `// SAFETY:` comments | Low |
| Builder pattern | Need to audit for existing builders | Low |

---

## 7. Technical Debt

### Debt Summary

| Debt Item | Severity | Est. Effort | Dependencies |
|-----------|----------|-------------|--------------|
| Remove unwrap() from core (~636) | P0 | High | Error refactor |
| Remove unwrap() from tools (~292) | P0 | High | Error refactor |
| Add repository traits | P1 | Medium | Design decision |
| Visibility audit (501 pub fn) | P1 | Medium | None |
| Add newtype wrappers | P2 | Low | None |
| unsafe SAFETY comments | P2 | Low | None |
| Coverage measurement | P2 | Low | None |

---

## 8. Implementation Roadmap

### Phase 1: Error Handling (Weeks 1-4)

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | Audit unwrap() in core | Categorized inventory |
| 2 | Refactor core error handling | thiserror-based errors |
| 3 | Refactor tools error handling | thiserror-based errors |
| 4 | Refactor server/agent errors | thiserror/anyhow as appropriate |

### Phase 2: Pattern Adoption (Weeks 5-8)

| Week | Task | Deliverable |
|------|------|-------------|
| 5 | Define repository traits | Trait definitions |
| 6 | Implement repository traits | SqliteSessionRepository |
| 7 | Visibility audit | Reduced pub fn count |
| 8 | Service layer refactor | Dependency injection |

### Phase 3: Polish (Weeks 9-10)

| Week | Task | Deliverable |
|------|------|-------------|
| 9 | Newtype wrappers, String→&str | Type safety improvements |
| 10 | Coverage, unsafe SAFETY | 80%+ coverage, documented unsafe |

---

## 9. Acceptance Criteria

### Must Pass (Release Blocker)

| Criteria | Verification |
|----------|--------------|
| Zero `unwrap()`/`expect()` in production | `grep -r "unwrap()\|expect(" crates/*/src/*.rs` returns 0 |
| `cargo clippy -- -D warnings` passes | Exit code 0 |
| `cargo fmt --all` passes | Exit code 0 |
| `cargo test` passes | All tests green |
| Repository traits defined and used | Code review |

### Should Pass (Quality Gate)

| Criteria | Verification |
|----------|--------------|
| 80%+ test coverage | `cargo llvm-cov --fail-under-lines 80` |
| No unsafe blocks without SAFETY | Code review |
| Visibility audit complete | <50 pub fn in core (reduced from 501) |

### Nice to Have (Polish)

| Criteria | Verification |
|----------|--------------|
| Newtype wrappers for IDs | Code review |
| Builder pattern for Config | Code review |
| All clippy suggestions addressed | `cargo clippy` warnings < 10 |

---

## Appendix: Reference Commands

```bash
# Count unwrap()/expect()
grep -rn "unwrap()\|expect(" opencode-rust/crates/ | wc -l

# Format check
cargo fmt --all -- --check

# Lint check
cargo clippy --all -- -D warnings

# Test coverage
cargo llvm-cov --fail-under-lines 80

# Audit public API
cargo doc --document-private-items 2>&1 | grep "warning: public item"

# Security audit
cargo audit
```

---

*Document generated from PRD: Code Refactor — Rust Conventions Compliance and Gap Analysis Report*
