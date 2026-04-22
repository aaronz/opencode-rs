# PRD: opencode vs opencode-rs Feature Gap Analysis

## Overview

This document tracks the behavioral differences, missing features, and bugs between the original `opencode` (TypeScript/Bun v1.4.5) and `opencode-rs` (Rust implementation).

---

## 1. Command Coverage

### 1.1 opencode Commands (Original)

| Command | Status in opencode-rs |
|---------|----------------------|
| `completion` | ❌ Missing |
| `acp` | ✅ Implemented |
| `mcp` | ✅ Implemented |
| `[project]` (default tui) | ✅ Implemented as `tui` |
| `attach` | ✅ Implemented |
| `run` | ✅ Implemented |
| `debug` | ✅ Implemented |
| `providers` | ⚠️ Partial (different auth flow) |
| `agent` | ✅ Implemented |
| `upgrade` | ✅ Implemented |
| `uninstall` | ✅ Implemented |
| `serve` | ✅ Implemented |
| `web` | ✅ Implemented |
| `models` | ⚠️ Different model catalog |
| `stats` | ✅ Implemented |
| `export` | ✅ Implemented |
| `import` | ✅ Implemented |
| `github` | ❌ Missing (has `git-hub` instead) |
| `pr` | ⚠️ Different implementation |
| `session` | ✅ Implemented |
| `plugin` | ❌ Missing |
| `db` | ✅ Implemented |

### 1.2 opencode-rs Exclusive Commands

| Command | Not in Original |
|---------|----------------|
| `account` | ✅ Rust-only |
| `config` | ✅ Rust-only |
| `bash` | ✅ Rust-only |
| `terminal` | ✅ Rust-only |
| `git-hub` | ✅ Rust-only (original has `github`) |
| `git-lab` | ✅ Rust-only |
| `generate` | ✅ Rust-only |
| `thread` | ✅ Rust-only |
| `workspace-serve` | ✅ Rust-only |
| `palette` | ✅ Rust-only |
| `shortcuts` | ✅ Rust-only |
| `workspace` | ✅ Rust-only |
| `ui` | ✅ Rust-only |
| `project` | ✅ Rust-only |
| `files` | ✅ Rust-only |
| `prompt` | ✅ Rust-only |
| `quick` | ✅ Rust-only |
| `desktop` | ✅ Rust-only |

---

## 2. Authentication Flow Differences

### 2.1 Original opencode (`providers login`)

```bash
opencode providers list
# Shows credential status with auth type:
# ●  Google      oauth
# ●  OpenAI     oauth
# ●  GitHub Copilot  oauth
# ●  MiniMax    api

opencode providers login
# Interactive: select provider and authenticate
```

**Auth Methods Supported:**
- **OAuth** - Browser-based flow (Google, OpenAI, GitHub Copilot)
- **API Key** - Direct key entry (Kimi, MiniMax, Z.AI)

### 2.2 opencode-rs (`/connect` in TUI)

```bash
opencode-rs tui
# Type /connect
# Select provider from list (18 providers)
# Select auth method (only OpenAI has options; others show empty dialog)
```

**Bug:** Only OpenAI has auth methods defined. All other providers show an **empty dialog** that closes silently when Enter is pressed.

**Affected Code:** `crates/tui/src/dialogs/connect_method.rs:21-28`

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

---

## 3. Model Catalog Differences

### 3.1 Model Count

| Implementation | Model Count |
|----------------|-------------|
| opencode (original) | **89 models** |
| opencode-rs | **~20 models** |

### 3.2 opencode Exclusive Models

The original has many models not in opencode-rs:

```
opencode/gpt-5-nano
opencode/minimax-m2.5-free
opencode/nemotron-3-super-free
github-copilot/* (all copilot models)
google/antigravity-* (internal models)
```

### 3.3 opencode-rs Exclusive Models

```
azure/gpt-4o-azure
xai/grok-2
ollama/codellama
cohere/cohere-command-r-plus
```

### 3.4 Model Format Difference

| opencode | opencode-rs |
|----------|-------------|
| `github-copilot/gpt-4o` | `openai/gpt-4o` |
| `google/antigravity-gemini-3-pro` | `google/gemini-1.5-pro` |

---

## 4. Provider Configuration

### 4.1 opencode Provider List

```
●  Google (oauth)
●  Kimi For Coding (api)
●  Z.AI Coding Plan (api)
●  OpenAI (oauth)
●  GitHub Copilot (oauth)
●  MiniMax (api)
```

### 4.2 opencode-rs Provider List (in /connect)

```
openai, anthropic, google, ollama, lmstudio, azure, openrouter,
mistral, groq, deepinfra, cerebras, cohere, togetherai,
perplexity, xai, huggingface, copilot, ai21
```

**Gap:** opencode-rs has more providers listed but:
1. Most don't have working auth flows
2. Missing "Kimi For Coding", "Z.AI Coding Plan" from opencode
3. No distinction between oauth vs api key auth

---

## 5. Bug: Silent Dialog Close in /connect

### 5.1 Bug Description

**Severity:** P1 (User-facing)

When selecting any provider except OpenAI in `/connect`:
1. `ConnectMethodDialog` appears with empty list
2. User presses Enter
3. Dialog closes immediately with no feedback
4. Returns to chat mode

### 5.2 Affected Files

- `crates/tui/src/dialogs/connect_method.rs` - Empty methods handling
- `crates/tui/src/dialogs/connect_provider.rs` - Provider selection

### 5.3 Workaround Applied

Temporary fix shows "Auth Method Not Available" message for providers without auth methods, but still closes on Enter.

### 5.4 Proper Fix Required

Implement auth flows for all providers, or at minimum:
1. Show API key input dialog for providers that support it
2. Show "Not yet implemented" message for OAuth-only providers
3. Distinguish between "no auth needed" (local) vs "not implemented"

---

## 6. Missing Features from opencode

### 6.1 Plugin System

| Feature | opencode | opencode-rs |
|---------|----------|-------------|
| `opencode plugin install` | ✅ | ❌ Missing |
| Plugin discovery | ✅ | ❌ Missing |
| Plugin config in `opencode.json` | ✅ | ❌ Partial |

### 6.2 GitHub Integration

| Feature | opencode | opencode-rs |
|---------|----------|-------------|
| `opencode github` | ✅ | ❌ Missing |
| `opencode pr <number>` | ✅ Fetches and checks out PR | ⚠️ `pr` command exists but different behavior |
| GitHub agent management | ✅ | ❌ Missing |

### 6.3 Shell Completion

| Feature | opencode | opencode-rs |
|---------|----------|-------------|
| `opencode completion` | ✅ | ❌ Missing |

---

## 7. Different Behaviors

### 7.1 Session Continuation

| opencode | opencode-rs |
|----------|-------------|
| `-c, --continue` | `-c, --continue <SESSION>` |
| Requires explicit session or continues last | Can continue without specifying session |

### 7.2 Model Selection

| opencode | opencode-rs |
|----------|-------------|
| `opencode models [provider]` | `opencode-rs models` |
| Filter by provider | Shows all models |
| Shows model context lengths | Shows context lengths |

### 7.3 Server Attachment

| opencode | opencode-rs |
|----------|-------------|
| `opencode attach <url>` | `opencode-rs attach` |
| URL as positional arg | Interactive or arg |

---

## 8. Priority Recommendations

### P0 - Critical (Blocking)

1. **Fix `/connect` silent dialog bug** - Users cannot authenticate with non-OpenAI providers
2. **Add API key auth flow** - Most providers only support API key, not OAuth

### P1 - High Priority

3. **Expand model catalog** - opencode-rs has 77% fewer models
4. **Add missing providers** - Kimi, Z.AI, Copilot OAuth
5. **Implement `github` command** - parity with opencode

### P2 - Medium Priority

6. **Add plugin system** - `opencode plugin` commands
7. **Add shell completion** - `opencode completion`
8. **Add GitHub Copilot models** - `github-copilot/*` models

### P3 - Low Priority

9. **Session continuation consistency** - align `-c` behavior
10. **Workspace serve** - different concept between implementations

---

## 9. Test Coverage Gaps

### 9.1 Missing Dialog Tests

| Dialog | Test Coverage |
|--------|--------------|
| `ConnectProviderDialog` | ✅ Has basic tests |
| `ConnectMethodDialog` | ❌ No tests (was just added) |
| `ConnectModelDialog` | ❌ No tests |
| Provider auth flow | ❌ No integration tests |

### 9.2 Recommended Tests

```rust
// Test empty auth methods shows message
#[test]
fn test_connect_method_dialog_shows_message_when_empty() {
    let dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
    // Should render with message, not empty
}

// Test pressing Enter on empty dialog
#[test]
fn test_connect_method_dialog_enter_on_empty_closes() {
    let mut dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(action, DialogAction::Close);
}
```

---

## 10. Cross-References

| Document | Topic |
|----------|-------|
| [22-provider-auth-expansion.md](./22-provider-auth-expansion.md) | Provider auth details |
| [10-provider-model-system.md](./10-provider-model-system.md) | Provider abstraction |
| [09-tui-system.md](./09-tui-system.md) | TUI dialog system |

---

## 11. Appendix: Command Mapping

| opencode | opencode-rs | Notes |
|----------|-------------|-------|
| `opencode providers login` | `/connect` in TUI | Different UX |
| `opencode providers list` | `opencode-rs providers` | Similar output |
| `opencode models` | `opencode-rs models` | Different catalogs |
| `opencode github` | `opencode-rs git-hub` | Different command names |
| `opencode pr 123` | `opencode-rs pr 123` | Similar |
| `opencode session` | `opencode-rs session` | Similar |
| `opencode serve` | `opencode-rs serve` | Similar |
| `opencode web` | `opencode-rs web` | Similar |
| `opencode plugin` | ❌ Not implemented | |
| `opencode completion` | ❌ Not implemented | |
| `opencode upgrade` | `opencode-rs upgrade` | Similar |
| `opencode uninstall` | `opencode-rs uninstall` | Similar |
