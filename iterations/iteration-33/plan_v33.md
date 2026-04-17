# Implementation Plan v33 - opencode-rs

**Project:** opencode-rs (Rust Implementation)
**Version:** 33
**Date:** 2026-04-17
**Status:** Based on Spec v33 + Gap Analysis

---

## 1. Priority Overview

| Priority | Count | Focus |
|----------|-------|-------|
| **P0** | 4 | Critical blockers - auth flow bugs |
| **P1** | 5 | High priority - missing commands/features |
| **P2** | 3 | Medium priority - integration completion |

---

## 2. P0 Critical Blockers (Required Before Release)

### P0-1: FR-007 - ConnectMethodDialog Fix
**File:** `opencode-rust/crates/tui/src/dialogs/connect_method.rs:21-28`
**Issue:** Non-OpenAI providers get empty auth methods

**Fix:** Show "API Key" method for all API-key providers. For OAuth-only providers (Google, Copilot), show "Not yet implemented" message and do NOT close on Enter.

**Acceptance:**
- [ ] All API-key providers show "API Key" option
- [ ] Empty dialog does NOT close silently on Enter
- [ ] Shows explanatory message for unimplemented auth

### P0-2: FR-008 - API Key Input Dialog
**Module:** `opencode-rust/crates/tui/src/dialogs/`
**Issue:** No API key input dialog exists

**Fix:** Implement `ApiKeyInputDialog` for API key entry with masking and validation.

**Acceptance:**
- [ ] API key input dialog renders correctly
- [ ] Input masked (dots, not plain text)
- [ ] API key validated before saving

### P0-3: FR-001 - Dynamic Provider Registry
**File:** `opencode-rust/crates/cli/src/cmd/providers.rs:94`
**Issue:** Hardcoded `["openai", "anthropic", "ollama"]`

**Fix:** Read providers dynamically from provider registry, display all 18 providers.

**Acceptance:**
- [ ] `providers` CLI lists all registered providers
- [ ] Provider list matches ConnectProviderDialog options

### P0-4: TD-1 (Same as FR-001)
See P0-3 above.

---

## 3. P1 High Priority (Next Sprint)

### P1-1: FR-002 - Expanded Model Catalog
**File:** `opencode-rust/crates/llm/src/models.rs`
**Issue:** Only ~17 models vs 89 in original (81% gap)

**Target:** 50+ models minimum

**Models to add:**
- github-copilot/* models
- opencode/gpt-5-nano
- opencode/minimax-m2.5-free
- opencode/nemotron-3-super-free
- google/antigravity-* models
- kimi/* models
- z.ai/* models

### P1-2: FR-003 - Shell Completion Command
**Module:** `opencode-rust/crates/cli/`
**Issue:** `opencode completion` not implemented

**Command Structure:**
```
opencode-rs completion [SHELL]
```

**Shells:** bash, zsh, fish, PowerShell

### P1-3: FR-004 - Plugin CLI Commands
**Module:** `opencode-rust/crates/cli/`
**Issue:** No `plugin` CLI command despite plugin crate existing

**Commands:**
- `opencode-rs plugin install <name>`
- `opencode-rs plugin list`
- `opencode-rs plugin remove <name>`
- `opencode-rs plugin search [query]`

### P1-4: FR-005 - PR Command Implementation
**File:** `opencode-rust/crates/cli/src/cmd/pr.rs:39-40`
**Issue:** Stub implementation prints debug only

**Commands:**
- `opencode-rs pr <number>` - Fetch and display PR details
- `opencode-rs pr checkout <number>` - Checkout PR branch
- `opencode-rs pr list` - List recent PRs

### P1-5: FR-009 - ConnectMethodDialog Tests
**File:** `opencode-rust/crates/tui/src/dialogs/connect_method.rs`
**Issue:** No unit tests despite being P0 user-facing dialog

**Required tests:**
- Empty methods + Enter closes dialog
- Empty methods + navigation doesn't panic
- Single item navigation works correctly
- Empty state renders visible message

---

## 4. P2 Medium Priority

### P2-1: FR-006 - GitHub Integration Completion
**File:** `opencode-rust/crates/cli/src/cmd/github.rs:80-86`
**Issue:** Login, RepoList, IssueList print TODO

**Commands:**
- `GitHubAction::Login` - OAuth flow
- `GitHubAction::RepoList` - List repositories
- `GitHubAction::IssueList { repo }` - List issues

### P2-2: FR-010 - ConnectModelDialog Tests
**File:** `opencode-rust/crates/tui/src/dialogs/connect_model.rs`
**Issue:** No tests for model selection dialog

**Required tests:**
- Empty model list handling
- Model selection with keyboard
- Model filtering/search
- Empty state renders with border

### P2-3: FR-011 - OAuth Flows for Google/Copilot
**Module:** `opencode-rust/crates/llm/src/auth_layered/`
**Issue:** Google, Copilot OAuth not implemented

**Providers needing OAuth:**
- Google
- GitHub Copilot

---

## 5. Technical Debt

| ID | Issue | Severity | Module |
|----|-------|----------|--------|
| TD-1 | Hardcoded provider list | High | providers.rs:94 |
| TD-2 | `connect_method.rs:105` modulo by 1 | Medium | tui/ |
| TD-3 | GitHub TODOs | Medium | github.rs:80-86 |
| TD-4 | PR stub | Medium | pr.rs:40 |

**TD-2 Detail:** Line 105 `self.selected_index = (self.selected_index + 1) % self.methods.len().max(1)` - when len is 0, max(1) returns 1, so modulo by 1 always gives 0. This is incorrect logic.

---

## 6. Implementation Order

1. **P0-1 (FR-007):** ConnectMethodDialog Fix - Quick fix, unblocks users
2. **P0-3 (FR-001):** Dynamic Provider Registry - Simple change
3. **P0-2 (FR-008):** API Key Input Dialog - Required for auth flow
4. **P1-5 (FR-009):** ConnectMethodDialog Tests - Low effort, high value
5. **P1-1 (FR-002):** Expanded Model Catalog - Large change, start early
6. **P1-2 (FR-003):** Shell Completion - Medium effort
7. **P1-3 (FR-004):** Plugin CLI - Medium effort
8. **P1-4 (FR-005):** PR Command - Medium effort
9. **P2-1 (FR-006):** GitHub Integration - Larger effort
10. **P2-2 (FR-010):** ConnectModelDialog Tests - Low effort
11. **P2-3 (FR-011):** OAuth Flows - Larger effort

---

## 7. Files to Modify

| File | Changes |
|------|---------|
| `opencode-rust/crates/tui/src/dialogs/connect_method.rs` | Fix auth method logic, add tests |
| `opencode-rust/crates/tui/src/dialogs/api_key_input.rs` | New file for API key dialog |
| `opencode-rust/crates/cli/src/cmd/providers.rs` | Dynamic provider registry |
| `opencode-rust/crates/llm/src/models.rs` | Expand model catalog |
| `opencode-rust/crates/cli/src/cmd/completion.rs` | New file for completion command |
| `opencode-rust/crates/cli/src/cmd/plugin.rs` | New file for plugin commands |
| `opencode-rust/crates/cli/src/cmd/pr.rs` | Implement PR functionality |
| `opencode-rust/crates/cli/src/cmd/github.rs` | Complete GitHub integration |
| `opencode-rust/crates/tui/src/dialogs/connect_model.rs` | Add tests |

---

*Plan Version: 33*
*Generated: 2026-04-17*
