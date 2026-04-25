# PRD: Provider Authentication Expansion

## Overview

This document describes the missing authentication implementation for non-OpenAI providers in the `/connect` flow. Currently, only OpenAI has a functional browser-based authentication flow. All other providers either require manual API key configuration or show an empty/inapplicable dialog.

---

## Current State

### `/connect` Flow

1. User types `/connect` in TUI
2. `ConnectProviderDialog` shows list of 18 providers:
   - openai, anthropic, google, ollama, lmstudio, azure, openrouter, mistral, groq, deepinfra, cerebras, cohere, togetherai, perplexity, xai, huggingface, copilot, ai21
3. User selects a provider
4. `ConnectMethodDialog` appears with auth method options

### The Bug

`ConnectMethodDialog` only populates methods for `provider_id == "openai"`:

```rust
let methods = if provider_id == "openai" {
    vec![
        ("browser".to_string(), "Browser auth".to_string()),
        ("api_key".to_string(), "API key".to_string()),
    ]
} else {
    Vec::new()  // Empty for all other providers!
};
```

For non-OpenAI providers:
- The dialog appears with border and title "Select Auth Method"
- **No list items render** (empty `Vec`)
- User presses Enter → `methods.is_empty()` returns `DialogAction::Close`
- Dialog closes immediately with **no user feedback**

### OpenAI Auth Flow

For OpenAI with "browser" method:
1. `start_openai_browser_connect()` spawns a thread
2. Thread starts local HTTP callback listener
3. Opens browser to OpenAI OAuth URL
4. Waits for callback, exchanges code for session
5. Fetches available models
6. Shows `ConnectModelDialog` to pick a model

---

## Problem Statement

Users expect all providers to have a functional authentication flow. Instead they encounter:

1. **Silent failure** for non-OpenAI providers (dialog flashes and closes)
2. **No indication** that only OpenAI browser auth is implemented
3. **Incomplete provider coverage** in the TUI connect flow

---

## Proposed Solution

### Option A: Implement Auth for All Providers (Recommended)

Each provider requires a different auth mechanism:

| Provider | Auth Method | Implementation Notes |
|----------|-------------|---------------------|
| OpenAI | Browser OAuth | Already implemented |
| Anthropic | API Key | Prompt for key, validate with `/v1/models` |
| Google | API Key / OAuth | Gemini API keys |
| Azure | API Key + Endpoint | Requires deployment-specific endpoint |
| Ollama | Local (no auth) | Connect directly to local endpoint |
| LM Studio | Local (no auth) | Connect directly to local endpoint |
| OpenRouter | API Key | Standard API key flow |
| Mistral | API Key | Standard API key flow |
| Groq | API Key | Standard API key flow |
| DeepInfra | API Key | Standard API key flow |
| Cerebras | API Key | Standard API key flow |
| Cohere | API Key | Standard API key flow |
| TogetherAI | API Key | Standard API key flow |
| Perplexity | API Key | Standard API key flow |
| xAI | API Key | Standard API key flow |
| HuggingFace | Token | HF inference endpoints |
| GitHub Copilot | Device flow | OAuth device code flow |
| AI21 | API Key | Standard API key flow |

### Option B: Informative Fallback (Quick Fix)

 пока the empty dialog properly:

```rust
if self.methods.is_empty() {
    // Show: "API key authentication only. Configure in settings."
    // Press Enter to go back
}
```

This is the minimal fix that provides user feedback.

---

## Implementation Plan

### Phase 1: Informative Fallback (Low Effort)

Update `ConnectMethodDialog` to handle empty methods gracefully:
- Show title "Auth Method Not Available" instead of "Select Auth Method"
- Display message explaining only API key config is available
- Link to settings or show inline input

### Phase 2: API Key Input Dialog (Medium Effort)

Add a new `ApiKeyInputDialog` that:
- Shows when provider supports API key auth
- Has input field for API key
- Validates key by making a test API call
- Stores credential in secure storage

### Phase 3: Provider-Specific Auth Flows (Higher Effort)

Implement per-provider auth:
- Anthropic: API key input + validation
- Google: API key input + validation
- OAuth providers: Browser-based flow similar to OpenAI

---

## User Experience

### Current (Broken)

```
/connect
→ Select Provider: [Anthropic]
→ Dialog flashes (no visible items)
→ Returns to chat, no feedback
```

### With Informative Fallback

```
/connect
→ Select Provider: [Anthropic]
→ Dialog: "Auth Method Not Available"
         "API key authentication only. Configure in settings."
         Press Enter to go back.
→ User presses Enter, returns to provider list
```

### With Full Implementation

```
/connect
→ Select Provider: [Anthropic]
→ Select Auth Method: [API Key] → [Connect]
→ API Key Input Dialog
→ Enter key → Validating...
→ Success: Select Model Dialog
```

---

## Technical Notes

### Provider Capability Discovery

Each provider should declare its supported auth methods:

```rust
pub enum AuthMethod {
    Browser,      // OAuth flow
    ApiKey,       // Direct API key
    Local,       // No auth (localhost)
    DeviceFlow,   // OAuth device code
}

pub trait ProviderAuth {
    fn supported_auth_methods(&self) -> Vec<AuthMethod>;
}
```

### Credential Storage

- Use `opencode-auth` crate's `CredentialStore`
- Support for multiple credentials per provider
- Secure storage (keychain on macOS, libsecret on Linux)

---

## Cross-References

| Document | Topic |
|----------|-------|
| [10-provider-model-system.md](./10-provider-model-system.md) | Provider abstraction and auth patterns |
| [09-tui-system.md](./09-tui-system.md) | TUI dialog system |
| [06-configuration-system.md](./06-configuration-system.md) | Config schema for credentials |
