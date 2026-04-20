# OpenCode RS - Specification Document v36

**Version:** 36
**Date:** 2026-04-20
**Status:** Active
**Based on:** PRD v1.0 + Gap Analysis (Iteration 36)

---

## 1. Overview

OpenCode RS is a Rust-based AI coding agent providing interactive developer assistance via TUI, HTTP API, and SDK. It integrates with multiple LLM providers and exposes extensible tooling for file operations, git, search, and custom plugins.

---

## 2. Architecture

### 2.1 Crate Structure

```
opencode-rust/
├── crates/
│   ├── core/           # Session management, tool registry, error handling
│   ├── cli/            # CLI commands and entry points
│   ├── llm/            # LLM provider integrations
│   ├── tools/          # Tool implementations
│   ├── agent/          # Agent orchestration
│   ├── tui/            # Terminal UI (ratatui)
│   ├── lsp/            # Language Server Protocol integration
│   ├── storage/        # SQLite-based session persistence
│   ├── server/         # HTTP API server (actix-web)
│   ├── auth/           # Authentication
│   ├── permission/     # Permission system
│   ├── plugin/         # Plugin architecture (WASM)
│   ├── git/            # Git operations
│   ├── mcp/            # Model Context Protocol
│   └── sdk/            # Rust SDK for programmatic access
├── tests/              # Integration tests
├── opencode-benches/   # Performance benchmarks
└── ratatui-testing/    # TUI testing framework
```

### 2.2 Component Responsibilities

| Crate | Responsibility | Implementation Status |
|-------|----------------|----------------------|
| `opencode-core` | Session management, tool registry, error system | ✅ Complete |
| `opencode-agent` | Orchestrates LLM + tools for task execution | ✅ Complete |
| `opencode-llm` | Multi-provider LLM adapter | ✅ Complete (20+ providers) |
| `opencode-tools` | Developer tools: file I/O, grep, git, search | ✅ Complete |
| `opencode-tui` | Interactive terminal user interface | ✅ Complete |
| `opencode-server` | HTTP REST API | ✅ Complete |
| `opencode-storage` | SQLite persistence for sessions | ✅ Complete |
| `opencode-sdk` | Public Rust API for external consumers | ✅ Partial (missing examples) |
| `opencode-auth` | Authentication (Argon2, JWT, AES-GCM) | ✅ Complete |
| `opencode-permission` | Role-based access control | ✅ Complete |
| `opencode-plugin` | WASM plugin system | ⚠️ Framework complete, WASM binaries missing |
| `opencode-git` | Git operations | ⚠️ Basic ops only (status/diff/commit) |
| `opencode-lsp` | LSP integration | ⚠️ Basic integration, limited tools |
| `opencode-mcp` | Model Context Protocol | ✅ Complete |

---

## 3. Functional Requirements

### 3.1 LLM Provider Integration

**FR-001** - The system SHALL support OpenAI models (GPT-4, GPT-3.5, GPT-4o).
**FR-002** - The system SHALL support Anthropic Claude models (Opus, Sonnet, Haiku).
**FR-003** - The system SHALL support Ollama local models (Llama2, Mistral, custom).
**FR-004** - The system SHALL support configuration via environment variables or config files.
**FR-005** - The system SHALL support model-specific parameters (temperature, max_tokens).
**FR-006** - The system SHALL support streaming responses with < 100ms time-to-first-token.
**FR-007** - The system SHOULD support additional providers: Azure, Google (Gemini), AWS (Bedrock), OpenRouter, Groq, Cohere, Mistral, and others.

### 3.2 Tool System

**FR-010** - The system SHALL implement the `Read` tool with line range support. *(P0)*
**FR-011** - The system SHALL implement the `Write` tool for creating/overwriting files. *(P0)*
**FR-012** - The system SHALL implement the `Edit` tool for targeted file edits. *(P0)*
**FR-013** - The system SHALL implement the `Grep` tool with regex search. *(P0)*
**FR-014** - The system SHALL implement the `Glob` tool for file pattern matching. *(P1)*
**FR-015** - The system SHALL implement the `Git` tool for git operations. *(P1)*
**FR-016** - The system SHALL implement the `Bash` tool for shell command execution. *(P1)*
**FR-017** - The system SHALL implement the `WebSearch` tool for internet search. *(P2)*
**FR-018** - The system SHALL implement the `Delete` tool for file deletion.
**FR-019** - The system SHALL implement the `MultiEdit` tool for batch file edits.
**FR-020** - The system SHALL implement LSP tools for language server integration.
**FR-021** - Plugins SHALL be able to register custom tools via the tool registry.
**FR-022** - Tools SHALL declare parameter schemas and permission levels.

#### 3.2.1 Git Tool Gaps (Gap G-003)

**FR-015a** - The `Git` tool SHALL support: `status`, `diff`, `log`, `commit`. *(Implemented)*
**FR-015b** - The `Git` tool SHOULD support: `branch`, `checkout`, `merge`. *(Gap - not yet implemented)*
**FR-015c** - The `Git` tool SHOULD support: `rebase`, `stash`, `push`, `pull`. *(Gap - not yet implemented)*

### 3.3 Agent Modes

**FR-030** - The system SHALL support `Build` mode with full read/write tool access.
**FR-031** - The system SHALL support `Plan` mode with read-only tool access.
**FR-032** - The system SHALL support `General` mode for search and investigation.

### 3.4 Session Management

**FR-040** - The system SHALL persist sessions to SQLite with full CRUD operations.
**FR-041** - The system SHALL support resuming interrupted sessions.
**FR-042** - The system SHALL support exporting sessions as JSON.
**FR-043** - The system SHALL support importing sessions from JSON.
**FR-044** - Sessions SHALL contain: `id`, `created_at`, `updated_at`, `mode`, `messages`, `metadata`.

### 3.5 TUI (Terminal UI)

**FR-050** - The TUI SHALL provide an interactive command palette.
**FR-051** - The TUI SHALL provide a session history browser.
**FR-052** - The TUI SHALL display real-time LLM output streaming.
**FR-053** - The TUI SHALL be built with ratatui.
**FR-054** - All TUI dialogs SHALL handle empty state gracefully (visible message, no panic).
**FR-055** - All TUI dialogs with lists SHALL handle Enter on empty list by closing the dialog.
**FR-056** - All TUI dialogs SHALL render correctly via `ratatui::backend::TestBackend`.

### 3.6 HTTP API Server

**FR-060** - `GET /api/status` SHALL return server status.
**FR-061** - `POST /api/session` SHALL create a new session.
**FR-062** - `GET /api/session/{id}` SHALL return session details.
**FR-063** - `POST /api/session/{id}/execute` SHALL execute an agent task.
**FR-064** - `GET /api/session/{id}/history` SHALL return conversation history (aliased as `/api/sessions/{id}/messages`).
**FR-065** - The server SHOULD support WebSocket streaming for real-time agent output. *(Gap G-006 - needs verification)*

### 3.7 ACP (Agent Communication Protocol)

**FR-070** - `GET /api/acp/status` SHALL return ACP status.
**FR-071** - `POST /api/acp/handshake` SHALL perform an ACP handshake.
**FR-072** - `POST /api/acp/connect` SHALL connect to an ACP server.
**FR-073** - `POST /api/acp/ack` SHALL acknowledge a handshake.

### 3.8 MCP (Model Context Protocol)

**FR-080** - The system SHALL connect to external MCP servers.
**FR-081** - The system SHALL discover tools from connected MCP servers.
**FR-082** - The system SHALL support both local and remote MCP server connections.

### 3.9 Plugin System

**FR-090** - The plugin framework SHALL support loading WASM plugin binaries.
**FR-091** - Plugins SHALL be able to register custom tools.
**FR-092** - A build script or bundled WASM binaries SHALL be provided for plugin deployment. *(Gap G-001)*

### 3.10 LSP Integration

**FR-100** - The system SHALL provide basic LSP integration for editor embedding.
**FR-101** - The system SHOULD expand LSP tool capabilities (diagnostics, completion, references). *(Gap G-004)*

### 3.11 SDK

**FR-110** - The `opencode-sdk` crate SHALL expose a public async/await Rust API.
**FR-111** - The SDK SHALL include comprehensive usage examples in `examples/`. *(Gap G-002)*
**FR-112** - The SDK SHOULD be published to crates.io. *(Gap G-007)*

---

## 4. Data Models

### 4.1 Session

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

### 4.2 Tool

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

## 5. Configuration

### 5.1 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENCODE_LLM_PROVIDER` | LLM provider name | `openai` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `OLLAMA_BASE_URL` | Ollama server URL | `http://localhost:11434` |
| `OPENCODE_DB_PATH` | SQLite database path | `./opencode.db` |

### 5.2 Config File (config.toml / config.json)

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

## 6. Non-Functional Requirements

### 6.1 Performance

| Metric | Target |
|--------|--------|
| Cold start time | < 2 seconds |
| Tool execution latency | < 500ms (local), < 2s (remote) |
| LLM response streaming | < 100ms time-to-first-token |
| Session load time | < 1 second |

### 6.2 Reliability

- **FR-200** - All crates SHALL compile with `cargo build --release`.
- **FR-201** - All integration tests SHALL pass with `cargo test`.
- **FR-202** - Zero clippy warnings: `cargo clippy --all -- -D warnings`. *(Needs verification)*
- **FR-203** - Code SHALL be formatted: `cargo fmt --all -- --check`. *(Needs verification)*

### 6.3 Security

- **FR-210** - No hardcoded credentials; all secrets via environment variables.
- **FR-211** - Passwords SHALL be hashed with Argon2 or bcrypt.
- **FR-212** - Data at rest SHALL be encrypted with AES-GCM.
- **FR-213** - API authentication SHALL use JWT.
- **FR-214** - Permission enforcement SHALL be applied per endpoint.

### 6.4 Compatibility

- **Rust**: 1.70+
- **Platforms**: macOS, Linux, Windows
- **LLM Providers**: OpenAI API compatible, Anthropic API, Ollama local

---

## 7. Testing Requirements

### 7.1 Unit Tests

**FR-300** - Each crate SHALL have inline `#[cfg(test)]` unit test modules.
**FR-301** - Run with: `cargo test --lib`

### 7.2 Integration Tests

**FR-310** - Integration tests SHALL cover agent + tool + LLM workflows.
**FR-311** - Location: `tests/` directory.
**FR-312** - Run with: `cargo test --test '*'`

### 7.3 TUI Tests

**FR-320** - TUI rendering SHALL be verified with `ratatui::backend::TestBackend`.
**FR-321** - PTY simulation SHALL be available via `ratatui-testing` crate.
**FR-322** - Run with: `cargo test -p ratatui-testing`

### 7.4 Benchmarks

**FR-330** - A benchmark suite SHALL exist in `opencode-benches/`. *(Gap G-005 - currently limited)*
**FR-331** - Benchmarks SHOULD cover tool execution, session load, and LLM round-trip latency.
**FR-332** - Run with: `cargo bench`

---

## 8. Known Gaps and Backlog Items

| Gap ID | Description | Priority | Estimated Effort |
|--------|-------------|----------|-----------------|
| G-001 | Plugin WASM binaries not in repo or build script missing | P1 | 1 day |
| G-002 | SDK lacks usage examples in `examples/` | P1 | 2 days |
| G-003 | Git tool missing branch/merge/rebase/stash/push/pull | P2 | 3 days |
| G-004 | LSP integration lacks diagnostics, completion, references | P2 | 1 week |
| G-005 | Benchmark suite has limited scenarios | P2 | 1 week |
| G-006 | WebSocket streaming not verified end-to-end | P2 | 2 days |
| G-007 | SDK not published to crates.io | P2 | 1 day |
| G-008 | Documentation scattered across crates; no consolidated guide | P2 | 3 days |

---

## 9. Implementation Progress

| Category | Completeness | Notes |
|----------|-------------|-------|
| Architecture (crates) | 100% | All 15 crates implemented |
| LLM Providers | 100% | 20+ providers (exceeds PRD) |
| Tool System | 95% | All P0/P1; Git/LSP gaps remain |
| Agent Modes | 100% | Build, Plan, General |
| TUI | 100% | Full ratatui implementation |
| HTTP API | 95% | All endpoints; WebSocket needs verification |
| Session Management | 100% | SQLite CRUD complete |
| MCP Protocol | 100% | Complete |
| Auth/Permission | 100% | Argon2, JWT, AES-GCM, RBAC |
| Plugin System | 85% | WASM framework done; binaries missing |
| Git Integration | 80% | Basic ops; advanced ops missing |
| LSP Integration | 70% | Basic; limited tool set |
| SDK | 90% | Public API exists; examples missing |
| **Overall** | **~87%** | |

---

## 10. Out of Scope

- Web UI (future iteration)
- Cloud deployment automation
- Team collaboration features
- Code hosting integration (GitHub, GitLab)
- Multi-agent coordination
- Formal verification

---

## 11. Open Questions

1. Should the SDK be published to crates.io (G-007)?
2. What is the plugin API stability guarantee for external WASM plugins?
3. Which LLM provider should be the default (`openai` currently)?
4. Should MCP support be a core dependency or an optional feature flag?
5. What is the migration path for session data between versions?

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-11 | Sisyphus | Initial draft |
| 36 | 2026-04-20 | Agent | Updated based on Gap Analysis Iteration 36; added FR-XXX numbering, gap table, implementation status |
