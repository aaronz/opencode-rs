# OpenCode Rust Port — Specification Document v1

**Version:** 1.0
**Generated:** 2026-04-09
**Based on:** PRD specifications and gap analysis
**Status:** Draft

---

## 1. Overview

This document defines the specification for the OpenCode Rust port implementation. It is derived from the PRD specifications (`01`–`15`) and updated based on gap analysis between planned features and current implementation status.

## 2. Implementation Status Summary

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | ✅ Complete | 90% |
| Phase 2 | Runtime Core | ✅ Complete | 85% |
| Phase 3 | Infrastructure Subsystems | ✅ Complete | 80% |
| Phase 4 | Interface Implementations | ⚠️ Partial | 60% |
| Phase 5 | Hardening | ⚠️ Partial | 70% |
| Phase 6 | Release Qualification | ⏳ Pending | 0% |

---

## 3. Feature Requirements

### 3.1 Core Architecture (Phase 1)

#### FR-001: Core Entity Model
- **Description:** Stable ID system, ownership tree, lifecycle invariants
- **Status:** ✅ Implemented
- **Components:**
  - `Project` type — stable ID, root path, VCS/worktree tracking
  - `Session` type — stable ID, parent lineage, status machine
  - `Message` type — ordered history within session, append-only mutation
  - `Part` type — extensible content parts with versioned surface
- **Gap:** Project VCS worktree root distinction missing (P2)

#### FR-002: Storage Layer
- **Description:** Persistence and recovery for projects, sessions, messages
- **Status:** ✅ Implemented
- **Components:**
  - Project/session/message persistence
  - Session recovery after restart
  - Snapshot create/load
  - Revert to checkpoint
  - Compaction with preserved resumability/shareability

#### FR-003: Config System
- **Description:** Configuration precedence, normalization, ownership boundaries
- **Status:** ✅ Implemented
- **Components:**
  - JSON and JSONC parsing
  - Config precedence: remote → global → custom → project → `.opencode` → inline
  - Variable expansion: `{env:VAR}` and `{file:PATH}`
  - `tools` legacy alias normalization into `permission`
  - Config ownership boundary: main config does not own TUI settings
  - Permission rule type with glob pattern support
  - Auth/secret storage paths

#### FR-004: HTTP API Surface
- **Description:** Route groups, auth, request validation
- **Status:** ✅ Implemented
- **Components:**
  - Route registration by canonical resource group
  - Auth enforcement per endpoint
  - Request validation
  - Session/message lifecycle endpoints
  - Streaming endpoints (SSE/websocket)
  - API error shape consistency

### 3.2 Runtime Core (Phase 2)

#### FR-005: Agent System
- **Description:** Primary/subagent model, permission boundaries
- **Status:** ✅ Implemented
- **Components:**
  - Primary agent execution loop
  - Exactly one active primary agent per session invariant
  - Hidden vs visible agent behavior
  - Subagent execution — child context, result handoff
  - Permission inheritance from parent to subagent
  - Runtime restriction of subagent permissions
- **Agents Implemented:**
  - `BuildAgent` — Full tool access
  - `PlanAgent` — Read-only analysis
  - `CompactionAgent` — Hidden, context compression
  - `TitleAgent` — Hidden, title generation
  - `SummaryAgent` — Hidden, session summarization
  - `GeneralAgent` — Full tool access for subagent
  - `ExploreAgent` — Read-only code exploration
  - `ReviewAgent` — Code review
  - `RefactorAgent` — Refactoring
  - `DebugAgent` — Debugging

#### FR-006: Tools System
- **Description:** Registry, execution pipeline, permission gate
- **Status:** ⚠️ Partial
- **Gap:** Custom tool file-based loader incomplete (P0)
- **Components:**
  - Tool registry — registration, lookup, listing
  - Built-in tool interface — stable name/description/args schema
  - Execution pipeline with permission gate (`allow`/`ask`/`deny`)
  - Argument validation
  - MCP tool qualification
  - Deterministic collision resolution
  - Result caching for safe tools
- **Built-in Tools Implemented:**
  - `read`, `write`, `edit`, `bash`, `grep`, `glob`, `ls`
  - `task`, `skill`, `lsp`, `session_tools`, `codesearch`
  - `multiedit`, `webfetch`, `websearch`, `batch`

#### FR-007: Custom Tool File Loader
- **Description:** File-based discovery and loading of custom tools
- **Status:** ❌ Not Implemented (P0)
- **PRD Reference:** 03-tools-system.md
- **Requirements:**
  - Project-level: `.opencode/tools/` directory
  - Global-level: `~/.config/opencode/tools/` directory
  - File-based tool registration to registry
  - Tool definition format: TypeScript/JavaScript files
- **FR Number:** FR-007

#### FR-008: Plugin System
- **Description:** Server/runtime plugin hooks and loading
- **Status:** ✅ Implemented
- **Components:**
  - Plugin source loading from configured paths
  - Hooks: `on_init`, `on_start`, `on_tool_call`, `on_message`, `on_session_end`
  - Failure containment — plugin errors do not crash runtime
- **Gap:** Plugin-provided tool registration not implemented (P2)

#### FR-009: TUI Plugin API
- **Description:** Plugin surface for terminal UI
- **Status:** ⚠️ Partial
- **Gap:** TypeScript SDK not implemented (P0)
- **Requirements:**
  - `tui.json` plugin configuration ownership
  - Plugin identity — runtime ID resolution
  - Plugin deduplication before activation
  - `plugin_enabled` semantics
  - TUI plugin module interface
  - Runtime `api.plugins.activate()` / `api.plugins.deactivate()`
- **FR Number:** FR-009

### 3.3 Infrastructure Subsystems (Phase 3)

#### FR-010: MCP Integration
- **Description:** Local/remote MCP server integration
- **Status:** ✅ Implemented
- **Components:**
  - Local MCP server connection and initialization
  - Remote MCP server connection with URL
  - Per-server OAuth configuration
  - Tool discovery from MCP servers
  - Tool naming with server qualification
  - Permission gating for MCP tools
  - Timeout and unavailable-server handling
  - Context cost warnings

#### FR-011: LSP Integration
- **Description:** Language server protocol integration
- **Status:** ✅ Implemented
- **Components:**
  - Built-in LSP server detection by language/file type
  - Custom LSP server registration via config
  - Diagnostics retrieval and surfacing
  - LSP failure handling

#### FR-012: Provider/Model System
- **Description:** LLM provider abstraction and model selection
- **Status:** ✅ Implemented
- **Providers:**
  - OpenAI, Anthropic, Google, Azure, Bedrock
  - Ollama, LM Studio, Local models
  - 20+ additional providers

#### FR-013: Formatters
- **Description:** Code formatting pipeline
- **Status:** ✅ Implemented
- **Components:**
  - Formatter detection by file type
  - Project config-based formatter selection
  - Disable-all and per-formatter disable
  - Custom formatter command invocation
  - Formatter absence/error handling

#### FR-014: Skills System
- **Description:** Skill discovery and loading
- **Status:** ✅ Implemented
- **Components:**
  - SKILL.md format support
  - Discovery precedence: project → global → compat paths
  - Deterministic duplicate resolution within a scope
  - Skill loading into runtime context
- **Gap:** Permission restrictions for skill usage (P2)

### 3.4 Interface Implementations (Phase 4)

#### FR-015: Desktop/Web/ACP Interface
- **Description:** Desktop app, web interface, ACP
- **Status:** ❌ Not Implemented (P1)
- **Gap:** Desktop app startup flow, web server mode, ACP startup/handshake
- **FR Number:** FR-015

#### FR-016: GitHub Integration
- **Description:** GitHub App integration, workflow file generation
- **Status:** ⚠️ Partial
- **Gap:** Workflow file generation missing (P1)
- **Requirements:**
  - `opencode github install` command
  - GitHub App installation
  - Workflow file at `.github/workflows/opencode.yml`
  - Required secrets setup
- **FR Number:** FR-016

#### FR-017: GitLab Integration
- **Description:** GitLab CI/CD integration
- **Status:** ❌ Not Implemented (P1)
- **Gap:** GitLab CI component not implemented (P1)
- **Requirements:**
  - GitHub workflow trigger examples
  - Comment/PR trigger parsing
  - CI secret loading for GitHub Actions
  - GitLab CI component support (experimental)
- **FR Number:** FR-017

### 3.5 TUI Plugin TypeScript SDK

#### FR-018: TUI Plugin TypeScript SDK
- **Description:** TypeScript SDK for TUI plugin development
- **Status:** ❌ Not Implemented (P0)
- **PRD Reference:** 15-tui-plugin-api.md
- **Requirements:**
  - `@opencode-ai/plugin/tui` package
  - `TuiPlugin` type definition
  - `TuiPluginModule` type definition
  - API surface:
    - `commands.register()`
    - `routes.register()`
    - `dialogs.register()`
    - `slots.register()`
    - `themes.install()` / `themes.set()`
    - `events.on()`
    - `state.get()` / `state.set()`
    - `onDispose` lifecycle
- **FR Number:** FR-018

### 3.6 Iterations Infrastructure

#### FR-019: Iterations Structure
- **Description:** Implementation tracking structure for iterations
- **Status:** ❌ Not Implemented (P0)
- **Requirements:**
  - Create `iterations/src/` module structure
  - Align with `iterate-prd.sh` workflow
  - Track implementation progress per iteration
- **FR Number:** FR-019

---

## 4. Convention Tests

### 4.1 Architecture Boundary Tests
- **Location:** `tests/src/conventions/architecture_boundaries.rs`
- **Status:** ✅ Implemented (5 tests)

### 4.2 Config Ownership Tests
- **Location:** `tests/src/conventions/config_ownership.rs`
- **Status:** ✅ Implemented (4 tests)

### 4.3 Route/Resource Group Tests
- **Location:** `tests/src/conventions/route_conventions.rs`
- **Status:** ✅ Implemented (4 tests)

### 4.4 Test Placement Tests
- **Location:** `tests/src/conventions/test_layout.rs`
- **Status:** ✅ Implemented (5 tests)

### 4.5 TUI Convention Tests
- **Location:** `tests/src/conventions/tui_conventions.rs`
- **Status:** ✅ Implemented (5 tests)
- **Requires:** `ratatui-testing/`

---

## 5. Technical Debt

| ID | Item | Module | Severity | Remediation |
|----|------|--------|----------|-------------|
| TD-001 | Deprecated `mode` field | config | Medium | Remove in major version |
| TD-002 | Deprecated `tools` field | config | Medium | Remove after migration |
| TD-003 | Deprecated `keybinds` field | config | Low | Moved to tui.json |
| TD-004 | Deprecated `layout` field | config | Low | Always uses stretch |
| TD-005 | Hardcoded built-in skills | core | Medium | Consider externalization |
| TD-006 | Magic numbers in compaction | core | Low | Make configurable |
| TD-007 | SHA256 args hashing | storage | Low | Consider CAS |
| TD-008 | Custom JSONC parser | config | Medium | Use existing crate |
| TD-009 | `#[serde(other)]` in Part | core | Low | Explicit error handling |

---

## 6. P0/P1/P2 Issue Tracking

### P0 - Blocking Issues

| ID | Issue | Module | FR Reference | Target Phase |
|----|-------|--------|---------------|--------------|
| P0-1 | Custom tool file loader | tools | FR-007 | Phase 2 |
| P0-2 | TUI Plugin TypeScript SDK | tui, sdk | FR-018 | Phase 2 |
| P0-3 | Iterations structure | project | FR-019 | Phase 0 |

### P1 - Important Issues

| ID | Issue | Module | FR Reference | Target Phase |
|----|-------|--------|---------------|--------------|
| P1-1 | GitHub workflow generation | git | FR-016 | Phase 4 |
| P1-2 | GitLab CI component | git | FR-017 | Phase 4 |
| P1-3 | tui.json ownership enforcement | config, tui | FR-009 | Phase 2 |
| P1-4 | Desktop/web/ACP interface | server | FR-015 | Phase 4 |

### P2 - Improvement Issues

| ID | Issue | Module | PRD Reference | Target Phase |
|----|-------|--------|----------------|--------------|
| P2-1 | VCS worktree root distinction | core | 01-core-arch | Phase 1 |
| P2-2 | AGENTS.md upward scanning | core | 06-config | Phase 1 |
| P2-3 | MCP OAuth CLI commands | cli | 04-mcp | Phase 3 |
| P2-4 | Session compaction boundaries | core | 01-core-arch | Phase 1 |
| P2-5 | Plugin-provided tool registration | plugin | 08-plugin | Phase 2 |
| P2-6 | Skill permission restrictions | core | 12-skills | Phase 3 |

---

## 7. Release Gates

| Gate | Criteria | Status |
|------|----------|--------|
| Phase 0 | Workspace builds, tests run, clippy clean | ✅ |
| Phase 1 | Authority tests green | ✅ |
| Phase 2 | Runtime tests green | ✅ |
| Phase 3 | Subsystem tests green | ✅ |
| Phase 4 | Interface smoke workflows pass | ⏳ |
| Phase 5a | Compatibility suite green | ⏳ |
| Phase 5b | Conventions suite green | ✅ |
| Phase 6 | Non-functional baselines recorded | ⏳ |

---

## 8. Crate Ownership Summary

| Crate | Phase | PRD | Status |
|-------|-------|-----|--------|
| `crates/core/` | 1 | `01`, `06` | ✅ |
| `crates/storage/` | 1 | `01` | ✅ |
| `crates/config/` | 1 | `06` | ✅ |
| `crates/server/` | 1, 4 | `07`, `13` | ⚠️ Partial |
| `crates/agent/` | 2 | `02` | ✅ |
| `crates/tools/` | 2, 3 | `03`, `11` | ⚠️ Partial |
| `crates/plugin/` | 2 | `08` | ✅ |
| `crates/tui/` | 2, 3 | `09`, `15` | ⚠️ Partial |
| `crates/mcp/` | 3 | `04` | ✅ |
| `crates/lsp/` | 3 | `05` | ✅ |
| `crates/llm/` | 3 | `10` | ✅ |
| `crates/git/` | 4 | `14` | ⚠️ Partial |
| `crates/sdk/` | 2 | `15` | ❌ |
| `ratatui-testing/` | 2, 3 | `09`, `15` | ✅ |

---

## 9. Cross-References

- [01-core-architecture.md](../01-core-architecture.md)
- [02-agent-system.md](../02-agent-system.md)
- [03-tools-system.md](../03-tools-system.md)
- [04-mcp-system.md](../04-mcp-system.md)
- [05-lsp-system.md](../05-lsp-system.md)
- [06-configuration-system.md](../06-configuration-system.md)
- [07-server-api.md](../07-server-api.md)
- [08-plugin-system.md](../08-plugin-system.md)
- [09-tui-system.md](../09-tui-system.md)
- [10-provider-model-system.md](../10-provider-model-system.md)
- [11-formatters.md](../11-formatters.md)
- [12-skills-system.md](../12-skills-system.md)
- [13-desktop-web-interface.md](../13-desktop-web-interface.md)
- [14-github-gitlab-integration.md](../14-github-gitlab-integration.md)
- [15-tui-plugin-api.md](../15-tui-plugin-api.md)
- [16-test-plan.md](../16-test-plan.md)
- [17-rust-test-implementation-roadmap.md](../17-rust-test-implementation-roadmap.md)
- [18-crate-by-crate-test-backlog.md](../18-crate-by-crate-test-backlog.md)
- [Gap Analysis Report](./gap_analysis.md)

---

## 10. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-04-09 | Initial version based on PRD and gap analysis |
