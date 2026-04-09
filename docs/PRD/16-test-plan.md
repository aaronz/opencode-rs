# PRD: Test Plan

## Scope

This document defines the validation strategy for the Rust port described by the PRD set in this directory.

It covers:

- Functional correctness against PRD requirements
- Cross-document consistency validation
- Regression coverage for authority documents and subsystem boundaries
- Test levels: unit, integration, end-to-end, compatibility, convention, and non-functional testing
- Release gates for implementation readiness

This document does **not** redefine product behavior. Each test area inherits its requirements from the corresponding PRD.

---

## Objectives

The test plan must prove that the Rust port:

1. Preserves the core entity and lifecycle invariants in [01-core-architecture.md](./01-core-architecture.md)
2. Implements configuration ownership and precedence exactly as defined in [06-configuration-system.md](./06-configuration-system.md)
3. Exposes a coherent API surface as defined in [07-server-api.md](./07-server-api.md)
4. Enforces agent, tool, permission, plugin, and TUI boundaries consistently across subsystem implementations
5. Preserves compatibility behavior only where explicitly intended
6. Fails safely on invalid input, denied permissions, and unsupported configurations

---

## Test Strategy

### Test Levels

| Level | Purpose | Typical Scope |
|---|---|---|
| Unit | Validate isolated invariants and transformations | entity models, config normalization, permission matching, tool argument validation |
| Integration | Validate subsystem interactions | session + storage, agent + tool + permission, server + config, plugin + runtime |
| End-to-End | Validate user-visible workflows | CLI/TUI flows, session lifecycle, sharing, plugin activation, API workflows |
| Compatibility | Validate legacy/interop behavior | config aliases, historical route adapters if implemented, compat skill/plugin paths |
| Convention | Validate implementation and test-suite structure | crate boundaries, naming, ownership rules, route grouping, test placement |
| Non-Functional | Validate system qualities | performance, reliability, recovery, security, observability |

### Coverage Model

Coverage should be organized by PRD authority:

- **Authority docs first**: `01`, `06`, `07`
- **Runtime architecture second**: `02`, `03`, `08`, `15`
- **Subsystem behavior third**: `04`, `05`, `09`, `10`, `11`, `12`, `13`, `14`

No implementation milestone is complete unless authority-doc requirements and subsystem cross-boundaries are both tested.

---

## Test Environment Matrix

### Operating Environments

- macOS
- Linux
- Windows / WSL where applicable

### Repository States

- Git-backed workspace
- Non-git workspace where supported
- Nested working directory inside a project root
- Multi-session / multi-workspace scenarios

### Configuration States

- Default configuration
- Global config only
- Project config only
- Combined global + project config
- Legacy compatibility keys present
- Invalid / conflicting config values

### Integration Modes

- CLI-only
- TUI-enabled
- Local server/API mode
- Plugin-enabled runtime
- MCP-enabled runtime
- LSP-enabled runtime

---

## Authority Document Test Coverage

### A. Core Architecture Tests (`01`)

#### Unit

- Validate `Project → Session → Message → Part` ownership is acyclic
- Validate stable identity semantics for project/session/message records
- Validate message ordering and part ordering invariants
- Validate fork creates a new child session identity without mutating parent identity
- Validate snapshot/checkpoint metadata can restore session-visible state

#### Integration

- Create project → session → messages → fork → share → compact → revert
- Verify session recovery after restart preserves ordering and identity
- Verify revert restores checkpointed state without requiring per-edit Git commits
- Verify compaction preserves resumability and shareability

#### Negative

- Reject orphan message insertion without valid session
- Reject invalid ownership references (message to unknown session, session to unknown project)
- Reject cyclic lineage or invalid parent relationships

### B. Configuration System Tests (`06`)

#### Unit

- Validate precedence ordering across config sources
- Validate `tools` legacy alias normalizes into `permission`
- Validate variable expansion for environment and file references
- Validate ownership boundaries: `opencode.json` vs `tui.json`
- Validate config parsing for JSON and JSONC

#### Integration

- Load combined global + project + inline config and verify deterministic merge
- Verify TUI settings are ignored in main config if the implementation enforces ownership boundaries
- Verify server/runtime plugin config loads from main config while TUI plugin config loads from `tui.json`
- Verify auth/secret storage lookup works across configured providers/MCP cases

#### Negative

- Invalid JSON/JSONC
- Unknown enum values for share/autoupdate/permission settings
- Conflicting plugin ownership declarations
- Missing env/file expansion targets

### C. HTTP Server API Tests (`07`)

#### Unit

- Validate route registration consistency by resource group
- Validate route naming/versioning conventions if versioned routes are implemented

#### Integration

- Health/global runtime endpoints
- Project/session/message lifecycle endpoints
- Permission response flow
- Config/provider/model endpoints
- MCP-related endpoints if exposed
- Streaming endpoints (SSE/websocket) where supported

#### Compatibility

- If legacy nested route shapes are implemented, verify they are explicitly adapter behavior and not the primary canonical surface

#### Negative

- Unauthorized access
- Invalid session/project/message identifiers
- Invalid request payloads
- Streaming disconnect / reconnect behavior

---

## Runtime Architecture Test Coverage

### D. Agent System Tests (`02`)

- Verify exactly one active primary agent per session
- Verify hidden vs visible agents behave correctly in selection flows
- Verify subagent execution creates child execution context without corrupting parent history
- Verify task/delegation results return to parent agent correctly
- Verify agent-level restrictions tighten effective permissions where applicable

### E. Tools System Tests (`03`)

- Verify tool registration and availability lifecycle
- Verify built-in, custom, and MCP tools all pass through the same execution pipeline
- Verify permission gate occurs before tool execution
- Verify argument validation errors are structured and non-destructive
- Verify deterministic name resolution for tool collisions
- Verify cache-sensitive and non-cacheable tool behavior where implemented

### F. Plugin System Tests (`08`)

- Verify server/runtime plugin loading from configured sources
- Verify hooks run in correct lifecycle stages
- Verify plugin-provided tools register and execute through normal tool flow
- Verify plugin isolation and failure handling do not destabilize the runtime
- Verify server/runtime plugin config is not conflated with TUI plugin config

### G. TUI Plugin API Tests (`15`)

- Verify `tui.json` plugin ownership and `plugin_enabled` behavior
- Verify deterministic plugin identity and deduplication
- Verify command, route, dialog, slot, theme, and event registrations
- Verify runtime activate/deactivate semantics
- Verify cleanup/disposal order and bounded cleanup behavior
- Verify plugin state persistence and restart behavior where applicable

---

## Subsystem Test Coverage

### H. MCP System Tests (`04`)

- Local MCP server registration and execution
- Remote MCP server connection and timeout handling
- Per-server OAuth configuration behavior
- MCP tool naming / qualification behavior
- Permission gating for MCP tools
- Failure handling for unavailable or misconfigured servers

### I. LSP System Tests (`05`)

- Built-in server detection for supported languages
- Custom LSP registration via config
- Diagnostics retrieval and surfacing into runtime workflows
- Experimental LSP tool gating where implemented
- Failure handling when servers are unavailable or disabled

### J. TUI System Tests (`09`)

- Session view rendering and message display
- Slash command execution
- Input model: multiline, history, autocomplete, file references, shell prefix
- Sidebar visibility and contextual sections
- Keybinding-driven session and message navigation

### K. Provider / Model Tests (`10`)

- Default model selection
- Per-agent model override behavior
- Provider credential resolution
- Variant / reasoning budget handling
- Local model provider behavior
- Provider fallback or unavailable-provider handling where supported

### L. Formatter Tests (`11`)

- Formatter detection by file type and project config
- Disable-all and per-formatter disable behavior
- Custom formatter command invocation
- Automatic formatting on write/edit flows
- Failure behavior when formatter is missing or errors

### M. Skills Tests (`12`)

- Deterministic skill discovery by precedence order
- Stable duplicate resolution within a scope
- Compatibility path loading for Claude/Agent-style directories
- Skill loading into runtime context
- Skill permission restrictions and denial behavior

### N. Desktop / Web / ACP Tests (`13`)

- Local desktop/web startup flow where implemented
- Runtime attachment between interfaces where supported
- Auth protection for served interfaces
- ACP/editor integration startup and connection lifecycle
- Sharing behavior in managed vs restricted deployment modes

### O. GitHub / GitLab Integration Tests (`14`)

- GitHub-triggered workflow examples with expected inputs/outputs
- Comment/PR-trigger parsing and execution path
- Auth/secret loading for CI environments
- GitLab CI component flow where supported
- Explicit marking/skipping of environment-dependent GitLab Duo behavior

---

## Cross-Document Consistency Tests

These tests verify the PRD boundaries themselves are preserved in implementation:

- `01` owns entities/invariants; no API or config behavior is implemented only there
- `06` owns config schema and ownership boundaries
- `07` owns HTTP route authority
- `08` owns server/runtime plugin behavior
- `15` owns TUI plugin behavior
- `09` does not redefine plugin/config authority
- `10`, `12`, `04`, and `13` reference `06` rather than redefining config schema

These checks should be automated where possible using spec-to-implementation mapping tests or documentation conformance checks.

---

## Convention Tests

Convention tests validate that the implementation and the test suite itself continue to follow the structural rules established by the PRD set and repository conventions.

### Architecture Boundary Conventions

- config ownership remains centered in `06`
- HTTP route ownership remains centered in `07`
- server/runtime plugin behavior remains centered in `08`
- TUI plugin behavior remains centered in `15`
- subsystem implementations do not silently reintroduce authority drift

### Rust Codebase Conventions

- crate and module placement follows repository architecture
- naming follows Rust conventions from `AGENTS.md`
- error types and error-code/category usage remain in the intended layer
- public API surface is exposed from the correct crate/module boundaries

### Test Suite Conventions

- crate-local invariants live in crate-local tests
- cross-crate workflows live under `opencode-rust/tests/`
- TUI interaction tests use `ratatui-testing`
- compatibility tests remain isolated from canonical behavior tests
- environment-dependent tests are clearly marked and gated

### API / Config / Permission Conventions

- route registration continues to follow the canonical resource-group layout
- config ownership split between `opencode.json` and `tui.json` is preserved
- `tools` compatibility handling normalizes into canonical permission behavior
- permission checks happen before tool execution
- MCP/tool naming conventions stay deterministic

### Convention Test Outputs

Convention tests should fail with actionable messages that identify the violated rule, the affected crate/file/suite, and the PRD or repo convention being broken.

---

## Negative and Failure Testing

Every subsystem should include at least:

- Invalid identifier handling
- Invalid config handling
- Permission denial behavior
- Missing dependency/tool/server behavior
- Restart/recovery behavior after interruption
- Timeout and cancellation behavior for long-running operations

Special attention areas:

- Revert/compaction safety
- Plugin cleanup failures
- MCP timeout/auth failures
- LSP server absence
- Formatter absence or malformed output

---

## Non-Functional Testing

### Performance

- Session creation latency
- Message append/read latency
- Tool execution overhead
- Config load/merge overhead
- TUI startup and plugin activation time
- API responsiveness under concurrent session/tool activity

### Reliability

- Crash recovery
- Snapshot/revert durability
- Concurrent session isolation
- Plugin failure containment
- Server restart behavior

### Security

- Permission enforcement before execution
- Secret storage and redaction checks
- Auth-protected API access
- Shared-session access control
- Plugin and MCP attack-surface review

### Observability

- Structured error reporting
- Diagnostic visibility for API, MCP, and LSP failures
- Ability to trace failed workflows in logs/test output

---

## Release Gates

An implementation milestone is ready only when all of the following pass:

1. **Authority tests**: `01`, `06`, and `07` coverage is green
2. **Runtime architecture tests**: `02`, `03`, `08`, and `15` coverage is green
3. **Subsystem tests**: each implemented subsystem has positive and negative coverage
4. **Compatibility tests**: intended legacy/interop behaviors are explicitly tested
5. **Recovery tests**: restart/revert/cleanup paths are exercised
6. **No ownership regressions**: config/API/plugin/TUI boundaries remain intact

Recommended CI gates:

- Unit test suite
- Integration test suite
- API contract test suite
- TUI/plugin integration suite
- Compatibility suite
- Convention suite
- Smoke end-to-end workflow suite

---

## Suggested Test Suite Layout

```text
tests/
  unit/
    core/
    config/
    agent/
    tools/
    providers/
  integration/
    session_lifecycle/
    config_merge/
    api/
    plugin_runtime/
    tui_plugins/
    mcp/
    lsp/
    formatters/
    skills/
  e2e/
    cli/
    tui/
    web_server/
    sharing/
  compatibility/
    config_aliases/
    skill_paths/
    legacy_routes/
  conventions/
    architecture/
    config/
    routes/
    naming/
    test_layout/
  nonfunctional/
    performance/
    recovery/
    security/
```

---

## Traceability Matrix

Each implemented feature should map to:

- a source PRD section
- one or more automated tests
- one or more failure-mode tests
- a release-gate category

Minimum traceability rule:

| PRD Type | Minimum Required Tests |
|---|---|
| Authority requirement | unit + integration |
| Runtime interaction | integration + negative |
| User-visible workflow | e2e + negative |
| Compatibility behavior | compatibility + regression |
| Convention rule | convention + regression |
| Reliability/security requirement | non-functional + regression |

---

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md)
- [02-agent-system.md](./02-agent-system.md)
- [03-tools-system.md](./03-tools-system.md)
- [04-mcp-system.md](./04-mcp-system.md)
- [05-lsp-system.md](./05-lsp-system.md)
- [06-configuration-system.md](./06-configuration-system.md)
- [07-server-api.md](./07-server-api.md)
- [08-plugin-system.md](./08-plugin-system.md)
- [09-tui-system.md](./09-tui-system.md)
- [10-provider-model-system.md](./10-provider-model-system.md)
- [11-formatters.md](./11-formatters.md)
- [12-skills-system.md](./12-skills-system.md)
- [13-desktop-web-interface.md](./13-desktop-web-interface.md)
- [14-github-gitlab-integration.md](./14-github-gitlab-integration.md)
- [15-tui-plugin-api.md](./15-tui-plugin-api.md)
