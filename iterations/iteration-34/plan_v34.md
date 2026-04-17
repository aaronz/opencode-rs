# Iteration 34 Implementation Plan

**Project:** opencode-rs
**Iteration:** 34
**Date:** 2026-04-17
**Based on:** Spec v34 + Gap Analysis

---

## 1. Priority Classification

### P0 - Critical Blockers (Must Fix)

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| **FR-012** | API key validation | `app.rs:773-787` | No API call to validate key |
| **FR-013** | Model selection after API key | `app.rs:773-787` | Returns to Chat instead of model picker |

### P1 - High Priority

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-014 | Google OAuth not wired | `app.rs` + `google_oauth.rs` | Exists but unused |
| FR-015 | Copilot OAuth not wired | `app.rs` + `copilot_oauth.rs` | Exists but unused |
| FR-002 | Model catalog expansion | `models.rs` | 81% gap (17 vs 89) |
| FR-003 | Shell completion | `cli/` | Missing command |
| FR-004 | Plugin CLI | `cli/` | Missing command |

### P2 - Medium Priority

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-016 | ProviderAuth trait | `llm/` | No standard capability discovery |
| FR-017 | Multiple credentials | `credential_store.rs` | Single credential per provider |

---

## 2. P0 Implementation Plan (Iteration 34 Focus)

### FR-012: API Key Validation Against Provider API

**Location:** `crates/tui/src/app.rs:773-787`

**Current Flow (Broken):**
```rust
fn handle_api_key_input_confirm(&mut self, api_key: String) {
    if let Err(e) = self.save_api_key_credential(&provider_id, &api_key) {
        // Only checks if save succeeds, NOT if key is valid
    }
    self.mode = AppMode::Chat;  // Returns directly to chat
}
```

**Required Changes:**
1. After API key save, make test API call to provider's `/v1/models` endpoint
2. For Anthropic: Call `https://api.anthropic.com/v1/models` with the key
3. For OpenAI-compatible: Call `/models` endpoint
4. If invalid: Show error message, allow retry
5. If valid: Proceed to model selection

**Implementation Steps:**
1. Create validation function in `app.rs` that calls provider's models endpoint
2. Handle different provider API structures (Anthropic vs OpenAI-compatible)
3. Add loading state during validation
4. Add error handling with user-friendly messages

---

### FR-013: Model Selection After API Key Save

**Location:** `crates/tui/src/app.rs:773-787`

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

**Implementation Steps:**
1. After successful validation (FR-012), fetch available models from provider
2. Transition to `ConnectModelDialog` with the model list
3. On model selection, configure provider with selected model
4. Return to Chat mode with provider configured

**Reference Implementation (OpenAI browser auth - correct):**
```rust
start_openai_browser_connect() → complete_browser_auth() → ConnectModelDialog
```

**API key auth should follow same pattern:**
```rust
handle_api_key_input_confirm() → validate_api_key() → ConnectModelDialog
```

---

## 3. P1 Implementation Plan

### FR-014: Wire Google OAuth to TUI

**Prerequisite:** OAuth implementation exists in `crates/llm/src/auth_layered/google_oauth.rs`

**Required Changes:**
1. Connect `GoogleOAuthService` to `handle_connect_method_confirm`
2. Launch browser OAuth flow for google when "browser" method selected
3. Handle OAuth callback and store credentials
4. Proceed to model selection after successful OAuth

**Code Location:** `crates/tui/src/app.rs` + `crates/tui/src/dialogs/connect_method.rs`

---

### FR-015: Wire Copilot OAuth to TUI

**Prerequisite:** OAuth implementation exists in `crates/llm/src/auth_layered/copilot_oauth.rs`

**Required Changes:**
1. Connect `CopilotOAuthService` to `handle_connect_method_confirm`
2. Launch device code flow for copilot when "browser" method selected
3. Handle OAuth callback and store credentials
4. Proceed to model selection after successful OAuth

---

### FR-002: Model Catalog Expansion

**Current:** ~17 models vs 89 original (81% gap)

**Target:** 50+ models minimum

**Required Changes:**
1. Add missing provider models:
   - `github-copilot/*` models
   - `opencode/gpt-5-nano`
   - `opencode/minimax-m2.5-free`
   - `opencode/nemotron-3-super-free`
   - `google/antigravity-*` models
   - `kimi/*` models
   - `z.ai/*` models
2. Update model context lengths
3. Align model naming with original

**Module:** `crates/llm/src/models.rs`

---

### FR-003: Shell Completion Command

**Required Commands:**
- `opencode-rs completion bash` - Generate bash completions
- `opencode-rs completion zsh` - Generate zsh completions
- `opencode-rs completion fish` - Generate fish completions
- `opencode-rs completion powershell` - Generate PowerShell completions

**Module:** `crates/cli/src/cmd/completion.rs` (new)

---

### FR-004: Plugin CLI Commands

**Required Commands:**
- `opencode-rs plugin install <name>` - Install plugin
- `opencode-rs plugin list` - List installed plugins
- `opencode-rs plugin remove <name>` - Remove plugin
- `opencode-rs plugin search [query]` - Search available plugins

**Module:** `crates/cli/src/cmd/plugin.rs` (new)

---

## 4. P2 Implementation Plan

### FR-016: ProviderAuth Trait

**Define:**
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

**Implementation:**
1. Define `AuthMethod` enum
2. Implement `ProviderAuth` trait on provider types
3. Update `ConnectMethodDialog` to query provider capabilities
4. Remove hardcoded `OAUTH_ONLY_PROVIDERS` list

**Module:** `crates/llm/src/`

---

### FR-017: Multiple Credentials Per Provider

**Required Changes:**
1. Add `store_named(provider_id, name, credential)`
2. Add `list_credentials(provider_id)`
3. Add `delete_named(provider_id, name)`
4. Update UI to allow credential management

**Module:** `crates/core/src/credential_store.rs`

---

## 5. Technical Debt

| TD | Issue | File | Severity | Fix |
|----|-------|------|----------|-----|
| TD-5 | Hardcoded OAuth-only list | `connect_method.rs:21` | Medium | FR-016 |
| TD-6 | Default model "gpt-4o" for all | `app.rs:972` | Medium | FR-013 |
| TD-7 | No base_url for API key providers | `app.rs:813-822` | Low | FR-012 |

---

## 6. Dependencies

### FR-012 requires:
- FR-008 (ApiKeyInputDialog) - ✅ Already implemented
- Provider API knowledge for validation endpoints

### FR-013 requires:
- FR-012 (API key validation) - Must complete first
- ConnectModelDialog - ✅ Already implemented
- Model fetching from provider API

### FR-014 requires:
- FR-011 (Google OAuth impl) - ✅ Already implemented
- TUI wiring in app.rs

### FR-015 requires:
- FR-011 (Copilot OAuth impl) - ✅ Already implemented
- TUI wiring in app.rs

---

## 7. Exit Criteria

Iteration 34 is complete when:
- [ ] FR-012: API key validation works for all providers
- [ ] FR-013: Model selection dialog shown after successful API key save
- [ ] FR-014: Google OAuth launches from TUI
- [ ] FR-015: Copilot OAuth launches from TUI
- [ ] All P0 tests pass
- [ ] No regressions in existing functionality

---

*Plan Version: 34*
*Generated: 2026-04-17*
