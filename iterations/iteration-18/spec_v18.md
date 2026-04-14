# OpenCode RS Specification - Iteration 18

**Version:** 1.8.0
**Generated:** 2026-04-14
**Status:** Implementation ~90-95% Complete
**Phase:** Phase 1-5 Complete, Phase 6 (Release Qualification) Pending

---

## Overview

This document describes the OpenCode Rust implementation specification, covering the MCP (Model Context Protocol) system, configuration, agent runtime, tools, and all supporting subsystems.

---

## 1. MCP System (Model Context Protocol)

### 1.1 MCP Server Configuration

#### FR-001: MCP Server Configuration Schema

MCP servers are configured under the `mcp` top-level key with per-server configuration:

```json
{
  "mcp": {
    "<server-name>": {
      "type": "local" | "remote",
      "command?: string[]",        // local only
      "url?: string",              // remote only
      "enabled": boolean,
      "environment?: object",      // local only
      "headers?: object",          // remote only
      "oauth?: object | false",
      "timeout?: number           // milliseconds
    }
  }
}
```

#### FR-002: Local MCP Server Support

Local MCP servers use stdio transport with JSON-RPC protocol:

```json
{
  "mcp": {
    "my-local-server": {
      "type": "local",
      "command": ["npx", "-y", "@modelcontextprotocol/server-everything"],
      "environment": {
        "MY_ENV_VAR": "value"
      },
      "timeout": 5000
    }
  }
}
```

#### FR-003: Remote MCP Server Support

Remote MCP servers use HTTP+SSE transport:

```json
{
  "mcp": {
    "my-remote-server": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "headers": {
        "Authorization": "Bearer $API_KEY"
      },
      "timeout": 10000
    }
  }
}
```

### 1.2 OAuth Support

#### FR-004: Automatic OAuth Flow

OpenCode automatically handles OAuth for supported servers:
1. Detects 401 response
2. Initiates OAuth flow
3. Uses dynamic client registration (RFC 7591) when supported
4. Securely stores tokens for subsequent requests

#### FR-005: OAuth Configuration Per Server

OAuth configuration is scoped per MCP server entry:

```json
{
  "mcp": {
    "my-remote-server": {
      "type": "remote",
      "url": "https://mcp.example.com/mcp",
      "oauth": {
        "clientId": "{env:MY_CLIENT_ID}",
        "clientSecret": "{env:MY_CLIENT_SECRET}",
        "scope": "tools:read tools:execute"
      }
    }
  }
}
```

#### FR-006: OAuth Disable

OAuth can be disabled per server:

```json
{
  "mcp": {
    "my-remote-server": {
      "oauth": false
    }
  }
}
```

#### FR-007: OAuth CLI Commands

```bash
opencode mcp auth <server-name>    # Authenticate specific server
opencode mcp auth list             # List auth status
opencode mcp debug <server-name>   # Debug OAuth issues
opencode mcp logout <server-name> # Remove credentials
```

### 1.3 Tool Integration

#### FR-008: MCP Tool Naming Convention

MCP tools are exposed with the format `<servername>_<toolname>`:
- Server `sentry` with tool `list_issues` → `sentry_list_issues`

#### FR-009: MCP Tool Permission Control

Permission rules use glob patterns:

```json
{
  "permission": {
    "mcp_sentry_*": "deny"        // disable all sentry tools
  }
}
```

#### FR-010: Per-Agent MCP Permission

```json
{
  "permission": {
    "mcp_*": "deny"               // disable globally
  },
  "agent": {
    "my-agent": {
      "permission": {
        "mcp_github_*": "allow"   // enable for specific agent
      }
    }
  }
}
```

### 1.4 Transport Specification

#### FR-011: Local Server Transport

Use stdio transport with JSON-RPC protocol:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

#### FR-012: Remote Server Transport

Use HTTP+SSE transport:
- POST requests for tool calls
- SSE stream for tool responses

Default timeout: 5000ms (5 seconds)

### 1.5 Built-in MCP Server Examples

#### FR-013: Sentry MCP Server

```json
{
  "mcp": {
    "sentry": {
      "type": "remote",
      "url": "https://mcp.sentry.dev/mcp",
      "oauth": {}
    }
  }
}
```

#### FR-014: Context7 MCP Server

```json
{
  "mcp": {
    "context7": {
      "type": "remote",
      "url": "https://mcp.context7.com/mcp"
    }
  }
}
```

#### FR-015: Vercel Grep MCP Server

```json
{
  "mcp": {
    "gh_grep": {
      "type": "remote",
      "url": "https://mcp.grep.app"
    }
  }
}
```

### 1.6 Context Usage

#### FR-016: Context Cost Warning

MCP servers consume context space. Each tool definition and its schema adds to the prompt size. Users should carefully select which MCP servers to enable.

---

## 2. Core Architecture

### 2.1 Entity Model

#### FR-017: Part Entity

Part type with extensible versioning surface.

**Implementation:** `crates/core/src/part.rs` - versioned enum with Unknown variant

#### FR-018: Project Entity

Project entity with stable ID.

**Implementation:** `crates/core/src/project.rs`

#### FR-019: Session Entity

Session entity with stable ID and parent lineage.

**Implementation:** `crates/core/src/session.rs`

#### FR-020: Message Entity

Message entity with ordered history.

**Implementation:** `crates/core/src/message.rs`

#### FR-021: Ownership Tree

Ownership tree (Project→Session→Message→Part) must be acyclic.

**Implementation:** 40+ acyclicity tests in `session.rs`

#### FR-022: Fork Model

Child session fork without parent mutation.

**Implementation:** `delegation.rs` + session fork logic

#### FR-023: Snapshot/Checkpoint Metadata

Snapshot and checkpoint metadata support.

**Implementation:** `crates/core/src/snapshot.rs`, `checkpoint.rs`

#### FR-024: Session Status Machine

Session status machine (idle→running→terminal).

**Implementation:** `session_state.rs`

---

## 3. Agent System

### 3.1 Agent Execution

#### FR-025: Primary Agent Execution Loop

Primary agent execution loop implementation.

**Implementation:** `crates/agent/src/runtime.rs`

#### FR-026: Exactly One Active Primary Agent Invariant

Exactly one active primary agent invariant must be maintained.

**Implementation:** 20+ invariant tests in `runtime.rs`

#### FR-027: Hidden vs Visible Agent Behavior

Hidden agents don't affect the active primary agent invariant.

**Implementation:** Tests verify hidden agents don't affect invariant

#### FR-028: Subagent Execution

Subagent execution with child context.

**Implementation:** `crates/agent/src/delegation.rs`

#### FR-029: Task/Delegation Mechanism

Task and delegation mechanism for agent coordination.

**Implementation:** `delegation.rs`

#### FR-030: Permission Inheritance

Permission inheritance from parent to subagent.

**Implementation:** Tests confirm intersection logic

#### FR-031: Runtime Subagent Permission Restriction

Runtime restriction of subagent permissions using `effective_scope = parent_scope.intersect(subagent_scope)`.

---

## 4. Tools System

### 4.1 Tool Registry

#### FR-032: Tool Registry

Tool registry with registration, lookup, and listing capabilities.

**Implementation:** `crates/tools/src/registry.rs` (2288 lines)

#### FR-033: Built-in Tool Interface

Built-in tool interface with stable name/description/args.

**Implementation:** Tool trait implementation

#### FR-034: Custom Tool Discovery

Custom tool discovery scanning `.ts/.js` files per PRD.

**Implementation:** Custom tools registered with ToolRegistry

#### FR-035: Execution Pipeline

Execution pipeline: name lookup → permission → validation → execute.

**Implementation:** Permission gate in AgentExecutor

#### FR-036: Argument Validation

Argument validation with schema validation.

#### FR-037: MCP Tool Qualification

MCP tool qualification with server-qualified naming.

**Implementation:** `crates/mcp/src/tool_bridge.rs`

#### FR-038: Deterministic Collision Resolution

Deterministic collision resolution with ToolSource priority (Builtin > Plugin > CustomProject > CustomGlobal).

#### FR-039: Result Caching

Result caching for safe tools with `CachedToolResult` with TTL and dependency tracking.

---

## 5. LSP System

### 5.1 LSP Features

#### FR-040: Built-in LSP Server Detection

Built-in LSP server detection.

**Implementation:** `crates/lsp/src/builtin.rs`

#### FR-041: Custom LSP Server Registration

Custom LSP server registration via config.

**Implementation:** `crates/lsp/src/custom.rs`

#### FR-042: Diagnostics Retrieval

Diagnostics retrieval and surfacing.

**Implementation:** `crates/lsp/src/client.rs`

#### FR-043: LSP Failure Handling

LSP failure handling.

**Implementation:** `crates/lsp/src/failure_handling_tests.rs`

#### FR-044: Experimental LSP Tool Behavior

Experimental LSP tool behavior.

**Implementation:** `crates/lsp/src/experimental.rs`

---

## 6. Configuration System

### 6.1 Config Features

#### FR-045: JSON and JSONC Parsing

JSON and JSONC parsing support.

**Implementation:** Full implementation in `crates/config/src/lib.rs` (1581+ lines)

#### FR-046: Config Precedence

Config precedence (remote→global→custom→project→.opencode→inline).

#### FR-047: Variable Expansion

Variable expansion: `{env:VAR}` and `{file:PATH}`.

**Implementation:** Implemented in config.rs

#### FR-048: Legacy Tools Alias

`tools` legacy alias normalization to `permission`.

#### FR-049: Config Ownership Boundary

Config ownership boundary (opencode.json vs tui.json) enforced with warnings.

#### FR-050: Permission Rule Type

Permission rule type with glob pattern support.

**Implementation:** `permission.rs`

#### FR-051: Auth/Secret Storage

Auth/secret storage paths at `~/.local/share/opencode/auth.json`.

---

## 7. HTTP Server API

### 7.1 API Features

#### FR-052: Route Registration

Route registration by resource group (session, config, provider, permission, share, MCP, SSE, acp, ws).

#### FR-053: Auth Enforcement

Auth enforcement per endpoint with middleware.

#### FR-054: Request Validation

Request validation.

**Implementation:** `validation.rs`

#### FR-055: Session/Message Lifecycle Endpoints

Session/message lifecycle endpoints.

**Implementation:** `session.rs`, `share.rs`

#### FR-056: Streaming Endpoints

Streaming endpoints (SSE/websocket).

**Implementation:** `sse.rs`, `ws.rs`

#### FR-057: API Error Shape Consistency

API error shape consistency.

**Implementation:** `error.rs`

---

## 8. Plugin System

### 8.1 Plugin Features

#### FR-058: Plugin Source Loading

Plugin source loading from configured paths.

**Implementation:** `crates/plugin/src/discovery.rs`

#### FR-059: Plugin Hooks

Hooks: on_init, on_start, on_tool_call, on_message, on_session_end.

**Implementation:** All implemented in `lib.rs`

#### FR-060: Hook Execution Order

Hook execution order deterministic using `IndexMap` with priority ordering.

#### FR-061: Plugin Tool Registration

Plugin-provided tool registration through standard registry with `Plugin::register_tool()` method.

#### FR-062: Failure Containment

Failure containment - plugin errors don't crash runtime. Hooks log warnings but don't panic.

#### FR-063: Server/Runtime Plugin Config Ownership

Server/runtime plugin config ownership split enforced.

---

## 9. TUI System

### 9.1 TUI Features

#### FR-064: Session View

Session view with markdown, syntax highlighting, diff.

**Implementation:** `app.rs` (191KB)

#### FR-065: Slash Commands

Slash commands with `/command` parsing.

**Implementation:** `command.rs`

#### FR-066: Input Model

Input model: multiline, history, autocomplete.

**Implementation:** `input/` module

#### FR-067: Sidebar

Sidebar with file tree, MCP/LSP status, diagnostics.

**Implementation:** `components/` and `widgets/`

#### FR-068: Keybinding System

Keybinding system with leader key.

**Implementation:** `keybinding.rs`

#### FR-069: @ File Reference

`@` file reference with fuzzy search.

**Implementation:** `file_ref_handler.rs`

#### FR-070: Shell Prefix Handling

`!` shell prefix handling.

**Implementation:** `shell_handler.rs`

---

## 10. Provider/Model System

### 10.1 Provider Features

#### FR-071: Provider Abstraction

Provider abstraction with registration, credential lookup.

**Implementation:** `crates/llm/src/provider_abstraction.rs`

#### FR-072: Default Model Selection

Default model selection.

**Implementation:** `crates/llm/src/model_selection.rs`

#### FR-073: Per-Agent Model Override

Per-agent model override support.

#### FR-074: Provider Credential Resolution

Provider credential resolution (env, file, secret store).

**Implementation:** `auth.rs`, layered auth

#### FR-075: Local Model Provider

Local model provider (Ollama, LM Studio).

**Implementation:** `crates/llm/src/ollama.rs`, `lm_studio.rs`

#### FR-076: Variant / Reasoning Budget

Variant / reasoning budget handling.

**Implementation:** `budget.rs`

---

## 11. Formatters

### 11.1 Formatter Features

#### FR-077: Formatter Detection

Formatter detection by file type.

**Implementation:** `FormatterEngine::match_formatters()`

#### FR-078: Project Config Formatter Selection

Project config-based formatter selection.

#### FR-079: Disable Formatter Control

Disable-all and per-formatter disable via `FormatterConfig::Disabled`.

#### FR-080: Custom Formatter Command

Custom formatter command invocation with env vars.

#### FR-081: Formatter Error Handling

Formatter absence/error handling (non-fatal, logs warnings).

---

## 12. Skills System

### 12.1 Skills Features

#### FR-082: SKILL.md Format

SKILL.md format support with frontmatter.

**Implementation:** `crates/core/src/skill.rs` (1400+ lines)

#### FR-083: Skill Discovery Precedence

Skill discovery precedence: project→global→compat.

#### FR-084: Deterministic Duplicate Resolution

Deterministic duplicate resolution within scope (first-found wins per scope).

#### FR-085: Compatibility Path Loading

Compatibility path loading (`.claude/skills/`, `.agents/skills/`).

#### FR-086: Skill Loading

Skill loading into runtime context via `inject_into_prompt()`.

#### FR-087: Skill Permission Restrictions

Permission restrictions for skill usage using tool permission system.

---

## 13. Desktop/Web Interface

### 13.1 Desktop/Web Features

#### FR-088: Desktop App Startup Flow

Desktop app startup flow.

**Implementation:** `crates/cli/src/cmd/desktop.rs` (207 lines)

#### FR-089: Web Server Mode

Web server mode.

**Implementation:** `crates/cli/src/cmd/web.rs` (86 lines)

#### FR-090: Auth-Protected Interface

Auth-protected interface access.

#### FR-091: Session Sharing

Session sharing between interface modes with ShareServer.

#### FR-092: ACP Startup/Handshake

ACP startup/handshake for editor integration.

**Implementation:** `crates/server/src/routes/acp.rs`, `acp_ws.rs`

#### FR-093: Sharing in Managed Deployments

Sharing behavior in managed/restricted deployments with `share` config option.

---

## 14. GitHub/GitLab Integration

### 14.1 Git Integration

#### FR-094: GitHub Workflow Trigger

GitHub workflow trigger examples.

**Implementation:** `crates/git/src/github.rs`

#### FR-095: Comment/PR Trigger Parsing

Comment/PR trigger parsing.

**Implementation:** `trigger.rs`

#### FR-096: CI Secret Loading

CI secret loading for GitHub Actions with auth integration.

#### FR-097: GitLab CI Component Support

GitLab CI component support.

**Implementation:** `crates/git/src/gitlab_ci.rs`

#### FR-098: GitLab Duo Behavior

GitLab Duo behavior (marked experimental).

---

## 15. TUI Plugin API

### 15.1 TUI Plugin Features

#### FR-099: TUI Plugin Config Ownership

`tui.json` plugin configuration ownership recognized in config system.

#### FR-100: Plugin Identity

Plugin identity with runtime ID resolution, file vs npm.

#### FR-101: Plugin Deduplication

Plugin deduplication before activation.

#### FR-102: Plugin Enable/Disable

`plugin_enabled` semantics for per-plugin enable/disable.

#### FR-103: Commands/Routes/Dialogs/Slots

Commands, routes, dialogs, slots registration.

**Implementation:** `plugin_api.rs` (54KB)

#### FR-104: Theme Management

Theme install/set.

**Implementation:** `theme.rs`

#### FR-105: Events Subscription

Events subscription via `api.event.on()`.

#### FR-106: State Management

State get/set with KV store + state.

#### FR-107: OnDispose Lifecycle

`onDispose` lifecycle cleanup registration.

#### FR-108: Runtime Plugin Management

Runtime `api.plugins.activate()`/`deactivate()` for plugin management.

#### FR-109: Bounded Cleanup

Bounded cleanup with AbortSignal enforcement.

#### FR-110: Theme Auto-Sync

Theme auto-sync on install.

---

## 16. Test Plan

### 16.1 Test Coverage

#### FR-111: Unit Tests

Unit tests for core entities.

#### FR-112: Integration Tests

Integration tests for agent flow.

**Implementation:** `agent_tool_tests.rs`, `agent_llm_tests.rs`

#### FR-113: Session Lifecycle Tests

Session lifecycle tests.

**Implementation:** `session_lifecycle_tests.rs` (21KB)

#### FR-114: Compaction/Shareability Tests

Compaction and shareability tests.

**Implementation:** `compaction_shareability_tests.rs` (17KB)

#### FR-115: MCP Protocol Tests

MCP protocol tests.

**Implementation:** `mcp_protocol_tests.rs`

#### FR-116: Session Storage Tests

Session storage tests.

**Implementation:** `session_storage_tests.rs`

#### FR-117: Agent Switch Tests

Agent switch tests.

**Implementation:** `agent_switch_tests.rs` (9KB)

#### FR-118: ACP Transport Tests

ACP transport tests.

**Implementation:** `acp_transport_tests.rs`

#### FR-119: Convention Tests

Convention tests in `conventions/` module.

#### FR-120: TUI Component Tests

TUI component tests.

**Implementation:** `slash_command_tests.rs`, `input_model_tests.rs`, etc.

#### FR-121: Ratatui Testing Framework

Ratatui testing framework crate.

**Implementation:** `ratatui-testing/` crate

---

## 17. Implementation Status

### 17.1 Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~100% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~100% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~100% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~95% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~95% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

### 17.2 Crate Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | Registry and custom tool discovery |
| `crates/plugin/` | ✅ Done | Hooks and tool registration |
| `crates/tui/` | ✅ Done | Full implementation, all tests passing |
| `crates/server/` | ✅ Done | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ✅ Done | Full config implementation (was empty re-export) |
| `crates/cli/` | ✅ Done | Desktop/web implemented |
| `crates/control-plane/` | ✅ Done | ACP stream, events, enterprise features |
| `ratatui-testing/` | ✅ Done | TUI testing framework crate |

---

## 18. Known Issues

### 18.1 Open Issues

| Issue | Severity | Module | Status |
|-------|----------|--------|--------|
| Phase 6 Release Qualification not started | P1 | all | ❌ NOT STARTED |
| GitLab CI integration tests failing (7 tests) | P2 | git | ❌ NEEDS FIX |
| Desktop/web smoke test port conflict | P2 | cli | ❌ NOT FIXED |
| Deprecated `mode` field | P2 | config | ⚠️ Deferred |
| Deprecated `tools` field | P2 | config | ⚠️ Deferred |

### 18.2 GitLab CI Integration Tests

7 integration tests in `crates/git/src/gitlab_ci.rs` require a real GitLab server at `http://127.0.0.1:63182`. These tests should be:
1. Marked with `#[ignore]` and documented as requiring external GitLab
2. Converted to use a mock GitLab server
3. Properly feature-gated

### 18.3 Desktop/Web Smoke Test

`desktop_web_different_ports` test uses hardcoded port 3000 which may conflict with other processes. Should use dynamic port allocation.

---

## 19. Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config ownership, `mcp` key schema, `permission` rules |
| [07-server-api.md](./07-server-api.md) | MCP API endpoints (if exposed via HTTP) |

---

## 20. Test Results

```
cargo test --all-features --all:
- ~1020 passed
- 8 failed

Failed tests breakdown:
- GitLab CI tests: 7 failures (integration tests require real GitLab server)
- CLI tests: 1 failure (desktop_web_different_ports - port conflict)
```

---

**Document Version:** 1.8
**Iteration:** 18
**Last Updated:** 2026-04-14
