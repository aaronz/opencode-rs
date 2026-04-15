# Specification Document - Iteration 23

**Project:** OpenCode Rust Monorepo
**Iteration:** 23
**Date:** 2026-04-15
**Phase:** Rust Conventions Compliance Implementation
**PRD Reference:** Code Refactor — Rust Conventions Compliance

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
10. [Rust Rules Reference](#10-rust-rules-reference)
11. [Cross-References](#11-cross-references)

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

- Coding style enforcement (`.opencode/rules/rust/coding-style.md`)
- Rust-specific patterns adoption (`.opencode/rules/rust/patterns.md`)
- Testing standards compliance (`.opencode/rules/rust/testing.md`)
- Hooks configuration (`.opencode/rules/rust/hooks.md`)
- Security practices verification (`.opencode/rules/rust/security.md`)

### Rule Hierarchy

1. **Rust-specific rules** (`.opencode/rules/rust/`) take precedence over common rules
2. **Language idioms** override generic recommendations — prefer idiomatic Rust over generic patterns
3. **Zero tolerance** for warnings — `cargo clippy -- -D warnings` must pass
4. **Immutability by default** — prefer `let` over `let mut`, borrow over mutate

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

| Priority | Count | Description | FR Reference |
|----------|-------|-------------|--------------|
| **P0** | 1 | `unwrap()`/`expect()` in production code (~1,137 occurrences) | FR-001 |
| **P1** | 5 | Missing repository traits, visibility audit, service layer, naming conventions, hooks | FR-002, FR-003, FR-009, FR-012, FR-013 |
| **P2** | 6 | String vs &str, newtype wrappers, test coverage, unsafe SAFETY, sealed traits, builder pattern | FR-004, FR-005, FR-006, FR-007, FR-010, FR-008 |

### Compliance Scorecard

| Category | Status | FR Reference | Gap Severity |
|----------|--------|--------------|--------------|
| Error Handling | ❌ Critical Gap | FR-001 | P0 |
| Repository Pattern | ❌ Not Compliant | FR-002 | P1 |
| Visibility Audit | ⚠️ Needs Review | FR-003 | P1 |
| Service Layer | ⚠️ Partial | FR-009 | P1 |
| Naming Conventions | ⚠️ Needs Audit | FR-012 | P1 |
| Hooks Configuration | ⚠️ Needs Setup | FR-013 | P1 |
| SQL Injection Prevention | ✅ Compliant | FR-014 | - |
| Secrets Management | ✅ Compliant | FR-015 | - |
| Module Organization | ✅ Compliant | - | - |
| Enum State Machines | ✅ Compliant | FR-011 | - |
| Ownership Compliance | ⚠️ ~60% | FR-004 | P2 |
| Newtype Wrappers | ❌ Not Compliant | FR-005 | P2 |
| Test Coverage | ⚠️ Unknown | FR-006 | P2 |
| Unsafe Code Safety | ⚠️ Needs Audit | FR-007 | P2 |
| Sealed Traits | ❌ Not Compliant | FR-010 | P2 |
| Builder Pattern | ⚠️ Not Audited | FR-008 | P2 |

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

#### Error Code Ranges

The codebase uses structured error codes per FR-118:

| Range | Category |
|-------|----------|
| 1xxx | Authentication errors |
| 2xxx | Authorization errors |
| 3xxx | Provider errors |
| 4xxx | Tool errors |
| 5xxx | Session errors |
| 6xxx | Config errors |
| 7xxx | Validation errors |
| 9xxx | Internal errors |

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

#### Refactor Philosophy

**DO:**
```rust
// Return new values, don't mutate in place
fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') {
        Cow::Owned(input.replace(' ', "_"))
    } else {
        Cow::Borrowed(input)
    }
}

// Take ownership in constructors via Into
fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
}
```

**DON'T:**
```rust
// Avoid unless mutation is genuinely required
fn normalize_bad(input: &mut String) {
    *input = input.replace(' ', "_");
}

// Take String when &str suffices
fn word_count_bad(text: String) -> usize {
    text.split_whitespace().count()
}
```

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

// Usage example
fn get_order(user: UserId, order: OrderId) -> anyhow::Result<Order>
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

#### Test Naming

Use descriptive names that explain the scenario:
- `creates_user_with_valid_email()`
- `rejects_order_when_insufficient_stock()`
- `returns_none_when_not_found()`

#### Async Tests

```rust
#[tokio::test]
async fn fetches_data_successfully() {
    let client = TestClient::new().await;
    let result = client.get("/data").await;
    assert!(result.is_ok());
}
```

#### Coverage Infrastructure

| Metric | Status |
|--------|--------|
| `#[cfg(test)]` modules | 214 files ✅ |
| `cargo-llvm-cov` integration | Unknown |
| Coverage gate in CI | Unknown |

#### Coverage Requirements by Crate

| Crate | Current Coverage | Target |
|-------|------------------|--------|
| `crates/core/` | ~60% | 80%+ |
| `crates/tools/` | ~50% | 80%+ |
| `crates/agent/` | ~45% | 80%+ |

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

#### Verification

```bash
# Find all unsafe blocks
grep -rn "unsafe" opencode-rust/crates/ --include="*.rs"

# Verify SAFETY comments
# Manual code review required
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

pub struct ServerConfigBuilder {
    host: String,
    port: u16,
    max_connections: u32,
}

impl ServerConfigBuilder {
    pub fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = n;
        self
    }

    pub fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host,
            port: self.port,
            max_connections: self.max_connections,
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

#### Required Pattern

```rust
pub struct OrderService {
    repo: Box<dyn OrderRepository>,
    payment: Box<dyn PaymentGateway>,
}

impl OrderService {
    pub fn new(repo: Box<dyn OrderRepository>, payment: Box<dyn PaymentGateway>) -> Self {
        Self { repo, payment }
    }
}
```

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

#### Module Organization

**DO:** Organize by domain

```text
src/
├── auth/
│   ├── mod.rs
│   ├── token.rs
│   └── middleware.rs
├── orders/
│   ├── mod.rs
│   └── service.rs
```

**DON'T:** Organize by type

```text
src/
├── structs.rs    # Don't do this
├── enums.rs
├── traits.rs
├── functions.rs
```

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

#### Required PostToolUse Hooks

| Hook | Trigger | Purpose |
|------|---------|---------|
| `cargo fmt` | After editing `.rs` files | Auto-format |
| `cargo clippy` | After editing `.rs` files | Lint checks |
| `cargo check` | After editing `.rs` files | Fast compilation verify |

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

**FORBIDDEN:**
```rust
// NEVER do this
let query = format!("SELECT * FROM users WHERE name = '{name}'");
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

### P1-004: Naming Conventions Audit

**Issue:** No systematic enforcement of Rust naming conventions across crates.

**Fix:**
1. Run `cargo clippy --all -- -D warnings` to identify violations
2. Fix snake_case for functions/variables
3. Fix PascalCase for types/traits/enums
4. Add naming lints to CI gate

### P1-005: Hooks Configuration

**Issue:** Pre-commit and post-tool hooks not configured for automated enforcement.

**Fix:**
1. Configure `cargo fmt` post-tool hook for .rs files
2. Configure `cargo clippy` post-tool hook for .rs files
3. Verify hooks active in CI pipeline

---

## 6. P2 - Medium Priority Issues

| Issue | Description | Fix Effort | FR Reference |
|-------|-------------|------------|--------------|
| String vs &str | Many `String` parameters that could be `&str` | Low | FR-004 |
| Newtype wrappers | Missing `SessionId`, `UserId`, `ProjectId` types | Low | FR-005 |
| Test coverage | Need to measure and reach 80% | Medium | FR-006 |
| unsafe SAFETY | 6 blocks need `// SAFETY:` comments | Low | FR-007 |
| Builder pattern | Need to audit for existing builders | Low | FR-008 |
| Sealed traits | Public traits should be sealed | Medium | FR-010 |

---

## 7. Technical Debt

### Debt Summary

| Debt Item | Severity | Est. Effort | Dependencies | FR Reference |
|-----------|----------|-------------|--------------|--------------|
| Remove unwrap() from core (~636) | P0 | High | Error refactor | FR-001 |
| Remove unwrap() from tools (~292) | P0 | High | Error refactor | FR-001 |
| Add repository traits | P1 | Medium | Design decision | FR-002 |
| Visibility audit (501 pub fn) | P1 | Medium | None | FR-003 |
| Service layer refinement | P1 | Medium | None | FR-009 |
| Naming conventions audit | P1 | Medium | None | FR-012 |
| Hooks configuration | P1 | Low | None | FR-013 |
| Add newtype wrappers | P2 | Low | None | FR-005 |
| unsafe SAFETY comments | P2 | Low | None | FR-007 |
| Coverage measurement | P2 | Low | None | FR-006 |
| Sealed traits adoption | P2 | Medium | None | FR-010 |
| Builder pattern audit | P2 | Low | None | FR-008 |
| String vs &str optimization | P2 | Low | None | FR-004 |

---

## 8. Implementation Roadmap

### Phase 1: Error Handling (Weeks 1-4)

| Week | Task | Deliverable | FR Reference |
|------|------|-------------|--------------|
| 1 | Audit unwrap() in core | Categorized inventory | FR-001 |
| 2 | Refactor core error handling | thiserror-based errors | FR-001 |
| 3 | Refactor tools error handling | thiserror-based errors | FR-001 |
| 4 | Refactor server/agent errors | thiserror/anyhow as appropriate | FR-001 |

### Phase 2: Pattern Adoption (Weeks 5-10)

| Week | Task | Deliverable | FR Reference |
|------|------|-------------|--------------|
| 5 | Define repository traits | Trait definitions | FR-002 |
| 6 | Implement repository traits | SqliteSessionRepository | FR-002 |
| 7 | Visibility audit | Reduced pub fn count | FR-003 |
| 8 | Service layer refactor | Dependency injection | FR-009 |
| 9 | Naming conventions audit | Clippy naming warnings fixed | FR-012 |
| 10 | Hooks configuration | CI/CD hooks active | FR-013 |

### Phase 3: Polish (Weeks 11-12)

| Week | Task | Deliverable | FR Reference |
|------|------|-------------|--------------|
| 11 | Newtype wrappers, String→&str | Type safety improvements | FR-005, FR-004 |
| 12 | Coverage, unsafe SAFETY, sealed traits | 80%+ coverage, documented unsafe | FR-006, FR-007, FR-010 |

### Phase 4: Builder & Verification (Week 13)

| Week | Task | Deliverable | FR Reference |
|------|------|-------------|--------------|
| 13 | Builder pattern audit, final verification | All FRs verified | FR-008, All |

---

## 9. Acceptance Criteria

### Must Pass (Release Blocker)

| Criteria | Verification | FR Reference |
|----------|--------------|--------------|
| Zero `unwrap()`/`expect()` in production | `grep -r "unwrap()\|expect(" crates/*/src/*.rs` returns 0 | FR-001 |
| `cargo clippy -- -D warnings` passes | Exit code 0 | All FRs |
| `cargo fmt --all` passes | Exit code 0 | All FRs |
| `cargo test` passes | All tests green | All FRs |
| Repository traits defined and used | Code review | FR-002 |
| SQL injection prevention verified | Parameterized queries only | FR-014 |
| Secrets management verified | No hardcoded secrets | FR-015 |

### Should Pass (Quality Gate)

| Criteria | Verification | FR Reference |
|----------|--------------|--------------|
| 80%+ test coverage | `cargo llvm-cov --fail-under-lines 80` | FR-006 |
| No unsafe blocks without SAFETY | Code review | FR-007 |
| Visibility audit complete | <50 pub fn in core (reduced from 501) | FR-003 |
| Naming conventions compliant | `cargo clippy` naming warnings = 0 | FR-012 |
| Hooks configured and active | CI/CD verification | FR-013 |

### Nice to Have (Polish)

| Criteria | Verification | FR Reference |
|----------|--------------|--------------|
| Newtype wrappers for IDs | Code review | FR-005 |
| Builder pattern for Config | Code review | FR-008 |
| Sealed traits for extensibility | Code review | FR-010 |
| All clippy suggestions addressed | `cargo clippy` warnings < 10 | FR-004 |

---

## 10. Rust Rules Reference

### Rule File Locations

| Rule | Location |
|------|----------|
| Coding Style | `.opencode/rules/rust/coding-style.md` |
| Patterns | `.opencode/rules/rust/patterns.md` |
| Testing | `.opencode/rules/rust/testing.md` |
| Hooks | `.opencode/rules/rust/hooks.md` |
| Security | `.opencode/rules/rust/security.md` |

### Command Reference

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Check
cargo check

# Test
cargo test

# Coverage
cargo llvm-cov --fail-under-lines 80

# Security
cargo audit
cargo deny check

# Count unwrap()/expect()
grep -rn "unwrap()\|expect(" opencode-rust/crates/ | wc -l

# Format check
cargo fmt --all -- --check

# Audit public API
cargo doc --document-private-items 2>&1 | grep "warning: public item"

# Security audit
cargo audit

# Naming conventions check
cargo clippy --all -- -D warnings 2>&1 | grep -i "naming\|snake\|pascal"

# Check for hardcoded secrets
rg 'sk-[a-zA-Z0-9]{20,}' --type rust
rg 'password\s*=\s*["\'][^"\']+["\']' --type rust

# Check SQL injection vectors (string interpolation in queries)
rg 'format!.*SELECT|format!.*INSERT|format!.*UPDATE|format!.*DELETE' --type rust

# Check for unsealed public traits
grep -rn "pub trait" opencode-rust/crates/*/src/ | grep -v "Sealed"

# Verify sealed trait implementations are internal
rg "impl.*Sealed for" --type rust
```

---

## 11. Cross-References

### PRD Cross-References

| Document | Topic |
|----------|-------|
| `01-core-architecture.md` | Entity definitions |
| `02-agent-system.md` | Agent patterns |
| `03-tools-system.md` | Tool registry |
| `16-test-plan.md` | Validation strategy |
| `17-rust-test-implementation-roadmap.md` | Test phasing |
| `18-crate-by-crate-test-backlog.md` | Backlog by crate |
| `19-implementation-plan.md` | Implementation phases |

### FR Cross-Reference

| FR-ID | Requirement | PRD Section | Gap Severity |
|-------|-------------|--------------|--------------|
| FR-001 | Error Handling - unwrap() Elimination | Error Handling | P0 |
| FR-002 | Repository Pattern Implementation | Pattern Requirements | P1 |
| FR-003 | Visibility Audit and Control | Visibility Rules | P1 |
| FR-004 | Ownership and Borrowing Compliance | Ownership and Borrowing | P2 |
| FR-005 | Newtype Pattern for Type Safety | Newtype Pattern | P2 |
| FR-006 | Test Coverage Target | Coverage Requirements | P2 |
| FR-007 | Unsafe Code Safety Documentation | Unsafe Code | P2 |
| FR-008 | Builder Pattern Adoption | Builder Pattern | P2 |
| FR-009 | Service Layer Pattern | Service Layer Pattern | P1 |
| FR-010 | Sealed Traits for Extensibility Control | Sealed Traits | P2 |
| FR-011 | Enum State Machines | Enum State Machines | P2 (Compliant) |
| FR-012 | Naming Conventions Enforcement | Naming Conventions | P1 |
| FR-013 | Hooks Configuration | Hooks Configuration | P1 |
| FR-014 | SQL Injection Prevention | SQL Injection Prevention | P1 (Compliant) |
| FR-015 | Secrets Management | Secrets Management | P1 (Compliant) |

---

## Enforcement Gates

### Pre-Commit Hooks

```bash
cargo fmt --all
cargo clippy --all -- -D warnings
cargo check
```

### CI Pipeline

| Stage | Command | Fail Condition |
|-------|---------|----------------|
| Format check | `cargo fmt --all -- --check` | Exit != 0 |
| Clippy | `cargo clippy --all -- -D warnings` | Warnings |
| Unit tests | `cargo test --lib` | Failures |
| Integration | `cargo test --test '*'` | Failures |
| Coverage | `cargo llvm-cov --fail-under-lines 80` | Below 80% |
| Security | `cargo audit` | CVEs found |
| Deny check | `cargo deny check` | Advisories |

---

*Document generated from PRD: Code Refactor — Rust Conventions Compliance and Gap Analysis Report*
*Iteration 23 - Rust Conventions Compliance Implementation*