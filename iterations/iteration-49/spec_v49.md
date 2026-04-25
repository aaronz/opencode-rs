# CLI Module Specification v1.0

**Document Version:** 49
**Generated:** 2026-04-26
**PRD Source:** `cli.md — CLI Module`
**Status:** Partially Implemented — Gap Analysis Applied

---

## 1. Overview

### 1.1 Purpose

This document describes the CLI implementation for OpenCode RS. It provides a command-line interface with subcommands, NDJSON output serialization for piping to other tools, and webview integration.

### 1.2 Module Information

- **Crate**: `opencode-cli`
- **Source**: `crates/cli/src/lib.rs`
- **Status**: Partially Implemented — Several stub commands require full implementation

---

## 2. Crate Layout

```
crates/cli/src/
├── lib.rs              ← Re-exports, tests
├── cmd/                ← Command implementations
├── output/             ← NdjsonSerializer for streaming output
└── webview.rs          ← Webview integration
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
tokio = { version = "1.45", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

opencode-core = { path = "../core" }
opencode-config = { path = "../config" }
opencode-agent = { path = "../agent" }
```

---

## 3. Functional Requirements

### FR-001: NdjsonSerializer Implementation

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system MUST implement `NdjsonSerializer<W: Write>` with all methods for streaming output:

```rust
pub struct NdjsonSerializer<W: Write> {
    writer: W,
}

impl<W: Write> NdjsonSerializer<W> {
    pub fn new(writer: W) -> Self;
    pub fn write_message(&mut self, role: &str, content: &str) -> Result<(), io::Error>;
    pub fn write_start(&mut self, model: &str) -> Result<(), io::Error>;
    pub fn write_chunk(&mut self, content: &str) -> Result<(), io::Error>;
    pub fn write_done(&mut self) -> Result<(), io::Error>;
    pub fn write_error(&mut self, error: &str) -> Result<(), io::Error>;
    pub fn write_tool_call(&mut self, tool: &str, args: &str) -> Result<(), io::Error>;
    pub fn write_tool_result(&mut self, tool: &str, result: &str) -> Result<(), io::Error>;
    pub fn flush(&mut self) -> Result<(), io::Error>;
}
```

**Implementation:** `crates/cli/src/output/ndjson.rs`

---

### FR-002: Output Events (NDJSON format)

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system MUST output events in NDJSON format:

```json
{"event": "start", "model": "gpt-4o"}
{"event": "message", "role": "user", "content": "Hello"}
{"event": "chunk", "content": "Hello"}
{"event": "tool_call", "tool": "read", "args": {"path": "foo.txt"}}
{"event": "tool_result", "tool": "read", "result": "file contents"}
{"event": "error", "error": "something went wrong"}
{"event": "done"}
```

---

### FR-003: CLI Commands Enum

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system MUST implement comprehensive command parsing via clap:

```rust
pub enum CliCommand {
    Run { model: Option<String>, prompt: Option<String> },
    Session { subcommand: SessionSubcommand },
    List,
    Info,
    Config { subcommand: ConfigSubcommand },
    Agent { agent_type: String, prompt: String },
    Web { port: Option<u16> },
    Serve { port: Option<u16> },
    Desktop { port: Option<u16> },
}

pub enum SessionSubcommand {
    List,
    Show { id: String },
    Delete { id: String },
    Export { id: String, format: String },
}

pub enum ConfigSubcommand {
    Get { key: String },
    Set { key: String, value: String },
    Show,
}
```

**Implementation:** `crates/cli/src/main.rs:59-237`

---

### FR-004: Agent Run Command

**Priority:** P0 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/agent.rs:48` — Only prints debug message

The system MUST implement actual agent execution logic with LLM provider:

```rust
pub fn run(agent_type: &str, prompt: &str, config: &Config) -> Result<ExitCode> {
    // TODO: Implement actual agent execution
    // 1. Create agent from AgentRegistry based on agent_type
    // 2. Initialize LLM provider from config
    // 3. Execute agent with prompt
    // 4. Stream output via NdjsonSerializer if --format is set
}
```

**Required Actions:**
- Query `AgentRegistry` for requested agent type
- Initialize LLM provider from `Config`
- Execute agent with provided prompt
- Handle streaming output based on `--format` flag

---

### FR-005: Config Set Command

**Priority:** P0 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/config.rs:199-202` — Always exits with error "Invalid setting key"

The system MUST implement config key-value persistence via Config:

```rust
pub enum ConfigSubcommand {
    Set { key: String, value: String },  // Must persist to config file
}
```

**Required Actions:**
- Parse dot-notation key (e.g., `agent.model`)
- Validate value type matches expected type
- Persist to config file via Config system
- Reload config to apply changes

---

### FR-006: Run Command with LLM Streaming

**Priority:** P0 — Incomplete (Placeholder)
**Status:** 🚧 Gap: `cmd/run.rs:228-255` — Outputs mock JSON instead of actual LLM streaming

The system MUST integrate with actual LLM streaming output for `--format ndjson/json`:

```rust
pub struct RunCommand {
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub format: Option<String>,  // "ndjson" or "json"
}

impl RunCommand {
    pub async fn execute(&self, config: &Config) -> Result<ExitCode> {
        // TODO: Implement actual LLM execution
        // 1. Load model from config or use default
        // 2. Initialize LLM provider
        // 3. Send prompt to LLM with streaming
        // 4. Write events to stdout via NdjsonSerializer
    }
}
```

**Required Actions:**
- Load default model from Config (not hardcoded "gpt-4o")
- Initialize appropriate LLM provider
- Stream responses via NdjsonSerializer
- Handle tool calls and results in output

---

### FR-007: Account Authentication Commands

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/account.rs` — Returns "not_implemented"

The system MUST implement login/logout/status with auth module:

```rust
pub enum AccountSubcommand {
    Login { provider: String },    // Browser-based OAuth
    Logout { provider: String },
    Status,
}
```

**Required Actions:**
- Integrate with `opencode-auth` crate
- Implement browser-based OAuth flow
- Store credentials securely
- Display authentication status

---

### FR-008: Attach Command

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/attach.rs:83-90` — Only prints URL/session_id

The system MUST implement actual session attachment to remote/local:

```rust
pub struct AttachCommand {
    pub session_id: Option<String>,
    pub url: Option<String>,
}

impl AttachCommand {
    pub async fn execute(&self, config: &Config) -> Result<ExitCode> {
        // TODO: Implement actual session attachment
        // 1. Connect to remote/local session via ACP protocol
        // 2. Transfer control to attached session
        // 3. Handle session state synchronization
    }
}
```

**Required Actions:**
- Connect via ACP protocol to remote session
- Handle local session attachment
- Manage session state transfer

---

### FR-009: MCP Add Subcommand

**Priority:** P1 — Incomplete (Missing)
**Status:** 🚧 Gap: `cmd/mcp.rs` — `Add` variant not in `McpAction` enum

The system MUST add `Add` variant to `McpAction` with server configuration:

```rust
pub enum McpAction {
    List,
    Add { name: String, command: String, args: Vec<String> },
    Remove { name: String },
}
```

**Required Actions:**
- Add `Add` variant to `McpAction` enum
- Validate MCP server command exists
- Persist MCP server configuration
- Initialize MCP client connection

---

### FR-010: Session Review and Diff Commands

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/session.rs:991-1011` — Stubs only

The system MUST implement actual file review/diff functionality:

```rust
pub enum SessionSubcommand {
    // ... existing ...
    Review { session_id: String, file_path: String },
    Diff { session_id: String },
}
```

**Required Actions:**
- Load session messages and context
- Implement file diff using session state changes
- Show review of modified files
- Format output for terminal display

---

### FR-011: Agent List Command

**Priority:** P1 — Incomplete (Missing)
**Status:** 🚧 Gap: `cmd/agent.rs:48` — Not implemented

The system MUST query `AgentRegistry` and list available agents:

```rust
pub enum AgentSubcommand {
    List,  // Not implemented
    Run { agent_type: String, prompt: String },
}
```

**Required Actions:**
- Query AgentRegistry for all registered agents
- Display agent name, description, and capabilities
- Support `--verbose` flag for detailed info

---

### FR-012: Auth Login Command

**Priority:** P1 — Not Started
**Status:** ❌ Gap: No `auth login` implementation

The system MUST implement provider credentials login:

```bash
opencode auth login --provider openai
opencode auth login --provider anthropic
```

**Required Actions:**
- Support multiple providers (OpenAI, Anthropic, etc.)
- Browser-based OAuth flow for each provider
- Secure credential storage
- Display login status

---

### FR-013: Agent Create Command

**Priority:** P2 — Not Started
**Status:** ❌ Gap: No `agent create` implementation

The system MUST support creating custom agents:

```bash
opencode agent create --name my-agent --type custom
```

**Required Actions:**
- Create agent configuration
- Register with AgentRegistry
- Persist to config

---

### FR-014: Session Fork Command

**Priority:** P1 — Incomplete (Partial)
**Status:** 🚧 Gap: `cmd/session.rs:673-698` — Only prints JSON output, no TUI integration

The system MUST implement session forking with proper TUI integration:

```rust
pub struct SessionForkCommand {
    pub session_id: String,
    pub new_session_id: Option<String>,
}
```

**Required Actions:**
- Create fork via SessionSharing
- Properly transfer session context
- Integrate with TUI for session management

---

### FR-015: GitHub Install Persistence

**Priority:** P1 — Incomplete
**Status:** 🚧 Gap: `cmd/github.rs:189-211` — Workflow not saved to workspace

The system MUST persist installed workflow to workspace config:

```rust
pub struct GithubInstallCommand {
    pub workflow_path: String,
    pub workspace: Option<String>,
}
```

**Required Actions:**
- Write workflow to `.opencode/` directory
- Persist to workspace config
- Verify installation on subsequent runs

---

### FR-016: Providers Login Multi-Provider Support

**Priority:** P1 — Incomplete (OpenAI only)
**Status:** 🚧 Gap: `cmd/providers.rs:142-145` — Limited to OpenAI

The system MUST extend to other providers (Anthropic, etc.):

```rust
pub enum ProviderLoginCommand {
    Login { provider: String },  // Currently only supports OpenAI
}
```

**Required Actions:**
- Extend browser auth to Anthropic
- Add Google AI Studio support
- Add Azure OpenAI support
- Add custom provider support

---

### FR-017: ACP Handshake Session Persistence

**Priority:** P1 — Incomplete
**Status:** 🚧 Gap: `cmd/acp.rs:250-292` — Session not stored for reconnection

The system MUST persist ACP session for reconnection:

```rust
pub struct AcpHandshakeCommand {
    pub client_id: String,
    pub capabilities: Vec<String>,
}
```

**Required Actions:**
- Store ACP session after successful handshake
- Implement session recovery on reconnect
- Handle session expiration

---

### FR-018: Config Migrate Command

**Priority:** P1 — Incomplete (Stub)
**Status:** 🚧 Gap: `cmd/config.rs:204-208` — Stub with TOML deprecation error

The system MUST implement TOML→JSONC migration or remove flag:

```rust
pub enum ConfigSubcommand {
    Migrate,  // Currently errors with "TOML config deprecated"
}
```

**Required Actions:**
- Either implement migration from TOML to JSONC format
- Or remove the `--migrate` flag with deprecation notice

---

### FR-019: Environment Variable Parsing

**Priority:** P2 — Incomplete
**Status:** 🚧 Gap: `main.rs` — Env vars not parsed before App init

The system MUST parse and apply environment variables before App init:

| Variable | Description | Status |
|----------|-------------|--------|
| `OPENCODE_AUTO_SHARE` | Auto-share sessions | 🚧 Missing |
| `OPENCODE_CONFIG` | Config file path | 🚧 Missing |
| `OPENCODE_CONFIG_DIR` | Config directory path | 🚧 Missing |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-update check | 🚧 Missing |
| `OPENCODE_ENABLE_EXA` | Enable Exa web search | 🚧 Missing |
| `OPENCODE_SERVER_PASSWORD` | Server basic auth password | 🚧 Missing |

**Required Actions:**
- Parse env vars before Config loading
- Apply env var overrides to configuration
- Document env var precedence order

---

### FR-020: Model Visibility Config Integration

**Priority:** P2 — Incomplete (Flat file)
**Status:** 🚧 Gap: `cmd/models.rs:238-267` — Uses flat JSON file instead of Config

The system MUST use Config system for model visibility:

```rust
pub struct ModelsVisibilityCommand {
    pub model_id: String,
    pub visible: bool,
}
```

**Required Actions:**
- Move model visibility to Config system
- Update Config schema to support model visibility
- Deprecate flat JSON file

---

### FR-021: Default Model from Config

**Priority:** P2 — Incomplete (Hardcoded)
**Status:** 🚧 Gap: `cmd/run.rs:226` — Hardcoded "gpt-4o"

The system MUST load default model from Config or registry:

```rust
impl RunCommand {
    fn get_default_model(&self, config: &Config) -> String {
        config.get("agent.model").unwrap_or_else(|_| "gpt-4o".to_string())
    }
}
```

---

### FR-022: Session Undo/Redo

**Priority:** P0 — Complete
**Status:** ✅ Implemented

The system provides working history operations:

```rust
// cmd/session.rs:603-667
pub enum SessionSubcommand {
    Undo { session_id: String },
    Redo { session_id: String },
}
```

---

### FR-023: Session Export

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/session.rs:743-779
pub enum SessionSubcommand {
    Export { id: String, format: String },
}
```

---

### FR-024: Serve Command

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/serve.rs — Full server initialization
pub struct ServeCommand {
    pub port: Option<u16>,
}
```

---

### FR-025: Web Command

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/web.rs — Full web interface
pub struct WebCommand {
    pub port: Option<u16>,
}
```

---

### FR-026: Desktop Command

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/desktop.rs — Full desktop mode
pub struct DesktopCommand {
    pub port: Option<u16>,
}
```

---

### FR-027: ACP Commands

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/acp.rs — HTTP API calls
pub enum AcpSubcommand {
    Status,
    Connect { url: String },
    Handshake { client_id: String, capabilities: Vec<String> },
    Ack,
}
```

---

### FR-028: GitHub OAuth Integration

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/github.rs — OAuth + GitHub API
pub enum GithubSubcommand {
    Login,
    RepoList,
    IssueList,
}
```

---

### FR-029: Providers List

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/providers.rs — Comprehensive provider list
pub enum ProvidersSubcommand {
    List,
    Login { provider: String },
}
```

---

### FR-030: Models List/Switch

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/models.rs — Full model registry access
pub enum ModelsSubcommand {
    List,
    Switch { model: String },
}
```

---

### FR-031: Session List/Show/Delete

**Priority:** P0 — Complete
**Status:** ✅ Implemented

```rust
// cmd/session.rs — Full CRUD operations
pub enum SessionSubcommand {
    List,
    Show { id: String },
    Delete { id: String },
}
```

---

## 4. CLI Usage Reference

```bash
# Start interactive TUI
opencode

# Run single prompt
opencode run "Explain this codebase"

# Session management
opencode session list
opencode session show <id>
opencode session delete <id>
opencode session export <id> --format json

# Config management
opencode config show
opencode config get agent.model
opencode config set agent.model gpt-4o

# Agent management
opencode agent run <type> "prompt"
opencode agent list

# MCP management
opencode mcp add --name my-server --command npx --args "[\"mcp\", \"serve\"]"
opencode mcp list
opencode mcp remove --name my-server

# Server modes
opencode serve --port 8080
opencode web --port 3000
opencode desktop --port 3000

# GitHub integration
opencode github login
opencode github repo-list
opencode github issue-list

# Provider management
opencode providers list
opencode providers login --provider openai

# ACP protocol
opencode acp status
opencode acp connect --url <url>
opencode acp handshake --client-id <id> --capabilities chat,tasks
```

---

## 5. User-Facing CLI Commands Reference

| Command | Description | Status |
|---------|-------------|--------|
| `opencode` | Start TUI (default) | ✅ |
| `opencode run` | Non-interactive mode with prompt | 🚧 FR-006 |
| `opencode serve` | Headless API server | ✅ |
| `opencode web` | Web interface server | ✅ |
| `opencode attach` | Connect TUI to remote server | 🚧 FR-008 |
| `opencode agent create` | Create custom agent | ❌ FR-013 |
| `opencode agent list` | List available agents | 🚧 FR-011 |
| `opencode agent run` | Run specific agent | 🚧 FR-004 |
| `opencode models` | List available models | ✅ |
| `opencode auth login` | Add provider credentials | ❌ FR-012 |
| `opencode session list` | List sessions | ✅ |
| `opencode session export` | Export session to JSON | ✅ |
| `opencode session review` | Review session changes | 🚧 FR-010 |
| `opencode session diff` | Show session diff | 🚧 FR-010 |
| `opencode mcp add` | Add MCP server | 🚧 FR-009 |
| `opencode mcp list` | List MCP servers | ✅ |
| `opencode github install` | Install GitHub agent | 🚧 FR-015 |

---

## 6. Environment Variables

| Variable | Description | Status |
|----------|-------------|--------|
| `OPENCODE_AUTO_SHARE` | Auto-share sessions | 🚧 FR-019 |
| `OPENCODE_CONFIG` | Config file path | 🚧 FR-019 |
| `OPENCODE_CONFIG_DIR` | Config directory path | 🚧 FR-019 |
| `OPENCODE_DISABLE_AUTOUPDATE` | Disable auto-update check | 🚧 FR-019 |
| `OPENCODE_ENABLE_EXA` | Enable Exa web search | 🚧 FR-019 |
| `OPENCODE_SERVER_PASSWORD` | Server basic auth password | 🚧 FR-019 |

---

## 7. Technical Debt

| Item | Location | Description | Priority |
|------|----------|-------------|----------|
| Magic string "gpt-4o" | `cmd/run.rs:226` | Default model should come from Config | P2 |
| Magic string "cmd+k" | `cmd/config.rs:217-218` | Hardcoded keybinds in JSON output | P2 |
| No error propagation | `cmd/providers.rs:67-88` | `open_browser()` uses `unwrap()` | P1 |
| Duplicated `load_config()` | Multiple cmd files | Each command loads Config independently | P2 |
| `#[allow(dead_code)]` on modules | `cmd/mod.rs` | Some modules may be unused | P3 |
| Hardcoded API base URL | `cmd/github.rs:5-8` | Should be configurable | P2 |
| `SessionRecord` duplication | `cmd/session.rs:11-24` | Duplicates core Session types | P2 |
| `ModelRow` struct | `cmd/models.rs:44-52` | Could use existing model types | P3 |
| `ProviderRow` struct | `cmd/providers.rs:23-30` | Duplicates provider info | P3 |
| Async runtime creation | Multiple cmd files | Each command creates own Runtime | P2 |
| No shared error handling | Throughout | Commands exit with `std::process::exit(1)` | P2 |

---

## 8. Implementation Status Summary

### Completed ✅

| Feature | FR Number |
|---------|-----------|
| NdjsonSerializer with all methods | FR-001 |
| Test suite for NdjsonSerializer | FR-001 |
| CLI argument parsing (clap) | FR-003 |
| `session list/show/delete` | FR-031 |
| `session export` | FR-023 |
| `session undo/redo` | FR-022 |
| `models list/switch` | FR-030 |
| `providers list` | FR-029 |
| `github login/repo-list/issue-list` | FR-028 |
| `serve` command | FR-024 |
| `web` command | FR-025 |
| `desktop` command | FR-026 |
| `acp status/connect/handshake` | FR-027 |

### In Progress 🚧

| Feature | FR Number | Gap Location |
|---------|-----------|--------------|
| `agent run` | FR-004 | `cmd/agent.rs:48` |
| `config set` | FR-005 | `cmd/config.rs:199-202` |
| `run --format ndjson/json` | FR-006 | `cmd/run.rs:228-255` |
| `account login/logout/status` | FR-007 | `cmd/account.rs` |
| `attach` | FR-008 | `cmd/attach.rs:83-90` |
| `mcp add` | FR-009 | `cmd/mcp.rs` |
| `session review/diff` | FR-010 | `cmd/session.rs:991-1011` |
| `agent list` | FR-011 | `cmd/agent.rs:48` |
| `session fork` | FR-014 | `cmd/session.rs:673-698` |
| `github install` persistence | FR-015 | `cmd/github.rs:189-211` |
| `providers login` multi-provider | FR-016 | `cmd/providers.rs:142-145` |
| `acp handshake` session storage | FR-017 | `cmd/acp.rs:250-292` |
| `config migrate` | FR-018 | `cmd/config.rs:204-208` |
| Environment variables | FR-019 | `main.rs` |
| Model visibility config | FR-020 | `cmd/models.rs:238-267` |
| Default model from config | FR-021 | `cmd/run.rs:226` |

### Not Started ❌

| Feature | FR Number |
|---------|-----------|
| `auth login` (provider credentials) | FR-012 |
| `agent create` | FR-013 |

---

## 9. Test Requirements

```rust
#[cfg(test)]
mod tests {
    use super::output::NdjsonSerializer;

    #[test]
    fn test_ndjson_serializer_write_message() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_message("user", "Hello").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"message\""));
        assert!(output.contains("\"role\":\"user\""));
        assert!(output.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_ndjson_serializer_write_start() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_start("gpt-4").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"start\""));
        assert!(output.contains("\"model\":\"gpt-4\""));
    }

    #[test]
    fn test_ndjson_serializer_write_error() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_error("something went wrong").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"error\""));
        assert!(output.contains("\"error\":\"something went wrong\""));
    }

    #[test]
    fn test_ndjson_serializer_write_tool_call() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_tool_call("read", r#"{"path": "foo.txt"}"#).unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"tool_call\""));
        assert!(output.contains("\"tool\":\"read\""));
    }

    #[test]
    fn test_ndjson_serializer_write_done() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_done().unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"done\""));
    }
}
```

---

## 10. Acceptance Criteria

### Must Have (P0)
- [ ] `agent run` executes actual agent with LLM provider
- [ ] `config set` persists key-value pairs to config file
- [ ] `run --format ndjson/json` streams actual LLM responses

### Should Have (P1)
- [ ] `account login/logout/status` works with auth module
- [ ] `attach` connects to remote/local sessions
- [ ] `mcp add` adds MCP servers
- [ ] `session review/diff` shows file changes
- [ ] `agent list` displays available agents
- [ ] ACP handshake persists session

### Could Have (P2)
- [ ] Environment variables are parsed before config load
- [ ] Model visibility uses Config system
- [ ] Default model loads from Config
- [ ] `providers login` supports multiple providers
- [ ] `github install` persists workflow to workspace

---

## 11. Cross-References

- [PRD](./cli.md) — Source PRD document
- [Gap Analysis](./gap-analysis.md) — Detailed gap analysis for this module

---

*Document generated from gap analysis. Implementation status tracked per FR number.*