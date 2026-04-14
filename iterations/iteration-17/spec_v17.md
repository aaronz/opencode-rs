# OpenCode RS Specification - Iteration 17

**Version:** 17
**Date:** 2026-04-14
**Status:** In Development (Phase 1-5 Complete, Phase 6 Pending)
**Implementation Progress:** ~85-90% Complete

---

## Document Overview

This specification documents the OpenCode Rust implementation, derived from the Product Requirements Document (PRD) and updated based on gap analysis iterations.

---

## 1. MCP System (Model Context Protocol)

### 1.1 MCP Server Configuration

#### FR-001: MCP Configuration Schema
MCP servers are configured under the `mcp` top-level key:

```json
{
  "mcp": {
    "server-name": {
      "type": "local" | "remote",
      "command?: string[]",        // local only
      "url?: string",              // remote only
      "enabled": boolean,
      "environment?: object",      // local only
      "headers?: object",          // remote only
      "oauth?: object | false",
      "timeout?: number"           // milliseconds
    }
  }
}
```

**Status:** ✅ Implemented

#### FR-002: Local MCP Server Support
Local servers use stdio transport with JSON-RPC protocol:

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

**Status:** ✅ Implemented

#### FR-003: Remote MCP Server Support
Remote servers use HTTP+SSE transport:

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

**Status:** ✅ Implemented

### 1.2 OAuth Support

#### FR-004: Automatic OAuth Flow
OpenCode automatically handles OAuth for supported servers:
1. Detects 401 response
2. Initiates OAuth flow
3. Uses dynamic client registration (RFC 7591) when supported
4. Securely stores tokens for subsequent requests

**Status:** ✅ Implemented

#### FR-005: Per-Server OAuth Configuration
OAuth is scoped per MCP server entry:

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

**Status:** ✅ Implemented

#### FR-006: OAuth Disable
OAuth can be explicitly disabled per server:

```json
{
  "mcp": {
    "my-remote-server": {
      "oauth": false
    }
  }
}
```

**Status:** ✅ Implemented

#### FR-007: OAuth CLI Commands
```bash
opencode mcp auth <server-name>    # Authenticate specific server
opencode mcp auth list             # List auth status
opencode mcp debug <server-name>   # Debug OAuth issues
opencode mcp logout <server-name> # Remove credentials
```

**Status:** ✅ Implemented

### 1.3 Tool Integration

#### FR-008: MCP Tool Naming Convention
MCP tools are exposed with format `<servername>_<toolname>`:
- Server `sentry` with tool `list_issues` → `sentry_list_issues`

**Status:** ✅ Implemented

#### FR-009: MCP Tool Permission Control
Permission rules use glob patterns:

```json
{
  "permission": {
    "mcp_sentry_*": "deny"        // disable all sentry tools
  }
}
```

**Status:** ✅ Implemented

#### FR-010: Per-Agent MCP Configuration
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

**Status:** ✅ Implemented

### 1.4 Built-in MCP Server Examples

#### FR-011: Sentry MCP Server
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

**Status:** ✅ Implemented

#### FR-012: Context7 MCP Server
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

**Status:** ✅ Implemented

#### FR-013: Vercel Grep MCP Server
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

**Status:** ✅ Implemented

### 1.5 Transport

#### FR-014: Local Server Transport
Local servers use stdio transport with JSON-RPC protocol:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
```

**Status:** ✅ Implemented

#### FR-015: Remote Server Transport
Remote servers use HTTP+SSE transport:
- POST requests for tool calls
- SSE stream for tool responses
- Default timeout: 5000ms (5 seconds)

**Status:** ✅ Implemented

### 1.6 Context Cost

#### FR-016: Context Usage Warning
MCP servers consume context space. Each tool definition and its schema adds to the prompt size.

**Status:** ✅ Implemented (context cost tracking in `crates/mcp/src/context_cost.rs`)

---

## 2. Core Architecture

### 2.1 Entity System

#### FR-020: Part Type with Extensible Versioning
- Versioned enum with Unknown variant for forward compatibility
- Location: `crates/core/src/part.rs`

**Status:** ✅ Done

#### FR-021: Project Entity with Stable ID
- Location: `crates/core/src/project.rs`

**Status:** ✅ Done

#### FR-022: Session Entity with Stable ID and Parent Lineage
- Location: `crates/core/src/session.rs`

**Status:** ✅ Done

#### FR-023: Message Entity with Ordered History
- Location: `crates/core/src/message.rs`

**Status:** ✅ Done

#### FR-024: Ownership Tree Acyclicity
- Project → Session → Message → Part hierarchy
- 40+ acyclicity tests in `session.rs`

**Status:** ✅ Done

### 2.2 Fork Model

#### FR-025: Child Session Fork Without Parent Mutation
- Implemented in `delegation.rs` + session fork logic
- Child sessions can be created without modifying parent

**Status:** ✅ Done

### 2.3 Snapshot and Checkpoint

#### FR-026: Snapshot/Checkpoint Metadata
- Location: `crates/core/src/snapshot.rs`, `checkpoint.rs`

**Status:** ✅ Done

### 2.4 Session State Machine

#### FR-027: Session Status Machine
States: idle → running → terminal
- Location: `crates/core/src/session_state.rs`

**Status:** ✅ Done

---

## 3. Agent System

### 3.1 Primary Agent

#### FR-030: Primary Agent Execution Loop
- Location: `crates/agent/src/runtime.rs`

**Status:** ✅ Done

#### FR-031: Exactly One Active Primary Agent Invariant
- 20+ invariant tests in `runtime.rs`

**Status:** ✅ Done

#### FR-032: Hidden vs Visible Agent Behavior
- Tests verify hidden agents don't affect invariant

**Status:** ✅ Done

### 3.2 Subagent System

#### FR-033: Subagent Execution with Child Context
- Location: `crates/agent/src/delegation.rs`

**Status:** ✅ Done

#### FR-034: Task/Delegation Mechanism
- Location: `crates/agent/src/delegation.rs`

**Status:** ✅ Done

#### FR-035: Permission Inheritance from Parent to Subagent
- Tests confirm intersection logic

**Status:** ✅ Done

#### FR-036: Runtime Restriction of Subagent Permissions
- `effective_scope = parent_scope.intersect(subagent_scope)`

**Status:** ✅ Done

---

## 4. Tools System

### 4.1 Tool Registry

#### FR-040: Tool Registry - Registration, Lookup, Listing
- Location: `crates/tools/src/registry.rs` (2288 lines)

**Status:** ✅ Done

#### FR-041: Built-in Tool Interface
- Stable name/description/args schema

**Status:** ✅ Done

### 4.2 Custom Tools

#### FR-042: Custom Tool Discovery
- Scans `.ts` and `.js` files per PRD specification
- Location: `crates/core/src/config/directory_scanner.rs:274`

**Status:** ✅ FIXED (Iteration 17)

#### FR-043: Custom Tools Registration with ToolRegistry
- `register_custom_tool()` and `register_discovered_custom_tools()`
- Location: `crates/core/src/tool.rs:246`

**Status:** ✅ FIXED (Iteration 17)

### 4.3 Execution Pipeline

#### FR-044: Execution Pipeline
Name lookup → permission → validation → execute

**Status:** ✅ Done

#### FR-045: Argument Validation
- Schema validation exists

**Status:** ✅ Done

#### FR-046: MCP Tool Qualification
- Server-qualified naming in `crates/mcp/src/tool_bridge.rs`

**Status:** ✅ Done

#### FR-047: Deterministic Collision Resolution
- ToolSource priority: Builtin > Plugin > CustomProject > CustomGlobal

**Status:** ✅ Done

#### FR-048: Result Caching for Safe Tools
- `CachedToolResult` with TTL and dependency tracking

**Status:** ✅ Done

---

## 5. LSP System

#### FR-050: Built-in LSP Server Detection
- Location: `crates/lsp/src/builtin.rs`

**Status:** ✅ Done

#### FR-051: Custom LSP Server Registration via Config
- Location: `crates/lsp/src/custom.rs`

**Status:** ✅ Done

#### FR-052: Diagnostics Retrieval and Surfacing
- Location: `crates/lsp/src/client.rs`

**Status:** ✅ Done

#### FR-053: LSP Failure Handling
- Location: `crates/lsp/src/failure_handling_tests.rs`

**Status:** ✅ Done

#### FR-054: Experimental LSP Tool Behavior
- Location: `crates/lsp/src/experimental.rs`

**Status:** ✅ Done

---

## 6. Configuration System

### 6.1 Config Parsing

#### FR-060: JSON and JSONC Parsing
- Location: `crates/core/src/config.rs` (3800+ lines)

**Status:** ✅ Done

#### FR-061: Config Precedence
Order: remote → global → custom → project → .opencode → inline

**Status:** ✅ Done

#### FR-062: Variable Expansion
- `{env:VAR}` and `{file:PATH}` supported

**Status:** ✅ Done

#### FR-063: Legacy Tools Alias Normalization
- `tools` legacy alias normalized to `permission`

**Status:** ✅ Done

### 6.2 Ownership

#### FR-064: Config Ownership Boundary
- Enforced between opencode.json and tui.json
- Warnings for violations

**Status:** ✅ Done

### 6.3 Permission System

#### FR-065: Permission Rule Type with Glob Patterns
- Location: `permission.rs`

**Status:** ✅ Done

#### FR-066: Auth/Secret Storage Paths
- `~/.local/share/opencode/auth.json`

**Status:** ✅ Done

### 6.4 Known Issues

#### FR-067: Config Crate Empty Re-export (P1 Issue)
- `crates/config/src/lib.rs` is empty re-export
- Should house config logic per PRD 19
- **Status:** ❌ NOT FIXED

---

## 7. HTTP Server API

#### FR-070: Route Registration by Resource Group
Routes: session, config, provider, permission, share, MCP, SSE, acp, ws

**Status:** ✅ Done

#### FR-071: Auth Enforcement Per Endpoint
- Middleware exists and is tested

**Status:** ✅ Done

#### FR-072: Request Validation
- Location: `validation.rs`

**Status:** ✅ Done

#### FR-073: Session/Message Lifecycle Endpoints
- Location: `session.rs`, `share.rs`

**Status:** ✅ Done

#### FR-074: Streaming Endpoints (SSE/WebSocket)
- Location: `sse.rs`, `ws.rs`

**Status:** ✅ Done

#### FR-075: API Error Shape Consistency
- Location: `error.rs`

**Status:** ✅ Done

---

## 8. Plugin System

#### FR-080: Plugin Source Loading from Configured Paths
- Location: `crates/plugin/src/discovery.rs`

**Status:** ✅ Done

#### FR-081: Hooks Implementation
- `on_init`, `on_start`, `on_tool_call`, `on_message`, `on_session_end`
- Location: `crates/plugin/src/lib.rs`

**Status:** ✅ Done

#### FR-082: Hook Execution Order Deterministic
- Uses priority ordering (lowest priority first)
- Location: `crates/plugin/src/lib.rs:604`

**Status:** ✅ FIXED (Iteration 17)

#### FR-083: Plugin-provided Tool Registration
- `Plugin::register_tool(&mut self, tool: PluginTool)` method
- `PluginManager::register_tools_in_registry()` method

**Status:** ✅ FIXED (Iteration 17)

#### FR-084: Failure Containment
- Plugin errors log warnings but don't crash runtime

**Status:** ✅ Done

#### FR-085: Server/Runtime Plugin Config Ownership
- Config ownership split enforced

**Status:** ✅ FIXED (Iteration 17)

---

## 9. TUI System

#### FR-090: Session View
- Markdown, syntax highlighting, diff support
- Location: `crates/tui/src/app.rs` (191KB)

**Status:** ✅ Done

#### FR-091: Slash Commands
- `/command` parsing in `command.rs`

**Status:** ✅ Done

#### FR-092: Input Model
- Multiline, history, autocomplete
- Location: `crates/tui/src/input/` module

**Status:** ✅ Done

#### FR-093: Sidebar Components
- File tree, MCP/LSP status, diagnostics
- Location: `components/` and `widgets/`

**Status:** ✅ Done

#### FR-094: Keybinding System with Leader Key
- Location: `keybinding.rs`

**Status:** ⚠️ PARTIAL (2 tests failing)

#### FR-095: @ File Reference with Fuzzy Search
- Location: `file_ref_handler.rs`

**Status:** ✅ Done

#### FR-096: ! Shell Prefix Handling
- Location: `shell_handler.rs`

**Status:** ✅ Done

### 9.1 TUI Tests

#### FR-097: TUI Component Test Coverage
- `slash_command_tests.rs` (287 lines)
- `input_model_tests.rs` (371 lines)
- `component_tests.rs`
- `file_references_tests.rs`

**Status:** ✅ Done (mostly)

### 9.2 Known TUI Issues

#### FR-098: TUI Keybinding Tests Failing (P2)
- 2 tests failing: case sensitivity and Space key handling
- **Status:** ❌ NOT FIXED

#### FR-099: TUI Theme Color Parsing Test Failing (P2)
- Hex color parsing returns wrong value
- **Status:** ❌ NOT FIXED

---

## 10. Provider/Model System

#### FR-100: Provider Abstraction
- Registration, credential lookup
- Location: `crates/llm/src/provider_abstraction.rs`

**Status:** ✅ Done

#### FR-101: Default Model Selection
- Location: `crates/llm/src/model_selection.rs`

**Status:** ✅ Done

#### FR-102: Per-Agent Model Override
- Implementation exists, not explicitly tested

**Status:** ⚠️ Deferred

#### FR-103: Provider Credential Resolution
- env, file, secret store layers
- Location: `auth.rs`

**Status:** ✅ Done

#### FR-104: Local Model Provider Support
- Ollama: `crates/llm/src/ollama.rs`
- LM Studio: `crates/llm/src/lm_studio.rs`

**Status:** ✅ Done

#### FR-105: Variant / Reasoning Budget Handling
- Location: `budget.rs`

**Status:** ✅ Done

---

## 11. Formatters

#### FR-110: Formatter Detection by File Type
- `FormatterEngine::match_formatters()`

**Status:** ✅ Done

#### FR-111: Project Config-Based Formatter Selection
- Config integration

**Status:** ✅ Done

#### FR-112: Disable-All and Per-Formatter Disable
- `FormatterConfig::Disabled`

**Status:** ✅ Done

#### FR-113: Custom Formatter Command Invocation
- `Command` execution with env vars

**Status:** ✅ Done

#### FR-114: Formatter Absence/Error Handling
- Non-fatal, logs warnings

**Status:** ✅ Done

---

## 12. Skills System

#### FR-120: SKILL.md Format Support with Frontmatter
- Location: `crates/core/src/skill.rs` (1400+ lines)

**Status:** ✅ Done

#### FR-121: Discovery Precedence
Priority order: project → global → compat

**Status:** ✅ Done

#### FR-122: Deterministic Duplicate Resolution Within Scope
- First-found wins per scope

**Status:** ✅ Done

#### FR-123: Compatibility Path Loading
- `.claude/skills/`, `.agents/skills/` paths

**Status:** ✅ Done

#### FR-124: Skill Loading into Runtime Context
- `inject_into_prompt()`

**Status:** ✅ Done

#### FR-125: Permission Restrictions for Skill Usage
- Uses tool permission system

**Status:** ✅ Done

---

## 13. Desktop/Web Interface

#### FR-130: Desktop App Startup Flow
- Location: `crates/cli/src/cmd/desktop.rs` (207 lines)

**Status:** ✅ Done

#### FR-131: Web Server Mode
- Location: `crates/cli/src/cmd/web.rs` (86 lines)

**Status:** ✅ Done

#### FR-132: Auth-Protected Interface Access
- Web UI has password protection

**Status:** ✅ Done

#### FR-133: Session Sharing Between Interface Modes
- ShareServer implemented

**Status:** ✅ Done

#### FR-134: ACP Startup/Handshake for Editor Integration
- Location: `crates/server/src/routes/acp.rs`, `acp_ws.rs`

**Status:** ✅ Done

#### FR-135: Sharing Behavior in Managed/Restricted Deployments
- `share` config option supported

**Status:** ✅ Done

### 13.1 Known Desktop/Web Issues

#### FR-136: Desktop/Web Smoke Test Port Conflict (P2)
- Test assumes specific port availability
- **Status:** ❌ NOT FIXED

---

## 14. GitHub/GitLab Integration

#### FR-140: GitHub Workflow Trigger Examples
- Location: `crates/git/src/github.rs`

**Status:** ✅ Done

#### FR-141: Comment/PR Trigger Parsing
- Location: `trigger.rs`

**Status:** ✅ Done

#### FR-142: CI Secret Loading for GitHub Actions
- Auth integration

**Status:** ✅ Done

#### FR-143: GitLab CI Component Support
- Location: `crates/git/src/gitlab_ci.rs`

**Status:** ✅ Done

#### FR-144: GitLab Duo Behavior
- Marked experimental, no explicit handling

**Status:** ⚠️ Marked Experimental

---

## 15. TUI Plugin API

#### FR-150: TUI Plugin Configuration Ownership
- `tui.json` recognized in config system

**Status:** ✅ Done

#### FR-151: Plugin Identity
- Runtime ID resolution, file vs npm distinction

**Status:** ✅ Done

#### FR-152: Plugin Deduplication Before Activation
- Deduplication logic exists

**Status:** ✅ Done

#### FR-153: Plugin Enabled Semantics
- Per-plugin enable/disable

**Status:** ✅ Done

#### FR-154: Commands, Routes, Dialogs, Slots Registration
- Location: `plugin_api.rs` (54KB)

**Status:** ✅ Done

#### FR-155: Theme Install/Set
- Location: `theme.rs`

**Status:** ✅ Done

#### FR-156: Events Subscription
- `api.event.on()`

**Status:** ✅ Done

#### FR-157: State Get/Set
- KV store + state

**Status:** ✅ Done

#### FR-158: OnDispose Lifecycle
- Cleanup registration

**Status:** ✅ Done

#### FR-159: Runtime Plugin Activate/Deactivate
- `api.plugins.activate()` / `deactivate()`

**Status:** ✅ Done

#### FR-160: Bounded Cleanup with AbortSignal
- AbortSignal enforcement

**Status:** ✅ Done

#### FR-161: Theme Auto-Sync on Install
- Implementation exists

**Status:** ✅ Done

### 15.1 TUI Plugin Tests

#### FR-162: TUI Plugin Test Coverage
- `plugin_lifecycle_tests.rs`
- `plugin_events_tests.rs`
- `plugin_state_tests.rs`
- `plugin_theme_tests.rs`
- `plugin_commands_tests.rs`
- `plugin_dispose_tests.rs`
- `plugin_enabled_tests.rs`
- `plugin_slots_tests.rs`

**Status:** ✅ Done

---

## 16. Test Plan

#### FR-170: Unit Tests for Core Entities
- Various test files

**Status:** ✅ Done

#### FR-171: Integration Tests for Agent Flow
- `agent_tool_tests.rs`, `agent_llm_tests.rs`

**Status:** ✅ Done

#### FR-172: Session Lifecycle Tests
- `session_lifecycle_tests.rs` (21KB)

**Status:** ✅ Done

#### FR-173: Compaction and Shareability Tests
- `compaction_shareability_tests.rs` (17KB)

**Status:** ✅ Done

#### FR-174: MCP Protocol Tests
- `mcp_protocol_tests.rs`

**Status:** ✅ Done

#### FR-175: Session Storage Tests
- `session_storage_tests.rs`

**Status:** ✅ Done

#### FR-176: Agent Switch Tests
- `agent_switch_tests.rs` (9KB)

**Status:** ✅ Done

#### FR-177: ACP Transport Tests
- `acp_transport_tests.rs`

**Status:** ✅ Done

#### FR-178: Convention Tests
- `conventions/` module

**Status:** ✅ Done

#### FR-179: TUI Component Tests
- `slash_command_tests.rs`, `input_model_tests.rs`, etc.

**Status:** ✅ Done

#### FR-180: ratatui-testing Crate
- `ratatui-testing/` crate exists with full implementation

**Status:** ✅ Done

---

## 17. Implementation Phases

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Done | ~95% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~95% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~95% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~90% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 18. Crate Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | Registry and custom tool discovery |
| `crates/plugin/` | ✅ Done | Hooks and tool registration |
| `crates/tui/` | ✅ Done | Full implementation, tests added |
| `crates/server/` | ✅ Done | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ⚠️ Broken | Empty re-export, not real crate |
| `crates/cli/` | ✅ Done | Desktop/web implemented |
| `crates/control-plane/` | ✅ Done | ACP stream, events, enterprise features |
| `ratatui-testing/` | ✅ Done | TUI testing framework crate |

---

## 19. Test Results Summary

```
cargo test --all-features --all:
- 610 passed
- 14 failed (across all packages)
```

### 19.1 Failing Tests

#### Config Tests (10 failures - P1)
All failures are PoisonError from `ENV_LOCK` mutex in parallel tests:
- `test_precedence_cli_none_values_dont_override_env`
- `test_precedence_cli_overrides_env`
- `test_precedence_env_config_content_overrides_file`
- `test_precedence_env_overrides_config_file`
- `test_precedence_full_chain_integration`
- `test_precedence_multiple_env_vars_stack`
- `test_precedence_opencode_dir_overrides_project`
- `test_precedence_project_config_overrides_global`
- `test_precedence_provider_api_keys_from_env`
- `test_load_multi_with_cli_overrides_full_chain`

**Root Cause:** Test infrastructure issue with shared mutex state

#### TUI Tests (3 failures - P2)
1. `keybinding::tests::test_key_parsing_simple` - case sensitivity issue
2. `keybinding::tests::test_key_parsing_space` - Space vs Char(' ') issue
3. `theme::tests::test_parse_hex_color` - color parsing wrong value

#### CLI Tests (1 failure - P2)
`desktop_web_different_ports` - port conflict in smoke test

---

## 20. Open Issues

### P1 - High Priority

| Issue | Status | Module |
|-------|--------|--------|
| Config crate empty re-export | ❌ NOT FIXED | config |
| Config tests failing with PoisonError | ❌ NOT FIXED | test infra |

### P2 - Medium Priority

| Issue | Status | Module |
|-------|--------|--------|
| TUI keybinding tests failing (2 tests) | ❌ NOT FIXED | tui |
| TUI theme color parsing test failing | ❌ NOT FIXED | tui |
| Desktop/web smoke test port conflict | ❌ NOT FIXED | cli |

### Deferred Items

| Issue | Status | Module |
|-------|--------|--------|
| Per-agent model override testing | ⚠️ Deferred | llm |
| Hidden vs visible agent UI behavior | ⚠️ Deferred | agent |

---

## 21. Technical Debt

| Item | Description | Status |
|------|-------------|--------|
| Empty `crates/config/` crate | Violates PRD 19 crate ownership | ❌ NOT FIXED |
| Config tests use ENV_LOCK with race condition | Tests fail with PoisonError | ❌ NOT FIXED |
| TUI test failures | 3 tests failing (keybinding 2, theme 1) | ❌ NOT FIXED |
| Desktop/web smoke test port conflict | Flaky test | ❌ NOT FIXED |
| Deprecated `mode` field | Marked for removal in v4.0 | ⚠️ Deferred |
| Deprecated `tools` field | Marked for removal after migration | ⚠️ Deferred |

---

## 22. Cross-Reference Index

| FR | Requirement | PRD Section |
|----|-------------|-------------|
| FR-001 - FR-016 | MCP System | PRD Section 4 |
| FR-020 - FR-027 | Core Architecture | PRD Section 1 |
| FR-030 - FR-036 | Agent System | PRD Section 2 |
| FR-040 - FR-048 | Tools System | PRD Section 3 |
| FR-050 - FR-054 | LSP System | PRD Section 5 |
| FR-060 - FR-067 | Configuration System | PRD Section 6 |
| FR-070 - FR-075 | HTTP Server API | PRD Section 7 |
| FR-080 - FR-085 | Plugin System | PRD Section 8 |
| FR-090 - FR-099 | TUI System | PRD Section 9 |
| FR-100 - FR-105 | Provider/Model System | PRD Section 10 |
| FR-110 - FR-114 | Formatters | PRD Section 11 |
| FR-120 - FR-125 | Skills System | PRD Section 12 |
| FR-130 - FR-136 | Desktop/Web Interface | PRD Section 13 |
| FR-140 - FR-144 | GitHub/GitLab Integration | PRD Section 14 |
| FR-150 - FR-162 | TUI Plugin API | PRD Section 15 |
| FR-170 - FR-180 | Test Plan | PRD Section 16 |

---

## 23. Iteration History

| Iteration | Date | Progress | Key Changes |
|-----------|------|----------|--------------|
| 16 | 2026-04-14 | ~75-80% | Initial implementation |
| 17 | 2026-04-14 | ~85-90% | Fixed P0 issues: custom tool discovery, plugin tool registration, hook execution order, desktop/web modes, ACP transport |

---

*Document Version: 17*
*Last Updated: 2026-04-14*
*Next Action: Fix P1 issues (config crate refactor, test infrastructure)*
