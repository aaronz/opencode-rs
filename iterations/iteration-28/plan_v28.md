# Implementation Plan v28: Rust Conventions Compliance

**Date:** 2026-04-17  
**Iteration:** 28  
**Status:** Active  
**Priority:** P0 items must be completed before release

---

## 1. P0 Critical (Blocker Issues) - MUST FIX

### 1.1 Production unwrap() Elimination

| ID | Action | File | Line | Status |
|----|--------|------|------|--------|
| P0-001 | Fix `unwrap()` on Option | `crates/tools/src/edit.rs` | 159 | Not Started |
| P0-002 | Fix `unwrap()` on Option | `crates/tools/src/web_search.rs` | 70 | Not Started |
| P0-003 | Audit remaining production code for `.unwrap()` | All crates | - | Not Started |

**Fix Pattern:**
```rust
// BEFORE (P0-001)
let idx = index.unwrap();

// AFTER
let idx = index.ok_or_else(|| MyError::InvalidIndex("index not found".into()))?;
```

### 1.2 Server Route Error Refactoring

| ID | Action | File | Status |
|----|--------|------|--------|
| P0-004 | Convert String errors to thiserror enums | `crates/server/src/routes/*.rs` | Not Started |
| P0-005 | Add SAFETY comments to unsafe blocks | `crates/server/src/routes/validation.rs:237,256` | Not Started |

### 1.3 Integration Test Fixes

| ID | Action | File | Status |
|----|--------|------|--------|
| P0-006 | Fix `test_tool_registry_execute_read_tool` | `tests/src/` | Failing |
| P0-007 | Fix `test_tool_registry_execute_write_tool` | `tests/src/` | Failing |
| P0-008 | Fix `test_path_normalization_prevents_traversal` | `tests/src/` | Failing |
| P0-009 | Fix `test_session_message_content_sanitization` | `tests/src/` | Failing |
| P0-010 | Fix `test_session_message_xss_prevention` | `tests/src/` | Failing |
| P0-011 | Fix `test_write_tool_path_validation` | `tests/src/` | Failing |

---

## 2. P1 High Priority

### 2.1 Visibility Audit

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| P1-001 | Visibility audit `crates/core/src/` | 3 days | Not Started |
| P1-002 | Reduce `pub` to `pub(crate)` in core | 2 days | Not Started |
| P1-003 | Visibility audit `crates/tools/src/` | 1 day | Not Started |
| P1-004 | Reduce `pub` to `pub(crate)` in tools | 1 day | Not Started |
| P1-005 | Visibility audit `crates/agent/src/` | 2 days | Not Started |

### 2.2 Test Coverage Improvement

| ID | Action | Target | Delta | Status |
|----|--------|--------|-------|--------|
| P1-006 | Increase `crates/core/` coverage to 80%+ | 80% | +20% | Not Started |
| P1-007 | Increase `crates/tools/` coverage to 80%+ | 80% | +30% | Not Started |
| P1-008 | Increase `crates/agent/` coverage to 80%+ | 80% | +35% | Not Started |
| P1-009 | Increase `crates/server/` coverage to 80%+ | 80% | +40% | Not Started |
| P1-010 | Increase `crates/llm/` coverage to 80%+ | 80% | +25% | Not Started |

### 2.3 Error Handling

| ID | Action | Status |
|----|--------|--------|
| P1-011 | Migrate legacy error variants to typed errors | Partial |
| P1-012 | Add `cargo-llvm-cov` CI gate | Not Started |

---

## 3. P2 Medium Priority

| ID | Action | Estimate | Status |
|----|--------|---------|--------|
| P2-001 | Add mockall dependency for trait mocking | 2 days | Not Started |
| P2-002 | Add service layer to server routes | 1 week | Not Started |
| P2-003 | Expand builder pattern to server routes | 1 week | Not Started |
| P2-004 | Add SAFETY comments to unsafe blocks | 1 day | Not Started |
| P2-005 | Add newtypes: `SessionId`, `UserId`, `ProjectId` | 2 days | Not Started |
| P2-006 | Audit enum matching for exhaustiveness | 2 days | Not Started |
| P2-007 | Add rstest dependency for parameterized tests | 2 days | Not Started |
| P2-008 | Seal additional traits | 1 day | Not Started |
| P2-009 | Audit `let mut` usage in `crates/core/src/session.rs` | 1 day | Not Started |
| P2-010 | Extend repository pattern to all data access | 1 week | Not Started |

---

## 4. CI Gate Status

| Gate | Command | Fail Condition | Status |
|------|---------|----------------|--------|
| Format | `cargo fmt --all -- --check` | Exit != 0 | ✅ Pass |
| Clippy | `cargo clippy --all -- -D warnings` | Warnings | 🔄 In progress |
| Unit tests | `cargo test --lib` | Failures | ✅ Pass |
| Coverage | `cargo llvm-cov --fail-under-lines 80` | Below 80% | ❌ Fail |
| Security | `cargo audit` | CVEs | Not run |
| Deny | `cargo deny check` | Advisories | Not run |

---

## 5. Technical Debt Summary

| Category | Est. Effort | Priority |
|----------|------------|----------|
| Error Handling (unwrap elimination) | 2-3 weeks | P0 |
| Server route errors | 1 week | P0 |
| Visibility audit | 1 week | P1 |
| Test coverage | 3 weeks | P1 |
| Pattern adoption | 2 weeks | P2 |
| CI/CD | 0.5 day | P1 |

**Total Estimated Debt:** 5-6 weeks

---

## 6. Verification Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Check clippy
cargo clippy --all -- -D warnings

# Count unwraps (excluding tests)
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l

# Audit pub visibility
grep -n "pub fn\|pub struct\|pub enum" crates/core/src/ | head -50

# Run tests
cargo test --all

# Check coverage
cargo llvm-cov --fail-under-lines 80
```

---

## 7. Implementation Sequence

1. **Week 1-2: P0 Fixes**
   - Fix P0-001 through P0-011 (unwrap(), server errors, failing tests)

2. **Week 3-4: P1 Foundation**
   - Visibility audit and fixes
   - Add CI coverage gate

3. **Week 5-6: Coverage Expansion**
   - Target each crate to 80%+
   - Integration test improvements

4. **Week 7+: P2 Patterns**
   - Service layer, newtypes, builders
   - Exhaustive enum matching
