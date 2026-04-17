# Iteration 34 Task List

**Project:** opencode-rs
**Iteration:** 34
**Date:** 2026-04-17
**Priority:** P0 tasks must be completed before release

---

## P0 - Critical Blockers

### FR-012: API Key Validation Against Provider API

**Status:** TODO
**Priority:** P0
**Module:** `crates/tui/src/app.rs:773-787`
**Issue:** API key saved without validation against provider API

#### Tasks:

- [ ] **T-001**: Create `validate_api_key()` function in `app.rs`
  - Make test API call to provider's `/v1/models` endpoint
  - Handle different response structures (Anthropic vs OpenAI-compatible)
  - Return validation result with error details if invalid

- [ ] **T-002**: Add loading state during API key validation
  - Show "Validating..." message in UI
  - Disable input during validation

- [x] **T-003**: Add error handling with retry option
  - Display user-friendly error message on validation failure
  - Allow user to re-enter API key
  - Do NOT save invalid keys

- [x] **T-004**: Handle provider-specific validation endpoints
  - Anthropic: `https://api.anthropic.com/v1/models`
  - OpenAI-compatible: `/models` endpoint
  - LM Studio: `/api/tags` endpoint

- [x] **T-005**: Add unit tests for API key validation
  - Test valid API key acceptance
  - Test invalid API key rejection
  - Test network error handling

---

### FR-013: Model Selection After API Key Save

**Status:** TODO
**Priority:** P0
**Module:** `crates/tui/src/app.rs:773-787`
**Issue:** Returns to Chat mode instead of showing model picker

#### Tasks:

- [x] **T-006**: Wire `ConnectModelDialog` after successful API key validation
  - Transition from API key input to model selection
  - Pass validated credentials to model dialog

- [x] **T-007**: Fetch available models from provider after validation
  - Query provider's `/v1/models` endpoint
  - Filter models based on provider
  - Populate model list for dialog

- [x] **T-008**: Configure provider with selected model
  - Store model selection with provider config
  - Set as active provider/model
  - Return to Chat mode

- [ ] **T-009**: Fix hardcoded default model (TD-6)
  - Remove `gpt-4o` as default for non-OpenAI providers
  - Use selected model instead

- [x] **T-010**: Add integration tests for complete API key flow
  - Test full flow: provider selection → API key → validation → model selection → chat
  - Test error recovery on validation failure

---

## P1 - High Priority

### FR-014: Wire Google OAuth to TUI

**Status:** TODO (OAuth impl exists)
**Priority:** P1
**Module:** `crates/tui/src/app.rs` + `crates/llm/src/auth_layered/google_oauth.rs`

#### Tasks:

- [x] **T-011**: Connect `GoogleOAuthService` to TUI connect flow
  - Wire to `handle_connect_method_confirm()`
  - Launch browser OAuth when "browser" selected for Google

- [x] **T-012**: Handle OAuth callback
   - Receive OAuth callback in app
   - Exchange code for tokens
   - Store credentials

- [ ] **T-013**: Wire model selection after successful Google OAuth
  - Proceed to `ConnectModelDialog` after OAuth complete

- [ ] **T-014**: Add integration tests for Google OAuth flow

---

### FR-015: Wire Copilot OAuth to TUI

**Status:** TODO (OAuth impl exists)
**Priority:** P1
**Module:** `crates/tui/src/app.rs` + `crates/llm/src/auth_layered/copilot_oauth.rs`

#### Tasks:

- [ ] **T-015**: Connect `CopilotOAuthService` to TUI connect flow
  - Wire to `handle_connect_method_confirm()`
  - Launch device code flow when "browser" selected for Copilot

- [ ] **T-016**: Handle device code flow
  - Display device code to user
  - Poll for token exchange
  - Store credentials

- [ ] **T-017**: Wire model selection after successful Copilot OAuth
  - Proceed to `ConnectModelDialog` after OAuth complete

- [ ] **T-018**: Add integration tests for Copilot OAuth flow

---

### FR-002: Model Catalog Expansion

**Status:** TODO
**Priority:** P1
**Module:** `crates/llm/src/models.rs`
**Gap:** 17 models vs 89 original (81% gap)

#### Tasks:

- [ ] **T-019**: Add GitHub Copilot models
  - `github-copilot/*` models

- [ ] **T-020**: Add OpenCode models
  - `opencode/gpt-5-nano`
  - `opencode/minimax-m2.5-free`
  - `opencode/nemotron-3-super-free`

- [ ] **T-021**: Add Google internal models
  - `google/antigravity-*` models

- [ ] **T-022**: Add Kimi models
  - `kimi/*` models

- [ ] **T-023**: Add Z.AI models
  - `z.ai/*` models

- [ ] **T-024**: Update model context lengths
  - Ensure all models have correct context window sizes

- [ ] **T-025**: Target 50+ models minimum
  - Verify model count meets requirement

---

### FR-003: Shell Completion Command

**Status:** TODO
**Priority:** P1
**Module:** `crates/cli/src/cmd/completion.rs` (new)

#### Tasks:

- [ ] **T-026**: Create `completion` command module
  - Add `crates/cli/src/cmd/completion.rs`

- [ ] **T-027**: Implement bash completion generation
  - `opencode-rs completion bash`

- [ ] **T-028**: Implement zsh completion generation
  - `opencode-rs completion zsh`

- [ ] **T-029**: Implement fish completion generation
  - `opencode-rs completion fish`

- [ ] **T-030**: Implement PowerShell completion generation
  - `opencode-rs completion powershell`

- [ ] **T-031**: Register completion command in CLI
  - Add to `crates/core/src/cli.rs`

---

### FR-004: Plugin CLI Commands

**Status:** TODO
**Priority:** P1
**Module:** `crates/cli/src/cmd/plugin.rs` (new)

#### Tasks:

- [ ] **T-032**: Create `plugin` command module
  - Add `crates/cli/src/cmd/plugin.rs`

- [ ] **T-033**: Implement `plugin install <name>` command
  - Download and install plugin

- [ ] **T-034**: Implement `plugin list` command
  - List installed plugins

- [ ] **T-035**: Implement `plugin remove <name>` command
  - Uninstall plugin

- [ ] **T-036**: Implement `plugin search [query]` command
  - Search available plugins

- [ ] **T-037**: Persist plugins in `opencode.json` config

- [ ] **T-038**: Register plugin command in CLI

---

## P2 - Medium Priority

### FR-016: ProviderAuth Trait

**Status:** TODO
**Priority:** P2
**Module:** `crates/llm/src/`

#### Tasks:

- [ ] **T-039**: Define `AuthMethod` enum
  - Browser, ApiKey, Local, DeviceFlow variants

- [ ] **T-040**: Create `ProviderAuth` trait
  - `fn supported_auth_methods(&self) -> Vec<AuthMethod>`

- [ ] **T-041**: Implement `ProviderAuth` on provider types

- [ ] **T-042**: Update `ConnectMethodDialog` to use trait
  - Remove hardcoded `OAUTH_ONLY_PROVIDERS` list

- [ ] **T-043**: Add tests for ProviderAuth trait

---

### FR-017: Multiple Credentials Per Provider

**Status:** TODO (partial)
**Priority:** P2
**Module:** `crates/core/src/credential_store.rs`

#### Tasks:

- [ ] **T-044**: Add `store_named()` method
  - `store_named(provider_id, name, credential)`

- [ ] **T-045**: Add `list_credentials()` method
  - `list_credentials(provider_id) -> Vec<NamedCredential>`

- [ ] **T-046**: Add `delete_named()` method
  - `delete_named(provider_id, name)`

- [ ] **T-047**: Update UI for credential management
  - Allow switching between stored credentials

- [ ] **T-048**: Add tests for multiple credentials

---

## Technical Debt Tasks

### TD-5: Hardcoded OAuth-only List

- [ ] **T-049**: Remove hardcoded `OAUTH_ONLY_PROVIDERS` in `connect_method.rs:21`
  - Replace with dynamic capability query (FR-016)

### TD-6: Default Model Hardcoded

- [ ] **T-050**: Fix default model in `app.rs:972`
  - Already covered by FR-013 (T-009)

### TD-7: No base_url for API Key Providers

- [ ] **T-051**: Set base_url during API key save
  - Already covered by FR-012

---

## Testing Tasks

### Unit Tests Required

- [ ] **T-052**: API key validation tests (FR-012)
- [ ] **T-053**: Model selection flow tests (FR-013)
- [ ] **T-054**: Google OAuth wiring tests (FR-014)
- [ ] **T-055**: Copilot OAuth wiring tests (FR-015)
- [ ] **T-056**: ProviderAuth trait tests (FR-016)
- [ ] **T-057**: Multiple credentials tests (FR-017)

### Integration Tests Required

- [ ] **T-058**: Complete API key flow integration test
- [ ] **T-059**: Google OAuth flow integration test
- [ ] **T-060**: Copilot OAuth flow integration test

---

## Completion Checklist

### P0 Must Complete:
- [ ] T-001 to T-005: FR-012 API Key Validation
- [ ] T-006 to T-010: FR-013 Model Selection After API Key

### P1 Should Complete:
- [ ] T-011 to T-014: FR-014 Google OAuth
- [ ] T-015 to T-018: FR-015 Copilot OAuth
- [ ] T-019 to T-025: FR-002 Model Catalog
- [ ] T-026 to T-031: FR-003 Shell Completion
- [ ] T-032 to T-038: FR-004 Plugin CLI

### P2 Nice to Have:
- [ ] T-039 to T-043: FR-016 ProviderAuth Trait
- [ ] T-044 to T-048: FR-017 Multiple Credentials

---

## Task Summary

| Priority | Tasks | Status |
|----------|-------|--------|
| P0 | T-001 to T-010 | TODO |
| P1 | T-011 to T-038 | TODO |
| P2 | T-039 to T-048 | TODO |
| Tech Debt | T-049 to T-051 | TODO |
| Testing | T-052 to T-060 | TODO |
| **Total** | **60 tasks** | |

---

*Task List Version: 34*
*Generated: 2026-04-17*
*Last Updated: 2026-04-17*
