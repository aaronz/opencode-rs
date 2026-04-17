# Task List v33 - opencode-rs

**Project:** opencode-rs (Rust Implementation)
**Version:** 33
**Date:** 2026-04-17
**Total Tasks:** 12
**P0:** 4 | **P1:** 5 | **P2:** 3

---

## P0 Critical Blockers

### Task P0-1: FR-007 - Fix ConnectMethodDialog
**Status:** TODO
**Priority:** P0
**Files:**
- `opencode-rust/crates/tui/src/dialogs/connect_method.rs`

**Changes:**
1. Line 21-28: Add "API Key" method for all non-OAuth-only providers
2. For OAuth-only providers (Google, Copilot), show "Not yet implemented" message
3. Ensure Enter on empty does NOT close silently

**Acceptance Criteria:**
- [ ] API-key providers show "API Key" option
- [ ] Empty dialog shows explanatory message
- [ ] Enter on empty closes with feedback

---

### Task P0-2: FR-008 - Implement API Key Input Dialog
**Status:** TODO
**Priority:** P0
**Files:**
- `opencode-rust/crates/tui/src/dialogs/api_key_input.rs` (NEW)

**Changes:**
1. Create `ApiKeyInputDialog` struct
2. Implement `Dialog` trait
3. Mask input (show dots, not plain text)
4. Add validation before saving
5. Wire into dialog flow: Provider Selection → Auth Method (API Key) → API Key Input → Validate → Save

**Acceptance Criteria:**
- [ ] Dialog renders correctly
- [ ] Input is masked
- [ ] Validation before saving
- [ ] Error message on invalid key

---

### Task P0-3: FR-001 - Dynamic Provider Registry
**Status:** ✅ Done
**Priority:** P0
**Files:**
- `opencode-rust/crates/cli/src/cmd/providers.rs`
- `opencode-rust/crates/llm/src/models.rs`

**Changes:**
1. Line 94: Remove hardcoded `["openai", "anthropic", "ollama"]`
2. Read providers dynamically from `ModelRegistry`
3. Match provider list with `ConnectProviderDialog` (18 providers)
4. Added `list_providers()` method to `ModelRegistry`

**Acceptance Criteria:**
- [x] `providers` CLI lists all 18 providers
- [x] List matches TUI dialog options
- [x] No duplicates or missing providers

---

### Task P0-4: TD-2 - Fix modulo by 1 bug
**Status:** TODO
**Priority:** P0
**Files:**
- `opencode-rust/crates/tui/src/dialogs/connect_method.rs`

**Changes:**
1. Line 105: Fix `(self.selected_index + 1) % self.methods.len().max(1)`
2. When `methods.len()` is 0, don't use modulo at all - just stay at index 0

**Acceptance Criteria:**
- [ ] Single item + Down stays at 0
- [ ] Empty list + navigation doesn't cause index issues

---

## P1 High Priority

### Task P1-1: FR-002 - Expanded Model Catalog
**Status:** TODO
**Priority:** P1
**Files:**
- `opencode-rust/crates/llm/src/models.rs`

**Changes:**
1. Add github-copilot/* models
2. Add opencode/gpt-5-nano
3. Add opencode/minimax-m2.5-free
4. Add opencode/nemotron-3-super-free
5. Add google/antigravity-* models
6. Add kimi/* models
7. Add z.ai/* models
8. Target: 50+ models total

**Acceptance Criteria:**
- [ ] Model catalog contains 50+ models
- [ ] All original provider models available
- [ ] Model context lengths displayed correctly

---

### Task P1-2: FR-003 - Shell Completion Command
**Status:** TODO
**Priority:** P1
**Files:**
- `opencode-rust/crates/cli/src/cmd/completion.rs` (NEW)

**Changes:**
1. Create `completion` command module
2. Support shells: bash, zsh, fish, PowerShell
3. Output completion script to stdout
4. Add command to CLI registry

**Command Structure:**
```
opencode-rs completion [SHELL]
```

**Acceptance Criteria:**
- [ ] `opencode-rs completion bash` generates bash completions
- [ ] `opencode-rs completion zsh` generates zsh completions
- [ ] Completions work for all CLI commands

---

### Task P1-3: FR-004 - Plugin CLI Commands
**Status:** TODO
**Priority:** P1
**Files:**
- `opencode-rust/crates/cli/src/cmd/plugin.rs` (NEW)

**Changes:**
1. Create `plugin` command module
2. Implement subcommands:
   - `plugin install <name>` - Install plugin
   - `plugin list` - List installed plugins
   - `plugin remove <name>` - Remove plugin
   - `plugin search [query]` - Search available plugins
3. Persist plugins in `opencode.json` config
4. Add to CLI registry

**Acceptance Criteria:**
- [ ] All plugin subcommands functional
- [ ] Plugins persisted in config
- [ ] Plugin discovery mechanism working

---

### Task P1-4: FR-005 - PR Command Implementation
**Status:** TODO
**Priority:** P1
**Files:**
- `opencode-rust/crates/cli/src/cmd/pr.rs`

**Changes:**
1. Remove stub implementation (lines 39-40)
2. Implement PR fetch functionality:
   - `pr <number>` - Fetch and display PR details
   - `pr checkout <number>` - Checkout PR branch
   - `pr list` - List recent PRs
3. Use GitHub API for PR data

**Acceptance Criteria:**
- [ ] Fetch PR details from GitHub API
- [ ] Checkout PR branch locally
- [ ] Display PR diff summary

---

### Task P1-5: FR-009 - ConnectMethodDialog Tests
**Status:** TODO
**Priority:** P1
**Files:**
- `opencode-rust/crates/tui/src/dialogs/connect_method.rs`

**Changes:**
Add tests per PRD Section 9.2:
1. `test_connect_method_dialog_shows_message_when_empty` - Empty methods shows message
2. `test_connect_method_dialog_enter_on_empty_closes` - Enter on empty closes dialog
3. `test_connect_method_dialog_empty_list_up_does_not_panic` - Navigation on empty doesn't panic
4. `test_connect_method_dialog_single_item_down_stays_at_zero` - Single item navigation

**Acceptance Criteria:**
- [ ] Empty methods + Enter closes dialog
- [ ] Empty methods + navigation doesn't panic
- [ ] Single item + Down at end stays at 0
- [ ] Empty state renders visible message with border

---

## P2 Medium Priority

### Task P2-1: FR-006 - GitHub Integration Completion
**Status:** TODO
**Priority:** P2
**Files:**
- `opencode-rust/crates/cli/src/cmd/github.rs`

**Changes:**
1. Remove TODO messages (lines 80-86)
2. Implement `GitHubAction::Login` - OAuth flow
3. Implement `GitHubAction::RepoList` - List user repositories
4. Implement `GitHubAction::IssueList { repo }` - List issues for repo

**Acceptance Criteria:**
- [ ] GitHub OAuth login functional
- [ ] Repository listing shows user repos
- [ ] Issue listing shows repo issues

---

### Task P2-2: FR-010 - ConnectModelDialog Tests
**Status:** TODO
**Priority:** P2
**Files:**
- `opencode-rust/crates/tui/src/dialogs/connect_model.rs`

**Changes:**
Add rendering and interaction tests:
1. `test_connect_model_dialog_empty_list_enter_closes` - Empty list + Enter closes
2. `test_connect_model_dialog_empty_list_up_does_not_panic` - Navigation on empty
3. `test_connect_model_dialog_single_item_down_stays_at_zero` - Single item navigation
4. `test_connect_model_dialog_renders_empty_state` - Empty state renders with border
5. `test_connect_model_dialog_renders_models` - Model list renders correctly

**Acceptance Criteria:**
- [ ] Empty state renders with border
- [ ] Model list scrollable
- [ ] Enter on model closes with selection

---

### Task P2-3: FR-011 - OAuth Flows for Google/Copilot
**Status:** TODO
**Priority:** P2
**Files:**
- `opencode-rust/crates/llm/src/auth_layered/`

**Changes:**
1. Implement Google OAuth flow
2. Implement GitHub Copilot OAuth flow
3. Update ConnectMethodDialog for OAuth-only providers

**Providers needing OAuth:**
- Google
- GitHub Copilot

**Acceptance Criteria:**
- [ ] Google OAuth login functional
- [ ] GitHub Copilot OAuth login functional

---

## Technical Debt Tasks

### Task TD-3: GitHub Command TODOs
**Status:** TODO
**Priority:** Medium
**See:** Task P2-1

### Task TD-4: PR Command Stub
**Status:** TODO
**Priority:** Medium
**See:** Task P1-4

---

## Summary

| Priority | Tasks | Status |
|----------|-------|--------|
| P0 | 4 | TODO |
| P1 | 5 | TODO |
| P2 | 3 | TODO |
| **Total** | **12** | |

---

*Task List Version: 33*
*Generated: 2026-04-17*
