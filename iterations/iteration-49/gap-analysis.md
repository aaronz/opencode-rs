# Gap Analysis Report: CLI Module vs PRD

**Module**: `opencode-cli`
**Source**: `opencode-rust/crates/cli/src/`
**Date**: 2026-04-26
**Status**: Partially Implemented — Several stub commands require full implementation

---

## 1. Gap Summary

| Category | Status | Notes |
|----------|--------|-------|
| **NdjsonSerializer** | ✅ Complete | All PRD methods implemented |
| **CLI Commands** | ⚠️ Partial | 43 command files, many stubs |
| **Test Coverage** | ✅ Good | Unit tests present for most commands |
| **Environment Variables** | ⚠️ Partial | Not explicitly parsed in CLI |

---

## 2. Gap List (Table Format)

| Gap Item | Severity | Module |修复建议 |
|----------|----------|--------|---------|
| `agent run` command only prints debug message | P0 | cmd/agent.rs:48 | Implement actual agent execution logic with LLM provider |
| `config --set` is stub with error message | P0 | cmd/config.rs:199-202 | Implement config key-value persistence via Config |
| `account` commands are stubs | P1 | cmd/account.rs:91 | Implement login/logout/status with auth module |
| `attach` command only prints messages | P1 | cmd/attach.rs:83-90 | Implement actual session attachment to remote/local |
| `mcp add` subcommand not in MCP action enum | P1 | cmd/mcp.rs | Add `Add` variant to `McpAction` with server configuration |
| `run --format ndjson/json` is placeholder output | P0 | cmd/run.rs:228-255 | Integrate with actual LLM streaming output |
| `models visibility` saves to flat JSON file | P2 | cmd/models.rs:238-267 | Consider using Config system for model visibility |
| `session review` and `session diff` are stubs | P1 | cmd/session.rs:991-1011 | Implement actual file review/diff functionality |
| `github install` doesn't persist installation | P1 | cmd/github.rs:189-211 | Persist installed workflow to workspace config |
| `providers --login` only supports OpenAI | P1 | cmd/providers.rs:142-145 | Extend to other providers (anthropic, etc.) |
| `acp` handshake doesn't store session | P1 | cmd/acp.rs:250-292 | Persist ACP session for reconnection |
| Hardcoded default model "gpt-4o" | P2 | cmd/run.rs:226 | Load default from Config or registry |
| Missing `OPENCODE_AUTO_SHARE` env var handling | P2 | main.rs | Parse and apply env vars before App init |
| Missing `OPENCODE_CONFIG` env var for config path | P2 | main.rs:66 | Use env var to override default config path |
| Missing `OPENCODE_DISABLE_AUTOUPDATE` handling | P3 | main.rs | Skip auto-update check when env var set |
| Missing `OPENCODE_ENABLE_EXA` env var | P3 | — | Not referenced in codebase |
| Missing `OPENCODE_SERVER_PASSWORD` handling | P2 | — | Pass to server config for basic auth |
| `config --migrate` is stub with error message | P1 | cmd/config.rs:204-208 | Either implement TOML→JSONC migration or remove flag |
| `agent list` subcommand not implemented | P1 | cmd/agent.rs:48 | Query AgentRegistry and list available agents |
| `session --fork` only prints JSON output | P1 | cmd/session.rs:673-698 | Actually creates fork via SessionSharing but no TUI integration |

---

## 3. Priority Classification

### P0 — Blockers (Must Fix)

1. **`agent run` is non-functional**
   - File: `cmd/agent.rs:48`
   - Issue: `run()` only prints debug message
   - Impact: Users cannot run custom agents from CLI

2. **`config --set` is stub**
   - File: `cmd/config.rs:199-202`
   - Issue: Always exits with error "Invalid setting key"
   - Impact: Users cannot modify config from CLI

3. **`run --format ndjson/json` is placeholder**
   - File: `cmd/run.rs:228-255`
   - Issue: Outputs mock JSON instead of actual LLM streaming
   - Impact: Cannot use CLI for scripted LLM interactions

### P1 — High Priority

4. **`account login/logout/status` are stubs**
   - File: `cmd/account.rs`
   - Impact: No authentication workflow from CLI

5. **`attach` command is non-functional**
   - File: `cmd/attach.rs`
   - Impact: Cannot attach to running sessions

6. **`mcp add` not implemented**
   - File: `cmd/mcp.rs`
   - Impact: Cannot add MCP servers via CLI

7. **`session review` / `session diff` are stubs**
   - File: `cmd/session.rs:991-1011`
   - Impact: Cannot review files or show diffs from session CLI

### P2 — Medium Priority

8. **`github install` doesn't persist**
   - File: `cmd/github.rs:189-211`
   - Impact: Workflow not saved to workspace

9. **`providers --login` limited to OpenAI**
   - File: `cmd/providers.rs:142-145`
   - Impact: Other providers cannot complete browser auth

10. **`models visibility` uses flat file**
    - File: `cmd/models.rs:238-267`
    - Impact: Model visibility not in Config system

11. **Hardcoded default model**
    - Multiple files
    - Impact: Should use Config default

12. **Environment variable parsing incomplete**
    - `OPENCODE_CONFIG`, `OPENCODE_AUTO_SHARE`, etc.
    - Impact: Reduced configurability

---

## 4. Technical Debt

| Item | Location | Description |
|------|----------|-------------|
| Magic string "gpt-4o" | `cmd/run.rs:226` | Default model should come from Config |
| Magic string "cmd+k" | `cmd/config.rs:217-218` | Hardcoded keybinds in JSON output |
| No error propagation | `cmd/providers.rs:67-88` | `open_browser()` uses `unwrap()` |
| Duplicated `load_config()` | Multiple cmd files | Each command loads Config independently |
| `#[allow(dead_code)]` on modules | `cmd/mod.rs` | Some modules may be unused |
| Hardcoded API base URL | `cmd/github.rs:5-8` | Should be configurable |
| `SessionRecord` duplication | `cmd/session.rs:11-24` | Duplicates core Session types |
| `ModelRow` struct | `cmd/models.rs:44-52` | Could use existing model types |
| `ProviderRow` struct | `cmd/providers.rs:23-30` | Duplicates provider info |
| Async runtime creation | Multiple cmd files | Each command creates own Runtime |
| No shared error handling | Throughout | Commands exit with `std::process::exit(1)` |

---

## 5. Implementation Progress

### Completed ✅

| Feature | Evidence |
|---------|----------|
| NdjsonSerializer with all methods | `output/ndjson.rs` — all 8 methods present |
| Test suite for NdjsonSerializer | `lib.rs:12-107` — 8 tests passing |
| CLI argument parsing (clap) | `main.rs:59-237` — comprehensive Commands enum |
| `session list/show/delete` | `cmd/session.rs` — full CRUD operations |
| `session export` | `cmd/session.rs:743-779` — JSON export |
| `session undo/redo` | `cmd/session.rs:603-667` — working history ops |
| `models list/switch` | `cmd/models.rs` — full model registry access |
| `providers list` | `cmd/providers.rs` — comprehensive provider list |
| `github login/repo-list/issue-list` | `cmd/github.rs` — OAuth + GitHub API |
| `serve` command | `cmd/serve.rs` — full server initialization |
| `web` command | `cmd/web.rs` — full web interface |
| `desktop` command | `cmd/desktop.rs` — full desktop mode |
| `acp status/connect/handshake` | `cmd/acp.rs` — HTTP API calls |
| Test coverage in most commands | Inline `#[cfg(test)]` modules |

### Not Started ❌

| Feature | PRD Reference |
|---------|---------------|
| `auth login` (provider credentials) | cli.mdx#auth-login |
| `agent create` | cli.mdx#agent-create |
| `agent list` | cli.mdx#agent-list |
| `mcp add` | cli.mdx#mcp-add |
| `session review` | cli.mdx (implied) |
| `session diff` | cli.mdx (implied) |
| `config set` persistence | ConfigSubcommand::Set |

### Stub/Placeholder 🚧

| Feature | Current State |
|---------|---------------|
| `agent run` | Prints debug message only |
| `account` commands | Returns "not_implemented" |
| `attach` | Prints URL/session_id only |
| `config set` | Errors immediately |
| `config migrate` | Errors with TOML deprecation |
| `run --format json` | Mock response only |

---

## 6. Recommendations

### Immediate (P0)
1. Implement actual LLM execution in `run` command for ndjson/json output
2. Fix `config --set` to persist key-value pairs
3. Implement `agent run` with proper agent execution

### Short Term (P1)
4. Implement account authentication workflow
5. Implement MCP server addition
6. Implement attach command for session connection
7. Implement session review/diff functionality

### Medium Term (P2)
8. Move model visibility to Config system
9. Add environment variable parsing to main.rs
10. Extend providers login to support more providers
11. Refactor duplicated config loading to shared helper

### Long Term (P3)
12. Add `agent create` command
13. Implement session sharing with proper URL handling
14. Add auto-update check with `OPENCODE_DISABLE_AUTOUPDATE`
