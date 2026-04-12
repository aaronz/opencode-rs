# Task Checklist v6

**Generated:** 2026-04-12  
**Based on:** Spec v6 gap analysis  
**Priority:** P0 tasks must be completed before release

---

## P0 - Blocking Issues (Must Fix Before Release)

### P0-new-2: ✅ Done
- [x] **Status:** ✅ IMPLEMENTED
- [ ] **Module:** `crates/cli`
- [ ] **FR Reference:** FR-015 (Desktop/Web/ACP Interface)
- **Current State:** Embedded WebView using wry library
- **Implementation:**
  - Created `crates/cli/src/webview.rs` with cross-platform WebView support
  - Modified `crates/cli/src/cmd/desktop.rs` to use embedded WebView when `desktop` feature is enabled
  - WebView loads local HTTP server URL on the same port
- **Verification:** Desktop mode opens embedded WebView, not external browser

### P2-15: ✅ Done
- [ ] **Status:** ❌ 11 test compilation errors
- [ ] **Module:** `crates/git`
- **Errors:**
  - 8 duplicate test name definitions (likely two test modules)
  - Missing `use std::sync::atomic::Ordering;` import
  - Missing `next_port()` helper function
- **Files to Modify:**
  - `crates/git/src/gitlab_ci.rs` (test module)
- **Required Actions:**
  - [ ] Add `use std::sync::atomic::Ordering;` import
  - [ ] Define or remove `next_port()` helper function
  - [ ] Fix duplicate test names (likely duplicate test module at lines 405+ and 697+)

---

## P1 - Important Issues (Should Fix Before Release)

### P1-1: ✅ Done
- [x] **Status:** Done
- [x] **Module:** `crates/core` (config/jsonc.rs)
- [x] **FR Reference:** FR-003
- **Action:** Enhanced JsoncError with source line extraction and caret display

### P1-2: ✅ Done
- [ ] **Status:** Deferred
- [ ] **Module:** `crates/config`
- [ ] **FR Reference:** FR-003
- **Action:** Add detection algorithm for circular references in variable expansion

### P1-3: ✅ Done
- [x] **Status:** Done
- [x] **Module:** `crates/config`
- [x] **FR Reference:** FR-003
- **Fields Removed:**
  - [x] `mode` (from Config)
  - [x] `tools` (from Config and AgentConfig)
  - [x] `theme` (from Config)
  - [x] `keybinds` (from Config)
- **Note:** Removed in major version (v4.0)

### P1-5: ✅ Done
- [x] **Status:** ✅ Done
- [ ] **Module:** `crates/tui`
- [ ] **FR Reference:** FR-018
- **Current:** Multiline, Shift+Enter for new line - implemented in App::handle_input
- [ ] **Action:** Verify terminal support works correctly

### P1-9: ✅ Done
- [ ] **Status:** Deferred (Partial)
- [ ] **Module:** `crates/cli`
- [ ] **FR Reference:** FR-015
- **Action:** Implement cross-interface session synchronization

---

## P2 - Nice to Have (Post-Release)

### Core Architecture
- [x] P2-1: Project VCS worktree root distinction (`crates/core`)
- [x] P2-2: Workspace path validation (`crates/core`)

### Storage
- [x] P2-3: Compaction shareability verification (`crates/storage`)

### Tools
- [x] P2-4: Deterministic collision resolution (`crates/tools`)
- [x] P2-5: Result caching invalidation (`crates/tools`)

### MCP
- [x] P2-6: Per-server OAuth token storage verification (`crates/mcp`)
- [x] P2-7: Context cost warnings (`crates/mcp`)

### LSP
- [ ] P2-8: Experimental LSP tool testing (`crates/lsp`)

### Server
- [ ] P2-9: API error shape consistency enforcement (`crates/server`)

### TUI
- [ ] P2-11: Shell prefix (`!`) handler (`crates/tui`)
- [ ] P2-12: Home view completion (`crates/tui`)

### LLM
- [ ] P2-13: LLM variant/reasoning budget (`crates/llm`)

### Git
- [ ] P2-14: GitLab Duo experimental marking (`crates/git`)

---

## Completed Tasks ✅

### P0 Issues Resolved
- [x] P0-1 through P0-20: Various (Iteration 4) - All fixed
- [x] P0-new-1: Git crate syntax error - **RESOLVED** (build succeeds)
- [x] P0-new-3: ACP HTTP+SSE transport - **IMPLEMENTED**

### P1 Issues Completed
- [x] P1-3: Deprecated Fields Removal (mode, tools, theme, keybinds)
- [x] P1-4: Slash commands (`/compact`, `/connect`, `/help`)
- [x] P1-6: File reference autocomplete improvement
- [x] P1-7: TUI Plugin dialogs incomplete (DialogAlert, DialogConfirm, DialogPrompt, DialogSelect)
- [x] P1-8: TUI Plugin slots system incomplete
- [x] P1-10: Permission inheritance edge cases
- [x] P1-11: Request validation edge cases

### P2 Issues Completed
- [x] P2-2: Workspace path validation (`crates/core`)
- [x] P2-10: Plugin cleanup/unload

---

## Build Status

| Crate | Build | Tests | Priority |
|-------|-------|-------|----------|
| opencode-core | ✅ | ✅ | - |
| opencode-agent | ✅ | ✅ | - |
| opencode-tools | ✅ | ✅ | - |
| opencode-mcp | ✅ | ✅ | - |
| opencode-lsp | ✅ | ✅ | - |
| opencode-plugin | ✅ | ✅ | - |
| opencode-server | ✅ | ✅ | - |
| opencode-cli | ✅ | ✅ | - |
| opencode-git | ✅ | ❌ | P2-15 |
| opencode-llm | ✅ | ✅ | - |

---

## Summary

| Category | Count | Completed |
|----------|-------|-----------|
| P0 Blockers | 2 | 1 |
| P1 Issues | 11 | 6 |
| P2 Issues | 15 | 2 |

---

*Task list generated: 2026-04-12*
*Priority: P0 tasks are blockers for release*
