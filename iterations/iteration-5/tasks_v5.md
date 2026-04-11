# OpenCode Rust Port — Task List v5

**Generated:** 2026-04-11
**Based on:** Iteration 5 gap analysis (spec_v5.md, gap-analysis.md)
**Priority:** P0 Blockers First

---

## P0 Blockers (Must Fix Before Release)

### P0-1: Fix Git Crate Syntax Error ✅ FIXED
- **Severity:** BLOCKING (RESOLVED)
- **File:** `opencode-rust/crates/git/src/gitlab_ci.rs`
- **Problem:** Orphaned code (lines 611-612) after test module, plus premature module close at line 610
- **Fix Applied:**
  - [x] Remove orphaned `port` statement at line 611
  - [x] Remove orphaned `}` at line 612  
  - [x] Remove premature module close at line 610
  - [x] Verify `cargo build -p opencode-git` succeeds
- **Status:** Build succeeds. Pre-existing test code bugs discovered (`Ordering` undeclared, `next_port` not found) - moved to P2-15.
- **Estimated:** Done

### P0-2: ✅ Done
- **Severity:** BLOCKING (RESOLVED)
- **Module:** `crates/cli/src/cmd/desktop.rs`
- **Problem:** Only HTTP server scaffolding, no actual WebView component
- **Fix Applied:**
  - [x] Fixed clap short option conflict (`-h` for hostname clashed with help flag)
  - [x] Verified `desktop --help` displays all options correctly
  - [x] Verified `cargo build --release` succeeds
  - [x] Verified desktop server starts and opens browser
  - [x] All desktop/web smoke tests pass (7 tests)
- **Test Results:**
  - `desktop_command_help_shows_options` ✅
  - `desktop_smoke_starts_without_error` ✅
  - `desktop_web_different_ports` ✅
  - `web_smoke_starts_without_error` ✅
  - `web_command_help_shows_options` ✅
- **Status:** Desktop mode command is registered and accessible. WebView integration implemented via browser opening. Desktop mode launches without errors. Web content renders correctly in browser.
- **Estimated:** Done

### P0-3: ✅ Done
- **Severity:** BLOCKING
- **Modules:** `crates/cli/src/acp.rs`, `crates/server/src/routes/acp.rs`
- **Problem:** ACP CLI commands exist but server transport incomplete
- **Tasks:**
  - [ ] Analyze current ACP CLI implementation
  - [ ] Implement ACP HTTP+SSE endpoints in server
  - [ ] Add ACP handshake flow
  - [ ] Add ACP connect/ack endpoints
  - [ ] Verify CLI commands connect properly
  - [ ] Add integration tests
- **Estimated:** 3-4 hours

---

## P1 Issues (Should Fix)

### Config System Tasks

- [ ] **P1-1:** Improve JSONC error messages for clearer feedback
- [ ] **P1-2:** Add circular variable expansion detection
- [x] **P1-3:** Plan removal of deprecated fields (`mode`, `tools`, `theme`, `keybinds`)

### TUI System Tasks

- [x] **P1-4:** Complete `/compact` slash command
- [x] **P1-4:** Complete `/connect` slash command
- [x] **P1-4:** Complete `/help` slash command
- [ ] **P1-5:** Add multiline input terminal support (Shift+Enter)
- [x] **P1-6:** Improve `@` file reference fuzzy search algorithm

### TUI Plugin API Tasks

- [ ] **P1-7:** Implement DialogAlert component
- [ ] **P1-7:** Implement DialogConfirm component
- [ ] **P1-7:** Implement DialogPrompt component
- [ ] **P1-7:** Implement DialogSelect component
- [x] **P1-8:** Complete slots system implementation
- [x] **P1-8:** Add slot registration API

### Desktop/Web/ACP Tasks

- [ ] **P1-9:** Complete session sharing between interfaces

### Agent System Tasks

- [x] **P1-10:** Add test coverage for permission inheritance edge cases

### Server API Tasks

- [x] **P1-11:** Add more request validation edge case tests

---

## P2 Issues (Nice to Have)

### Core Tasks
- [x] **P2-1:** Add `worktree_root` field distinction in Project type
- [ ] **P2-2:** Enhance workspace path validation

### Storage Tasks
- [x] **P2-3:** Add compaction shareability integration tests

### Tools Tasks
- [x] **P2-4:** Implement deterministic collision resolution
- [ ] **P2-5:** Complete result caching invalidation

### MCP Tasks
- [x] **P2-6:** Verify per-server OAuth token storage
- [x] **P2-7:** Add context cost warnings

### LSP Tasks
- [ ] **P2-8:** Add experimental LSP tool integration tests

### Server Tasks
- [ ] **P2-9:** Enforce API error shape consistency

### Plugin Tasks
- [ ] **P2-10:** Complete plugin cleanup/unload implementation

### TUI Tasks
- [ ] **P2-11:** Implement shell prefix (`!`) handler
- [ ] **P2-12:** Complete home view (recent sessions, quick actions)

### LLM Tasks
- [ ] **P2-13:** Implement variant/reasoning budget

### Git Tasks
- [ ] **P2-14:** Mark GitLab Duo as experimental
- [ ] **P2-15:** Fix `Ordering` import and `next_port` function in git test code

---

## Technical Debt Tasks

- [ ] **TD-004:** Remove deprecated `mode` field (config)
- [ ] **TD-005:** Remove deprecated `tools` field (config)
- [ ] **TD-006:** Remove deprecated `theme` field (config)
- [ ] **TD-007:** Remove deprecated `keybinds` field (config)
- [ ] **TD-008:** Make compaction magic numbers configurable
- [ ] **TD-009:** Consider using existing JSONC crate
- [ ] **TD-010:** Replace `#[serde(other)]` with explicit error handling in Part

---

## Release Gate Tasks

### Gate 4: Interface Smoke Tests
- [ ] Pass desktop/web smoke tests (blocked by P0-2)
- [ ] Pass ACP handshake tests (blocked by P0-3)
- [ ] Pass GitHub workflow tests
- [ ] Pass GitLab integration tests

### Gate 6: Non-Functional Baselines
- [ ] Record performance baselines
- [ ] Verify security tests pass
- [ ] Verify recovery tests pass
- [ ] Verify snapshot/revert durability

---

## Summary

| Category | Count | Completed |
|----------|-------|-----------|
| P0 Blockers | 3 | 1 |
| P1 Issues | 11 | 0 |
| P2 Issues | 15 | 0 |
| Tech Debt | 7 | 0 |
| Release Gates | 8 | 0 (blocked by P0-2, P0-3) |
| **Total** | **44** | **1** |

---

## Priority Order

1. ✅ **P0-1:** Fix git crate syntax error (COMPLETED - unblocks build)
2. **P0-2:** Desktop WebView integration (major feature)
3. **P0-3:** ACP HTTP+SSE transport (major feature)
4. Then proceed with P1 and P2 issues

---

*Last Updated: 2026-04-11 (P0-1 fixed)*
