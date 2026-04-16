# Specification Document: Rust Conventions Compliance (Iteration 28)

**Date:** 2026-04-17
**Iteration:** 28
**Status:** Active Development
**Target:** Full Rust conventions compliance per `.opencode/rules/rust/`

---

## 1. Overview

This document defines the specification for achieving and maintaining Rust conventions compliance across the OpenCode Rust codebase. It serves as the authoritative reference for all Rust coding standards, patterns, and enforcement mechanisms.

**Compliance Status:** ~60% Compliant (reassessed per gap analysis)

| Category | Status | Compliance |
|----------|--------|------------|
| Error Handling | ⚠️ Partial | ~60% |
| Visibility Rules | ❌ Non-compliant | ~40% |
| Ownership/Borrowing | ⚠️ Partial | ~70% |
| Test Coverage | ⚠️ Below target | ~55% |
| Pattern Adoption | ⚠️ Limited | ~65% |
| Security Practices | ✅ Good | ~95% |
| Repository Pattern | ✅ Partial | ~80% |
| Service Layer | ⚠️ Limited | ~50% |
| Builder Pattern | ✅ Partial | ~80% |
| Unsafe Code | ⚠️ Needs audit | ~90% |

---

## 2. Feature Requirements

### FR-001: Error Handling Standardization

**Priority:** P1
**Status:** Partial Compliance (~60%)

**Requirements:**
- All library crates must use `thiserror` for typed errors
- Application crates may use `anyhow` for flexible context
- Production code must not use `.unwrap()` or `.expect()`
- All errors must be propagated with `?` operator

**Current State:**
- ✅ `crates/core/src/error.rs` - Uses thiserror correctly
- ✅ `crates/storage/src/error.rs` - Uses thiserror correctly
- ✅ `crates/llm/src/error.rs` - Uses thiserror correctly
- ❌ `crates/tools/` - Contains unwrap() instances (P0)
- ❌ `crates/server/src/routes/` - Contains untyped String errors (P0)

**Gap Analysis:**
| Location | Issue | Fix Effort |
|----------|-------|------------|
| `crates/server/src/routes/*.rs` | Untyped String errors | Medium |
| `crates/tools/src/edit.rs:159` | `unwrap()` on Option | Low |
| `crates/tools/src/web_search.rs:70` | `unwrap()` on Option | Low |

---

### FR-002: Integration Test Compliance

**Priority:** P0
**Status:** Blocking

**Requirements:**
- All integration tests must pass before release
- Security tests must validate path normalization and XSS prevention
- Tool execution tests must verify read/write functionality

**Failing Tests (Must Fix):**
| Test Name | File | Issue |
|-----------|------|-------|
| `test_tool_registry_execute_read_tool` | `tests/src/` | Tool execution returns failure |
| `test_tool_registry_execute_write_tool` | `tests/src/` | Tool execution returns failure |
| `test_path_normalization_prevents_traversal` | `tests/src/` | Path validation broken |
| `test_session_message_content_sanitization` | `tests/src/` | XSS/injection prevention broken |
| `test_session_message_xss_prevention` | `tests/src/` | XSS prevention broken |
| `test_write_tool_path_validation` | `tests/src/` | Path validation broken |

---

### FR-003: Test Coverage Enforcement

**Priority:** P1
**Status:** Not Compliant (no CI gate)

**Requirements:**
- Minimum 80% line coverage for all crates
- Use `cargo-llvm-cov` for coverage reporting
- CI gate must fail below 80% threshold

**Current State:**
| Crate | Current Coverage | Target | Delta |
|-------|------------------|--------|-------|
| `core` | ~60% | 80%+ | +20% |
| `storage` | ~70% | 80%+ | +10% |
| `llm` | ~55% | 80%+ | +25% |
| `tools` | ~50% | 80%+ | +30% |
| `agent` | ~45% | 80%+ | +35% |
| `server` | ~40% | 80%+ | +40% |
| `tui` | ~60% | 80%+ | +20% |
| `plugin` | ~70% | 80%+ | +10% |
| `auth` | ~75% | 80%+ | +5% |
| `config` | ~70% | 80%+ | +10% |
| `cli` | ~50% | 80%+ | +30% |

**CI Gate Requirement:**
```bash
cargo llvm-cov --fail-under-lines 80
```

---

### FR-004: LlmError Migration to thiserror

**Priority:** P1
**Status:** Compliant

**Requirements:**
- `LlmError` enum must use `#[derive(thiserror::Error)]`
- `RetryConfig` error types must use `thiserror`
- All error variants must have structured fields

**Current State:**
- ✅ `crates/llm/src/error.rs` - Uses thiserror correctly

---

### FR-005: Production unwrap() Elimination

**Priority:** P0
**Status:** Not Compliant (3484+ instances)

**Requirements:**
- Zero `.unwrap()` or `.expect()` in production code
- Use proper error propagation with `?`
- Provide meaningful error messages

**Distribution by Crate:**
| Crate | Est. unwrap() Count | Priority |
|-------|---------------------|----------|
| `tools` | ~500 | P0 |
| `server/routes/` | ~100 | P0 |
| `core` | ~50 | P1 |
| Other crates | ~100 | P1 |

**P0 High-Risk Locations:**
| File | Line | Code |
|------|------|------|
| `crates/tools/src/edit.rs` | 159 | `let idx = index.unwrap();` |
| `crates/tools/src/web_search.rs` | 70 | `let api_key = api_key.unwrap();` |

**Command to Audit:**
```bash
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l
```

---

### FR-006: Visibility Boundary Audit

**Priority:** P1
**Status:** Not Compliant (~3896+ pub declarations)

**Requirements:**
- Default to private visibility
- Use `pub(crate)` for internal crate sharing
- Only mark `pub` what is part of public API
- `lib.rs` must re-export only intended public API

**Excessive Public Items by Module:**
| Module | Public Item Count (est.) | Should be pub(crate) (est.) |
|--------|--------------------------|-----------------------------|
| `crates/core/src/` | ~200 | ~80 |
| `crates/tools/src/` | ~50 | ~30 |
| `crates/agent/src/` | ~80 | ~40 |

**Audit Command:**
```bash
grep -n "pub fn\|pub struct\|pub enum" crates/core/src/
```

---

### FR-007: Mockall Integration for Testing

**Priority:** P2
**Status:** Not Implemented

**Requirements:**
- Add `mockall` dependency for trait mocking
- Use `mockall::mock!` for interface mocking
- Standardize mocking patterns across tests

---

### FR-008: Server Route Error Refactoring

**Priority:** P0
**Status:** Not Compliant

**Requirements:**
- Each route module should have a `thiserror` enum for errors
- Route handlers must not use `.unwrap()`
- Consistent error wrapping across all routes

**Files Requiring Changes:**
| File | Issue |
|------|-------|
| `crates/server/src/routes/*.rs` | Untyped String errors |
| `crates/server/src/routes/validation.rs` | Two unsafe blocks need SAFETY comments |

---

### FR-009: Repository Pattern (Maintained)

**Status:** ✅ Compliant (80%)

**Requirements:**
- All data access must be encapsulated behind traits
- Current: `SessionRepository`, `ProjectRepository` in storage crate

**Gap:** Only 2 repository traits found; need to extend to all data access

---

### FR-010: Service Layer Pattern

**Priority:** P2
**Status:** Limited (~50%)

**Requirements:**
- Business logic in service structs with injected dependencies
- Current: `StorageService` exists, need per-domain services

**Gap:** Business logic still in handlers; needs extraction to service layer

---

### FR-011: Builder Pattern (Expanding)

**Status:** ✅ Partial (~80%)

**Requirements:**
- Use for structs with many optional parameters
- Current: `ContextBuilder`, `ClientBuilder`, `SamlAuthnRequestBuilder` exist

**Gap:** Other complex structs not following pattern

---

### FR-012: Secrets Management (Maintained)

**Status:** ✅ Compliant (~95%)

**Requirements:**
- No hardcoded API keys or secrets
- Use environment variables via `std::env::var()`
- Validate presence of required secrets at startup

---

### FR-013: SQL Injection Prevention (Maintained)

**Status:** ✅ Compliant

**Requirements:**
- Use parameterized queries exclusively
- Never interpolate user input into SQL strings

---

### FR-014: Unsafe Code Audit

**Priority:** P2
**Status:** Needs Audit (~90%)

**Requirements:**
- Minimize `unsafe` blocks
- Every `unsafe` block requires `// SAFETY:` comment
- Audit `unsafe` usage during code review

**Unsafe Blocks Missing SAFETY Comments:**
| File | Line | Issue |
|------|------|-------|
| `crates/plugin/src/lib.rs` | 661 | Missing SAFETY comment |
| `crates/tui/src/app.rs` | 4677, 4690 | Missing SAFETY comments |
| `crates/server/src/routes/validation.rs` | 237, 256 | Missing SAFETY comments |

---

### FR-015: Ownership and Borrowing Compliance

**Priority:** P1
**Status:** Partial (~70%)

**Requirements:**
- Prefer `let` over `let mut` where mutation isn't required
- Take `&str` instead of `String` when ownership isn't needed
- Use `Into<String>` for constructors that need ownership
- Prefer returning new values over mutating in place

**Current Issues:**
| Location | Issue | Fix |
|----------|-------|-----|
| `crates/core/src/session.rs` | Many `let mut` | Audit for immutability |

**Pattern to Follow:**
```rust
// GOOD: Borrow when ownership isn't needed
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

// GOOD: Take ownership in constructors via Into
fn new(name: impl Into<String>) -> Self {
    Self { name: name.into() }
}
```

---

### FR-016: Newtype Pattern for Type Safety

**Priority:** P2
**Status:** Underutilized

**Requirements:**
- Prevent argument mix-ups with distinct wrapper types
- Add newtypes for ID types: `UserId`, `SessionId`, `ProjectId`

**Current State:**
- Only 2 newtypes found: `SlotId`, `TaskId`

**Pattern to Follow:**
```rust
struct UserId(u64);
struct OrderId(u64);

fn get_order(user: UserId, order: OrderId) -> anyhow::Result<Order>
```

---

### FR-017: Enum State Machines with Exhaustive Matching

**Priority:** P2
**Status:** Partial

**Requirements:**
- Model states as enums — make illegal states unrepresentable
- Always match exhaustively — no wildcard `_` for business-critical enums

**Current Issues:**
| Location | Issue |
|----------|-------|
| `crates/agent/` | Some state enums use wildcards |
| `crates/mcp/` | Some state enums need audit |

**Pattern to Follow:**
```rust
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { reason: String, retries: u32 },
}
```

---

### FR-018: Parameterized Testing with rstest

**Priority:** P2
**Status:** Limited

**Requirements:**
- Add `rstest` for parameterized tests
- Convert basic `#[test]` to parameterized where applicable

**Current State:**
- Basic `#[test]` only; no rstest usage found

---

### FR-019: Sealed Traits for Extensibility Control

**Priority:** P2
**Status:** Limited

**Requirements:**
- Use a private module to seal traits, preventing external implementations
- Only `sealed::Sealed` for Tool trait currently exists

**Pattern to Follow:**
```rust
mod private {
    pub trait Sealed {}
}

pub trait Format: private::Sealed {
    fn encode(&self, data: &[u8]) -> Vec<u8>;
}
```

---

### FR-020: Immutability by Default

**Priority:** P2
**Status:** Partial

**Requirements:**
- Prefer `let` over `let mut`
- Borrow over mutate
- Return new values instead of mutating in place

**Current Issues:**
| Location | Issue |
|----------|-------|
| `crates/core/src/session.rs` | Many `let mut` bindings |

---

## 3. Enforcement Gates

### CI Pipeline Requirements

| Stage | Command | Fail Condition |
|-------|---------|----------------|
| Format check | `cargo fmt --all -- --check` | Exit != 0 |
| Clippy | `cargo clippy --all -- -D warnings` | Warnings |
| Unit tests | `cargo test --lib` | Failures |
| Integration | `cargo test --test '*'` | Failures |
| **Coverage** | `cargo llvm-cov --fail-under-lines 80` | **Below 80%** |
| Security | `cargo audit` | CVEs found |
| Deny check | `cargo deny check` | Advisories |

### Current CI Gate Status

| Gate | Current Status | Action Required |
|------|----------------|-----------------|
| `cargo fmt --all -- --check` | ✅ Pass | Maintain |
| `cargo clippy -- -D warnings` | 🔄 In progress | Fix warnings before merge |
| `cargo test --lib` | ✅ Pass (136 tests) | Extend coverage |
| `cargo llvm-cov --fail-under-lines 80` | ❌ Fail | Increase coverage to 80%+ |
| `cargo audit` | Not run | Schedule regular runs |
| `cargo deny check` | Not run | Add to CI pipeline |

---

## 4. Action Items

### P0 — Critical (Blocker Issues)

| ID | Action | Files | Status |
|----|--------|-------|--------|
| FR-005 | Fix production unwrap() in `crates/tools/src/edit.rs:159` | `let idx = index.unwrap();` | Not Started |
| FR-005 | Fix production unwrap() in `crates/tools/src/web_search.rs:70` | `let api_key = api_key.unwrap();` | Not Started |
| FR-005 | Audit remaining production code for `.unwrap()` | Search non-test code | Not Started |
| FR-008 | Convert `crates/server/src/routes/` String errors to thiserror | Route handlers | Not Started |
| FR-002 | Fix failing integration tests | `tests/src/` | Failing |

### P1 — High Priority

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| FR-001 | Migrate legacy error variants to typed errors | 2 days | Partial |
| FR-003 | Add `cargo-llvm-cov` CI gate | 0.5 day | Not Started |
| FR-003 | Increase `crates/core/` coverage to 80%+ | 2 days | Not Started |
| FR-003 | Increase `crates/tools/` coverage to 80%+ | 3 days | Not Started |
| FR-003 | Increase `crates/agent/` coverage to 80%+ | 3 days | Not Started |
| FR-003 | Increase `crates/server/` coverage to 80%+ | 3 days | Not Started |
| FR-006 | Visibility audit across all crates | 3 days | Not Started |
| FR-006 | Reduce `pub` to `pub(crate)` in `crates/core/` | 2 days | Not Started |
| FR-006 | Reduce `pub` to `pub(crate)` in `crates/tools/` | 1 day | Not Started |
| FR-009 | Extend repository pattern to all data access | 1 week | Not Started |
| FR-015 | Audit `let mut` usage in `crates/core/src/session.rs` | 1 day | Not Started |

### P2 — Medium Priority

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| FR-007 | Add mockall dependency and patterns | 2 days | Not Started |
| FR-010 | Add service layer to server routes | 1 week | Not Started |
| FR-011 | Expand builder pattern to server routes | 1 week | Not Started |
| FR-014 | Add SAFETY comments to unsafe blocks | 1 day | Not Started |
| FR-016 | Add newtypes: `SessionId`, `UserId`, `ProjectId` | 2 days | Not Started |
| FR-017 | Audit enum matching for exhaustiveness | 2 days | Not Started |
| FR-018 | Add rstest dependency and convert tests | 2 days | Not Started |
| FR-019 | Seal additional traits | 1 day | Not Started |
| FR-020 | Prefer `let` + new values over `let mut` | 2 days | Not Started |

---

## 5. Crate-by-Crate Compliance Matrix

| Crate | Error Handling | Visibility | Coverage | Patterns | Unsafe | Priority Issues |
|-------|---------------|------------|----------|----------|--------|----------------|
| `core` | ✅ Good (~90%) | ❌ Poor (~40%) | ~60% | ✅ Good | ✅ Good | Visibility audit needed |
| `storage` | ✅ Good (~90%) | ✅ Good (~80%) | ~70% | ✅ Good | ✅ Good | Coverage gap |
| `llm` | ✅ Good (~95%) | ✅ Good (~80%) | ~55% | ✅ Good | ✅ Good | Coverage gap |
| `tools` | ❌ Poor (~40%) | ❌ Poor (~40%) | ~50% | ✅ Good | ✅ Good | unwrap(), visibility |
| `agent` | ✅ Good (~85%) | ❌ Poor (~50%) | ~45% | ⚠️ Partial | ✅ Good | Coverage, enum matching |
| `server` | ❌ Poor (~30%) | ✅ Good (~70%) | ~40% | ⚠️ Partial | ⚠️ Needs audit | Route errors, SAFETY |
| `tui` | ✅ Good (~85%) | ✅ Good (~80%) | ~60% | ✅ Good | ⚠️ Needs audit | SAFETY comments |
| `plugin` | ✅ Good (~85%) | ✅ Good (~80%) | ~70% | ✅ Good | ⚠️ Needs audit | SAFETY comments |
| `auth` | ✅ Good (~90%) | ✅ Good (~80%) | ~75% | ✅ Good | ✅ Good | Coverage gap |
| `config` | ✅ Good (~90%) | ✅ Good (~80%) | ~70% | ✅ Good | ✅ Good | Coverage gap |
| `cli` | ✅ Good (~85%) | ✅ Good (~80%) | ~50% | ✅ Good | ✅ Good | Coverage gap |

---

## 6. Files Requiring Immediate Attention

### P0 - Critical

| File | Line | Issue | Priority |
|------|------|-------|----------|
| `crates/tools/src/edit.rs` | 159 | `let idx = index.unwrap();` | P0 |
| `crates/tools/src/web_search.rs` | 70 | `let api_key = api_key.unwrap();` | P0 |
| `crates/server/src/routes/*.rs` | Various | Untyped String errors | P0 |
| `tests/src/security_tests.rs` | Various | Failing tests | P0 |
| `tests/src/tool_registry_audit_tests.rs` | Various | Failing tests | P0 |

### P1 - High Priority

| File | Lines | Issue | Priority |
|------|-------|-------|----------|
| `crates/core/src/lib.rs` | Various | Mix of pub/pub(crate) | P1 |
| `crates/tools/src/lib.rs` | Various | All tools exported pub | P1 |
| `crates/core/src/session.rs` | Various | `let mut` overuse | P1 |
| `crates/agent/src/**/*.rs` | Various | Coverage ~45% | P1 |
| `crates/server/src/**/*.rs` | Various | Coverage ~40% | P1 |

### P2 - Needs SAFETY Comments

| File | Line | Issue |
|------|------|-------|
| `crates/plugin/src/lib.rs` | 661 | `unsafe` needs SAFETY comment |
| `crates/tui/src/app.rs` | 4677, 4690 | `unsafe` needs SAFETY comments |
| `crates/server/src/routes/validation.rs` | 237, 256 | `unsafe` needs SAFETY comments |

---

## 7. Technical Debt Summary

| Category | Est. Effort | Items |
|----------|------------|-------|
| Error Handling | 2-3 weeks | 3484+ unwraps, server error types |
| Visibility | 1 week | ~3896+ pub declarations audit |
| Testing | 3 weeks | Coverage gaps across all crates |
| Pattern Adoption | 2 weeks | Service layer, newtypes, builders |
| CI/CD | 0.5 day | Add llvm-cov, audit, deny gates |

**Total Estimated Debt:** 5-6 weeks

---

## 8. Rust Rules Compliance Checklist

### Coding Style Requirements

| Requirement | Current | Target | Gap |
|-------------|---------|--------|-----|
| rustfmt formatting | ✅ | ✅ | None |
| clippy linting | ⚠️ | ✅ | Warnings exist |
| 4-space indent | ✅ | ✅ | None |
| Max 100 char line | ✅ | ✅ | None |

### Immutability Standards

| Requirement | Current | Target | Gap |
|-------------|---------|--------|-----|
| `let` by default | ⚠️ | ✅ | Some `let mut` overuse |
| Return new values | ⚠️ | ✅ | Some mutation in place |
| `Cow<'_, T>` usage | ⚠️ | ✅ | Rare usage |

### Error Handling

| Requirement | Current | Target | Gap |
|-------------|---------|--------|-----|
| thiserror for libraries | ✅ | ✅ | Core uses it correctly |
| anyhow for applications | ⚠️ | ✅ | Limited usage |
| No unwrap() in production | ❌ | ✅ | 3484+ violations |
| `?` for propagation | ⚠️ | ✅ | Partial adoption |

### Naming Conventions

| Element | Current | Target | Gap |
|---------|---------|--------|-----|
| snake_case functions | ✅ | ✅ | Compliant |
| PascalCase types | ✅ | ✅ | Compliant |
| SCREAMING_SNAKE_CASE const | ✅ | ✅ | Compliant |
| Lifetime parameters | ✅ | ✅ | Compliant |

### Visibility Rules

| Requirement | Current | Target | Gap |
|-------------|---------|--------|-----|
| Default to private | ❌ | ✅ | Excessive pub |
| pub(crate) for internal | ❌ | ✅ | Not adopted |
| pub for public API only | ❌ | ✅ | Leaks implementation |

### Module Organization

| Requirement | Current | Target | Gap |
|-------------|---------|--------|-----|
| Organized by domain | ✅ | ✅ | Compliant |
| No type-based organization | ✅ | ✅ | Compliant |

---

## 9. Verification Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Check clippy
cargo clippy --all -- -D warnings

# Run tests
cargo test --all

# Check coverage (requires cargo-llvm-cov)
cargo llvm-cov --fail-under-lines 80

# Count unwraps (excluding tests)
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l

# Audit pub visibility
grep -n "pub fn\|pub struct\|pub enum" crates/core/src/ | head -50

# Audit unsafe blocks
grep -n "unsafe" crates/*/src/*.rs | grep -v "test"
```

---

## 10. Cross-References

- [PRD: Rust Conventions Compliance](./iterations/iteration-28/prd_r28.md)
- [Gap Analysis](./iterations/iteration-28/gap-analysis.md)
- [01-core-architecture.md](../01-core-architecture.md)
- [02-agent-system.md](../02-agent-system.md)
- [03-tools-system.md](../03-tools-system.md)
- [16-test-plan.md](../16-test-plan.md)
- [17-rust-test-implementation-roadmap.md](../17-rust-test-implementation-roadmap.md)
- [18-crate-by-crate-test-backlog.md](../18-crate-by-crate-test-backlog.md)
- [19-implementation-plan.md](../19-implementation-plan.md)

---

## 11. Change Log

| Version | Date | Changes |
|---------|------|---------|
| v28 | 2026-04-17 | Reassessed compliance to ~60%; added FR-015 to FR-020; updated P0/P1/P2 priorities; updated unwrap() count to 3484+; added visibility audit requirements; added unsafe SAFETY comments requirement; updated crate-by-crate matrix |
| v27 | 2026-04-17 | Initial spec based on gap analysis; FR-001 to FR-014 added |

(End of file - total 531 lines)
