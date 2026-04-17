# opencode-rs Specification v33

**Project:** opencode-rs (Rust Implementation)
**Version:** 33
**Date:** 2026-04-17
**Based on:** PRD (opencode vs opencode-rs Feature Gap Analysis) + Gap Analysis

---

## 1. Overview

opencode-rs is a Rust reimplementation of the original opencode (TypeScript/Bun v1.4.5) AI coding agent. This document defines the specification for opencode-rs, tracking feature requirements, implementation status, and gap analysis against the original implementation.

---

## 2. Command Specification

### 2.1 Core Commands

| Command | Status | Implementation | Notes |
|---------|--------|----------------|-------|
| `tui` | ✅ Implemented | Default TUI interface | `[project]` alias |
| `attach` | ✅ Implemented | Server attachment | URL as arg or interactive |
| `run` | ✅ Implemented | Agent execution | |
| `debug` | ✅ Implemented | Debug mode | |
| `agent` | ✅ Implemented | Agent selection | |
| `upgrade` | ✅ Implemented | Auto-upgrade | |
| `uninstall` | ✅ Implemented | Removal | |
| `serve` | ✅ Implemented | HTTP server | |
| `web` | ✅ Implemented | Web interface | |
| `stats` | ✅ Implemented | Statistics | |
| `session` | ✅ Implemented | Session management | |
| `db` | ✅ Implemented | Database CLI | |
| `export` | ✅ Implemented | Session portability | |
| `import` | ✅ Implemented | Session portability | |

### 2.2 Provider/Model Commands

| Command | Status | Implementation | Notes |
|---------|--------|----------------|-------|
| `providers` | ⚠️ Partial | `crates/cli/src/cmd/providers.rs` | **FR-001**: Only 3 hardcoded providers |
| `models` | ⚠️ Partial | `crates/llm/src/models.rs` | **FR-002**: ~17 models vs 89 original |
| `acp` | ✅ Implemented | ACP protocol | |
| `mcp` | ✅ Implemented | MCP client/server | |

### 2.3 Missing Commands

| Command | Original | Status | Priority | Implementation |
|---------|----------|--------|----------|----------------|
| `completion` | ✅ | ❌ Missing | P1 | **FR-003**: Shell completion generation |
| `plugin` | ✅ | ❌ Missing CLI | P1 | **FR-004**: Plugin CLI commands |

### 2.4 Incomplete Commands

| Command | Status | Issue | Priority | Implementation |
|---------|--------|-------|----------|----------------|
| `pr` | ❌ Stub | Prints debug only | P1 | **FR-005**: PR fetch/checkout |
| `github` | ⚠️ Renamed | `git-hub` exists, incomplete | P2 | **FR-006**: GitHub integration |

### 2.5 Rust-Exclusive Commands

| Command | Status | Notes |
|---------|--------|-------|
| `account` | ✅ Implemented | Account management |
| `config` | ✅ Implemented | Config management |
| `bash` | ✅ Implemented | Shell integration |
| `terminal` | ✅ Implemented | Terminal integration |
| `git-hub` | ✅ Implemented | GitHub workflows |
| `git-lab` | ✅ Implemented | GitLab integration |
| `generate` | ✅ Implemented | Code generation |
| `thread` | ✅ Implemented | Thread management |
| `workspace-serve` | ✅ Implemented | Workspace server |
| `palette` | ✅ Implemented | Command palette |
| `shortcuts` | ✅ Implemented | Keyboard shortcuts |
| `workspace` | ✅ Implemented | Workspace management |
| `ui` | ✅ Implemented | UI controls |
| `project` | ✅ Implemented | Project management |
| `files` | ✅ Implemented | File operations |
| `prompt` | ✅ Implemented | Prompt management |
| `quick` | ✅ Implemented | Quick actions |
| `desktop` | ✅ Implemented | Desktop mode |

---

## 3. Feature Requirements

### FR-001: Dynamic Provider Registry

**Priority:** P0 (Critical)
**Module:** `crates/cli/src/cmd/providers.rs:94`
**Issue:** Hardcoded `["openai", "anthropic", "ollama"]`

**Specification:**
- Remove hardcoded provider list
- Read providers dynamically from provider registry
- Display all 18 providers available in TUI dialog

**Acceptance Criteria:**
- [ ] `opencode-rs providers` lists all registered providers
- [ ] Provider list matches ConnectProviderDialog options
- [ ] No duplicate or missing providers

---

### FR-002: Expanded Model Catalog

**Priority:** P1 (High)
**Module:** `crates/llm/src/models.rs`
**Issue:** Only ~17 models vs 89 in original (81% gap)

**Specification:**
- Expand model registry to 50+ models minimum
- Add missing provider models:
  - `github-copilot/*` models
  - `opencode/gpt-5-nano`
  - `opencode/minimax-m2.5-free`
  - `opencode/nemotron-3-super-free`
  - `google/antigravity-*` models
- Align model naming with original where applicable

**Target Model Count:** 50+ models

**Acceptance Criteria:**
- [ ] Model catalog contains 50+ models
- [ ] All original provider models available
- [ ] Model context lengths displayed correctly

---

### FR-003: Shell Completion Command

**Priority:** P1 (High)
**Module:** `crates/cli/`
**Issue:** `opencode completion` not implemented

**Specification:**
- Add `completion` command for shell completion generation
- Support shells: bash, zsh, fish, PowerShell
- Output completion script to stdout or install to standard locations

**Command Structure:**
```
opencode-rs completion [SHELL]
```

**Acceptance Criteria:**
- [ ] `opencode-rs completion bash` generates bash completions
- [ ] `opencode-rs completion zsh` generates zsh completions
- [ ] Completions work for all CLI commands

---

### FR-004: Plugin CLI Commands

**Priority:** P1 (High)
**Module:** `crates/cli/`
**Issue:** No `plugin` CLI command despite plugin crate existing

**Specification:**
- Add plugin management commands:
  - `opencode-rs plugin install <name>` - Install plugin
  - `opencode-rs plugin list` - List installed plugins
  - `opencode-rs plugin remove <name>` - Remove plugin
  - `opencode-rs plugin search [query]` - Search available plugins

**Acceptance Criteria:**
- [ ] All plugin subcommands functional
- [ ] Plugins persisted in `opencode.json` config
- [ ] Plugin discovery mechanism working

---

### FR-005: PR Command Implementation

**Priority:** P1 (High)
**Module:** `crates/cli/src/cmd/pr.rs:39-40`
**Issue:** Stub implementation prints debug only

**Specification:**
- Implement PR fetch and checkout functionality:
  - `opencode-rs pr <number>` - Fetch and display PR details
  - `opencode-rs pr checkout <number>` - Checkout PR branch
  - `opencode-rs pr list` - List recent PRs

**Acceptance Criteria:**
- [ ] Fetch PR details from GitHub API
- [ ] Checkout PR branch locally
- [ ] Display PR diff summary

---

### FR-006: GitHub Integration Completion

**Priority:** P2 (Medium)
**Module:** `crates/cli/src/cmd/github.rs:80-86`
**Issue:** Login, RepoList, IssueList print TODO

**Specification:**
- Complete GitHub integration:
  - `GitHubAction::Login` - OAuth flow implementation
  - `GitHubAction::RepoList` - List user repositories
  - `GitHubAction::IssueList { repo }` - List issues for repo

**Acceptance Criteria:**
- [ ] GitHub OAuth login functional
- [ ] Repository listing shows user repos
- [ ] Issue listing shows repo issues

---

### FR-007: ConnectMethodDialog Fix

**Priority:** P0 (Critical)
**Module:** `crates/tui/src/dialogs/connect_method.rs:21-28`
**Issue:** Empty methods for non-OpenAI providers, silent close on Enter

**Specification:**
- For all providers except those with only OAuth:
  - Show "API Key" as available auth method
- For OAuth-only providers (Google, Copilot):
  - Show "Not yet implemented" message
  - Do NOT close on Enter - require explicit Cancel/ESC

**Current Bug:**
```rust
let methods = if provider_id == "openai" {
    vec![
        ("browser".to_string(), "Browser auth".to_string()),
        ("api_key".to_string(), "API key".to_string()),
    ]
} else {
    Vec::new()  // BUG: Empty for all other providers!
};
```

**Acceptance Criteria:**
- [ ] All API-key providers show "API Key" option
- [ ] Empty dialog does NOT close silently on Enter
- [ ] Shows explanatory message for unimplemented auth

---

### FR-008: API Key Input Flow

**Priority:** P0 (Critical)
**Module:** `crates/tui/src/dialogs/`
**Issue:** No API key input dialog exists

**Specification:**
- Implement API key text input dialog
- Flow: Provider Selection → Auth Method (API Key) → API Key Input → Validate → Save

**Dialog Sequence:**
1. `ConnectProviderDialog` - Select provider
2. `ConnectMethodDialog` - Select "API Key"
3. `ApiKeyInputDialog` (NEW) - Enter API key
4. Validate and store credentials

**Acceptance Criteria:**
- [ ] API key input dialog renders correctly
- [ ] Input masked (dots, not plain text)
- [ ] API key validated before saving
- [ ] Error message on invalid key

---

### FR-009: ConnectMethodDialog Tests

**Priority:** P1 (High)
**Module:** `crates/tui/src/dialogs/connect_method.rs`
**Issue:** No unit tests despite being P0 user-facing dialog

**Specification:**
Add tests per PRD Section 9.2:

```rust
#[test]
fn test_connect_method_dialog_shows_message_when_empty() {
    let dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
    // Should render with message, not empty
}

#[test]
fn test_connect_method_dialog_enter_on_empty_closes() {
    let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(action, DialogAction::Close);
}
```

**Acceptance Criteria:**
- [ ] Empty methods + Enter closes dialog
- [ ] Empty methods + navigation doesn't panic
- [ ] Single item navigation works correctly
- [ ] Empty state renders visible message

---

### FR-010: ConnectModelDialog Tests

**Priority:** P2 (Medium)
**Module:** `crates/tui/src/dialogs/connect_model.rs`
**Issue:** No tests for model selection dialog

**Specification:**
Add rendering and interaction tests:
- Empty model list handling
- Model selection with keyboard
- Model filtering/search

**Acceptance Criteria:**
- [ ] Empty state renders with border
- [ ] Model list scrollable
- [ ] Enter on model closes with selection

---

## 4. Authentication Specification

### 4.1 Auth Method Matrix

| Provider | opencode Auth | opencode-rs Auth | Status |
|----------|--------------|-----------------|--------|
| Google | OAuth | ❌ None | **FR-011** needed |
| OpenAI | OAuth + API Key | ✅ OAuth + API Key | Complete |
| GitHub Copilot | OAuth | ❌ None | **FR-011** needed |
| Kimi | API Key | ❌ None | **FR-008** needed |
| Z.AI | API Key | ❌ None | **FR-008** needed |
| MiniMax | API Key | ❌ None | **FR-008** needed |
| Anthropic | API Key | ✅ API Key | Complete |
| Ollama | Local | ✅ Local | Complete |

### 4.2 Auth Flow Types

**OAuth Flow:**
1. User selects provider with OAuth
2. System opens browser for authentication
3. OAuth callback received
4. Credentials stored securely

**API Key Flow:**
1. User selects provider with API Key
2. User enters API key in input dialog (**FR-008**)
3. Key validated against provider
4. Credentials stored securely

---

## 5. Model Catalog Specification

### 5.1 Current Models (17)

```
openai: gpt-4o, gpt-4o-mini, gpt-4-turbo
anthropic: claude-sonnet-4-20250514, claude-haiku-3, claude-opus-4-20250514
ollama: llama3, codellama
azure: gpt-4o-azure
google: gemini-1.5-pro, gemini-1.5-flash
openrouter: openrouter/gpt-4o
xai: grok-2
mistral: mistral-large-latest
groq: llama-3.1-70b-versatile
deepinfra: deepinfra/llama-3.1-70b
cerebras: cerebras/llama-3.1-70b
cohere: cohere-command-r-plus
togetherai: togetherai/llama-3.1-70b
perplexity: perplexity/llama-3.1-sonar-large
```

### 5.2 Missing Models (Target: 50+)

```
github-copilot/* (all copilot models)
opencode/gpt-5-nano
opencode/minimax-m2.5-free
opencode/nemotron-3-super-free
google/antigravity-* (internal models)
kimi/* (kimi models)
z.ai/* (z.ai models)
```

---

## 6. Dialog System Specification

### 6.1 Dialog Inventory

| Dialog | File | Tests | P0 Bug | Status |
|--------|------|-------|--------|--------|
| ConnectProviderDialog | `connect_provider.rs` | ✅ 2 tests | No | Complete |
| ConnectMethodDialog | `connect_method.rs` | ❌ None | **YES** | **FR-007** |
| ConnectModelDialog | `connect_model.rs` | ❌ None | No | **FR-010** |
| ApiKeyInputDialog | (missing) | ❌ None | N/A | **FR-008** |

### 6.2 Dialog Interaction Rules

1. **Empty list + Enter** → Close dialog (not None)
2. **Empty list + navigation** → Must not panic
3. **Single item + Down at end** → Stay at index 0
4. **Empty state** → Always show visible message with border

---

## 7. Technical Debt

| ID | Issue | Module | Severity | Fix |
|----|-------|--------|----------|-----|
| TD-1 | `providers.rs:94` hardcode | `cli/providers.rs` | High | **FR-001** |
| TD-2 | `connect_method.rs:105` modulo by 1 | `tui/` | Medium | Fix logic |
| TD-3 | GitHub TODOs | `github.rs:80-86` | Medium | **FR-006** |
| TD-4 | PR stub | `pr.rs:40` | Medium | **FR-005** |

---

## 8. Priority Matrix

| Priority | Count | Items |
|----------|-------|-------|
| P0 | 4 | FR-001, FR-007, FR-008, TD-1 |
| P1 | 5 | FR-002, FR-003, FR-004, FR-005, FR-009 |
| P2 | 3 | FR-006, FR-010, FR-011 |
| P3 | 0 | - |

---

## 9. Acceptance Criteria Summary

### P0 Criteria (Required Before Release)

- [ ] **FR-001**: `providers` CLI lists all providers dynamically
- [ ] **FR-007**: ConnectMethodDialog shows API Key option or message, does not close silently
- [ ] **FR-008**: API key input dialog implemented and functional

### P1 Criteria (Next Sprint)

- [ ] **FR-002**: Model catalog expanded to 50+ models
- [ ] **FR-003**: Shell completion command functional
- [ ] **FR-004**: Plugin CLI commands implemented
- [ ] **FR-005**: PR command fetches and checks out PRs
- [ ] **FR-009**: ConnectMethodDialog has unit tests

### P2 Criteria (Medium Term)

- [ ] **FR-006**: GitHub integration completed
- [ ] **FR-010**: ConnectModelDialog has tests
- [ ] **FR-011**: OAuth flows for Google/Copilot

---

## 10. Cross-References

| File | Topic | Related FR |
|------|-------|------------|
| `crates/tui/src/dialogs/connect_method.rs` | Auth method dialog | FR-007, FR-009 |
| `crates/tui/src/dialogs/connect_provider.rs` | Provider selection | FR-001 |
| `crates/cli/src/cmd/providers.rs` | CLI provider list | FR-001 |
| `crates/llm/src/models.rs` | Model registry | FR-002 |
| `crates/cli/src/cmd/github.rs` | GitHub command | FR-006 |
| `crates/cli/src/cmd/pr.rs` | PR command | FR-005 |
| `crates/llm/src/auth_layered/layer2_auth_mechanism.rs` | Auth mechanisms | FR-008, FR-011 |

---

*Specification Version: 33*
*Generated: 2026-04-17*
*Based on: PRD Gap Analysis + Iteration 33 Review*
