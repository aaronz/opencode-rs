# OpenCode RS - Product Requirements Document

## 1. Overview

### Project Name
**OpenCode RS** - A Rust implementation of OpenCode AI coding agent

### Project Type
Monorepo containing multiple crates for an AI-powered coding agent with TUI, HTTP API, and SDK components.

### Core Value Proposition
OpenCode RS provides an extensible AI coding agent that integrates with multiple LLM providers, offers developer tools (file operations, grep, git), and exposes capabilities via TUI, HTTP API, and Rust SDK.

### Target Users
- **Individual Developers**: Use via TUI for interactive coding assistance
- **Enterprise Teams**: Deploy via HTTP API for centralized coding workflows
- **Tool Builders**: Extend via SDK for custom integrations
- **IDE/Editor Extensions**: LSP integration for editor embedding

---

## 2. Architecture Overview

### Crate Structure

```
opencode-rust/
├── crates/
│   ├── core/           # Core runtime, session management, tool registry, error handling
│   ├── cli/            # CLI commands and entry points
│   ├── llm/            # LLM provider integrations (OpenAI, Claude, Ollama)
│   ├── tools/          # Tool implementations (read, write, grep, git, search)
│   ├── agent/          # Agent orchestration, planning, task execution
│   ├── tui/            # Terminal UI (ratatui-based)
│   ├── lsp/            # Language Server Protocol integration
│   ├── storage/        # SQLite-based session persistence
│   ├── server/         # HTTP API server (actix-web)
│   ├── auth/           # Authentication
│   ├── permission/     # Permission system
│   ├── plugin/         # Plugin architecture
│   ├── git/            # Git operations
│   ├── mcp/            # Model Context Protocol support
│   └── sdk/            # Rust SDK for programmatic access
├── tests/              # Integration tests
├── opencode-benches/   # Performance benchmarks
└── ratatui-testing/   # TUI testing framework
```

### Key Components

| Component | Responsibility |
|-----------|----------------|
| **opencode-core** | Session management, tool registry, error system |
| **opencode-agent** | Orchestrates LLM + tools for task execution |
| **opencode-llm** | Multi-provider LLM adapter (OpenAI, Anthropic, Ollama) |
| **opencode-tools** | Developer tools: file I/O, grep, git, search |
| **opencode-tui** | Interactive terminal user interface |
| **opencode-server** | HTTP REST API for remote access |
| **opencode-storage** | SQLite persistence for sessions |
| **opencode-sdk** | Public Rust API for external consumers |

---

## 3. Core Features

### 3.1 LLM Provider Support

| Provider | Status | Models |
|----------|--------|--------|
| OpenAI | ✅ Supported | GPT-4, GPT-3.5 |
| Anthropic Claude | ✅ Supported | Claude 3 Opus, Sonnet, Haiku |
| Ollama (local) | ✅ Supported | Llama2, Mistral, custom models |

**Requirements:**
- Configurable via environment variables or config files
- Support for model-specific parameters (temperature, max tokens)
- Streaming support for real-time responses

### 3.2 Tool System

**Built-in Tools:**

| Tool | Description | Priority |
|------|-------------|----------|
| `Read` | Read file contents with line range support | P0 |
| `Write` | Create or overwrite files | P0 |
| `Edit` | Apply targeted edits to files | P0 |
| `Grep` | Search file contents with regex | P0 |
| `Glob` | Find files by pattern | P1 |
| `Git` | Git operations (status, diff, log, commit) | P1 |
| `Bash` | Execute shell commands | P1 |
| `WebSearch` | Search the web | P2 |

**Tool Registry:**
- Plugins can register custom tools
- Tools have names, descriptions, parameter schemas
- Permission system controls tool access

### 3.3 Agent Modes

| Mode | Capabilities | Use Case |
|------|--------------|----------|
| `Build` | Full tool access, can modify files | Implementation |
| `Plan` | Read-only, analysis only | Planning, review |
| `General` | Search and research | Investigation |

### 3.4 User Interfaces

**TUI (Terminal UI):**
- Interactive command palette
- Session history browser
- Real-time output streaming
- Built with ratatui

**HTTP API:**
- REST endpoints for all agent operations
- WebSocket support for streaming
- ACP (Agent Communication Protocol) routes

**CLI:**
- Shell-like command interface
- Scriptable batch operations

**SDK:**
- Rust library for programmatic access
- Async/await API design

### 3.5 Session Management

- **Persistence**: SQLite-based storage
- **Session Types**: Conversations, implementations, code reviews
- **Resume**: Continue interrupted sessions
- **Export**: JSON export/import for portability

### 3.6 MCP (Model Context Protocol)

- Connect to external MCP servers
- Tool discovery from MCP servers
- Remote MCP server connections
- Local MCP server support

---

## 4. Feature Requirements

### 4.1 P0 - Must Have

| Feature | Description | Verification |
|---------|-------------|--------------|
| Session management | Create, save, resume sessions | `cargo test test_session_*` |
| Tool execution | Execute tools via agent | `cargo test test_tool_*` |
| LLM integration | Connect to at least one provider | Manual verification |
| TUI basic operations | Navigate, select, execute | `cargo test test_tui_*` |
| File operations | Read/write/edit files | `cargo test test_file_*` |
| Build verification | `cargo build` passes | CI pipeline |

### 4.2 P1 - Should Have

| Feature | Description |
|---------|-------------|
| Multi-provider LLM | Support 2+ LLM providers |
| Permission system | Role-based tool access |
| MCP integration | Connect to MCP servers |
| Plugin system | Load external plugins |
| HTTP API | Remote agent access |
| Git integration | Full git workflow |

### 4.3 P2 - Nice to Have

| Feature | Description |
|---------|-------------|
| LSP integration | IDE editor support |
| WebSocket streaming | Real-time agent output |
| SDK documentation | Public API docs |
| Benchmark suite | Performance regression tests |

---

## 5. Non-Functional Requirements

### 5.1 Performance

| Metric | Target |
|--------|--------|
| Cold start time | < 2 seconds |
| Tool execution latency | < 500ms (local), < 2s (remote) |
| LLM response streaming | < 100ms time-to-first-token |
| Session load time | < 1 second |

### 5.2 Reliability

- **Build**: All crates compile with `cargo build --release`
- **Tests**: Integration tests pass with `cargo test`
- **Clippy**: Zero warnings with `cargo clippy --all -- -D warnings`
- **Formatting**: Pass `cargo fmt --all -- --check`

### 5.3 Security

- No hardcoded credentials (env vars for secrets)
- Argon2/bcrypt for password hashing
- AES-GCM for data at rest encryption
- JWT for API authentication
- Permission enforcement per endpoint

### 5.4 Compatibility

- **Rust**: 1.70+
- **Platforms**: macOS, Linux, Windows
- **LLM Providers**: OpenAI API compatible, Anthropic API, Ollama local

---

## 6. API Specification

### 6.1 HTTP Server Endpoints

```
GET  /api/status              - Server status
POST /api/session             - Create new session
GET  /api/session/{id}       - Get session details
POST /api/session/{id}/execute - Execute agent task
GET  /api/session/{id}/history - Get conversation history
```

### 6.2 ACP Routes

```
GET  /api/acp/status         - ACP status
POST /api/acp/handshake      - Perform handshake
POST /api/acp/connect        - Connect to server
POST /api/acp/ack            - Acknowledge handshake
```

---

## 7. Data Models

### 7.1 Session

```json
{
  "id": "uuid",
  "created_at": "timestamp",
  "updated_at": "timestamp",
  "mode": "build|plan|general",
  "messages": [
    {
      "role": "user|assistant",
      "content": "string",
      "timestamp": "timestamp"
    }
  ],
  "metadata": {}
}
```

### 7.2 Tool

```json
{
  "name": "string",
  "description": "string",
  "parameters": {
    "type": "object",
    "properties": {}
  },
  "permission_level": "read|write|admin"
}
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

- Each crate has inline `#[cfg(test)]` modules
- Test individual functions and traits
- Run: `cargo test --lib`

### 8.2 Integration Tests

- `tests/` directory with full workflow tests
- Test agent + tool + LLM integration
- Run: `cargo test --test '*'`

### 8.3 TUI Testing

- `ratatui-testing` framework for snapshot tests
- PTY simulation for interactive testing
- Run: `cargo test -p ratatui-testing`

### 8.4 Benchmarking

- `opencode-benches/` for performance regression
- Run: `cargo bench`

---

## 9. Configuration

### 9.1 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_LLM_PROVIDER` | LLM provider name | openai |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `OLLAMA_BASE_URL` | Ollama server URL | http://localhost:11434 |
| `OPENCODE_DB_PATH` | SQLite database path | ./opencode.db |

### 9.2 Config File (config.toml)

```toml
[server]
port = 3000
hostname = "127.0.0.1"

[server.desktop]
enabled = true
auto_open_browser = true

[server.acp]
enabled = true
server_id = "local"
version = "1.0"
```

---

## 10. Out of Scope

- Web UI (future work)
- Cloud deployment automation
- Team collaboration features
- Code hosting integration (GitHub, GitLab)
- Multi-agent coordination
- Formal verification

---

## 11. Open Questions

1. Should the SDK be published to crates.io?
2. What is the plugin API stability guarantee?
3. Which LLM provider should be the default?
4. Should MCP support be core or optional?
5. What is the migration path for session data between versions?

---

## 12. References

- [AGENTS.md](./AGENTS.md) - AI agent instructions and workflows
- [Cargo.toml](./opencode-rust/Cargo.toml) - Workspace manifest
- [ratatui-testing/](./ratatui-testing/) - TUI testing framework

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-11 | Sisyphus | Initial draft based on codebase analysis |
