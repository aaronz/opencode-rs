# Specification Document: Rust Conventions Compliance (Iteration 27)

**Date:** 2026-04-17
**Iteration:** 27
**Status:** Active Development
**Target:** Full Rust conventions compliance per `.opencode/rules/rust/`

---

## 1. Overview

This document defines the specification for achieving and maintaining Rust conventions compliance across the OpenCode Rust codebase. It serves as the authoritative reference for all Rust coding standards, patterns, and enforcement mechanisms.

**Compliance Status:** ~70% Compliant

| Category | Status | Compliance |
|----------|--------|------------|
| Error Handling | ⚠️ Partial | ~70% |
| Repository Pattern | ✅ Good | 90% |
| Service Layer | ✅ Good | 85% |
| State Machines | ✅ Good | 95% |
| Builder Pattern | ✅ Good | 80% |
| Unsafe Code | ✅ Good | 90% |
| Visibility | ✅ Good | 85% |
| Secrets Management | ✅ Good | 95% |
| Test Coverage | ⚠️ Partial | ~60% |

---

## 2. Feature Requirements

### FR-001: Error Handling Standardization

**Priority:** P1
**Status:** Partial Compliance (~70%)

**Requirements:**
- All library crates must use `thiserror` for typed errors
- Application crates may use `anyhow` for flexible context
- Production code must not use `.unwrap()` or `.expect()`
- All errors must be propagated with `?` operator

**Current State:**
- ✅ `crates/core/src/error.rs` - Uses thiserror
- ✅ `crates/storage/src/error.rs` - Uses thiserror
- ⚠️ `crates/llm/src/error.rs` - Uses manual `Display` implementation
- ⚠️ `crates/tools/` - Contains ~500 `.unwrap()` instances
- ⚠️ `crates/server/src/routes/` - Contains `.unwrap()` in handlers

**Target State:**
```rust
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("rate limit exceeded: {retry_after:?}")]
    RateLimitExceeded { retry_after: Option<u64> },
    #[error("provider error: {0}")]
    Provider(String),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}
```

---

### FR-002: Integration Test Compliance

**Priority:** P0
**Status:** Blocking (9 tests failing)

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
| 4 more in `phase6_regression_tests` | `tests/src/` | Regression failures |

---

### FR-003: Test Coverage Enforcement

**Priority:** P1
**Status:** Not Compliant (no CI gate)

**Requirements:**
- Minimum 80% line coverage for all crates
- Use `cargo-llvm-cov` for coverage reporting
- CI gate must fail below 80% threshold

**Current State:**
| Crate | Current Coverage | Target |
|-------|------------------|--------|
| `core` | ~60% | 80%+ |
| `storage` | ~70% | 80%+ |
| `llm` | ~55% | 80%+ |
| `tools` | ~50% | 80%+ |
| `agent` | ~45% | 80%+ |
| `server` | ~65% | 80%+ |
| `tui` | ~60% | 80%+ |
| `plugin` | ~70% | 80%+ |
| `auth` | ~75% | 80%+ |
| `config` | ~70% | 80%+ |
| `cli` | ~50% | 80%+ |

**CI Gate Requirement:**
```bash
cargo llvm-cov --fail-under-lines 80
```

---

### FR-004: LlmError Migration to thiserror

**Priority:** P1
**Status:** Not Compliant

**Requirements:**
- `LlmError` enum must use `#[derive(thiserror::Error)]`
- `RetryConfig` error types must use `thiserror`
- All error variants must have structured fields

**Files Requiring Changes:**
| File | Lines | Issue |
|------|-------|-------|
| `crates/llm/src/error.rs` | 4-17 | `LlmError` not using thiserror |
| `crates/llm/src/error.rs` | 88-104 | `RetryConfig` not using thiserror |

---

### FR-005: Production unwrap() Elimination

**Priority:** P1
**Status:** Not Compliant (3516 instances)

**Requirements:**
- Zero `.unwrap()` or `.expect()` in production code
- Use proper error propagation with `?`
- Provide meaningful error messages

**Distribution by Crate:**
| Crate | Est. unwrap() Count | Priority |
|-------|---------------------|----------|
| `tools` | ~500 | High |
| `server/routes/` | ~100 | High |
| `core` | ~50 | Medium |
| Other crates | ~100 | Medium |

---

### FR-006: Visibility Boundary Audit

**Priority:** P2
**Status:** Partial Compliance

**Requirements:**
- Default to private visibility
- Use `pub(crate)` for internal crate sharing
- Only mark `pub` what is part of public API
- `lib.rs` must re-export only intended public API

**Audit Scope:**
- Reduce unnecessary `pub` to `pub(crate)` where possible
- Verify no implementation leaks through `pub` interfaces
- Ensure internal types are not accidentally made public

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

**Priority:** P2
**Status:** Partial Compliance

**Requirements:**
- Each route module should have a `thiserror` enum for errors
- Route handlers must not use `.unwrap()`
- Consistent error wrapping across all routes

---

## 3. Pattern Requirements

### FR-009: Repository Pattern (Maintained)

**Status:** ✅ Compliant (90%)

All data access must be encapsulated behind traits:
```rust
pub trait OrderRepository: Send + Sync {
    fn find_by_id(&self, id: u64) -> Result<Option<Order>, StorageError>;
    fn find_all(&self) -> Result<Vec<Order>, StorageError>;
}
```

---

### FR-010: Service Layer Pattern (Maintained)

**Status:** ✅ Compliant (85%)

Business logic in service structs with injected dependencies:
```rust
pub struct OrderService {
    repo: Box<dyn OrderRepository>,
    payment: Box<dyn PaymentGateway>,
}
```

---

### FR-011: Builder Pattern (Expanding)

**Status:** ✅ Compliant (80%)

Existing builders: `ContextBuilder`, `ClientBuilder`, `SamlAuthnRequestBuilder`
Target: Expand to `crates/server/routes/`

---

## 4. Security Requirements

### FR-012: Secrets Management (Maintained)

**Status:** ✅ Compliant (95%)

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

### FR-014: Unsafe Code Audit (Maintained)

**Status:** ✅ Compliant (90%)

**Requirements:**
- Minimize `unsafe` blocks
- Every `unsafe` block requires `// SAFETY:` comment
- Audit `unsafe` usage during code review

---

## 5. Enforcement Gates

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

---

## 6. Action Items

### P0 — Immediate (Fix Before Continue)

| ID | Action | Files | Status |
|----|--------|-------|--------|
| FR-002 | Fix `test_tool_registry_execute_read_tool` | `tests/src/` | Failing |
| FR-002 | Fix `test_tool_registry_execute_write_tool` | `tests/src/` | Failing |
| FR-002 | Fix `test_path_normalization_prevents_traversal` | `tests/src/` | Failing |
| FR-002 | Fix `test_session_message_content_sanitization` | `tests/src/` | Failing |
| FR-002 | Fix `test_session_message_xss_prevention` | `tests/src/` | Failing |
| FR-002 | Fix `test_write_tool_path_validation` | `tests/src/` | Failing |

### P1 — Short-term (1-2 Sprints)

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| FR-004 | Convert `LlmError` to thiserror | 1 day | Not Started |
| FR-004 | Convert `RetryConfig` to thiserror | 0.5 day | Not Started |
| FR-005 | Replace `.unwrap()` in `crates/tools/src/` | 1 week | Not Started |
| FR-005 | Replace `.unwrap()` in `crates/server/src/routes/` | 2 days | Not Started |
| FR-003 | Add `cargo-llvm-cov` CI gate | 0.5 day | Not Started |
| FR-001 | Migrate legacy error variants to typed errors | 2 days | Partial |

### P2 — Medium-term (1 Month)

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| FR-006 | Visibility audit across all crates | 3 days | Not Started |
| FR-007 | Add mockall dependency and patterns | 2 days | Not Started |
| FR-008 | Refactor server route errors | 1 week | Partial |
| FR-011 | Expand builder pattern to server routes | 1 week | Not Started |
| FR-009 | Audit repository trait abstraction in llm | 1 week | Not Started |
| FR-010 | Add service layer to server routes | 1 week | Not Started |

---

## 7. Crate-by-Crate Compliance Matrix

| Crate | Error Handling | Patterns | Coverage | Priority Issues |
|-------|---------------|----------|----------|----------------|
| `core` | ✅ Good | ✅ Good | ~60% | Legacy variants |
| `storage` | ✅ Good | ✅ Good | ~70% | None |
| `llm` | ⚠️ Poor | ✅ Good | ~55% | `LlmError` manual impl |
| `tools` | ⚠️ Mixed | ✅ Good | ~50% | 500+ unwraps |
| `agent` | ✅ Good | ✅ Good | ~45% | Coverage gap |
| `server` | ⚠️ Mixed | ✅ Good | ~65% | Route errors |
| `tui` | ✅ Good | ✅ Good | ~60% | None |
| `plugin` | ✅ Good | ✅ Good | ~70% | None |
| `auth` | ✅ Good | ✅ Good | ~75% | None |
| `config` | ✅ Good | ✅ Good | ~70% | None |
| `cli` | ✅ Good | ✅ Good | ~50% | Coverage gap |

---

## 8. Files Requiring Immediate Attention

| File | Lines | Issue | Priority |
|------|-------|-------|----------|
| `crates/llm/src/error.rs` | 4-17 | `LlmError` not using thiserror | P1 |
| `crates/llm/src/error.rs` | 88-104 | `RetryConfig` not using thiserror | P1 |
| `crates/tools/src/*.rs` | Various | ~500 `.unwrap()` instances | P1 |
| `crates/server/src/routes/run.rs` | 293, 303 | `.unwrap()` in handlers | P1 |
| `crates/core/src/tool.rs` | 773, 774 | `.unwrap()` in tests | P2 |
| `tests/src/security_tests.rs` | 162 | Failing test | P0 |
| `tests/src/tool_registry_audit_tests.rs` | 63 | Failing test | P0 |

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
```

---

## 10. Technical Debt Summary

| Category |Est. Effort | Items |
|----------|------------|-------|
| Error Handling | 2-3 weeks | 3516 unwraps, LlmError migration |
| Testing | 2 weeks | Coverage gaps in agent, tools, cli |
| Pattern Adoption | 2 weeks | Visibility audit, service layer |
| CI/CD | 0.5 day | Add llvm-cov gate |

**Total Estimated Debt:** 4-5 weeks

---

## 11. Cross-References

- [PRD: Rust Conventions Compliance](./iterations/iteration-27/prd_r27.md)
- [Gap Analysis](./iterations/iteration-27/gap-analysis.md)
- [01-core-architecture.md](../01-core-architecture.md)
- [02-agent-system.md](../02-agent-system.md)
- [03-tools-system.md](../03-tools-system.md)
- [16-test-plan.md](../16-test-plan.md)
- [17-rust-test-implementation-roadmap.md](../17-rust-test-implementation-roadmap.md)
- [18-crate-by-crate-test-backlog.md](../18-crate-by-crate-test-backlog.md)
- [19-implementation-plan.md](../19-implementation-plan.md)

---

## 12. Change Log

| Version | Date | Changes |
|---------|------|---------|
| v27 | 2026-04-17 | Initial spec based on gap analysis; FR-001 to FR-014 added |
