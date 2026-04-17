# Implementation Plan v30: OpenCode RS (Iteration 30)

**Date:** 2026-04-17
**Iteration:** 30
**Focus:** ratatui-testing crate completion + PRD gap resolution

---

## 1. Priority Classification

### P0 - Critical (Must Fix)
| ID | Task | Status | Notes |
|----|------|--------|-------|
| FR-017 | Fix production unwrap() in all crates | Not Started | 3484+ instances found |

### P1 - High Priority
| ID | Task | Status | Notes |
|----|------|--------|-------|
| FR-018 | Add `cargo-llvm-cov` CI gate | Not Started | Enforce 80% coverage |
| FR-018 | Increase coverage to 80%+ across all crates | Not Started | |
| FR-028 | Visibility audit across all crates | ✅ Done | Added visibility_audit.rs tests |
| FR-024 | Define plugin API version stability policy | Not Started | |
| FR-025 | Verify WebSocket streaming capability | Not Started | ws.rs vs SSE |
| FR-026 | Add SDK documentation to CI | Not Started | `cargo doc --no-deps` |

### P2 - Medium Priority
| ID | Task | Status | Notes |
|----|------|--------|-------|
| FR-037 | Add `similar-asserts` to dev-dependencies | Optional | Visual snapshot diffing |

---

## 2. Focus Area: ratatui-testing (~95% Complete)

### Completed Modules (5/5 Core)
- [x] PtySimulator (FR-030) - 100%
- [x] BufferDiff (FR-031) - 100%
- [x] StateTester (FR-032) - 100%
- [x] TestDsl (FR-033) - 100%
- [x] CliTester (FR-034) - 100%
- [x] Snapshot Management (FR-035) - 100%
- [x] DialogRenderTester (FR-036) - Extra/Approved

### Known Limitations
- Windows PTY: Stub returns generic errors (documented limitation)

---

## 3. Implementation Order

### Phase 1: P0 - Critical
1. **FR-017**: Eliminate production `.unwrap()` and `.expect()` calls
   - Scope: All crates except test code
   - Approach: Replace with proper error propagation using `?`
   - Target: Zero unwrap in production code

### Phase 2: P1 - High Priority
2. **FR-025**: Verify WebSocket streaming capability
   - Check `routes/ws.rs` vs SSE in `routes/stream.rs`
   - Document actual capability

3. **FR-026**: Add SDK documentation to CI
   - Add `cargo doc --no-deps` to CI pipeline
   - Configure docs.rs publishing

4. **FR-024**: Define plugin API version stability policy
   - Document version stability guarantees
   - Define deprecation policy

5. **FR-028**: Visibility audit
   - Audit public APIs across all crates
   - Ensure consistent visibility modifiers

6. **FR-018**: Coverage enforcement
   - Add `cargo-llvm-cov` to CI
   - Set 80% threshold
   - Increase coverage incrementally

### Phase 3: P2 - Medium Priority
7. **FR-037**: Optional - Add `similar-asserts` dev dependency
   - Only if visual snapshot diffing is needed

---

## 4. Verification Commands

```bash
# Check formatting
cargo fmt --all -- --check

# Check clippy
cargo clippy --all -- -D warnings

# Run tests
cargo test -p ratatui-testing --all

# Build release
cargo build --release

# Check coverage (requires cargo-llvm-cov)
cargo llvm-cov --fail-under-lines 80

# Count unwraps (excluding tests)
grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l
```

---

## 5. Dependencies Status

### Required (All Present)
- ratatui 0.28 ✅
- crossterm 0.28 ✅
- portable-pty 0.8 ✅
- anyhow 1.0 ✅
- thiserror 2.0 ✅
- serde 1.0 ✅
- serde_json 1.0 ✅
- tempfile 3.14 ✅
- tokio 1.45 ✅

### Optional
- similar-asserts 1.5 ⚠️ Not present (optional)

---

## 6. CI Pipeline Status

| Stage | Command | Status |
|-------|---------|--------|
| Format check | `cargo fmt --all -- --check` | ✅ Pass |
| Clippy | `cargo clippy --all -- -D warnings` | ⚠️ Warnings exist |
| Unit tests | `cargo test --lib` | ✅ Pass |
| Integration | `cargo test --test '*'` | ✅ Pass |
| Build | `cargo build --release` | ✅ Pass |
| Coverage | `cargo llvm-cov --fail-under-lines 80` | ❌ Not in CI |
| Audit | `cargo audit` | ❌ Not in CI |
| Deny | `cargo deny check` | ❌ Not in CI |
| Benchmarks | `cargo bench` | ❌ Not in CI |
| Doc build | `cargo doc --no-deps` | ❌ Not in CI |

---

**End of Plan**
