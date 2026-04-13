# OpenCode Rust Port — Specification Document v15

**Version:** 15.0
**Generated:** 2026-04-13
**Based on:** PRD specifications and gap analysis (Iteration 15)
**Status:** Draft

---

## 1. Overview

This document defines the specification for the OpenCode Rust port implementation. It is derived from the PRD specifications (`01`–`19`) and updated based on gap analysis between planned features and current implementation status.

**Overall Completion Estimate: ~65-70%**
**Phase Status:** Phase 1-2 of 6 (Authority Implementation, Runtime Core)

---

## 2. Implementation Status Summary

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Complete | ~90% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ⚠️ Partial | ~70% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Mostly Complete | ~85% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ❌ Not Started | ~20% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Mostly Complete | ~80% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 3. Feature Requirements

### 3.1 Core Architecture (Phase 1)

#### FR-001: Core Entity Model
- **Description:** Stable ID system, ownership tree, lifecycle invariants
- **Status:** ✅ Implemented
- **Components:**
  - `Project` type — stable ID, root path, VCS/worktree tracking ✅
  - `Session` type — stable ID, parent lineage, status machine ✅
  - `Message` type — ordered history within session, append-only mutation ✅
  - `Part` type — extensible content parts with versioned surface ✅
- **Gaps:**
  - No explicit unit tests verifying ownership tree acyclicity
  - No integration tests for complete session lifecycle: create→fork→share→compact→revert

#### FR-002: Storage Layer
- **Description:** Persistence and recovery for projects, sessions, messages
- **Status:** ✅ Implemented
- **Components:**
  - Project/session/message persistence ✅
  - Session recovery after restart ✅
  - Snapshot create/load ✅
  - Revert to checkpoint ✅
  - Compaction with preserved resumability/shareability ✅

### 3.2 Configuration System (Phase 1)

#### FR-003: Config System
- **Description:** Configuration precedence, normalization, ownership boundaries
- **Status:** ⚠️ Partial (P2 gap: config crate is empty re-export)
- **Components:**
  - JSON and JSONC parsing ✅
  - Config precedence: remote → global → custom → project → `.opencode` → inline ✅
  - Variable expansion: `{env:VAR}` and `{file:PATH}` ✅
  - `tools` legacy alias normalization into `permission` ✅
  - Permission rule type with glob pattern support ✅
  - Auth/secret storage paths ✅
  - Config ownership boundary: `opencode.json` vs `tui.json` split ✅
- **Gap:**
  - `crates/config/src/lib.rs` is nearly empty (just re-exports core). Per PRD 19, config should be in dedicated `crates/config/` crate

### 3.3 Server API (Phase 1)

#### FR-004: HTTP API Surface
- **Description:** Route groups, auth, request validation
- **Status:** ⚠️ Partial
- **Components:**
  - Route registration by canonical resource group ✅
  - Auth enforcement per endpoint ⚠️ (middleware exists, needs verification)
  - Request validation ✅
  - Session/message lifecycle endpoints (CRUD) ✅
  - Streaming endpoints (SSE, WebSocket) ✅
  - API error shape consistency ✅
- **Gaps:**
  - No explicit tests verifying route-group presence
  - No negative tests for unauthorized/malformed requests

### 3.4 Agent System (Phase 2)

#### FR-005: Agent System
- **Description:** Primary/subagent model, permission boundaries
- **Status:** ⚠️ Partial
- **Components:**
  - Primary agent execution loop ✅
  - Exactly one active primary agent per session invariant ⚠️ (unverified)
  - Hidden vs visible agent behavior ⚠️ (UI integration unclear)
  - Subagent execution — child context, result handoff ✅
  - Permission inheritance from parent to subagent ✅
  - Runtime restriction of subagent permissions ✅
- **Agents Implemented:**
  - `build` — Full tool access (default, visible)
  - `plan` — Read-only analysis (visible)
  - `compaction` — Hidden, context compression
  - `title` — Hidden, title generation
  - `summary` — Hidden, session summarization
  - `general` — Full tool access for subagent (write, edit, bash, read, grep, glob, list)
  - `explore` — Read-only code exploration (read, grep, glob, list)
- **Gaps:**
  - No tests for hidden vs visible agent behavior in selection flows
  - No tests verifying exactly-one-active-primary-agent invariant

### 3.5 Tools System (Phase 2-3) ❌ CRITICAL GAPS

#### FR-006: Tools System
- **Description:** Registry, execution pipeline, permission gate
- **Status:** ✅ Mostly Implemented
- **Components:**
  - Tool registry — registration, lookup, listing ✅
  - Built-in tool interface — stable name/description/args schema ✅
  - Execution pipeline with permission gate (`allow`/`ask`/`deny`) ✅
  - Argument validation ✅
  - MCP tool qualification with server prefix ✅
  - Deterministic collision resolution ✅
  - Result caching for safe tools ✅
- **Built-in Tools Implemented:**
  - `read`, `write`, `edit`, `bash`, `grep`, `glob`, `ls`
  - `task`, `skill`, `lsp`, `session_tools`, `codesearch`
  - `multiedit`, `webfetch`, `websearch`, `batch`

#### FR-007: Custom Tool File Loader
- **Description:** File-based discovery and loading of custom tools
- **Status:** ❌ **BROKEN** (P0 Gap)
- **PRD Reference:** 03-tools-system.md
- **Components:**
  - Project-level: `.opencode/tools/` directory ✅
  - Global-level: `~/.config/opencode/tools/` directory ✅
- **Critical Gap - Discovery Format Mismatch:**
  - **PRD requires:** TypeScript/JavaScript files with `export default tool({...})`
  - **Implementation:** Scans `TOOL.md` files
  - **Location:** `crates/core/src/config/directory_scanner.rs:228`
- **Critical Gap - Custom Tools Not Registered:**
  - Discovered tools recorded in config but NOT registered with ToolRegistry
  - `DirectoryScanner` records tools in `tools_info` but `ToolRegistry` only has `register()` for programmatic registration

### 3.6 Plugin System (Phase 2) ❌ CRITICAL GAPS

#### FR-008: Plugin System
- **Description:** Server/runtime plugin hooks and loading
- **Status:** ⚠️ Partial (P0 gaps)
- **PRD Reference:** 08-plugin-system.md
- **Components:**
  - Plugin source loading from configured paths (npm, local) ✅
  - npm plugin configuration via `opencode.json` `plugin` key ✅
  - Local plugin structure in `.opencode/plugins/` ✅
  - Plugin context: project, directory, worktree, client, shell API ✅
  - **Event Hooks ✅ (All implemented):**
    - Command Events: `command.executed` ✅
    - File Events: `file.edited`, `file.watcher.updated` ✅
    - Installation Events: `installation.updated` ✅
    - LSP Events: `lsp.client.diagnostics`, `lsp.updated` ✅
    - Message Events: `message.part.removed`, `message.part.updated`, `message.removed`, `message.updated` ✅
    - Permission Events: `permission.asked`, `permission.replied` ✅
    - Server Events: `server.connected` ✅
    - Session Events: `session.created`, `session.compacted`, `session.deleted`, `session.diff`, `session.error`, `session.idle`, `session.status`, `session.updated` ✅
    - Todo Events: `todo.updated` ✅
    - Shell Events: `shell.env` ✅
    - Tool Events: `tool.execute.after`, `tool.execute.before` ✅
    - Experimental: `experimental.session.compacting` ✅
  - Plugin dependency management via `.opencode/package.json` ✅
  - Plugin logging API via `client.app.log()` ✅
  - Failure containment — plugin errors do not crash runtime ✅
  - Plugin cleanup/unload ✅
- **P0 Critical Gaps:**

1. **Non-deterministic Hook Execution Order**
   - Location: `crates/plugin/src/lib.rs:358-369` (on_tool_call_all iterates `self.plugins`)
   - Uses `HashMap` iteration order (non-deterministic)
   - `IndexMap` insertion order is preserved BUT plugin registration order depends on discovery order
   - **PRD requires:** Deterministic execution order

2. **Plugin Tool Registration Missing**
   - `PluginToolAdapter` exists and implements `Tool` trait
   - But no mechanism to register plugin tools with `ToolRegistry`
   - `Plugin::register_tool()` method does not exist
   - **Impact:** Plugin system cannot extend agent capabilities

3. **Plugin Config Ownership Not Enforced**
   - Server/runtime and TUI plugin configs can be mixed
   - **PRD requires:** Config ownership boundary enforcement

### 3.7 TUI Plugin API (Phase 2)

#### FR-009: TUI Plugin API
- **Description:** Plugin surface for terminal UI
- **Status:** ✅ Mostly Implemented
- **PRD Reference:** 15-tui-plugin-api.md
- **Components:**
  - `tui.json` plugin configuration ownership ✅
  - Plugin identity — runtime ID resolution ✅
  - Plugin deduplication before activation ✅
  - `plugin_enabled` semantics ✅
  - TUI plugin module interface ✅
  - Runtime `api.plugins.activate()` / `api.plugins.deactivate()` ✅
  - Commands registration ✅
  - Routes registration ✅
  - **Dialogs ✅ (All 4 dialogs implemented)**
    - `DialogAlert` ✅
    - `DialogConfirm` ✅
    - `DialogPrompt` ✅
    - `DialogSelect` ✅
  - **Slots system ✅ (Full implementation)**
  - Theme API ✅
  - Events ✅
  - State API ✅
  - `onDispose` lifecycle ✅
- **Gap:**
  - Theme auto-sync on install not explicitly tested

### 3.8 MCP System (Phase 3)

#### FR-010: MCP Integration
- **Description:** Local/remote MCP server integration
- **Status:** ✅ Implemented
- **PRD Reference:** 04-mcp-system.md
- **Components:**
  - Local MCP server connection (stdio transport + JSON-RPC) ✅
  - Remote MCP server connection (HTTP+SSE) ✅
  - Per-server OAuth configuration ✅
  - Automatic OAuth flow (401 detection, RFC 7591 dynamic client registration) ✅
  - Tool discovery from MCP servers ✅
  - Tool naming with server qualification (`<servername>_<toolname>`) ✅
  - Permission gating for MCP tools ✅
  - Timeout configuration per server ✅
  - Unavailable-server handling ✅
  - Context cost warnings ✅
  - Built-in server examples: Sentry, Context7, Vercel Grep ✅

### 3.9 LSP System (Phase 3)

#### FR-011: LSP Integration
- **Description:** Language server protocol integration
- **Status:** ✅ Implemented
- **PRD Reference:** 05-lsp-system.md
- **Components:**
  - Built-in LSP server detection by language/file extension ✅
  - Custom LSP server registration via config ✅
  - Diagnostics retrieval and surfacing ✅
  - LSP failure handling (graceful degradation) ✅
  - Experimental LSP tool (`goToDefinition`, `findReferences`) ✅

### 3.10 Provider/Model System (Phase 3)

#### FR-012: Provider/Model System
- **Description:** LLM provider abstraction and model selection
- **Status:** ✅ Mostly Implemented
- **Components:**
  - Provider abstraction ✅
  - Default model selection ✅
  - Per-agent model override ⚠️ (unverified)
  - Local model providers (Ollama, LM Studio) ✅
  - Variant/reasoning budget support ✅
- **Providers Implemented:**
  - OpenAI, Anthropic, Google, Azure, Bedrock ✅
  - Ollama, LM Studio, Local models ✅
- **Gap:**
  - Per-agent model override not explicitly tested

### 3.11 Formatters (Phase 3)

#### FR-013: Formatters
- **Description:** Code formatting pipeline
- **Status:** ✅ Implemented
- **Components:**
  - Formatter detection by file type ✅
  - Project config-based formatter selection ✅
  - Disable-all and per-formatter disable ✅
  - Custom formatter command invocation ✅
  - Formatter absence/error handling ✅

### 3.12 Skills System (Phase 3)

#### FR-014: Skills System
- **Description:** Skill discovery and loading
- **Status:** ✅ Implemented
- **Components:**
  - SKILL.md format support ✅
  - Discovery precedence: project → global → compat paths ✅
  - Deterministic duplicate resolution within a scope ✅
  - Skill loading into runtime context ✅
  - Permission restrictions for skill usage ⚠️ (uses tool permission system)

### 3.13 Desktop/Web/ACP Interface (Phase 4) ❌ NOT STARTED

#### FR-015: Desktop/Web/ACP Interface
- **Description:** Desktop app, web interface, ACP
- **Status:** ❌ **NOT STARTED**
- **PRD Reference:** 13-desktop-web-interface.md
- **Components:**
  - Desktop app shell (WebView integration) ❌ (stubs only)
  - Web server mode ❌ (stub in `crates/cli/src/cmd/web.rs`)
  - Auth protection ❌ (no implementation)
  - Session sharing between interfaces ❌ (no mechanism)
  - ACP startup/handshake ⚠️ (event structs exist, no transport)
  - Sharing behavior in managed/restricted deployments ❌ (no implementation)
- **Gap:**
  - Major interface features missing - only stubs exist

### 3.14 GitHub/GitLab Integration (Phase 4)

#### FR-016: GitHub Integration
- **Description:** GitHub App integration, workflow triggers, comment parsing
- **Status:** ⚠️ Partial
- **PRD Reference:** 14-github-gitlab-integration.md
- **Components:**
  - GitHub workflow trigger parsing (`issue_comment`, `pull_request_review`) ✅
  - Comment/PR trigger parsing (`/oc` or `/opencode` command) ✅
  - CI secret loading for GitHub Actions ✅

#### FR-017: GitLab Integration
- **Description:** GitLab CI/CD integration
- **Status:** ⚠️ Partial
- **PRD Reference:** 14-github-gitlab-integration.md
- **Components:**
  - GitLab CI component ✅
  - GitLab Duo support ⚠️ (marked experimental)

### 3.15 TUI System (Phase 2-3)

#### FR-018: TUI Core System
- **Description:** Terminal user interface components
- **Status:** ⚠️ Partial (needs tests)
- **PRD Reference:** 09-tui-system.md
- **Components:**
  - Session view rendering (markdown, syntax highlighting, diff) ✅
  - Slash commands (`/compact`, `/connect`, `/help`) ✅
  - Multiline input ✅
  - File references (`@`) fuzzy search ✅
  - Shell prefix (`!`) execution ✅
  - Keybinding system (leader key, categories) ✅
  - Sidebar (file tree, MCP/LSP status, diagnostics) ✅
  - Home view ✅
- **Gaps:**
  - No automated tests for slash command execution
  - No tests for input model (multiline, history, autocomplete)
  - No tests for sidebar visibility and content

### 3.16 Test Infrastructure (Phase 5-6)

#### FR-019: Authority Document Tests
- **Description:** Tests for core authority documents (01, 06, 07)
- **Status:** ⚠️ Partial
- **PRD Reference:** 16-test-plan.md
- **Components:**
  - Core ownership tree tests (unit + integration) ⚠️ (acyclicity not tested)
  - Config precedence merge tests ✅
  - API route-group tests ❌ (missing)
  - Session/message lifecycle tests ⚠️ (incomplete)
- **Gap:**
  - Missing create→fork→share→compact→revert integration test

#### FR-020: Runtime Architecture Tests
- **Description:** Tests for runtime systems (02, 03, 08, 15)
- **Status:** ⚠️ Partial
- **Components:**
  - Agent primary invariant tests (exactly one active) ❌ (missing)
  - Subagent execution tests ✅
  - Tool registry tests ✅
  - Plugin hook order tests ❌ (non-deterministic order)
  - TUI plugin lifecycle tests ⚠️ (partial)

#### FR-021: Subsystem Tests
- **Description:** Tests for infrastructure subsystems (04, 05, 10, 11, 12)
- **Status:** ✅ Implemented
- **Components:**
  - MCP integration tests ✅
  - LSP integration tests ✅
  - Provider/model tests ⚠️ (per-agent override untested)
  - Skills discovery tests ✅

#### FR-022: Interface Tests
- **Description:** Tests for desktop/web/ACP/git interfaces (13, 14)
- **Status:** ❌ Not Implemented
- **Components:**
  - Desktop/web smoke tests ❌ (no implementation)
  - ACP handshake tests ❌ (no transport)
  - GitHub workflow tests ✅
  - GitLab integration tests ✅

### 3.17 Convention Tests

#### FR-023: Convention Tests
- **Description:** Architecture, config, route, layout, and TUI conventions
- **Status:** ✅ Implemented
- **Components:**
  - Architecture Boundary Tests ✅
  - Config Ownership Tests ✅
  - Route Conventions Tests ✅
  - Test Layout Tests ✅
  - TUI Convention Tests ✅

---

## 4. P0/P1/P2 Issue Tracking

### P0 - Blocking Issues (Must Fix Before Release)

| ID | Issue | Module | FR Reference | Gap |
|----|-------|--------|---------------|-----|
| **P0-1** | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | FR-007 | Discovery format mismatch - custom tools non-functional |
| **P0-2** | Custom tools not registered with ToolRegistry | tools | FR-007 | Discovered tools not registered - custom tools non-functional |
| **P0-3** | Plugin tool registration missing | plugin | FR-008 | Plugin cannot add tools to agent toolset |

### P1 - High Priority Issues

| ID | Issue | Module | FR Reference | Gap |
|----|-------|--------|---------------|-----|
| P1-1 | Non-deterministic hook execution order | plugin | FR-008 | Uses HashMap iteration - unpredictable plugin behavior |
| P1-2 | Plugin config ownership not enforced | plugin | FR-008 | Config boundary violation |
| P1-3 | Exactly-one-active-primary-agent invariant untested | agent | FR-005 | No invariant verification |
| P1-4 | Ownership tree acyclicity not tested | core | FR-001 | Core data integrity not guaranteed |
| P1-5 | Session lifecycle integration tests incomplete | storage | FR-002 | Missing create→fork→share→compact→revert |
| P1-6 | Desktop app not implemented | cli | FR-015 | Major feature missing |
| P1-7 | Web server mode incomplete | cli | FR-015 | Major feature missing |
| P1-8 | ACP transport not implemented | control-plane | FR-015 | Editor integration incomplete |
| P1-9 | Config crate is empty re-export | config | FR-003 | Violates PRD 19 crate ownership |

### P2 - Medium Priority Issues

| ID | Issue | Module | FR Reference | Gap |
|----|-------|--------|---------------|-----|
| P2-1 | TUI slash command tests missing | tui | FR-018 | UI regression risk |
| P2-2 | TUI input model tests missing | tui | FR-018 | UI regression risk |
| P2-3 | TUI sidebar tests missing | tui | FR-018 | UI regression risk |
| P2-4 | Per-agent model override untested | llm | FR-012 | Uncertain correctness |
| P2-5 | Route-group presence tests missing | server | FR-004 | Regression risk |
| P2-6 | API negative tests (auth, malformed) missing | server | FR-004 | Security gaps |
| P2-7 | Hidden vs visible agent UI behavior untested | agent | FR-005 | UI integration unclear |
| P2-8 | Theme auto-sync on install not tested | tui | FR-009 | Feature gap |

---

## 5. Technical Debt

| ID | Item | Module | Severity | Remediation | Status |
|----|------|--------|----------|-------------|--------|
| TD-001 | Empty `crates/config/` crate | config | Medium | Move config logic to dedicated crate | ❌ Not Fixed |
| TD-002 | `DirectoryScanner` discovery mismatch | tools | **CRITICAL** | Implement TypeScript/JavaScript discovery | ❌ Not Fixed |
| TD-003 | Custom tools discovered but not registered | tools | **CRITICAL** | Add registration flow to ToolRegistry | ❌ Not Fixed |
| TD-004 | Non-deterministic plugin hook execution | plugin | High | Use explicit priority ordering | ❌ Not Fixed |
| TD-005 | Plugin `register_tool()` method missing | plugin | **CRITICAL** | Add method to Plugin trait | ❌ Not Fixed |
| TD-006 | ACP transport layer missing | control-plane | High | Implement ACP handshake and transport | ❌ Not Fixed |
| TD-007 | Deprecated `mode` field | config | Medium | Remove in v4.0 | Deferred |
| TD-008 | Deprecated `tools` field | config | Medium | Remove after migration | Deferred |
| TD-009 | Deprecated `theme` field | config | Low | Moved to tui.json | Deferred |
| TD-010 | Deprecated `keybinds` field | config | Low | Moved to tui.json | Deferred |

---

## 6. Crate-Level Implementation Status

| Crate | Phase | PRD | Status | Notes |
|-------|-------|-----|--------|-------|
| `crates/core/` | 1 | `01`, `06` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | 1 | `01` | ✅ Done | Persistence, recovery, snapshots |
| `crates/permission/` | 1 | `02` | ✅ Done | Permission system |
| `crates/server/` | 1, 4 | `07`, `13` | ✅ Done | API routes, auth, streaming |
| `crates/agent/` | 2 | `02` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | 2, 3 | `03`, `11` | ⚠️ Partial | Registry done, custom tool discovery **broken** |
| `crates/plugin/` | 2 | `08` | ⚠️ Partial | Hooks done, tool registration **missing** |
| `crates/tui/` | 2, 3 | `09`, `15` | ✅ Done | Full implementation, needs tests |
| `crates/mcp/` | 3 | `04` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | 3 | `05` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | 3 | `10` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | 4 | `14` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | 1 | `06` | ❌ Broken | Empty re-export, not real crate |
| `crates/cli/` | 4 | `13` | ⚠️ Partial | Desktop/web stubs exist, not implemented |
| `crates/control-plane/` | 4 | `13` | ⚠️ Partial | ACP event structs exist, no transport |
| `ratatui-testing/` | 2, 3 | `09`, `15` | ✅ Done | TUI testing framework |

---

## 7. Recommendations

### Immediate Actions (P0 Fixes)

1. **Fix Custom Tool Discovery (P0-1)**
   - Implement TypeScript/JavaScript file discovery in `DirectoryScanner`
   - Parse and execute tool definitions using dynamic import
   - Register discovered tools with `ToolRegistry`

2. **Implement Plugin Tool Registration (P0-3)**
   - Add `register_tool()` method to `Plugin` trait
   - Integrate `PluginManager` with `ToolRegistry`
   - Add tests verifying plugin tools appear in registry

3. **Fix Hook Execution Determinism (P1-1)**
   - Add explicit `priority` field to plugins or hooks
   - Execute hooks in priority order
   - Document execution order guarantees

### Short-term Actions (P1)

4. **Complete Desktop/Web/ACP (P1-6, P1-7, P1-8)**
   - Implement desktop app shell with WebView
   - Implement web server mode with proper auth
   - Implement ACP transport layer

5. **Add Critical Invariant Tests (P1-3, P1-4, P1-5)**
   - Test exactly-one-active-primary-agent
   - Test ownership tree acyclicity
   - Test session lifecycle (create→fork→share→compact→revert)

### Medium-term Actions (P2)

6. **Complete Test Coverage (P2-1 through P2-8)**
   - Add tests for TUI components
   - Add tests for API route groups
   - Add negative tests for permissions and auth

7. **Refactor Config Crate (P1-9)**
   - Move config logic from `core` to dedicated `config` crate
   - Align with PRD 19 crate ownership intentions

---

## 8. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 15.0 | 2026-04-13 | **Major revision based on Iteration 15 gap analysis.** Overall completion revised from ~92-94% to ~65-70%. Identified 3 critical P0 gaps: custom tool discovery format mismatch (scans TOOL.md instead of .ts/.js), custom tools not registered with ToolRegistry, and plugin tool registration missing. Added P0-1 through P0-3 blocking issues. Revised P1/P2 issue tables significantly. Added TD-001 through TD-010 technical debt items. Updated Phase 4 status to "Not Started" (~20% completion). All previously "resolved" P0/P1 items reverted to open status. |
| 11.0 | 2026-04-13 | Previous version claimed ~92-94% completion (over-optimistic) |
| 1.0-10.0 | Various | Previous iterations |

---

*Document generated: 2026-04-13*
*Iteration: 15*
*Phase: Phase 1-2 of 6 (Authority Implementation, Runtime Core)*
*Priority: Fix P0 gaps in custom tool and plugin tool registration before proceeding with interface implementations*
