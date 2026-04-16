# Implementation Plan: Rust Conventions Compliance (Iteration 27)

**Date:** 2026-04-17
**Iteration:** 27
**Status:** Active Development
**Target:** Full Rust conventions compliance per `.opencode/rules/rust/`

---

## 1. Priority Framework

| Priority | Definition | Timeline |
|----------|-------------|----------|
| **P0** | Blocking issues - must fix before any other work | Immediate |
| **P1** | High priority - should fix within 1-2 sprints | 1-2 weeks |
| **P2** | Medium priority - technical debt | 1 month |

---

## 2. P0 — Immediate (Blocking)

### FR-002: Fix 9 Failing Integration Tests

| Test | File | Root Cause |
|------|------|------------|
| `test_tool_registry_execute_read_tool` | `tests/src/tool_registry_audit_tests.rs` | Tool execution returns failure |
| `test_tool_registry_execute_write_tool` | `tests/src/tool_registry_audit_tests.rs` | Tool execution returns failure |
| `test_path_normalization_prevents_traversal` | `tests/src/security_tests.rs` | Path validation broken |
| `test_session_message_content_sanitization` | `tests/src/security_tests.rs` | XSS/injection prevention broken |
| `test_session_message_xss_prevention` | `tests/src/security_tests.rs` | XSS prevention broken |
| `test_write_tool_path_validation` | `tests/src/security_tests.rs` | Path validation broken |
| 4 more in `phase6_regression_tests` | `tests/src/` | Regression failures |

**Steps:**
1. Run `cargo test --all` to identify all failing tests
2. Analyze `test_tool_registry_execute_read_tool` - verify tool registry registration
3. Analyze `test_tool_registry_execute_write_tool` - verify write tool implementation
4. Fix `test_path_normalization_prevents_traversal` - add proper path normalization
5. Fix XSS prevention tests - validate sanitization logic
6. Run tests again to verify fixes

**Verification:** `cargo test --all` passes

---

## 3. P1 — Short-term (1-2 Weeks)

### FR-004: Convert `LlmError` to thiserror

**Files:** `crates/llm/src/error.rs`

**Changes:**
1. Add `#[derive(thiserror::Error)]` to `LlmError` enum
2. Convert manual `Display` implementations to `#[error(...)]` attributes
3. Convert `RetryConfig` errors to thiserror (lines 88-104)

**Target State:**
```rust
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("rate limit exceeded: {retry_after:?}")]
    RateLimitExceeded { retry_after: Option<u64> },
    #[error("provider error: {context}")]
    Provider { context: String },
    #[error("invalid response: {detail}")]
    InvalidResponse { detail: String },
}
```

**Verification:** `cargo check -p opencode-llm` and `cargo clippy -p opencode-llm`

---

### FR-005: Replace `.unwrap()` in Production Code

**Scope:** 3516 instances across all crates

**Priority Order:**
1. `crates/tools/src/` (~500 instances)
2. `crates/server/src/routes/` (~100 instances)
3. `crates/core/` (~50 instances)

**Approach:**
1. Run `grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l` to get baseline
2. For each `.unwrap()`:
   - Replace with `?` if Result-returning function
   - Replace with `.ok()` or `.ok_or()` if Option-returning
   - Add context with `.context()` or map_err
3. Verify with `cargo clippy --all -- -D warnings`

**Verification:** `grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l` shows reduction

---

### FR-003: Add `cargo-llvm-cov` CI Gate

**File:** CI configuration

**Changes:**
1. Add `cargo install cargo-llvm-cov` to CI setup
2. Add stage: `cargo llvm-cov --fail-under-lines 80`

**Verification:** Coverage reports generated on CI

---

## 4. P2 — Medium-term (1 Month)

### FR-006: Visibility Audit

**Scope:** All crates

**Steps:**
1. Review all `pub` items that could be `pub(crate)`
2. Ensure `lib.rs` re-exports only intended public API
3. Verify no implementation leaks through `pub` interfaces

**Files to audit:**
- `crates/core/src/lib.rs`
- `crates/storage/src/lib.rs`
- `crates/llm/src/lib.rs`
- `crates/tools/src/lib.rs`
- `crates/server/src/lib.rs`

---

### FR-007: Add mockall Dependency

**File:** `Cargo.toml` (workspace or test dependencies)

**Steps:**
1. Add `mockall = "0.12"` to test dependencies
2. Identify traits requiring mocking (e.g., `LLMProvider`, `Tool`)
3. Add `#[cfg(test)]` mock implementations

---

### FR-008: Server Route Error Refactoring

**File:** `crates/server/src/routes/`

**Changes:**
1. Create route-specific error enums using thiserror
2. Remove `.unwrap()` from route handlers
3. Consistent error wrapping across all routes

---

### FR-011: Expand Builder Pattern to Server Routes

**Scope:** `crates/server/routes/`

**Target:** Apply builder pattern similar to `ContextBuilder`, `ClientBuilder`

---

## 5. Verification Commands

```bash
# Format check
cargo fmt --all -- --check

# Clippy check
cargo clippy --all -- -D warnings

# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Count unwraps (exclude tests)
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l

# Coverage (requires cargo-llvm-cov)
cargo llvm-cov --fail-under-lines 80
```

---

## 6. Timeline Estimate

| Phase | Tasks | Estimate |
|-------|-------|----------|
| P0 | Fix 9 failing tests | 2-3 days |
| P1 | LlmError migration | 1.5 days |
| P1 | unwrap() elimination | 2-3 weeks |
| P1 | Coverage CI gate | 0.5 day |
| P2 | Visibility audit | 3 days |
| P2 | mockall integration | 2 days |
| P2 | Server route errors | 1 week |
| P2 | Builder pattern expansion | 1 week |

**Total:** 4-5 weeks

---

## 7. Success Criteria

- [ ] All 9 P0 tests pass
- [ ] `LlmError` uses thiserror
- [ ] Production code has zero `.unwrap()`/`.expect()`
- [ ] 80% line coverage enforced in CI
- [ ] All P1 and P2 items addressed
