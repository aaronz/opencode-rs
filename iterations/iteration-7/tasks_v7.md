# Task Checklist v7

**Generated:** 2026-04-12  
**Based on:** Spec v7 gap analysis (Iteration 7)  
**Priority:** P0 tasks must be completed before release

---

## P0 - Blocking Issues (Must Fix Before Release)

### P0-new-2: ✅ Done
- [x] **Status:** ✅ Done (Iteration 7)
- [x] **Module:** `crates/cli`
- [x] **FR Reference:** FR-015 (Desktop/Web/ACP Interface)
- [x] **PRD Reference:** 13-desktop-web-interface.md
- **Implementation:**
  - Created `WebViewManager` struct with proper lifecycle management
  - WebView is created with `WebViewManager::new(url, title)`
  - Runs in background thread and properly shuts down via `stop()` and `wait_until_stopped()`
  - Drop implementation ensures cleanup
  - State shared via existing `ServerState` (event_bus, storage, etc.)
- **Verification:**
  - [x] WebView successfully spawns when desktop feature is enabled
  - [x] WebView properly shares state with TUI/server
  - [x] WebView loads local HTTP server URL correctly
  - [x] WebView integrates with desktop mode lifecycle (start/stop)

---

## P1 - Important Issues (Should Fix Before Release)

### P1-1: JSONC Error Messages Clarity ✅ Done
- [x] **Status:** Done
- [x] **Module:** `crates/core` (config/jsonc.rs)
- [x] **FR Reference:** FR-003
- [x] **Action:** Enhanced JsoncError with source line extraction and caret display

### P1-2: Circular Variable Expansion Detection ✅ Done
- [x] **Status:** ✅ Done
- [x] **Module:** `crates/config`
- [x] **FR Reference:** FR-003
- [x] **Action:** Circular variable expansion detection implemented - Fixed test bugs in opencode-core (incorrect variable paths), added integration tests in opencode-config/tests/circular_variable_tests.rs

### P1-3: Deprecated Fields Planning
- [ ] **Status:** Deferred (planning phase)
- [ ] **Module:** `crates/config`
- [ ] **FR Reference:** FR-003
- [ ] **Fields:** `mode`, `tools`, `theme`, `keybinds`
- [ ] **Action:** Plan removal in v4.0 major version

### P1-5: Multiline Input Terminal Support ✅ Done (Iteration 7)
- [x] **Status:** ✅ Done (Iteration 7)
- [x] **Module:** `crates/tui`
- [x] **FR Reference:** FR-018
- [x] **Implementation:** Shift+Enter for new line in `input_widget.rs`
- [x] **Verification:** Tests pass

### P1-9: Session Sharing Between Interfaces
- [ ] **Status:** Deferred (Partial)
- [ ] **Module:** `crates/cli`
- [ ] **FR Reference:** FR-015
- [ ] **Action:** Implement cross-interface session synchronization

---

## P2 - Nice to Have (Post-Release)

### Core Architecture
- [ ] P2-1: Project VCS worktree root distinction (`crates/core`)
- [x] P2-2: Workspace path validation (`crates/core`) ✅ Done

### Storage
- [x] P2-3: Compaction shareability verification (`crates/storage`) ✅ Done

### Tools
- [x] P2-4: Deterministic collision resolution (`crates/tools`) ✅ Done
- [ ] P2-5: Result caching invalidation (`crates/tools`)

### MCP
- [x] P2-6: Per-server OAuth token storage (`crates/mcp`) ✅ Done (Iteration 7)
- [x] P2-7: Context cost warnings (`crates/mcp`) ✅ Done (Iteration 7, context_cost.rs)

### LSP
- [ ] P2-8: Experimental LSP tool testing (`crates/lsp`)

### Server
- [ ] P2-9: API error shape consistency enforcement (`crates/server`)

### Plugin
- [x] P2-10: Plugin cleanup/unload (`crates/plugin`) ✅ Done (Iteration 7)

### TUI
- [ ] P2-11: Shell prefix (`!`) handler (`crates/tui`)
- [ ] P2-12: Home view completion (`crates/tui`)

### LLM
- [ ] P2-13: LLM variant/reasoning budget (`crates/llm`)

### Git
- [ ] P2-14: GitLab Duo experimental marking (`crates/git`) - marked experimental in docs
- [ ] **P2-15: Git test code cleanup** ❌ BUG (not feature gap)
  - [ ] Remove unused `next_port()` function (line 413)
  - [ ] Remove or use `GitLabMockServer` struct (line 706)
  - [ ] Clean up associated unused items: `new`, `handle_request`, `url`, `stop`

---

## Completed Tasks ✅

### P0 Issues Resolved
- [x] P0-1 through P0-20: Various (Iteration 4) - All fixed
- [x] P0-new-1: Git crate syntax error - **RESOLVED** (build succeeds)
- [x] P0-new-2: Desktop WebView Integration - **IMPLEMENTED** (Iteration 7)
- [x] P0-new-3: ACP HTTP+SSE transport - **IMPLEMENTED**

### P1 Issues Completed (Iteration 7)
- [x] **P1-5: Multiline input terminal support** - Shift+Enter for new line ✅

### P1 Issues Completed (Prior)
- [x] P1-1: JSONC error messages clarity
- [x] P1-3: Deprecated Fields Removal (mode, tools, theme, keybinds)
- [x] P1-4: Slash commands (`/compact`, `/connect`, `/help`)
- [x] P1-6: File reference autocomplete improvement
- [x] P1-7: TUI Plugin dialogs incomplete (DialogAlert, DialogConfirm, DialogPrompt, DialogSelect)
- [x] P1-8: TUI Plugin slots system incomplete
- [x] P1-10: Permission inheritance edge cases
- [x] P1-11: Request validation edge cases

### P2 Issues Completed (Iteration 7)
- [x] **P2-6: Per-server OAuth token storage** ✅
- [x] **P2-7: Context cost warnings** ✅ (context_cost.rs)
- [x] **P2-10: Plugin cleanup/unload** ✅

### P2 Issues Completed (Prior)
- [x] P2-2: Workspace path validation (`crates/core`)
- [x] P2-3: Compaction shareability verification
- [x] P2-4: Deterministic collision resolution

---

## Code Quality Warnings (Cleanup Tasks)

| ID | Item | File | Severity | Fix Action |
|----|------|------|----------|------------|
| CQ-1 | Unused `Message` import | core/crash_recovery.rs:1 | Low | Remove unused import |
| CQ-2 | Unused `SecretStorage` methods | core/config/secret_storage.rs:36 | Low | Remove or use 6 methods |
| CQ-3 | Unused `e` variable | tools/lsp_tool.rs:311,526,626,783 | Low | Prefix with `_e` |
| CQ-4 | Unused `body` variable | git/github.rs:566 | Low | Prefix with `_body` |
| CQ-5 | Unused `next_port` function | git/gitlab_ci.rs:413 | Low | Remove or use |
| CQ-6 | Unused `GitLabMockServer` | git/gitlab_ci.rs:706 | Low | Remove or use |
| CQ-7 | Unused imports | cli/src/cmd/quick.rs:5-6 | Low | Remove unused |
| CQ-8 | Unused `save_session_records` | cli/src/cmd/session.rs:42 | Low | Remove or use |
| CQ-9 | Unused `complete` variable | cli/src/cmd/mcp_auth.rs:216 | Low | Prefix with `_complete` |

---

## Build Status

| Crate | Build | Tests | Warnings | Priority |
|-------|-------|-------|----------|----------|
| opencode-core | ✅ | ✅ | 2 | Low (TD-012, TD-010) |
| opencode-agent | ✅ | ✅ | 0 | - |
| opencode-tools | ✅ | ✅ | 4 | Low (TD-013, CQ-3) |
| opencode-mcp | ✅ | ✅ | 3 | Low |
| opencode-lsp | ✅ | ✅ | 0 | - |
| opencode-plugin | ✅ | ✅ | 1 | Low |
| opencode-server | ✅ | ✅ | 2 | Low |
| opencode-cli | ✅ | ✅ | 5 | Low (CQ-7, CQ-8, CQ-9) |
| opencode-git | ✅ | ⚠️ | 1 | P2-15 (cleanup) |
| opencode-llm | ✅ | ✅ | 12 | Low |

---

## Summary

| Category | Count | Completed | Remaining |
|----------|-------|-----------|-----------|
| P0 Blockers | 1 | 1 (P0-new-2) | 0 |
| P1 Issues | 5 | 1 (P1-5) | 4 deferred |
| P2 Issues | 14 | 6 | 8 deferred + 1 bug |

---

## Release Blocker Summary

**All P0 blockers have been resolved!** 🎉
- **P0-new-2: Desktop WebView integration** - ✅ IMPLEMENTED (Iteration 7)

**All other issues are either completed or deferred to post-release.**

---

*Task list generated: 2026-04-12*
*Updated: 2026-04-12 - P0-new-2 completed*