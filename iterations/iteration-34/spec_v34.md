# opencode-rs Specification v34

**Project:** opencode-rs (Rust Implementation)
**Version:** 34
**Date:** 2026-04-17
**Based on:** PRD #22 (Provider Authentication Expansion) + Iteration 34 Gap Analysis

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
| `providers` | ✅ Implemented | `crates/cli/src/cmd/providers.rs` | **FR-001**: Dynamic provider registry |
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
| `pr` | ✅ Implemented | PR fetch/checkout/list | P1 | **FR-005**: Complete |
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
**Status:** ✅ Completed

**Specification:**
- Remove hardcoded provider list
- Read providers dynamically from provider registry
- Display all 18 providers available in TUI dialog

**Acceptance Criteria:**
- [x] `opencode-rs providers` lists all registered providers
- [x] Provider list matches ConnectProviderDialog options
- [x] No duplicate or missing providers

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
**Module:** `crates/cli/src/cmd/pr.rs`
**Status:** ✅ Implemented

**Specification:**
- Implement PR fetch and checkout functionality:
  - `opencode-rs pr <number>` - Fetch and display PR details
  - `opencode-rs pr checkout <number>` - Checkout PR branch
  - `opencode-rs pr list` - List recent PRs

**Acceptance Criteria:**
- [x] Fetch PR details from GitHub API
- [x] Checkout PR branch locally
- [x] Display PR diff summary

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
**Module:** `crates/tui/src/dialogs/connect_method.rs`
**Status:** ✅ Completed (Iteration 33)

**Specification:**
- For all providers except those with only OAuth:
  - Show "API Key" as available auth method
- For OAuth-only providers (Google, Copilot):
  - Show "Not yet implemented" message
  - Do NOT close on Enter - require explicit Cancel/ESC

**Acceptance Criteria:**
- [x] All API-key providers show "API Key" option
- [x] Empty dialog does NOT close silently on Enter
- [x] Shows explanatory message for unimplemented auth
- [x] Unit tests: 14 tests pass

---

### FR-008: API Key Input Flow

**Priority:** P0 (Critical)
**Module:** `crates/tui/src/dialogs/api_key_input.rs`
**Status:** ✅ Implemented (Iteration 33)

**Specification:**
- Implement API key text input dialog
- Flow: Provider Selection → Auth Method (API Key) → API Key Input → Validate → Save

**Dialog Sequence:**
1. `ConnectProviderDialog` - Select provider
2. `ConnectMethodDialog` - Select "API Key"
3. `ApiKeyInputDialog` (NEW) - Enter API key
4. Validate and store credentials

**Acceptance Criteria:**
- [x] API key input dialog renders correctly
- [x] Input masked (dots, not plain text)
- [x] Basic validation (non-empty, min length, no whitespace)
- [ ] **API key validated against provider API** (P0 Gap - FR-012)
- [ ] **Model selection after API key save** (P0 Gap - FR-013)

---

### FR-009: ConnectMethodDialog Tests

**Priority:** P1 (High)
**Module:** `crates/tui/src/dialogs/connect_method.rs`
**Status:** ✅ Completed (Iteration 33)

**Specification:**
Add tests per PRD Section 9.2:
- Empty methods + Enter closes dialog
- Empty methods + navigation doesn't panic
- Single item navigation works correctly
- Empty state renders visible message

**Acceptance Criteria:**
- [x] Empty methods + Enter closes dialog
- [x] Empty methods + navigation doesn't panic
- [x] Single item navigation works correctly
- [x] Empty state renders visible message

---

### FR-010: ConnectModelDialog Tests

**Priority:** P2 (Medium)
**Module:** `crates/tui/src/dialogs/connect_model.rs`
**Status:** ✅ Completed (Iteration 33)

**Acceptance Criteria:**
- [x] Empty state renders with border
- [x] Model list scrollable
- [x] Enter on model closes with selection

---

### FR-011: OAuth Flows for Google/Copilot

**Priority:** P1 (High)
**Module:** `crates/llm/src/auth_layered/google_oauth.rs`, `copilot_oauth.rs`
**Status:** ⚠️ Implementation exists, NOT wired to TUI

**Specification:**
OAuth implementations exist in auth_layered module with 109 tests. TUI shows "not yet implemented" instead of launching OAuth flow.

**Current State:**
- Google OAuth: `crates/llm/src/auth_layered/google_oauth.rs` ✅ (109 tests)
- Copilot OAuth: `crates/llm/src/auth_layered/copilot_oauth.rs` ✅ (109 tests)
- TUI wiring: ❌ Not connected

**Acceptance Criteria:**
- [ ] Google OAuth flow launches from TUI when "browser" selected
- [ ] Copilot OAuth flow launches from TUI when "browser" selected
- [ ] OAuth callback handled correctly
- [ ] Credentials stored after successful OAuth

---

### FR-012: API Key Validation Against Provider API

**Priority:** P0 (Critical)
**Module:** `crates/tui/src/app.rs:773-787`
**Status:** ❌ Not Implemented (Gap from Iteration 33)

**Issue:**
API key is saved without any validation against the provider's API. User can enter invalid API keys with no feedback until actual API calls fail.

**PRD Requirement:**
> "Validates key by making a test API call"

**Current Flow (Broken):**
```rust
fn handle_api_key_input_confirm(&mut self, api_key: String) {
    // ...
    if let Err(e) = self.save_api_key_credential(&provider_id, &api_key) {
        // Only checks if save succeeds, NOT if key is valid
    }
    self.mode = AppMode::Chat;  // Returns directly to chat
}
```

**Expected Flow:**
1. User enters API key
2. Make test API call to provider's `/v1/models` or equivalent
3. If valid: Show `ConnectModelDialog` with available models
4. If invalid: Show error message, allow retry

**Implementation Notes:**
- For Anthropic: Call `/v1/models` with the key
- For OpenAI-compatible: Call `/models` endpoint
- For other providers: Use provider-specific validation endpoint

**Acceptance Criteria:**
- [ ] API key validated by calling provider's models endpoint
- [ ] Invalid key shows error message with retry option
- [ ] Valid key proceeds to model selection dialog
- [ ] User feedback during validation (loading state)

---

### FR-013: Model Selection After API Key Save

**Priority:** P0 (Critical)
**Module:** `crates/tui/src/app.rs:773-787`
**Status:** ❌ Not Implemented (Gap from Iteration 33)

**Issue:**
After successful API key save, the flow returns directly to Chat mode. Users cannot select a model for the provider.

**PRD Requirement:**
> "Enter key → Validating... → Success: Select Model Dialog"

**Current Flow (Broken):**
1. User selects provider (e.g., Anthropic)
2. User selects "API key" method
3. User enters API key
4. **"API key saved successfully"** → Returns to Chat
5. No model selection happens

**Expected Flow:**
1-3. Same
4. Validate API key (FR-012)
5. **Show ConnectModelDialog with available models**
6. User selects model
7. Set up provider with selected model

**Technical Note:**
OpenAI browser auth correctly wires: `start_openai_browser_connect()` → `complete_browser_auth()` → `ConnectModelDialog`

API key auth should follow same pattern:
`handle_api_key_input_confirm()` → validate → `ConnectModelDialog`

**Acceptance Criteria:**
- [ ] After successful API key validation, show model picker
- [ ] Available models fetched from the provider
- [ ] Selected model stored with provider configuration
- [ ] User returns to Chat with provider configured

---

### FR-014: Wire Google OAuth to TUI

**Priority:** P1 (High)
**Module:** `crates/tui/src/app.rs` + `crates/llm/src/auth_layered/google_oauth.rs`
**Status:** ❌ Not Wired

**Issue:**
`GoogleOAuthService` exists but is not connected to the TUI connect flow.

**Current ConnectMethodDialog behavior:**
```rust
} else if is_oauth_only {
    Vec::new()  // Shows "OAuth not yet implemented"
}
```

**Specification:**
- Connect `GoogleOAuthService` to `handle_connect_method_confirm`
- Launch browser OAuth flow for google when "browser" method selected
- Handle OAuth callback and store credentials

**Acceptance Criteria:**
- [ ] Google "browser" auth method appears in dialog
- [ ] Selecting "browser" launches Google OAuth flow
- [ ] OAuth callback handled and credentials stored
- [ ] User proceeds to model selection

---

### FR-015: Wire Copilot OAuth to TUI

**Priority:** P1 (High)
**Module:** `crates/tui/src/app.rs` + `crates/llm/src/auth_layered/copilot_oauth.rs`
**Status:** ❌ Not Wired

**Issue:**
`CopilotOAuthService` exists but is not connected to the TUI connect flow.

**Specification:**
- Connect `CopilotOAuthService` to `handle_connect_method_confirm`
- Launch device code flow for copilot when "browser" method selected
- Handle OAuth callback and store credentials

**Acceptance Criteria:**
- [ ] Copilot "browser" auth method appears in dialog
- [ ] Selecting "browser" launches Copilot OAuth device flow
- [ ] OAuth callback handled and credentials stored
- [ ] User proceeds to model selection

---

### FR-016: ProviderAuth Trait

**Priority:** P2 (Medium)
**Module:** `crates/llm/src/`
**Status:** ❌ Not Implemented

**Issue:**
No standard way to query provider capabilities for supported authentication methods.

**PRD Technical Note:**
```rust
pub enum AuthMethod {
    Browser,      // OAuth flow
    ApiKey,       // Direct API key
    Local,        // No auth (localhost)
    DeviceFlow,   // OAuth device code
}

pub trait ProviderAuth {
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
}
```

**Current Implementation:**
- `AuthMechanism` enum exists in `crates/llm/src/auth_layered/layer2_auth_mechanism.rs`
- Different variants: ApiKey, BearerToken, BasicAuth, OAuthBrowser, DeviceCode, etc.
- No `ProviderAuth` trait with `supported_auth_methods()`

**Benefits:**
- Replace hardcoded `OAUTH_ONLY_PROVIDERS` list
- Dynamic auth method discovery per provider
- Better separation of concerns

**Acceptance Criteria:**
- [ ] `AuthMethod` enum defined with Browser, ApiKey, Local, DeviceFlow variants
- [ ] `ProviderAuth` trait implemented on provider types
- [ ] `ConnectMethodDialog` queries provider capabilities
- [ ] No hardcoded provider lists in dialog code

---

### FR-017: Multiple Credentials Per Provider

**Priority:** P2 (Medium)
**Module:** `crates/core/src/credential_store.rs`
**Status:** ⚠️ Partial Implementation

**Issue:**
`CredentialStore` stores single credential per provider. Users cannot store multiple named API keys for the same provider.

**PRD Requirement:**
> "Support for multiple credentials per provider"
> "Support for multiple named credentials per provider"

**Current Implementation:**
- `CredentialStore::store(provider_id, credential)` overwrites existing
- No support for credential naming/labeling

**Specification:**
- Add support for named credentials: `store_named(provider_id, name, credential)`
- Add `list_credentials(provider_id)` to enumerate stored credentials
- Add `delete_named(provider_id, name)` to remove specific credentials
- Update UI to allow credential management

**Acceptance Criteria:**
- [ ] Can store multiple credentials per provider with unique names
- [ ] Can list all stored credentials for a provider
- [ ] Can delete specific named credentials
- [ ] UI allows switching between stored credentials

---

## 4. Authentication Specification

### 4.1 Auth Method Matrix

| Provider | opencode Auth | opencode-rs Auth | Status |
|----------|--------------|-----------------|--------|
| Google | OAuth | ❌ Not wired | **FR-014** needed |
| OpenAI | OAuth + API Key | ✅ OAuth + API Key | Complete |
| GitHub Copilot | OAuth | ❌ Not wired | **FR-015** needed |
| Kimi | API Key | ❌ None | **FR-012, FR-013** needed |
| Z.AI | API Key | ❌ None | **FR-012, FR-013** needed |
| MiniMax | API Key | ❌ None | **FR-012, FR-013** needed |
| Anthropic | API Key | ⚠️ API Key (no validation) | **FR-012** needed |
| Ollama | Local | ✅ Local | Complete |

### 4.2 Auth Flow Types

**OAuth Flow (OpenAI - Complete):**
1. User selects provider with OAuth
2. System opens browser for authentication
3. OAuth callback received
4. Credentials stored securely
5. Model selection dialog shown

**OAuth Flow (Google/Copilot - FR-014/FR-015):**
1. User selects provider with OAuth
2. System opens browser for authentication (FR-014/FR-015)
3. OAuth callback received
4. Credentials stored securely
5. Model selection dialog shown

**API Key Flow (FR-012/FR-013):**
1. User selects provider with API Key
2. User enters API key in input dialog (FR-008)
3. Key validated against provider API (FR-012)
4. Credentials stored securely
5. Model selection dialog shown (FR-013)

**Local Flow (Ollama/LM Studio - Complete):**
1. User selects local provider
2. System connects directly (no auth)
3. Model selection dialog shown

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
| ConnectMethodDialog | `connect_method.rs` | ✅ 14 tests | No | **FR-007** Complete |
| ConnectModelDialog | `connect_model.rs` | ✅ 13 tests | No | **FR-010** Complete |
| ApiKeyInputDialog | `api_key_input.rs` | ✅ 15 tests | No | **FR-008** Implemented |

### 6.2 Dialog Interaction Rules

1. **Empty list + Enter** → Close dialog (not None)
2. **Empty list + navigation** → Must not panic
3. **Single item + Down at end** → Stay at index 0
4. **Empty state** → Always show visible message with border

### 6.3 Connect Flow State Machine

```
[User types /connect]
        ↓
[ConnectProviderDialog] → Select provider
        ↓
[ConnectMethodDialog] → Select auth method
    ├─ "browser" → OAuth flow (FR-011 for Google/Copilot)
    └─ "api_key" → [ApiKeyInputDialog] (FR-008)
        ↓
[ApiKeyInputDialog] → Enter key (FR-008)
        ↓
[Validate API Key] (FR-012) ── Invalid ──→ [Show error, retry]
        ↓ Valid
[ConnectModelDialog] (FR-013)
        ↓
[Configure provider with selected model]
        ↓
[Return to Chat]
```

---

## 7. Technical Debt

| ID | Issue | Module | Severity | Fix |
|----|-------|--------|----------|-----|
| TD-1 | `providers.rs:94` hardcode | `cli/providers.rs` | High | **FR-001** ✅ Fixed |
| TD-2 | `connect_method.rs:105` modulo by 1 | `tui/` | Medium | ✅ Fixed (Iteration 33) |
| TD-3 | GitHub TODOs | `github.rs:80-86` | Medium | **FR-006** |
| TD-4 | PR stub | `pr.rs:40` | Medium | **FR-005** ✅ Fixed |
| TD-5 | Hardcoded OAuth-only list | `connect_method.rs:21` | Medium | **FR-016** |
| TD-6 | Default model "gpt-4o" for all | `app.rs:972` | Medium | **FR-013** |
| TD-7 | No base_url for API key providers | `app.rs:813-822` | Low | **FR-012** |

---

## 8. Priority Matrix

### P0 - Critical Blockers (Iteration 34)

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-012 | API key validation | `app.rs:773-787` | No API call to validate key |
| FR-013 | Model selection after API key | `app.rs:773-787` | Returns to Chat instead of model picker |

### P1 - High Priority

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-011 | OAuth implementations not wired | `google_oauth.rs`, `copilot_oauth.rs` | 109 tests exist but TUI shows "not implemented" |
| FR-014 | Google OAuth not wired | `app.rs` + `google_oauth.rs` | Exists but unused |
| FR-015 | Copilot OAuth not wired | `app.rs` + `copilot_oauth.rs` | Exists but unused |
| FR-002 | Model catalog expansion | `models.rs` | 81% gap (17 vs 89) |
| FR-003 | Shell completion | `cli/` | Missing command |
| FR-004 | Plugin CLI | `cli/` | Missing command |
| FR-009 | ConnectMethodDialog tests | `connect_method.rs` | ✅ Done (Iteration 33) |

### P2 - Medium Priority

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-016 | ProviderAuth trait | `llm/` | No standard capability discovery |
| FR-017 | Multiple credentials | `credential_store.rs` | Single credential per provider |
| FR-006 | GitHub integration | `github.rs` | Login/RepoList/IssueList not implemented |
| FR-010 | ConnectModelDialog tests | `connect_model.rs` | ✅ Done (Iteration 33) |

---

## 9. Acceptance Criteria Summary

### P0 Criteria (Required Before Release)

- [x] **FR-001**: `providers` CLI lists all providers dynamically
- [x] **FR-007**: ConnectMethodDialog shows API Key option or message, does not close silently
- [x] **FR-008**: API key input dialog implemented and functional
- [ ] **FR-012**: API key validated against provider API before saving
- [ ] **FR-013**: Model selection dialog shown after successful API key save

### P1 Criteria (Next Sprint)

- [ ] **FR-002**: Model catalog expanded to 50+ models
- [ ] **FR-003**: Shell completion command functional
- [ ] **FR-004**: Plugin CLI commands implemented
- [x] **FR-005**: PR command fetches and checks out PRs
- [x] **FR-009**: ConnectMethodDialog has unit tests (14 tests)
- [ ] **FR-011**: OAuth flows for Google/Copilot wired to TUI
- [ ] **FR-014**: Google OAuth flow launches from TUI
- [ ] **FR-015**: Copilot OAuth flow launches from TUI

### P2 Criteria (Medium Term)

- [ ] **FR-006**: GitHub integration completed
- [x] **FR-010**: ConnectModelDialog has tests (13 tests)
- [ ] **FR-016**: ProviderAuth trait with `supported_auth_methods()`
- [ ] **FR-017**: Multiple credentials per provider

---

## 10. Iteration 34 Deliverables

Based on gap analysis, Iteration 34 should prioritize:

### Must Fix (P0)
1. **FR-012**: Add API key validation - Call provider's `/v1/models` to validate key
2. **FR-013**: Wire model selection after API key - Show `ConnectModelDialog` after validation

### Should Fix (P1)
3. **FR-014**: Wire Google OAuth to TUI
4. **FR-015**: Wire Copilot OAuth to TUI

### Nice to Have (P2)
5. **FR-016**: Implement ProviderAuth trait
6. **FR-017**: Multiple credentials support

---

## 11. Cross-References

| File | Topic | Related FR |
|------|-------|------------|
| `crates/tui/src/dialogs/connect_method.rs` | Auth method dialog | FR-007, FR-009, FR-016 |
| `crates/tui/src/dialogs/connect_provider.rs` | Provider selection | FR-001 |
| `crates/tui/src/dialogs/api_key_input.rs` | API key input | FR-008, FR-012, FR-013 |
| `crates/tui/src/dialogs/connect_model.rs` | Model selection | FR-010, FR-013 |
| `crates/tui/src/app.rs` | App flow orchestration | FR-012, FR-013, FR-014, FR-015 |
| `crates/cli/src/cmd/providers.rs` | CLI provider list | FR-001 |
| `crates/llm/src/models.rs` | Model registry | FR-002 |
| `crates/llm/src/auth_layered/google_oauth.rs` | Google OAuth | FR-011, FR-014 |
| `crates/llm/src/auth_layered/copilot_oauth.rs` | Copilot OAuth | FR-011, FR-015 |
| `crates/llm/src/auth_layered/layer2_auth_mechanism.rs` | Auth mechanisms | FR-016 |
| `crates/core/src/credential_store.rs` | Credential storage | FR-017 |

---

## 12. Gap Analysis Summary

| Gap Item | Severity | Module | File:Line | Status |
|----------|----------|--------|-----------|--------|
| No API key validation | **P0** | TUI/App | `app.rs:773-787` | **FR-012** |
| No model selection after API key save | **P0** | TUI/App | `app.rs:773-787` | **FR-013** |
| Google OAuth not wired | P1 | TUI/App | `app.rs` + `google_oauth.rs` | **FR-014** |
| Copilot OAuth not wired | P1 | TUI/App | `app.rs` + `copilot_oauth.rs` | **FR-015** |
| ProviderAuth trait missing | P2 | LLM | N/A | **FR-016** |
| Single credential per provider | P2 | Core | `credential_store.rs` | **FR-017** |
| OAuth-only list hardcoded | P2 | TUI | `connect_method.rs:21` | **FR-016** |

---

*Specification Version: 34*
*Generated: 2026-04-17*
*Based on: PRD #22 Gap Analysis + Iteration 33 Review*