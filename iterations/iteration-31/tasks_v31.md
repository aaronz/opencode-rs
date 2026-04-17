# Task List: ratatui-testing v0.1.0

**Project:** ratatui-testing
**Date:** 2026-04-17
**Iteration:** 31
**Total Tasks:** 5

---

## P1 Tasks (High Priority)

### TASK-P1-001: Fix PtySimulator read_output buffer drain issue
**Status:** ✅ Done
**Priority:** P1
**Issue ID:** GAP-001
**Estimated Time:** 15 minutes

**Description:**
The `read_output()` function in `src/pty.rs:130` breaks after the first successful read instead of continuing to drain the buffer. This causes tests to miss output that arrives in multiple chunks.

**Root Cause:**
```rust
Ok(n) => {
    buffer.extend_from_slice(&temp_buf[..n]);
    break;  // <-- Premature break
}
```

**Fix:**
Remove the `break` statement at line 130 to allow the loop to continue reading until timeout.

**Files Affected:**
- `src/pty.rs`

**Acceptance Criteria:**
- [ ] `read_output()` continues reading until timeout
- [ ] All buffered data is captured
- [ ] `cargo test -p ratatui-testing` passes
- [ ] PTY integration tests pass

**Verification Command:**
```bash
cd ratatui-testing && cargo test
```

---

## P2 Tasks (Medium Priority)

### TASK-P2-001: ✅ Done
**Status:** PENDING
**Priority:** P2
**Issue ID:** GAP-002
**Estimated Time:** 1 minute

**Description:**
PRD specification requires `similar-asserts = "1.5"` in dev-dependencies but Cargo.toml is missing this entry.

**Fix:**
Add `similar-asserts = "1.5"` to `[dev-dependencies]` in Cargo.toml.

**Files Affected:**
- `Cargo.toml`

**Acceptance Criteria:**
- [ ] Cargo.toml includes `similar-asserts = "1.5"` in dev-dependencies
- [ ] `cargo check` passes

---

### TASK-P2-002: Document DialogTester in PRD
**Status:** ✅ Done
**Priority:** P2
**Issue ID:** GAP-003
**Estimated Time:** 30 minutes

**Description:**
DialogTester module exists in `dialog_tester.rs` but is not documented in the PRD specification.

**Fix:**
Update `spec_v31.md` to include DialogRenderTester module documentation (FR-DIALOG-001).

**Files Affected:**
- `spec_v31.md` (or create gap-analysis update)

**Acceptance Criteria:**
- [x] PRD includes DialogRenderTester API documentation
- [x] FR-DIALOG-001 section added with Public API
- [x] Acceptance criteria for DialogTester documented

---

### TASK-P2-003: Remove dead code warnings
**Status:** PENDING
**Priority:** P2
**Issue ID:** GAP-004
**Estimated Time:** 5 minutes

**Description:**
Two functions in `dialog_tester.rs` have `#[allow(dead_code)]` decorators but are not used anywhere:
- `assert_render_result` at line 92
- `assert_empty_state` at line 100

**Fix:**
Either remove the functions if not needed, or integrate them into tests.

**Files Affected:**
- `src/dialog_tester.rs`
- `tests/dialog_tests.rs` (if adding usage)

**Acceptance Criteria:**
- [ ] No `#[allow(dead_code)]` decorators remain
- [ ] No compiler warnings about dead code
- [ ] All dialog tests still pass

---

### TASK-P2-004: Document snapshot.rs in PRD
**Status:** PENDING
**Priority:** P2
**Issue ID:** GAP-005
**Estimated Time:** 10 minutes

**Description:**
The `snapshot.rs` module exists and is functional but is not documented in the PRD specification.

**Fix:**
Add FR-SNAP-001 section to PRD documenting the snapshot module API.

**Files Affected:**
- `spec_v31.md` (or create gap-analysis update)

**Acceptance Criteria:**
- [ ] PRD includes snapshot module documentation
- [ ] FR-SNAP-001 section added with Public API
- [ ] File structure in PRD updated

---

## Task Summary

| Task ID | Priority | Description | Estimated Time | Status |
|---------|----------|-------------|---------------|--------|
| TASK-P1-001 | P1 | Fix PtySimulator read_output buffer drain | 15 min | ✅ Done |
| TASK-P2-001 | P2 | Add similar-asserts dev-dependency | 1 min | PENDING |
| TASK-P2-002 | P2 | Document DialogTester in PRD | 30 min | ✅ Done |
| TASK-P2-003 | P2 | Remove dead code warnings | 5 min | PENDING |
| TASK-P2-004 | P2 | Document snapshot.rs in PRD | 10 min | PENDING |

**Total Estimated Time:** 61 minutes

---

## Completion Checklist

- [x] TASK-P1-001 completed and verified
- [ ] TASK-P2-001 completed and verified
- [x] TASK-P2-002 completed and verified
- [ ] TASK-P2-003 completed and verified
- [ ] TASK-P2-004 completed and verified
- [ ] Full test suite passes: `cargo test -p ratatui-testing`
- [ ] Clippy passes: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Formatting correct: `cargo fmt --all -- --check`

---

*Task list version: 31.1*
*Generated: 2026-04-17*