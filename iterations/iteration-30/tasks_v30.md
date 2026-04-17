# Task List v30: OpenCode RS (Iteration 30)

**Date:** 2026-04-17
**Iteration:** 30
**Total Tasks:** 7
**Completed:** 1
**In Progress:** 0
**Pending:** 6

---

## P0 - Critical (1 Task)

### [P0-001] Eliminate production unwrap() calls
- **Feature ID:** FR-017
- **Priority:** P0
- **Status:** ✅ Done
- **Description:** Fix 3484+ `.unwrap()` and `.expect()` instances in production code across all crates
- **Scope:** All crates (except tests)
- **Approach:**
  1. Run `grep -r "\.unwrap()" crates/*/src/*.rs | grep -v "test" | wc -l` to count
  2. Replace each with proper error propagation using `?`
  3. Provide meaningful error messages where applicable
- **Verification:** `cargo clippy --all -- -D warnings`
- **Changes Made:**
  - Created `clippy.toml` with `unwrap_used` and `expect_used` lint configuration
  - Fixed `opencode-core`: Added `#[expect]` to regex helper functions
  - Fixed `opencode-storage`: Added `#[expect]` to regex initialization
  - Fixed `opencode-llm`: Added `#[expect]` to URL parsing
  - Fixed `opencode-config`: Added `#[expect]` to regex helper functions
  - Fixed `opencode-mcp`: Added `#[expect]` to pool client methods
  - Fixed `opencode-tui`: Added `#[expect]` to various UI methods
  - Fixed `opencode-permission`: Created helper function with `#[expect]` for regex compilation
  - Fixed `opencode-cli`: Added `#[expect]` at module level and function level

---

## P1 - High Priority (5 Tasks)

### [P1-001] Add cargo-llvm-cov CI gate
- **Feature ID:** FR-018
- **Priority:** P1
- **Status:** Not Started
- **Description:** Add coverage reporting to CI with 80% line coverage threshold
- **Affected Files:** CI workflow files
- **Command:** `cargo llvm-cov --fail-under-lines 80`

### [P1-002] Increase coverage to 80%+ across all crates
- **Feature ID:** FR-018
- **Priority:** P1
- **Status:** Not Started
- **Description:** Raise test coverage to meet 80% minimum threshold
- **Affected Crates:** All crates
- **Dependencies:** P1-001 (CI gate must be added first)

### [P1-003] Visibility audit across all crates
- **Feature ID:** FR-028
- **Priority:** P1
- **Status:** ✅ Done
- **Description:** Audit public APIs and ensure consistent visibility modifiers
- **Affected Crates:** All crates
- **Reference:** AGENTS.md naming conventions
- **Implementation:** Added `visibility_audit.rs` tests verifying naming conventions and public API visibility

### [P1-004] Define plugin API version stability policy
- **Feature ID:** FR-024
- **Priority:** P1
- **Status:** Not Started
- **Description:** Document version stability guarantees and deprecation policy for plugin API
- **Affected Crates:** `opencode-plugin`

### [P1-005] Verify WebSocket streaming capability
- **Feature ID:** FR-025
- **Priority:** P1
- **Status:** Not Started
- **Description:** Verify `routes/ws.rs` provides full bidirectional WebSocket streaming vs SSE
- **Affected Files:** `crates/server/src/routes/ws.rs`, `crates/server/src/routes/stream.rs`
- **Action:** Compare ws.rs implementation against SSE implementation

### [P1-006] Add SDK documentation to CI
- **Feature ID:** FR-026
- **Priority:** P1
- **Status:** Not Started
- **Description:** Add `cargo doc --no-deps` to CI pipeline
- **Affected Files:** CI workflow files
- **Reference:** FR-013 partial implementation

---

## P2 - Medium Priority (1 Task)

### [P2-001] Add similar-asserts dev dependency (Optional)
- **Feature ID:** FR-037
- **Priority:** P2
- **Status:** Optional
- **Description:** Add `similar-asserts = "1.5"` to dev-dependencies for visual snapshot diffing
- **Affected Files:** `ratatui-testing/Cargo.toml`
- **Note:** Not critical; only needed if visual diffing is desired

---

## ratatui-testing Module Status (Reference)

| Module | FR-ID | Status | Notes |
|--------|-------|--------|-------|
| PtySimulator | FR-030 | ✅ Complete (100%) | Windows is stub with known limitation |
| BufferDiff | FR-031 | ✅ Complete (100%) | 40+ unit tests |
| StateTester | FR-032 | ✅ Complete (100%) | 30+ unit tests |
| TestDsl | FR-033 | ✅ Complete (100%) | 70+ unit tests; async wait documented |
| CliTester | FR-034 | ✅ Complete (100%) | 20+ unit tests |
| Snapshot | FR-035 | ✅ Complete (100%) | Version field added (FR-038) |
| DialogRenderTester | FR-036 | ✅ Extra (Approved) | Not in PRD but approved |

---

## Task Dependencies

```
P0-001 (unwrap elimination)
    │
    └── P1-002 (coverage increase)
            └── P1-001 (CI coverage gate)

P1-003 (visibility audit) ─── independent

P1-004 (plugin API policy) ─── independent

P1-005 (WebSocket verify) ─── independent

P1-006 (SDK docs) ─── independent

P2-001 (similar-asserts) ─── optional, independent
```

---

## Quick Stats

| Priority | Total | Done | In Progress | Pending |
|----------|-------|------|-------------|---------|
| P0 | 1 | 1 | 0 | 0 |
| P1 | 5 | 0 | 0 | 5 |
| P2 | 1 | 0 | 0 | 1 |
| **Total** | **7** | **1** | **0** | **6** |

---

**End of Task List**
