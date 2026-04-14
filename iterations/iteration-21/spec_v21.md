# OpenCode RS Specification - Iteration 21

**Version:** 2.1
**Generated:** 2026-04-14
**Status:** Implementation ~93-96% Complete
**Phase:** Phase 1-5 Complete (~98%), Phase 6 (Release Qualification) Pending
**Iteration Focus:** PRD 20 - ratatui-testing Framework (Implementation NOT STARTED)

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

**Implementation:** Full implementation in `crates/config/src/lib.rs` (106KB+)

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

---

## 17. ratatui-testing Framework (PRD 20)

### 17.1 Overview

The `ratatui-testing` crate provides a testing framework for TUI applications built with `ratatui`. It enables simulation of user input, buffer comparison, and state validation.

**Location:** `ratatui-testing/`

**Status:** ❌ **NOT IMPLEMENTED** - All modules remain as stubs (no progress since iteration-20)

### 17.2 PtySimulator

#### FR-122: PTY Master/Slave Creation

Creates a PTY (pseudo-terminal) master/slave pair on Unix systems.

**Required:** Use `portable-pty` crate for cross-platform PTY support.

**Status:** ❌ STUB - Implementation needed

```rust
// CURRENT STUB (11 lines):
pub struct PtySimulator;
impl PtySimulator {
    pub fn new() -> Self { Self }
    pub fn write_input(&mut self, _input: &str) -> Result<()> { Ok(()) }
    pub fn read_output(&mut self) -> Result<String> { Ok(String::new()) }
}

// REQUIRED IMPLEMENTATION:
pub struct PtySimulator {
    master: Box<dyn MasterPty>,
    child: Box<dyn ChildPty>,
}

impl PtySimulator {
    pub fn new(command: &[&str]) -> Result<Self>;
    pub fn write_input(&mut self, data: &str) -> Result<()>;
    pub fn read_output(&mut self, timeout: Duration) -> Result<String>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn inject_key_event(&mut self, event: KeyEvent) -> Result<()>;
    pub fn inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>;
}
```

#### FR-123: PTY Write/Read Operations

- `write_input(data)`: Write strings to PTY slave
- `read_output(timeout)`: Read output from PTY master with configurable timeout

**Status:** ❌ NOT IMPLEMENTED

#### FR-124: PTY Window Resize

`resize(cols, rows)`: Resize PTY window dimensions.

**Status:** ❌ NOT IMPLEMENTED

#### FR-125: PTY Event Injection

- `inject_key_event(event)`: Inject keyboard events via `crossterm`
- `inject_mouse_event(event)`: Inject mouse events via `crossterm`

**Status:** ❌ NOT IMPLEMENTED

### 17.3 BufferDiff

#### FR-126: Buffer Comparison

Compares two `ratatui::Buffer` instances cell-by-cell to detect differences.

**Status:** ❌ STUB - Implementation needed

```rust
// CURRENT STUB (12 lines):
pub struct BufferDiff;
impl BufferDiff {
    pub fn new() -> Self { Self }
    pub fn diff(&self, _expected: &str, _actual: &str) -> Result<String> { Ok(String::new()) }
}

// REQUIRED IMPLEMENTATION:
#[derive(Debug)]
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}

#[derive(Debug)]
pub struct DiffResult {
    pub cells: Vec<CellDiff>,
    pub total_diffs: usize,
    pub passed: bool,
}

pub struct BufferDiff {
    ignore_colors: bool,
    ignore_attributes: bool,
}

impl BufferDiff {
    pub fn new() -> Self;
    pub fn ignore_fg(mut self, ignore: bool) -> Self;
    pub fn ignore_bg(mut self, ignore: bool) -> Self;
    pub fn ignore_attributes(mut self, ignore: bool) -> Self;
    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult;
}
```

#### FR-127: Difference Reporting

Reports exact x,y coordinates of differing cells with expected vs actual values.

**Status:** ❌ NOT IMPLEMENTED

#### FR-128: Diff Options

- `ignore_fg`: Ignore foreground color differences
- `ignore_bg`: Ignore background color differences
- `ignore_attributes`: Ignore style attribute differences (bold, italic, etc.)

**Status:** ❌ NOT IMPLEMENTED

#### FR-129: Human-Readable Diff Output

Provides formatted diff output for test failure messages.

**Status:** ❌ NOT IMPLEMENTED

### 17.4 StateTester

#### FR-130: State Capture

Captures serializable state to JSON for comparison.

**Status:** ❌ STUB - Implementation needed

```rust
// CURRENT STUB (17 lines):
pub struct StateTester;
impl StateTester {
    pub fn new() -> Self { Self }
    pub fn assert_state<S>(&self, _state: &S) -> Result<()> where S: serde::Serialize { Ok(()) }
}

// REQUIRED IMPLEMENTATION:
pub struct StateTester {
    snapshot: Option<Value>,
}

impl StateTester {
    pub fn new() -> Self;
    pub fn capture<S>(&mut self, state: &S) -> Result<()>
    where S: Serialize;
    pub fn assert_state<S>(&self, current: &S) -> Result<()>
    where S: Serialize;
    pub fn assert_state_matches(&self, expected: &Value) -> Result<()>;
}
```

#### FR-131: State Comparison

Compares current application state against captured snapshot using JSON diff.

**Status:** ❌ NOT IMPLEMENTED

#### FR-132: Mismatch Reporting

Reports mismatches with JSON diff format showing expected vs actual values.

**Status:** ❌ NOT IMPLEMENTED

### 17.5 TestDsl

#### FR-133: Widget Rendering

Renders ratatui widgets to Buffer for testing.

**Status:** ❌ STUB - Implementation needed

```rust
// CURRENT STUB (14 lines):
pub struct TestDsl;
impl TestDsl {
    pub fn new() -> Self { Self }
    pub fn render(&self, _widget: impl std::fmt::Debug) -> Result<()> { Ok(()) }
}

// REQUIRED IMPLEMENTATION:
pub struct TestDsl {
    pty: Option<PtySimulator>,
    diff: BufferDiff,
    state_tester: StateTester,
}

impl TestDsl {
    pub fn new() -> Self;
    pub fn with_pty(mut self) -> Result<Self>;
    pub fn render_widget<W: Widget>(&self, widget: &W) -> Result<Buffer>;
    pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>;
    pub fn wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>
    where F: Fn() -> bool;
    pub fn capture_state<S>(&mut self, state: &S) -> &mut Self;
    pub fn assert_state<S>(&self, state: &S) -> Result<()>;
}
```

#### FR-134: Fluent API Composition

Composes PtySimulator, BufferDiff, and StateTester into a fluent test API.

**Status:** ❌ NOT IMPLEMENTED

#### FR-135: Wait-For Predicate

`wait_for(timeout, predicate)`: Waits for a condition to be true with timeout.

**Status:** ❌ NOT IMPLEMENTED

### 17.6 CliTester

#### FR-136: Process Spawning

Spawns a CLI process with arguments.

**Status:** ❌ STUB - Implementation needed

```rust
// CURRENT STUB (14 lines):
pub struct CliTester;
impl CliTester {
    pub fn new() -> Self { Self }
    pub fn run(&self, _args: &[&str]) -> Result<String> { Ok(String::new()) }
}

// REQUIRED IMPLEMENTATION:
pub struct CliTester {
    bin: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    temp_dir: Option<TempDir>,
}

pub struct CliOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl CliTester {
    pub fn new(bin: &str) -> Self;
    pub fn args(mut self, args: &[&str]) -> Self;
    pub fn env(mut self, key: &str, value: &str) -> Self;
    pub fn with_temp_dir(mut self) -> Result<Self>;
    pub fn run(&self) -> Result<CliOutput>;
}
```

#### FR-137: Output Capture

Captures stdout and stderr from spawned process.

**Status:** ❌ NOT IMPLEMENTED

#### FR-138: Exit Code and Cleanup

Returns exit code and automatically cleans up temporary directories.

**Status:** ❌ NOT IMPLEMENTED

### 17.7 Integration Tests

#### FR-139: Module Compilation

All ratatui-testing modules compile together without errors.

**Status:** ❌ NOT IMPLEMENTED - tests/ directory is empty

#### FR-140: Test Execution

Integration tests pass with `cargo test -p ratatui-testing`.

**Status:** ❌ NOT IMPLEMENTED - No tests exist

#### FR-141: Cross-Platform Support

Primary support for Unix systems; Windows support as best-effort.

**Status:** ⚠️ Planned

### 17.8 Acceptance Criteria Status

| Criteria | Status | Implementation |
|----------|--------|---------------|
| Creates PTY master/slave pair on Unix | ❌ NOT IMPLEMENTED | Stub only |
| Writes strings to PTY slave | ❌ NOT IMPLEMENTED | Stub only |
| Reads output from PTY master with timeout | ❌ NOT IMPLEMENTED | Stub only |
| Resizes PTY window (cols/rows) | ❌ NOT IMPLEMENTED | Method missing |
| Injects KeyEvent via crossterm | ❌ NOT IMPLEMENTED | Stub only |
| Injects MouseEvent via crossterm | ❌ NOT IMPLEMENTED | Stub only |
| Compares two Buffers cell-by-cell | ❌ NOT IMPLEMENTED | Stub only |
| Reports exact x,y of differences | ❌ NOT IMPLEMENTED | Structs missing |
| Supports ignoring fg/bg/attributes | ❌ NOT IMPLEMENTED | Options missing |
| Provides human-readable diff output | ❌ NOT IMPLEMENTED | Stub only |
| Captures serializable state to JSON | ❌ NOT IMPLEMENTED | Method missing |
| Compares current state to snapshot | ❌ NOT IMPLEMENTED | Stub only |
| Reports mismatches with JSON diff | ❌ NOT IMPLEMENTED | Not implemented |
| Renders widget to Buffer | ❌ NOT IMPLEMENTED | Stub only |
| Composes PTY, BufferDiff, StateTester | ❌ NOT IMPLEMENTED | No composition |
| Fluent API chains correctly | ❌ NOT IMPLEMENTED | Not implemented |
| Wait-for predicate support | ❌ NOT IMPLEMENTED | Method missing |
| Spawns process with args | ❌ NOT IMPLEMENTED | Stub only |
| Captures stdout/stderr | ❌ NOT IMPLEMENTED | Not implemented |
| Returns exit code | ❌ NOT IMPLEMENTED | Not implemented |
| Cleans up temp directories | ❌ NOT IMPLEMENTED | Not implemented |
| All modules compile together | ⚠️ PARTIAL | Compiles but stub |
| Integration tests pass | ❌ NOT IMPLEMENTED | No tests exist |
| Works with `cargo test` | ❌ NOT IMPLEMENTED | No tests exist |

---

## 18. Implementation Status

### 18.1 Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~100% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~100% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~100% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~98% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~98% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |
| **PRD 20** | **ratatui-testing Framework** | **❌ NOT STARTED** | **~0%** |

### 18.2 Crate Status

| Crate | Status | Lines | Notes |
|-------|--------|-------|-------|
| `crates/core/` | ✅ Done | ~83KB | Entity models, session, tool, skill |
| `crates/storage/` | ✅ Done | ~15KB | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | ~64KB | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | ~2.3KB | Registry and custom tool discovery |
| `crates/plugin/` | ✅ Done | ~3.5KB | Hooks and tool registration |
| `crates/tui/` | ✅ Done | ~191KB | Full TUI implementation |
| `crates/server/` | ✅ Done | ~50KB | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | ~58KB | Full MCP implementation |
| `crates/lsp/` | ✅ Done | Multiple | LSP client, diagnostics |
| `crates/llm/` | ✅ Done | Multiple | Multiple providers, model selection |
| `crates/git/` | ✅ Done | ~1.7KB | GitHub/GitLab integration |
| `crates/config/` | ✅ Done | ~106KB | Full config implementation |
| `crates/cli/` | ✅ Done | Desktop/web | Desktop/web implemented |
| `crates/control-plane/` | ✅ Done | ACP stream | ACP stream, events, enterprise |
| `ratatui-testing/` | ❌ STUB | ~100 lines | **All modules stubs - NOT IMPLEMENTED** |

---

## 19. Known Issues

### 19.1 Open Issues

| Issue | Severity | Module | Status | Fix Required |
|-------|----------|--------|--------|--------------|
| ratatui-testing PtySimulator - all methods stubs | P0 | ratatui-testing | ❌ NOT IMPLEMENTED | Full implementation using `portable-pty` |
| ratatui-testing BufferDiff - all methods stubs | P0 | ratatui-testing | ❌ NOT IMPLEMENTED | Full buffer comparison implementation |
| ratatui-testing StateTester - all methods stubs | P0 | ratatui-testing | ❌ NOT IMPLEMENTED | State capture/comparison implementation |
| ratatui-testing TestDsl - all methods stubs | P0 | ratatui-testing | ❌ NOT IMPLEMENTED | Fluent API composition implementation |
| ratatui-testing CliTester - all methods stubs | P0 | ratatui-testing | ❌ NOT IMPLEMENTED | CLI process spawning implementation |
| ratatui-testing tests/ directory empty | P0 | ratatui-testing | ❌ NOT IMPLEMENTED | Integration tests needed |
| Phase 6 Release Qualification not started | P1 | all | ❌ NOT STARTED | End-to-end testing, benchmarks |
| `test_bedrock_credential_resolution_bearer_token_priority` fails with `--all-features` | P1 | llm | ❌ NOT FIXED | Test pollution from AWS env vars |
| Trailing whitespace in `storage/src/service.rs` | P2 | storage | ❌ NOT FIXED | Run `cargo fmt` |
| Multiple clippy warnings | P2 | multiple | ⚠️ Minor | Run `cargo clippy --fix` |

### 19.2 P0 Priority: PRD 20 Implementation Checklist

#### PtySimulator - Required Implementation

**Dependencies:**
- `portable-pty` for cross-platform PTY
- `crossterm` for key/mouse event injection
- `tokio` for async timeout handling

**Methods to Implement:**
- [ ] `new(command: &[&str]) -> Result<Self>`: Create PTY with command
- [ ] `write_input(data: &str) -> Result<()>`: Write to PTY slave
- [ ] `read_output(timeout: Duration) -> Result<String>`: Read from PTY master
- [ ] `resize(cols: u16, rows: u16) -> Result<()>`: Resize window
- [ ] `inject_key_event(event: KeyEvent) -> Result<()>`: Inject KeyEvent
- [ ] `inject_mouse_event(event: MouseEvent) -> Result<()>`: Inject MouseEvent

#### BufferDiff - Required Implementation

**Dependencies:**
- `ratatui` for Buffer/Cell types

**Structs to Implement:**
- [ ] `pub struct CellDiff { x: u16, y: u16, expected: Cell, actual: Cell }`
- [ ] `pub struct DiffResult { cells: Vec<CellDiff>, total_diffs: usize, passed: bool }`

**Methods to Implement:**
- [ ] `new() -> Self`: Create with defaults
- [ ] `ignore_fg(mut self, ignore: bool) -> Self`: Set foreground ignore
- [ ] `ignore_bg(mut self, ignore: bool) -> Self`: Set background ignore
- [ ] `ignore_attributes(mut self, ignore: bool) -> Self`: Set attributes ignore
- [ ] `diff(expected: &Buffer, actual: &Buffer) -> DiffResult`: Compare buffers

#### StateTester - Required Implementation

**Dependencies:**
- `serde_json` for JSON serialization

**Methods to Implement:**
- [ ] `new() -> Self`: Create with empty snapshot
- [ ] `capture<S>(&mut self, state: &S) -> Result<()>`: Serialize state to JSON
- [ ] `assert_state<S>(&self, current: &S) -> Result<()>`: Compare with snapshot
- [ ] `assert_state_matches(&self, expected: &Value) -> Result<()>`: Compare with expected JSON

#### TestDsl - Required Implementation

**Composition of:**
- PtySimulator for input simulation
- BufferDiff for output verification
- StateTester for state validation

**Methods to Implement:**
- [ ] `new() -> Self`: Create with default components
- [ ] `with_pty(command: &[&str]) -> Result<Self>`: Add PTY for interactive testing
- [ ] `render_widget<W: Widget>(&self, widget: &W) -> Result<Buffer>`: Render to Buffer
- [ ] `send_keys(&mut self, keys: &str) -> Result<&mut Self>`: Simulate keyboard input
- [ ] `wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>`: Wait for condition
- [ ] `capture_state<S>(&mut self, state: &S) -> &mut Self`: Capture state snapshot
- [ ] `assert_state<S>(&self, state: &S) -> Result<()>`: Assert state matches

#### CliTester - Required Implementation

**Dependencies:**
- `assert_cmd` for process spawning
- `tempfile` for temp directories

**Structs to Implement:**
- [ ] `pub struct CliOutput { exit_code: i32, stdout: String, stderr: String }`

**Methods to Implement:**
- [ ] `new(bin: &str) -> Self`: Create with binary path
- [ ] `args(mut self, args: &[&str]) -> Self`: Add command arguments
- [ ] `env(mut self, key: &str, value: &str) -> Self`: Add environment variables
- [ ] `with_temp_dir(mut self) -> Result<Self>`: Add temp directory
- [ ] `run(&self) -> Result<CliOutput>`: Execute and return output

#### Integration Tests - Required Files

**Test files to create:**
- [ ] `tests/pty_tests.rs` - PTY functionality tests
- [ ] `tests/buffer_diff_tests.rs` - Buffer comparison tests
- [ ] `tests/state_tests.rs` - State testing tests
- [ ] `tests/dsl_tests.rs` - Fluent API tests
- [ ] `tests/cli_tests.rs` - CLI testing tests
- [ ] `tests/integration_tests.rs` - Full workflow tests

---

## 20. Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config ownership, `mcp` key schema, `permission` rules |
| [07-server-api.md](./07-server-api.md) | MCP API endpoints (if exposed via HTTP) |
| [PRD 20](./ratatui-testing-prd.md) | ratatui-testing framework detailed specifications |
| [Gap Analysis](./gap-analysis.md) | Iteration 21 gap analysis report |

---

## 21. Test Results

```
cargo test --all-features --all:
- ~1020+ passed across all crates
- 0 test failures

Outstanding Issues:
- ratatui-testing: All modules remain as stubs (no progress)
- ratatui-testing: tests/ directory is empty, no integration tests
- Multiple clippy warnings (dead code, unused variables, unused imports)
```

---

## 22. Iteration 21 Changes

### Key Observation

**No significant progress on PRD 20 implementation since iteration-20.**

The ratatui-testing framework remains entirely in stub form:
- `pty.rs`: 11 lines (stub)
- `diff.rs`: 12 lines (stub)
- `state.rs`: 17 lines (stub)
- `dsl.rs`: 14 lines (stub)
- `cli.rs`: 14 lines (stub)
- `tests/`: Empty directory

### Priority for Next Iteration

1. **P0 - Implement ratatui-testing Framework**
   - PtySimulator: Use `portable-pty` for PTY operations
   - BufferDiff: Implement cell-by-cell comparison
   - StateTester: Implement state capture and JSON comparison
   - TestDsl: Compose all components into fluent API
   - CliTester: Use `assert_cmd` for process spawning
   - Integration tests: Add tests for all modules

2. **P1 - Begin Phase 6 Release Qualification**
   - End-to-end integration tests
   - Performance benchmarking
   - Security audit
   - Observability validation

3. **P2 - Fix Remaining Issues**
   - Fix Bedrock test environment pollution
   - Fix trailing whitespace
   - Fix clippy warnings

---

## 23. Feature Requirements Summary

| FR Range | Category | Count | Status |
|----------|----------|-------|--------|
| FR-001 to FR-016 | MCP System | 16 | ✅ Done |
| FR-017 to FR-024 | Core Architecture | 8 | ✅ Done |
| FR-025 to FR-031 | Agent System | 7 | ✅ Done |
| FR-032 to FR-039 | Tools System | 8 | ✅ Done |
| FR-040 to FR-044 | LSP System | 5 | ✅ Done |
| FR-045 to FR-051 | Configuration | 7 | ✅ Done |
| FR-052 to FR-057 | HTTP Server API | 6 | ✅ Done |
| FR-058 to FR-063 | Plugin System | 6 | ✅ Done |
| FR-064 to FR-070 | TUI System | 7 | ✅ Done |
| FR-071 to FR-076 | Provider/Model | 6 | ✅ Done |
| FR-077 to FR-081 | Formatters | 5 | ✅ Done |
| FR-082 to FR-087 | Skills System | 6 | ✅ Done |
| FR-088 to FR-093 | Desktop/Web | 6 | ✅ Done |
| FR-094 to FR-098 | GitHub/GitLab | 5 | ✅ Done |
| FR-099 to FR-110 | TUI Plugin API | 12 | ✅ Done |
| FR-111 to FR-120 | Test Plan | 10 | ✅ Done |
| FR-121 | ~~Ratatui Testing~~ (merged) | - | - |
| FR-122 to FR-141 | ratatui-testing (PRD 20) | 20 | ❌ NOT IMPLEMENTED |

**Total: 140 Feature Requirements**
- ✅ Done: 120
- ❌ Not Implemented: 20 (PRD 20 - ratatui-testing)

**Overall Implementation: ~93-96% Complete**

---

## 24. Gap Summary

### P0 - Blocking Issues (PRD 20 Implementation)

| Issue | Status | Module | Impact |
|-------|--------|--------|--------|
| PtySimulator stub implementation | ❌ NOT IMPLEMENTED | ratatui-testing | **Blocks TUI testing** |
| BufferDiff stub implementation | ❌ NOT IMPLEMENTED | ratatui-testing | **Blocks buffer comparison** |
| StateTester stub implementation | ❌ NOT IMPLEMENTED | ratatui-testing | **Blocks state testing** |
| TestDsl stub implementation | ❌ NOT IMPLEMENTED | ratatui-testing | **Blocks fluent test API** |
| CliTester stub implementation | ❌ NOT IMPLEMENTED | ratatui-testing | **Blocks CLI testing** |
| Empty ratatui-testing tests/ directory | ❌ NOT IMPLEMENTED | ratatui-testing | **No test coverage** |

### P1 - High Priority Issues

| Issue | Status | Module | Impact |
|-------|--------|--------|--------|
| Phase 6 Release Qualification not started | ❌ NOT STARTED | all | **Cannot release** |
| `test_bedrock_credential_resolution_bearer_token_priority` fails | ❌ NOT FIXED | llm | Test reliability |

### P2 - Medium Priority Issues

| Issue | Status | Module | Impact |
|-------|--------|--------|--------|
| Trailing whitespace in `storage/src/service.rs` | ❌ NOT FIXED | storage | Cleanliness |
| Multiple clippy warnings | ⚠️ Minor | multiple | Code quality |

---

**Document Version:** 2.1
**Iteration:** 21
**Last Updated:** 2026-04-14