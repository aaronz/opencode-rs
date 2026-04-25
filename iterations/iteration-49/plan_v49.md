# Implementation Plan v49 — CLI Module

**Document Version:** 49.1
**Updated:** 2026-04-26
**Based on:** CLI Module Specification v1.0 (Spec Version 49)

---

## 1. Priority Classification

### P0 — Blockers (Must Fix Before Release)

| Priority | FR | Feature | Gap Location | Required Actions |
|----------|-----|---------|--------------|------------------|
| P0 | FR-004 | `agent run` | `cmd/agent.rs:48` | Query AgentRegistry, initialize LLM provider, execute agent, stream output |
| P0 | FR-005 | `config set` | `cmd/config.rs:199-202` | Parse dot-notation key, validate value type, persist to config file, reload config |
| P0 | FR-006 | `run --format ndjson/json` | `cmd/run.rs:228-255` | Load model from Config, initialize LLM provider, stream via NdjsonSerializer |

### P1 — High Priority

| Priority | FR | Feature | Gap Location | Required Actions |
|----------|-----|---------|--------------|------------------|
| P1 | FR-007 | `account login/logout/status` | `cmd/account.rs` | Integrate with auth module, browser-based OAuth, secure credential storage |
| P1 | FR-008 | `attach` | `cmd/attach.rs:83-90` | Connect via ACP protocol, transfer session control, handle state sync |
| P1 | FR-009 | `mcp add` | `cmd/mcp.rs` | Add `Add` variant to McpAction, validate server command, persist config |
| P1 | FR-010 | `session review/diff` | `cmd/session.rs:991-1011` | Load session messages, implement diff, format for terminal |
| P1 | FR-011 | `agent list` | `cmd/agent.rs:48` | Query AgentRegistry, display agent info, support `--verbose` |
| P1 | FR-014 | `session fork` | `cmd/session.rs:673-698` | Create fork via SessionSharing, integrate with TUI |
| P1 | FR-015 | `github install` persistence | `cmd/github.rs:189-211` | Write workflow to `.opencode/`, persist to workspace config |
| P1 | FR-016 | `providers login` multi-provider | `cmd/providers.rs:142-145` | Extend browser auth to Anthropic, Google AI Studio, Azure OpenAI |
| P1 | FR-017 | `acp handshake` session storage | `cmd/acp.rs:250-292` | Store ACP session, implement recovery, handle expiration |
| P1 | FR-018 | `config migrate` | `cmd/config.rs:204-208` | Either implement TOML→JSONC migration or remove with deprecation notice |
| P1 | FR-019 | Environment variables | `main.rs` | Parse `OPENCODE_*` vars before Config load, apply overrides |
| P1 | FR-020 | Model visibility config | `cmd/models.rs:238-267` | Move visibility to Config system, deprecate flat JSON |
| P1 | FR-021 | Default model from config | `cmd/run.rs:226` | Load from Config not hardcoded "gpt-4o" |

### P2 — Medium Priority

| Priority | FR | Feature | Required Actions |
|----------|-----|---------|------------------|
| P2 | FR-012 | `auth login` (provider credentials) | Support multiple providers, browser-based OAuth, secure storage |
| P2 | FR-013 | `agent create` | Create agent config, register with AgentRegistry, persist |

---

## 2. Implementation Order

### Phase 1: P0 Blockers
1. **`run --format ndjson/json`** (FR-006) — Foundation for CLI scripting
2. **`config set`** (FR-005) — Enables runtime configuration
3. **`agent run`** (FR-004) — Core agent execution

### Phase 2: P1 High Priority
4. **`agent list`** (FR-011) — Simple registry query
5. **`account login/logout/status`** (FR-007) — Auth workflow
6. **`attach`** (FR-008) — Session connectivity
7. **`mcp add`** (FR-009) — MCP server management
8. **`session review/diff`** (FR-010) — File review functionality
9. **`session fork`** (FR-014) — TUI integration
10. **`github install`** persistence (FR-015)
11. **`providers login`** multi-provider (FR-016)
12. **`acp handshake`** session storage (FR-017)
13. **`config migrate`** (FR-018)
14. **Environment variables** (FR-019)
15. **Model visibility config** (FR-020)
16. **Default model from config** (FR-021)

### Phase 3: P2 Enhancement
17. **`auth login`** (FR-012)
18. **`agent create`** (FR-013)

---

## 3. Technical Debt Items

| Item | Location | Description | Priority |
|------|----------|-------------|----------|
| Magic string "gpt-4o" | `cmd/run.rs:226` | Default model should come from Config | P2 |
| Magic string "cmd+k" | `cmd/config.rs:217-218` | Hardcoded keybinds in JSON output | P2 |
| No error propagation | `cmd/providers.rs:67-88` | `open_browser()` uses `unwrap()` | P1 |
| Duplicated `load_config()` | Multiple cmd files | Each command loads Config independently | P2 |
| `#[allow(dead_code)]` on modules | `cmd/mod.rs` | Some modules may be unused | P3 |
| Hardcoded API base URL | `cmd/github.rs:5-8` | Should be configurable | P2 |
| `SessionRecord` duplication | `cmd/session.rs:11-24` | Duplicates core Session types | P2 |
| Async runtime creation | Multiple cmd files | Each command creates own Runtime | P2 |

---

## 4. Dependencies

- FR-006 depends on FR-005 (config set) for model configuration
- FR-004 depends on AgentRegistry from `opencode-agent`
- FR-007 depends on `opencode-auth` crate integration
- FR-008 depends on ACP protocol implementation
- FR-009 depends on MCP client in `opencode-mcp`

---

## 5. Verification

After each FR implementation:
1. Run `cargo build -p opencode-cli --all-features`
2. Run `cargo test -p opencode-cli`
3. Run `cargo clippy -p opencode-cli -- -D warnings`
