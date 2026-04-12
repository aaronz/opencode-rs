# Task List v8

**Version:** 8.0
**Generated:** 2026-04-12
**Based on:** Spec v8 (Iteration 8 gap analysis)
**Status:** Draft

---

## Priority Legend

| Priority | Description |
|----------|-------------|
| P0 | **BLOCKING** - Must fix before any release |
| P1 | **IMPORTANT** - Should fix before release |
| P2 | **NICE TO HAVE** - Can defer to post-release |
| DC | **DEAD CODE** - Cleanup, low severity |

---

## P0 Tasks (BLOCKING - Must Fix)

### P0-8: ✅ Done

| Attribute | Value |
|-----------|-------|
| **Priority** | P0 |
| **Severity** | CRITICAL |
| **Module** | permission |
| **File** | `crates/permission/src/models.rs:28` |
| **Issue** | `intersect()` function has unreachable pattern that fails clippy |
| **Build Impact** | `cargo clippy --all -- -D warnings` fails |
| **Status** | ✅ Done |

**Task:**
- [x] Fix the `intersect()` function pattern matching logic
- [x] Verify: `cargo clippy --all -- -D warnings` passes
- [x] Verify: `cargo test -p opencode-permission` passes

**Pattern Error:**
```
error: unreachable pattern
28 |             (AgentPermissionScope::ReadOnly, _) | (_, AgentPermissionScope::ReadOnly) => {
   |                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no value can reach this
```

---

### P0-new-2: ✅ Done

| Attribute | Value |
|-----------|-------|
| **Priority** | P0 |
| **Severity** | BLOCKING |
| **Module** | cli |
| **File** | `crates/cli/src/desktop.rs`, `crates/cli/src/webview.rs`, `crates/server/src/lib.rs` |
| **PRD Reference** | FR-015, PRD 13 |
| **Issue** | WebView is stub-only, not integrated with app lifecycle |
| **Phase Impact** | Phase 4 (Interface Implementations) |
| **Status** | ✅ Done |

**Task:**
- [x] Implement actual WebView component per PRD 13
- [x] Connect WebView to desktop mode properly
- [x] Share state between TUI and WebView via oneshot channel
- [x] Verify: Desktop mode starts without errors
- [x] Verify: WebView renders correctly

**Implementation Details:**
- Added `WebViewManager` with `close_receiver()` to notify when WebView closes
- Added `run_server_with_shutdown()` in server module for coordinated shutdown
- Updated desktop.rs to use `tokio::select!` to wait for WebView close, server shutdown, or Ctrl+C
- When WebView window is closed, it signals the server to shut down gracefully

---

## P1 Tasks (IMPORTANT - Should Fix)

### P1-3: Deprecated Fields Removal

| Attribute | Value |
|-----------|-------|
| **Priority** | P1 |
| **Module** | config |
| **PRD Reference** | FR-003 |
| **Issue** | Deprecated fields remain in codebase |
| **Status** | 🚧 In Progress |

**Subtasks:**

| Field | Severity | Status | Task |
|-------|----------|--------|------|
| `mode` | Medium | 🚧 In Progress | Complete removal of `AgentMode` enum and `mode` field |
| `tools` | Medium | 📋 Todo | Plan removal after migration path |
| `theme` | Low | 📋 Todo | Confirm moved to tui.json |
| `keybinds` | Low | 📋 Todo | Confirm moved to tui.json |

**Task:**
- [ ] Complete `mode` field removal (in progress)
- [ ] Add deprecation warnings for v4.0 removal
- [ ] Document migration path for `tools` field
- [ ] Verify all `mode` usages removed from codebase

---

### P1-2: ✅ Done

| Attribute | Value |
|-----------|-------|
| **Priority** | P1 |
| **Module** | config |
| **PRD Reference** | FR-003 |
| **Issue** | Circular references in `{env:VAR}` and `{file:PATH}` expansion not detected |
| **Status** | ✅ Done |

**Task:**
- [x] Add detection algorithm for circular references
- [x] Add error message for circular expansion
- [x] Add tests for circular expansion scenarios

---

### P1-9: ✅ Done

| Attribute | Value |
|-----------|-------|
| **Priority** | P1 |
| **Module** | cli |
| **PRD Reference** | FR-015 |
| **Issue** | Cross-interface session synchronization is partial |
| **Status** | ✅ Done |

**Task:**
- [x] Design session sharing mechanism
- [x] Implement cross-interface sync
- [x] Add tests for session sharing

**Implementation Notes:**
Session sharing mechanism already implemented in `session_sharing.rs`. Fixed pre-existing UUID bugs in crash_recovery tests that were causing test failures. All 121+ session tests now pass.

---

### CLI Test Failures: test_prompt_history_persistence

| Attribute | Value |
|-----------|-------|
| **Priority** | P1 |
| **Module** | cli |
| **File** | `crates/cli/tests/e2e_prompt_history.rs` |
| **Issue** | Assertion failed in history persistence test |
| **Status** | ❌ Failing |

**Task:**
- [ ] Investigate root cause of persistence failure
- [ ] Fix history persistence logic
- [ ] Verify: `cargo test test_prompt_history_persistence` passes

---

### CLI Test Failures: test_prompt_history_navigation

| Attribute | Value |
|-----------|-------|
| **Priority** | P1 |
| **Module** | cli |
| **File** | `crates/cli/tests/e2e_prompt_history.rs` |
| **Issue** | `history.len() >= 3` assertion failed |
| **Status** | ❌ Failing |

**Task:**
- [ ] Investigate root cause of navigation logic issue
- [ ] Fix history navigation
- [ ] Verify: `cargo test test_prompt_history_navigation` passes

---

## P2 Tasks (Nice to Have - Can Defer)

### Core Architecture Improvements

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-1 | Project VCS worktree root distinction | core | 📋 Deferred |
| P2-2 | Workspace path validation | core | ✅ Done |

### Tool System

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-4 | Deterministic collision resolution | tools | ✅ Done |
| P2-5 | Result caching invalidation | tools | ✅ Done |

### MCP System

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-6 | Per-server OAuth token storage | mcp | ✅ Done |
| P2-7 | Context cost warnings | mcp | ✅ Done |

### LSP System

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-8 | Experimental LSP tool testing | lsp | ✅ Done |

### Server API

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-9 | API error shape consistency | server | ✅ Done |

### Plugin System

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-10 | Plugin cleanup/unload | plugin | ✅ Done |

### TUI System

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-11 | Shell prefix (`!`) handler | tui | ✅ Done |
| P2-12 | Home view completion | tui | ✅ Done |

### Storage

| ID | Task | Module | Status |
|----|------|--------|--------|
| P2-3 | Compaction shareability verification | storage | ✅ Done |

---

## DC Tasks (Dead Code Cleanup)

| ID | Item | Module | File | Severity | Task |
|----|------|--------|------|----------|------|
| DC-1 | Unused `Message` import | core | crash_recovery.rs:1 | Low | Remove unused import |
| DC-2 | Unused `SecretStorage` methods | core | secret_storage.rs:36 | Low | Remove or use |
| DC-3 | Unused `e` variable | tools | lsp_tool.rs:311,526,626,783 | Low | Rename to `_e` |
| DC-4 | Unused `body` variable | git | github.rs:566 | Low | Rename to `_body` |
| DC-5 | `open_browser` function unused | cli | desktop.rs:141 | Low | Remove or use ✅ |
| DC-6 | `format_time_elapsed` function unused | tui | app.rs:534 | Low | Remove or use ✅ |
| DC-7 | Unused `complete` variable | cli | mcp_auth.rs:216 | Low | Rename to `_complete` ✅ |
| DC-8 | Unused `models_url` function | llm | ollama.rs | Low | Remove or use |
| DC-9 | Unused `ChatStreamChunk` struct | llm | ollama.rs | Low | Remove or use |
| DC-10 | Unused `role` field | llm | ollama.rs:48 | Low | Remove or use |

---

## Release Gate Checklist

### Gate 0: Project Foundation

| Criteria | Status | Verification |
|----------|--------|---------------|
| Workspace builds | ✅ | `cargo build --release` |
| Tests run | ⚠️ | `cargo test` (2 CLI tests failing) |
| Clippy clean | ❌ | `cargo clippy --all -- -D warnings` |

### Gate 1: Authority Tests

| Criteria | Status | Test Count |
|----------|--------|------------|
| Core ownership tests | ✅ | 4 suites |
| Config precedence tests | ✅ | 4 suites |
| API route-group tests | ✅ | 4 suites |
| Session/message lifecycle tests | ✅ | 4 suites |

### Gate 2: Runtime Tests

| Criteria | Status | Test Count |
|----------|--------|------------|
| Agent primary invariant tests | ✅ | 5 suites |
| Subagent execution tests | ✅ | 5 suites |
| Tool registry tests | ✅ | 5 suites |
| Plugin hook order tests | ✅ | 5 suites |
| TUI plugin lifecycle tests | ✅ | 5 suites |

### Gate 3: Subsystem Tests

| Criteria | Status | Test Count |
|----------|--------|------------|
| MCP integration tests | ✅ | 4 suites |
| LSP integration tests | ✅ | 4 suites |
| Provider/model tests | ✅ | 4 suites |
| Skills discovery tests | ✅ | 4 suites |

### Gate 4: Interface Tests

| Criteria | Status | Notes |
|----------|--------|-------|
| Desktop/web smoke tests | 🚧 | Desktop WebView P0 blocks |
| ACP handshake tests | ✅ | Implemented |
| GitHub workflow tests | ✅ | Implemented |
| GitLab integration tests | ✅ | Implemented |

### Gate 5: Compatibility & Conventions

| Criteria | Status | Test Count |
|----------|--------|------------|
| Compatibility suite | ✅ | 3 suites |
| Conventions suite | ✅ | 23 tests |

### Gate 6: Non-Functional

| Criteria | Status | Notes |
|----------|--------|-------|
| Performance baselines | 🚧 | Partial - needs verification |
| Security tests | 🚧 | Partial - needs verification |
| Recovery tests | 🚧 | Partial - needs verification |

---

## Summary

| Category | Count | Completed | In Progress | Open | Deferred |
|----------|-------|----------|-------------|------|----------|
| P0 Blockers | 2 | 1 | 0 | 1 | 0 |
| P1 Issues | 6 | 0 | 1 | 5 | 5 |
| P2 Issues | 12 | 7 | 0 | 0 | 5 |
| DC Cleanup | 10 | 0 | 0 | 0 | 10 |

**Total Active Tasks:** 7 (1 P0 + 6 P1)

---

*Task list generated: 2026-04-12*
*Iteration: 8*
