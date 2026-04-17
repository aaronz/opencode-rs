# Gap Analysis: Provider Authentication Expansion (PRD #22)

**Project:** opencode-rs (Rust Implementation)
**Analysis Date:** 2026-04-17
**PRD Reference:** `docs/PRD/22-provider-auth-expansion.md`
**Iteration:** 34
**Analysis Mode:** Current Implementation vs. PRD Requirements

---

## 1. Executive Summary

Iteration 33 addressed the P0 **silent dialog close bug** and implemented the `ApiKeyInputDialog`. However, several gaps remain between the current implementation and the PRD's full vision for provider authentication.

| Category | PRD Requirement | Current Status | Gap |
|----------|-----------------|---------------|-----|
| ConnectMethodDialog Bug Fix | Show API key or informative message | ✅ Fixed | None |
| API Key Input Dialog | Implement with validation | ✅ Done | None |
| API Key Validation | Validate with test API call | ❌ Not done | **P0 Gap** |
| Model Selection After API Key | Show model picker | ❌ Not done | **P0 Gap** |
| OAuth for Google | Wire existing impl to TUI | ❌ Not wired | **P1 Gap** |
| OAuth for Copilot | Wire existing impl to TUI | ❌ Not wired | **P1 Gap** |
| ProviderAuth Trait | `supported_auth_methods()` | ❌ Not done | **P2 Gap** |
| Credential Storage | Multiple credentials per provider | ⚠️ Partial | **P2 Gap** |

---

## 2. Gap Analysis by Feature

### 2.1 ConnectMethodDialog (FR-007 - RESOLVED)

**PRD State:** Dialog showed empty list for non-OpenAI providers, closing silently on Enter.

**Current Implementation (`crates/tui/src/dialogs/connect_method.rs`):**

```rust
const OAUTH_ONLY_PROVIDERS: [&str; 2] = ["google", "copilot"];

let methods = if provider_id == "openai" {
    vec![
        ("browser".to_string(), "Browser auth".to_string()),
        ("api_key".to_string(), "API key".to_string()),
    ]
} else if is_oauth_only {
    Vec::new()  // Shows "OAuth not yet implemented"
} else {
    vec![("api_key".to_string(), "API key".to_string())]
};
```

| Provider | Dialog Behavior | Status |
|----------|---------------|--------|
| openai | Shows "Browser auth" + "API key" | ✅ Complete |
| anthropic, ollama, etc. | Shows "API key" | ✅ Complete |
| google, copilot | Shows "OAuth not yet implemented" | ✅ Fixed (informative) |

**Gap:** None - FR-007 is resolved.

---

### 2.2 API Key Input Dialog (FR-008 - RESOLVED)

**Current Implementation (`crates/tui/src/dialogs/api_key_input.rs`):**

- Masked input (dots by default, Tab to reveal) ✅
- Validation: non-empty, minimum 10 chars, no whitespace ✅
- 15 unit tests ✅

**Gap:** None - FR-008 is resolved.

---

### 2.3 API Key Validation Gap (P0 - OPEN)

**PRD Requirement:**
> "Validates key by making a test API call"

**Current Implementation (`crates/tui/src/app.rs:773-787`):**

```rust
fn handle_api_key_input_confirm(&mut self, api_key: String) {
    let provider_id = self.pending_connect_provider.clone().unwrap_or_default();
    if let Err(e) = self.save_api_key_credential(&provider_id, &api_key) {
        self.add_message(format!("Failed to save API key: {}", e), false);
    } else {
        self.add_message(
            format!("API key saved successfully for {}", ...),
            false,
        );
    }
    self.mode = AppMode::Chat;  // <-- Returns directly to chat
}
```

**Problem:** API key is saved without any validation against the provider's API.

**Impact:**
- User can enter invalid API keys
- No feedback until actual API call fails
- Poor UX for authentication errors

---

### 2.4 Model Selection After API Key (P0 - OPEN)

**PRD Requirement:**
> "Enter key → Validating... → Success: Select Model Dialog"

**Current Flow:**
1. User selects provider (e.g., Anthropic)
2. User selects "API key" method
3. User enters API key
4. **"API key saved successfully"** → Returns to Chat
5. No model selection happens

**Expected Flow (from PRD):**
1-3. Same
4. Validate API key
5. **Show ConnectModelDialog with available models**
6. User selects model
7. Set up provider with selected model

**Current Code Path:**
- OpenAI browser auth: `start_openai_browser_connect()` → `complete_browser_auth()` → `ConnectModelDialog` ✅
- API key auth: `handle_api_key_input_confirm()` → returns to Chat ❌

**Gap:** API key auth flow does not invoke model selection dialog.

---

### 2.5 OAuth Flows for Google/Copilot (P1 - OPEN)

**PRD Requirement:**
> "Google: API Key / OAuth" and "GitHub Copilot: Device flow"

**Current State:**
- Google OAuth implementation exists in `crates/llm/src/auth_layered/google_oauth.rs` ✅
- Copilot OAuth implementation exists in `crates/llm/src/auth_layered/copilot_oauth.rs` ✅
- 109 tests exist in `crates/llm/src/auth_layered/tests.rs` ✅

**Problem:** These are NOT wired into the TUI connect flow.

**Current ConnectMethodDialog behavior for google/copilot:**
```rust
} else if is_oauth_only {
    Vec::new()  // Shows "OAuth not yet implemented"
}
```

**Gap:** OAuth implementations exist but TUI shows "not yet implemented" instead of launching OAuth flow.

---

### 2.6 ProviderAuth Trait (P2 - OPEN)

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
- `AuthMechanism` enum exists in `crates/llm/src/auth_layered/layer2_auth_mechanism.rs` but has different variants (ApiKey, BearerToken, BasicAuth, OAuthBrowser, DeviceCode, etc.)
- No `ProviderAuth` trait with `supported_auth_methods()`

**Gap:** Provider capability discovery via trait not implemented.

---

### 2.7 Credential Storage (P2 - PARTIAL)

**PRD Requirement:**
> "Use opencode-auth crate's CredentialStore"
> "Support for multiple credentials per provider"

**Current Implementation:**
- `CredentialStore` exists in `crates/core/src/credential_store.rs`
- Stores single credential per provider
- No support for multiple named credentials per provider

**Gap:** Limited credential management (single credential per provider).

---

## 3. P0/P1/P2 Issue Classification

### P0 - Critical Blockers

| # | Issue | Module | Impact | Fix Suggestion |
|---|-------|--------|--------|----------------|
| P0-1 | **No API key validation** | `app.rs:773-787` | Invalid keys accepted silently | Call provider's `/v1/models` or equivalent to validate |
| P0-2 | **No model selection after API key** | `app.rs:773-787` | Users cannot select models | Wire `ConnectModelDialog` after API key save |

### P1 - High Priority

| # | Issue | Module | Impact | Fix Suggestion |
|---|-------|--------|--------|----------------|
| P1-1 | **Google OAuth not wired to TUI** | `app.rs` + `google_oauth.rs` | Google OAuth exists but unusable | Connect `GoogleOAuthService` to TUI flow |
| P1-2 | **Copilot OAuth not wired to TUI** | `app.rs` + `copilot_oauth.rs` | Copilot OAuth exists but unusable | Connect `CopilotOAuthService` to TUI flow |

### P2 - Medium Priority

| # | Issue | Module | Impact | Fix Suggestion |
|---|-------|--------|--------|----------------|
| P2-1 | **ProviderAuth trait missing** | `llm/` | No standard way to query provider capabilities | Implement `ProviderAuth` trait |
| P2-2 | **Single credential per provider** | `core/credential_store.rs` | Can't store multiple API keys | Add credential naming/labeling |

---

## 4. Technical Debt

| TD # | Issue | File | Severity | Description |
|------|-------|------|----------|-------------|
| TD-1 | Hardcoded OAuth-only list | `connect_method.rs:21` | Medium | `["google", "copilot"]` hardcoded instead of derived from provider capabilities |
| TD-2 | Default model "gpt-4o" for all | `app.rs:972` | Medium | After API key save, defaults to gpt-4o even for non-OpenAI providers |
| TD-3 | No base_url for API key providers | `app.rs:813-822` | Low | `Credential` struct has `base_url` field but it's not set during API key save |

---

## 5. Implementation Progress Summary

### Completed (from iteration-33)

| Feature | Status | Notes |
|---------|--------|-------|
| ConnectMethodDialog fix | ✅ Done | FR-007 |
| API Key Input Dialog | ✅ Done | FR-008 with 15 tests |
| Dynamic Provider Registry | ✅ Done | FR-001 |
| Modulo by 1 bug fix | ✅ Done | TD-2 |
| ConnectMethodDialog Tests | ✅ Done | 14 tests (FR-009) |
| ConnectModelDialog Tests | ✅ Done | 13 tests (FR-010) |
| Google OAuth impl | ✅ Done | 109 tests in auth_layered (FR-011) |
| Copilot OAuth impl | ✅ Done | 109 tests in auth_layered (FR-011) |

### Open Issues

| Issue | Priority | Status |
|-------|----------|--------|
| API Key Validation | P0 | Not implemented |
| Model Selection After API Key | P0 | Not wired |
| Wire Google OAuth to TUI | P1 | OAuth impl exists but not connected |
| Wire Copilot OAuth to TUI | P1 | OAuth impl exists but not connected |
| ProviderAuth trait | P2 | Not implemented |
| Multiple credentials per provider | P2 | Partial implementation |

---

## 6. Gap Summary Table

| Gap Item | Severity | Module | File:Line | 修复建议 |
|----------|----------|--------|-----------|---------|
| No API key validation | **P0** | TUI/App | `app.rs:773-787` | Call provider API to validate key before saving |
| No model selection after API key save | **P0** | TUI/App | `app.rs:773-787` | Show `ConnectModelDialog` after successful key save |
| Google OAuth not connected | P1 | TUI/App | `app.rs` + `google_oauth.rs` | Wire `GoogleOAuthService` into connect flow |
| Copilot OAuth not connected | P1 | TUI/App | `app.rs` + `copilot_oauth.rs` | Wire `CopilotOAuthService` into connect flow |
| ProviderAuth trait missing | P2 | LLM | N/A | Define trait with `supported_auth_methods()` |
| Single credential per provider | P2 | Core | `credential_store.rs` | Support multiple named credentials |
| Default model hardcoded | P2 | TUI/App | `app.rs:972` | Query available models from provider |
| OAuth-only list hardcoded | P2 | TUI | `connect_method.rs:21` | Derive from provider capabilities |

---

## 7. Recommended Priority for Iteration 34

### Immediate (P0 - Required)

1. **P0-1: Add API Key Validation**
   - After user enters API key, make a test API call to validate
   - For Anthropic: Call `/v1/models` with the key
   - For OpenAI-compatible: Call `/models` endpoint
   - Show error if validation fails, allow retry

2. **P0-2: Wire Model Selection After API Key**
   - After successful API key validation
   - Fetch available models from provider
   - Show `ConnectModelDialog` with model list
   - Set provider with selected model

### Short-term (P1)

3. **P1-1: Wire Google OAuth to TUI**
   - Connect `GoogleOAuthService` to `handle_connect_method_confirm`
   - Launch browser OAuth flow for google when "browser" selected

4. **P1-2: Wire Copilot OAuth to TUI**
   - Connect `CopilotOAuthService` to `handle_connect_method_confirm`
   - Launch device code flow for copilot

### Medium-term (P2)

5. **P2-1: Implement ProviderAuth trait**
   - Define `AuthMethod` enum with Browser, ApiKey, Local, DeviceFlow
   - Implement `ProviderAuth` trait on providers
   - Update `ConnectMethodDialog` to query provider capabilities

6. **P2-2: Support multiple credentials per provider**
   - Add credential naming/labeling
   - Allow users to manage multiple API keys per provider

---

## 8. Code References

| Component | File | Key Lines |
|-----------|------|-----------|
| ConnectMethodDialog | `crates/tui/src/dialogs/connect_method.rs` | 21-45, 75-90 |
| ApiKeyInputDialog | `crates/tui/src/dialogs/api_key_input.rs` | 55-108, 220-283 |
| API key save | `crates/tui/src/app.rs` | 773-787 |
| Browser auth flow | `crates/tui/src/app.rs` | 825-888 |
| Google OAuth | `crates/llm/src/auth_layered/google_oauth.rs` | 1-412 |
| Copilot OAuth | `crates/llm/src/auth_layered/copilot_oauth.rs` | 1-422 |
| AuthMechanism | `crates/llm/src/auth_layered/layer2_auth_mechanism.rs` | 4-33 |
| CredentialStore | `crates/core/src/credential_store.rs` | 1-300 |

---

## 9. Test Coverage Status

| Component | Unit Tests | Integration Tests |
|-----------|------------|-------------------|
| ConnectMethodDialog | 14 ✅ | 4 ✅ |
| ApiKeyInputDialog | 15 ✅ | 0 ❌ |
| Google OAuth | N/A | 109 ✅ (auth_layered tests) |
| Copilot OAuth | N/A | 109 ✅ (auth_layered tests) |
| API key validation | 0 ❌ | 0 ❌ |
| Model selection flow | 0 ❌ | 0 ❌ |

---

*Report Generated: 2026-04-17*
*Analysis Method: Direct codebase inspection*
*PRD Reference: docs/PRD/22-provider-auth-expansion.md*
*Previous Analysis: iteration-33/gap-analysis.md*
