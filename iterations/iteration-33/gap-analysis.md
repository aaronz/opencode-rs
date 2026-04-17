# Gap Analysis: opencode vs opencode-rs Feature Parity

**Project:** opencode-rs (Rust Implementation)
**Analysis Date:** 2026-04-17
**PRD Reference:** User-provided PRD (opencode vs opencode-rs Feature Gap Analysis)
**Analysis Mode:** Implementation vs Original opencode (TypeScript/Bun v1.4.5)
**Iteration:** 33

---

## 1. Executive Summary

The opencode-rs implementation has **significant feature gaps** compared to the original TypeScript implementation. The most critical issue is the **P0 silent dialog bug** in `/connect` which prevents users from authenticating with any non-OpenAI provider. Secondary concerns include a **77% smaller model catalog** and several missing CLI commands.

| Category | opencode (Original) | opencode-rs | Gap |
|----------|-------------------|-------------|-----|
| Commands | 21 listed | 18 providers in dialog / 3 in CLI | ⚠️ Parity |
| Model Count | 89 models | ~17 models (ModelRegistry) | ❌ 81% fewer |
| Auth Methods | OAuth + API Key | Only OpenAI OAuth | ❌ Partial |
| Provider List | 6 providers | 18 in dialog / 3 in CLI | ⚠️ Inconsistent |

---

## 2. Gap Analysis by Category

### 2.1 Command Coverage Gaps

| opencode Command | opencode-rs Status | Gap Severity |
|-----------------|---------------------|--------------|
| `completion` | ❌ **Missing** | P1 |
| `github` | ⚠️ Renamed to `git-hub` | P2 |
| `plugin` | ❌ **Missing** (no CLI command) | P1 |
| `pr` | ⚠️ Stub implementation | P1 |
| `providers` | ⚠️ Only 3 providers listed | P1 |

### 2.2 Authentication Flow Gaps

**P0 Bug Confirmed:** `crates/tui/src/dialogs/connect_method.rs:21-28`

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

| Provider | opencode Auth | opencode-rs Auth | Status |
|----------|--------------|-----------------|--------|
| Google | OAuth | ❌ None | P0 Gap |
| OpenAI | OAuth + API Key | ✅ OAuth + API Key | Complete |
| GitHub Copilot | OAuth | ❌ None | P0 Gap |
| Kimi | API Key | ❌ None | P0 Gap |
| Z.AI | API Key | ❌ None | P0 Gap |
| MiniMax | API Key | ❌ None | P0 Gap |

### 2.3 Model Catalog Gap

| Metric | opencode | opencode-rs | Gap |
|--------|----------|-------------|-----|
| Total Models | 89 | ~17 (ModelRegistry::new) | **-81%** |
| Provider Models | github-copilot/* (all) | ❌ None | P1 |
| MiniMax Models | minimax-m2.5-free | ❌ None | P1 |
| OpenCode Exclusive | opencode/gpt-5-nano | ❌ None | P1 |

**Confirmed Models in opencode-rs** (`crates/llm/src/models.rs`):
- openai: gpt-4o, gpt-4o-mini, gpt-4-turbo
- anthropic: claude-sonnet-4-20250514, claude-haiku-3, claude-opus-4-20250514
- ollama: llama3, codellama
- azure: gpt-4o-azure
- google: gemini-1.5-pro, gemini-1.5-flash
- openrouter: openrouter/gpt-4o
- xai: grok-2
- mistral: mistral-large-latest
- groq: llama-3.1-70b-versatile
- deepinfra: deepinfra/llama-3.1-70b
- cerebras: cerebras/llama-3.1-70b
- cohere: cohere-command-r-plus
- togetherai: togetherai/llama-3.1-70b
- perplexity: perplexity/llama-3.1-sonar-large

### 2.4 Test Coverage Gaps

| Dialog | Test Coverage | Status |
|--------|--------------|--------|
| `ConnectProviderDialog` | ✅ Has tests | Complete |
| `ConnectMethodDialog` | ❌ **No tests** | **P1 Gap** |
| `ConnectModelDialog` | ❌ **No tests** | P2 |
| Provider auth flow | ❌ No integration tests | P1 |

**Missing Tests per PRD Section 9.2:**
```rust
// Test empty auth methods shows message
#[test]
fn test_connect_method_dialog_shows_message_when_empty()

// Test pressing Enter on empty dialog
#[test]
fn test_connect_method_dialog_enter_on_empty_closes()
```

---

## 3. P0/P1/P2 Issue Classification

### P0 - Critical Blockers (Must Fix)

| # | Issue | Module | Impact |
|---|-------|--------|--------|
| P0-1 | **Silent dialog close bug** - `ConnectMethodDialog` returns empty for all providers except OpenAI. User presses Enter and dialog closes with no feedback. | `tui/dialogs/connect_method.rs:21-28` | Users cannot authenticate with non-OpenAI providers |
| P0-2 | **No API key auth flow** - Most providers (Kimi, Z.AI, MiniMax) only support API key, but no input dialog exists | `tui/dialogs/` | Cannot use most providers |
| P0-3 | **`providers` CLI only lists 3 providers** - `crates/cli/src/cmd/providers.rs:94` hardcodes `["openai", "anthropic", "ollama"]` but dialog shows 18 | `cli/providers.rs` | Inconsistent UX, missing providers |

### P1 - High Priority Issues

| # | Issue | Module | Impact |
|---|-------|--------|--------|
| P1-1 | **`completion` command missing** - Shell completion generation not implemented | `cli/` | User setup friction |
| P1-2 | **`plugin` CLI command missing** - `opencode plugin install` not available despite plugin crate existing | `cli/` | No plugin extensibility |
| P1-3 | **`pr` command is stub** - Just prints "PR action: {:?}" without actual implementation | `cli/pr.rs:39-40` | GitHub PR integration incomplete |
| P1-4 | **Model catalog 81% smaller** - Only ~17 models vs 89 in original | `llm/models.rs` | Limited model choice |
| P1-5 | **No `ConnectMethodDialog` tests** - Dialog has no unit tests despite being user-facing | `tui/dialogs/connect_method.rs` | Regression risk |
| P1-6 | **`github` vs `git-hub` naming** - Original has `github`, rs has `git-hub` | `cli/` | Command naming inconsistency |

### P2 - Medium Priority

| # | Issue | Module | Impact |
|---|-------|--------|--------|
| P2-1 | **`github` Login/RepoList/IssueList are TODO** - `crates/cli/src/cmd/github.rs:80-86` prints "TODO" messages | `cli/github.rs` | Incomplete GitHub integration |
| P2-2 | **No `ConnectModelDialog` tests** | `tui/` | Test gap |
| P2-3 | **Missing Kimi and Z.AI providers** - opencode has these but opencode-rs does not | `llm/` | Provider gap |
| P2-4 | **OAuth only implemented for OpenAI** - Google, Copilot OAuth not implemented | `llm/` | Limited auth options |

---

## 4. Technical Debt

| # | Issue | Module | Severity | Description |
|---|-------|--------|----------|-------------|
| TD-1 | **Hardcoded provider list** | `providers.rs:94` | High | `["openai", "anthropic", "ollama"]` should be dynamic |
| TD-2 | **connect_method.rs:105** | `tui/` | Medium | `self.methods.len().max(1)` - modulo by 1 always returns 0 |
| TD-3 | **GitHub command TODOs** | `github.rs:80-86` | Medium | Login, RepoList, IssueList not implemented |
| TD-4 | **PR command stub** | `pr.rs:40` | Medium | Just prints debug output |
| TD-5 | **Catalog models_dev.rs unused** | `llm/catalog/` | Low | Type definition exists but may not be populated |

---

## 5. Implementation Progress Summary

### 5.1 Command Implementation Status

| Command | Status | Notes |
|---------|--------|-------|
| `acp` | ✅ Implemented | Full ACP protocol |
| `mcp` | ✅ Implemented | MCP client/server |
| `tui` | ✅ Implemented | Default interface |
| `attach` | ✅ Implemented | Server attachment |
| `run` | ✅ Implemented | Agent execution |
| `debug` | ✅ Implemented | Debug mode |
| `providers` | ⚠️ Partial | Only 3 providers hardcoded |
| `agent` | ✅ Implemented | Agent selection |
| `upgrade` | ✅ Implemented | Auto-upgrade |
| `uninstall` | ✅ Implemented | Removal |
| `serve` | ✅ Implemented | HTTP server |
| `web` | ✅ Implemented | Web interface |
| `models` | ⚠️ Partial | Different catalog |
| `stats` | ✅ Implemented | Statistics |
| `export/import` | ✅ Implemented | Session portability |
| `github` | ⚠️ Renamed to `git-hub` + incomplete | P1 issues |
| `pr` | ❌ Stub | Just prints debug |
| `session` | ✅ Implemented | Session management |
| `plugin` | ❌ Missing CLI command | P1 |
| `db` | ✅ Implemented | Database CLI |
| `completion` | ❌ Missing | P1 |
| `git-hub` | ✅ Rust-only extra | GitHub workflow |
| `git-lab` | ✅ Rust-only extra | GitLab integration |
| `account` | ✅ Rust-only extra | Account management |
| `config` | ✅ Rust-only extra | Config management |
| `bash/terminal` | ✅ Rust-only extra | Shell integration |

### 5.2 Dialog Implementation Status

| Dialog | File | Tests | Status |
|--------|------|-------|--------|
| ConnectProviderDialog | `connect_provider.rs` | ✅ 2 tests | Complete |
| ConnectMethodDialog | `connect_method.rs` | ❌ None | **P0 Bug** |
| ConnectModelDialog | `connect_model.rs` | ❌ None | Needs tests |

---

## 6. Gap Summary Table

| Gap Item | Severity | Module | 修复建议 |
|----------|----------|--------|---------|
| Silent dialog close (empty auth methods) | **P0** | `tui/dialogs/connect_method.rs:21-28` | Add API key auth method for all providers, or show "Not implemented" message |
| `providers` CLI hardcoded to 3 providers | **P0** | `cli/cmd/providers.rs:94` | Read from provider registry dynamically |
| No API key input dialog for providers | **P0** | `tui/dialogs/` | Implement API key input flow |
| `completion` command missing | P1 | `cli/` | Add shell completion generation |
| `plugin` CLI command missing | P1 | `cli/` | Add `plugin install/list/remove` commands |
| `pr` command is stub | P1 | `cli/pr.rs:40` | Implement PR fetch/checkout |
| Model catalog 81% smaller | P1 | `llm/models.rs` | Expand model catalog from catalog service |
| `ConnectMethodDialog` has no tests | P1 | `tui/dialogs/connect_method.rs` | Add unit tests per PRD section 9.2 |
| `github` vs `git-hub` naming | P2 | `cli/` | Align command names or add alias |
| GitHub Login/RepoList/IssueList not implemented | P2 | `cli/github.rs:80-86` | Complete implementation or remove |
| `ConnectModelDialog` has no tests | P2 | `tui/` | Add rendering and interaction tests |
| Missing Kimi, Z.AI providers | P2 | `llm/` | Add to provider registry |
| Google/Copilot OAuth not implemented | P2 | `llm/` | Add OAuth flows |

---

## 7. Priority Recommendations

### Immediate (P0 - Required Before Release)

1. **Fix silent dialog bug in `ConnectMethodDialog`**
   - Option A: Show "API Key" method for all providers
   - Option B: Show "Not yet implemented" message with explanation
   - Must not close silently on Enter

2. **Fix `providers` CLI hardcoded list**
   - Remove hardcoded `["openai", "anthropic", "ollama"]`
   - Use dynamic provider registry

3. **Implement API key input flow**
   - Add text input dialog for API key entry
   - Validate and store credentials

### Short-term (P1 - Next Sprint)

1. Add `completion` command for shell setup
2. Add `plugin` CLI command (`install`, `list`, `remove`)
3. Implement `pr` command properly (fetch PR details, checkout)
4. Expand model catalog to 50+ models
5. Add tests for `ConnectMethodDialog`

### Medium-term (P2)

1. Complete GitHub integration (`Login`, `RepoList`, `IssueList`)
2. Add OAuth flows for Google and Copilot
3. Add Kimi and Z.AI providers
4. Align `github` command naming with original

---

## 8. Cross-References

| Document | Topic |
|----------|-------|
| `crates/tui/src/dialogs/connect_method.rs` | Auth method dialog bug |
| `crates/tui/src/dialogs/connect_provider.rs` | Provider selection (18 providers) |
| `crates/cli/src/cmd/providers.rs` | CLI provider list (3 hardcoded) |
| `crates/llm/src/models.rs` | Model registry (~17 models) |
| `crates/llm/src/auth_layered/layer2_auth_mechanism.rs` | Auth mechanism enum |
| `crates/cli/src/cmd/github.rs` | GitHub command (TODOs) |
| `crates/cli/src/cmd/pr.rs` | PR command (stub) |

---

## 9. Appendix: Confirmed Bug Locations

### A.1 ConnectMethodDialog Empty Methods (P0)

**File:** `crates/tui/src/dialogs/connect_method.rs`
**Lines:** 21-28

```rust
let methods = if provider_id == "openai" {
    vec![
        ("browser".to_string(), "Browser auth".to_string()),
        ("api_key".to_string(), "API key".to_string()),
    ]
} else {
    Vec::new()  // BUG: All other providers get empty list
};
```

### A.2 Hardcoded Provider List (P0)

**File:** `crates/cli/src/cmd/providers.rs`
**Line:** 94

```rust
let providers = ["openai", "anthropic", "ollama"]
    .into_iter()
    // ...
```

### A.3 Stub PR Command

**File:** `crates/cli/src/cmd/pr.rs`
**Lines:** 39-40

```rust
pub(crate) fn run(args: PrArgs) {
    println!("PR action: {:?}", args.action);  // STUB - no actual implementation
}
```

### A.4 GitHub TODO Commands

**File:** `crates/cli/src/cmd/github.rs`
**Lines:** 79-86

```rust
GitHubAction::Login => {
    println!("GitHub login - TODO: Implement OAuth flow");  // TODO
}
GitHubAction::RepoList => {
    println!("GitHub repo list - TODO: List repositories");  // TODO
}
GitHubAction::IssueList { repo } => {
    println!("GitHub issues for {} - TODO", repo);  // TODO
}
```

---

*Report Generated: 2026-04-17*
*Analysis Method: Direct codebase inspection of opencode-rust/crates/*
*PRD Reference: User-provided opencode vs opencode-rs Feature Gap Analysis*
*Previous Analysis: iteration-32/gap-analysis.md (ratatui-testing)*
