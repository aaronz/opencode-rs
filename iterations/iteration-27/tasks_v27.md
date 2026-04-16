# Task List: Rust Conventions Compliance (Iteration 27)

**Date:** 2026-04-17
**Priority:** P0 > P1 > P2

---

## P0 — Immediate (Fix Before Continue)

| ID | Task | Status | Files | Notes |
|----|------|--------|-------|-------|
| P0-01 | Fix `test_tool_registry_execute_read_tool` | ✅ Done | `tests/src/tool_registry_audit_tests.rs` | Tool execution returns failure |
| P0-02 | Fix `test_tool_registry_execute_write_tool` | ✅ Done | `tests/src/tool_registry_audit_tests.rs` | Tool execution returns failure |
| P0-03 | Fix `test_path_normalization_prevents_traversal` | ✅ Done | `tests/src/security_tests.rs` | Path validation broken |
| P0-04 | Fix `test_session_message_content_sanitization` | ✅ Done | `tests/src/security_tests.rs` | XSS/injection prevention broken |
| P0-05 | Fix `test_session_message_xss_prevention` | TODO | `tests/src/security_tests.rs` | XSS prevention broken |
| P0-06 | Fix `test_write_tool_path_validation` | TODO | `tests/src/security_tests.rs` | Path validation broken |
| P0-07 | Fix 4 remaining phase6_regression_tests | TODO | `tests/src/` | Regression failures |
| P0-08 | Verify all tests pass with `cargo test --all` | TODO | - | After P0-01 through P0-07 |

---

## P1 — Short-term (1-2 Weeks)

### Error Handling

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P1-01 | Convert `LlmError` to thiserror | ✅ Done | `crates/llm/src/error.rs:4-17` | 1 day |
| P1-02 | Convert `RetryConfig` errors to thiserror | TODO | `crates/llm/src/error.rs:88-104` | 0.5 day |
| P1-03 | Audit `OpenCodeError` legacy variants | ✅ Done | `crates/core/src/error.rs` | 1 day |

### unwrap() Elimination

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P1-04 | Replace `.unwrap()` in `crates/tools/src/` | TODO | `crates/tools/src/*.rs` (~500) | 1 week |
| P1-05 | Replace `.unwrap()` in `crates/server/src/routes/` | TODO | `crates/server/src/routes/*.rs` (~100) | 2 days |
| P1-06 | Replace `.unwrap()` in `crates/core/` | TODO | `crates/core/src/*.rs` (~50) | 1 day |
| P1-07 | Verify zero `.unwrap()` with clippy | TODO | All crates | - |

### Coverage

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P1-08 | Add `cargo-llvm-cov` to CI pipeline | TODO | CI config | 0.5 day |
| P1-09 | Add coverage threshold `cargo llvm-cov --fail-under-lines 80` | TODO | CI config | - |
| P1-10 | Increase `crates/agent/` coverage from 45% to 80% | TODO | `crates/agent/` | 1 week |
| P1-11 | Increase `crates/tools/` coverage from 50% to 80% | TODO | `crates/tools/` | 1 week |

---

## P2 — Medium-term (1 Month)

### Visibility

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P2-01 | Audit `pub` vs `pub(crate)` in `crates/core/` | TODO | `crates/core/src/lib.rs` | 0.5 day |
| P2-02 | Audit `pub` vs `pub(crate)` in `crates/llm/` | TODO | `crates/llm/src/lib.rs` | 0.5 day |
| P2-03 | Audit `pub` vs `pub(crate)` in `crates/storage/` | TODO | `crates/storage/src/lib.rs` | 0.5 day |
| P2-04 | Audit `pub` vs `pub(crate)` in `crates/tools/` | TODO | `crates/tools/src/lib.rs` | 0.5 day |
| P2-05 | Audit `pub` vs `pub(crate)` in `crates/server/` | TODO | `crates/server/src/lib.rs` | 0.5 day |

### Testing Infrastructure

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P2-06 | Add `mockall` dependency | TODO | `Cargo.toml` | 0.5 day |
| P2-07 | Add mock implementations for `LLMProvider` trait | TODO | `tests/src/` | 1 day |
| P2-08 | Add mock implementations for `Tool` trait | TODO | `tests/src/` | 1 day |

### Server Routes

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P2-09 | Create route-specific error enums in `crates/server/src/routes/` | TODO | `crates/server/src/routes/` | 2 days |
| P2-10 | Remove `.unwrap()` from route handlers | TODO | `crates/server/src/routes/run.rs` | 1 day |
| P2-11 | Expand builder pattern to server routes | TODO | `crates/server/routes/` | 1 week |

### Repository/Service Patterns

| ID | Task | Status | Files | Estimate |
|----|------|--------|-------|----------|
| P2-12 | Audit repository trait abstraction in `crates/llm/` | TODO | `crates/llm/` | 1 week |
| P2-13 | Add service layer to server routes | TODO | `crates/server/routes/` | 1 week |

---

## Verification Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo test --all` passes (P0 tests fixed)
- [ ] `cargo llvm-cov --fail-under-lines 80` passes
- [ ] `LlmError` uses `#[derive(thiserror::Error)]`
- [ ] Zero `.unwrap()`/`.expect()` in production code

---

## Progress Tracking

| Metric | Baseline | Target | Current |
|--------|----------|--------|---------|
| P0 Tests Passing | 0/9 | 9/9 | 0/9 |
| LlmError thiserror | No | Yes | No |
| unwrap() count | 3516 | 0 | 3516 |
| Coverage - core | ~60% | 80% | ~60% |
| Coverage - llm | ~55% | 80% | ~55% |
| Coverage - tools | ~50% | 80% | ~50% |
| Coverage - agent | ~45% | 80% | ~45% |
| Coverage - server | ~65% | 80% | ~65% |
| Overall Compliance | ~70% | 100% | ~70% |
