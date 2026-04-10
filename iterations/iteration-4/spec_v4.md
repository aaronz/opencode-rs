# OpenCode Rust Port — Specification Document v4

**Version:** 4.0
**Generated:** 2026-04-10
**Based on:** PRD specifications and gap analysis (Iteration 4)
**Status:** Draft

---

## 1. Overview

This document defines the specification for the OpenCode Rust port implementation. It is derived from the PRD specifications (`01`–`15`) and updated based on gap analysis between planned features and current implementation status.

**Overall Completion Estimate: ~35-40%**
**Phase Status:** Phase 1-2 of 6

---

## 2. Implementation Status Summary

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | 🚧 In Progress | ~60% |
| Phase 2 | Runtime Core | 🚧 In Progress | ~50% |
| Phase 3 | Infrastructure Subsystems | 🚧 In Progress | ~40% |
| Phase 4 | Interface Implementations | ❌ Not Started | 0% |
| Phase 5 | Hardening | 🚧 Partial | ~30% |
| Phase 6 | Release Qualification | ❌ Not Started | 0% |

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
- **Test Coverage Required:** Unit tests for ownership tree acyclicity, serialization roundtrips

#### FR-002: Storage Layer
- **Description:** Persistence and recovery for projects, sessions, messages
- **Status:** ⚠️ Partial
- **Components:**
  - Project/session/message persistence
  - Session recovery after restart
  - Snapshot create/load
  - Revert to checkpoint
  - Compaction with preserved resumability/shareability
- **Gap:** Snapshot/Revert model not fully implemented and tested (P1)

#### FR-003: Config System
- **Description:** Configuration precedence, normalization, ownership boundaries
- **Status:** ⚠️ Partial (P0 blockers remain)
- **Components:**
  - JSON and JSONC parsing
  - Config precedence: remote → global → custom → project → `.opencode` → inline
  - Variable expansion: `{env:VAR}` and `{file:PATH}`
  - `tools` legacy alias normalization into `permission`
  - Config ownership boundary: `opencode.json` vs `tui.json` split NOT enforced (P0)
  - Permission rule type with glob pattern support
  - Auth/secret storage paths
- **P0 Blockers:**
  - Config precedence not fully enforced
  - Config ownership boundary not enforced

#### FR-004: HTTP API Surface
- **Description:** Route groups, auth, request validation
- **Status:** ⚠️ Partial (P0 blockers remain)
- **Components:**
  - Route registration by canonical resource group (NOT fully implemented - P0)
  - Auth enforcement per endpoint (NOT fully implemented - P0)
  - Request validation
  - Session/message lifecycle endpoints (CRUD incomplete - P0)
  - Streaming endpoints (SSE/websocket - partial)
  - API error shape consistency
- **P0 Blockers:**
  - Route registration by resource group incomplete
  - Auth enforcement incomplete
  - Session/message lifecycle CRUD incomplete

### 3.2 Runtime Core (Phase 2)

#### FR-005: Agent System
- **Description:** Primary/subagent model, permission boundaries
- **Status:** ⚠️ Partial (P0 blockers remain)
- **Components:**
  - Primary agent execution loop ✅
  - Exactly one active primary agent per session invariant ❌ (P0 - NOT enforced)
  - Hidden vs visible agent behavior ❌ (P1 - not implemented)
  - Subagent execution — child context, result handoff ❌ (P0 - not implemented)
  - Permission inheritance from parent to subagent ❌ (P1 - not implemented)
  - Runtime restriction of subagent permissions ✅
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
- **P0 Blockers:**
  - Exactly one active primary agent invariant not enforced
  - Subagent execution not implemented
  - Task/delegation mechanism not implemented

#### FR-006: Tools System
- **Description:** Registry, execution pipeline, permission gate
- **Status:** ⚠️ Partial (P0 blockers remain)
- **Components:**
  - Tool registry — registration, lookup, listing
  - Built-in tool interface — stable name/description/args schema ✅
  - Execution pipeline with permission gate (`allow`/`ask`/`deny`) ✅
  - Argument validation ✅
  - MCP tool qualification ❌ (P1 - not implemented)
  - Deterministic collision resolution ❌ (P2)
  - Result caching for safe tools ❌ (P2)
- **Built-in Tools Implemented:**
  - `read`, `write`, `edit`, `bash`, `grep`, `glob`, `ls`
  - `task`, `skill`, `lsp`, `session_tools`, `codesearch`
  - `multiedit`, `webfetch`, `websearch`, `batch`
- **P0 Blockers:**
  - Custom tool discovery format mismatch (scanning TOOL.md instead of .ts/.js)
  - Custom tool registration disconnected from ToolRegistry

#### FR-007: Custom Tool File Loader
- **Description:** File-based discovery and loading of custom tools
- **Status:** ❌ Not Implemented (P0)
- **PRD Reference:** 03-tools-system.md
- **Requirements:**
  - Project-level: `.opencode/tools/` directory
  - Global-level: `~/.config/opencode/tools/` directory
  - File-based tool registration to registry
  - Tool definition format: TypeScript/JavaScript files (NOT .md)
- **Current Gap:** Scanner looks for `TOOL.md` but PRD specifies `.ts/.js`

#### FR-008: Plugin System
- **Description:** Server/runtime plugin hooks and loading
- **Status:** ⚠️ Partial (P0 blockers remain)
- **Components:**
  - Plugin source loading from configured paths ✅
  - Hooks: `on_init`, `on_start`, `on_tool_call`, `on_message`, `on_session_end` ✅
  - Failure containment — plugin errors do not crash runtime ✅
  - Plugin-provided tool registration ❌ (P0 - no `register_tool()` method)
  - Plugin cleanup/unload ❌ (P2)
- **P0 Blockers:**
  - Hook execution order non-deterministic (HashMap iteration - must use IndexMap)
  - Plugin-provided tool registration not implemented
  - Plugin config not separated from TUI plugin config

#### FR-009: TUI Plugin API
- **Description:** Plugin surface for terminal UI
- **Status:** ❌ Not Fully Implemented (P0)
- **PRD Reference:** 15-tui-plugin-api.md
- **Components:**
  - `tui.json` plugin configuration ownership ❌ (P0 - not isolated)
  - Plugin identity — runtime ID resolution ❌ (P0)
  - Plugin deduplication before activation ❌ (P0)
  - `plugin_enabled` semantics ❌ (P0 - not implemented)
  - TUI plugin module interface ⚠️ (partial)
  - Runtime `api.plugins.activate()` / `api.plugins.deactivate()` ❌ (P0)
  - Commands registration ⚠️ (partial - P1)
  - Routes registration ⚠️ (partial - P1)
  - Dialogs ❌ (P2)
  - Slots ❌ (P2)
  - Theme API ⚠️ (partial - P1)
  - Events ⚠️ (partial - P1)
  - State API ⚠️ (partial - P1)
  - `onDispose` lifecycle ⚠️ (partial - P1)
- **P0 Blockers:**
  - `tui.json` ownership not enforced
  - Plugin identity/deduplication not implemented
  - `plugin_enabled` semantics not implemented
  - Runtime activate/deactivate not implemented

### 3.3 Infrastructure Subsystems (Phase 3)

#### FR-010: MCP Integration
- **Description:** Local/remote MCP server integration
- **Status:** ❌ Not Started (P0 blockers)
- **PRD Reference:** 04-mcp-system.md
- **Components:**
  - Local MCP server connection (stdio transport + JSON-RPC) ❌ (P0)
  - Remote MCP server connection (HTTP+SSE) ❌ (P0)
  - Per-server OAuth configuration ❌ (P1)
  - Tool discovery from MCP servers ❌ (P0)
  - Tool naming with server qualification (`<servername>_<toolname>`) ❌ (P1)
  - Permission gating for MCP tools ⚠️ (depends on tool registry fix)
  - Timeout and unavailable-server handling ❌ (P1)
  - Context cost warnings ❌ (P2)
- **Current State:** Only struct definitions exist, no actual transport implementation
- **P0 Blockers:**
  - Local MCP server connection not implemented
  - Remote MCP server connection not implemented
  - Tool discovery from MCP servers not implemented

#### FR-011: LSP Integration
- **Description:** Language server protocol integration
- **Status:** ❌ Not Started (P0 blockers)
- **PRD Reference:** 05-lsp-system.md
- **Components:**
  - Built-in LSP server detection by language/file extension ❌ (P0)
  - Custom LSP server registration via config ❌ (P1)
  - Diagnostics retrieval and surfacing ❌ (P0)
  - LSP failure handling ❌ (P1)
  - Experimental LSP tool (`goToDefinition`, `findReferences`, etc.) ❌ (P2)
- **Current State:** Only stubs exist
- **P0 Blockers:**
  - Built-in LSP server detection not implemented
  - Diagnostics retrieval not implemented

#### FR-012: Provider/Model System
- **Description:** LLM provider abstraction and model selection
- **Status:** ⚠️ Partial
- **Components:**
  - Provider abstraction ⚠️ (partial - P1)
  - Default model selection ⚠️ (precedence not fully implemented - P1)
  - Per-agent model override ❌ (P1)
  - Local model providers (Ollama, LM Studio) ⚠️ (partial - P1)
  - Variant/reasoning budget ❌ (P2)
- **Providers Implemented:**
  - OpenAI, Anthropic, Google, Azure, Bedrock (basic)
  - Ollama, LM Studio, Local models (partial)

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
- **Status:** ✅ Implemented (with P1/P2 gaps)
- **Components:**
  - SKILL.md format support ✅
  - Discovery precedence: project → global → compat paths ⚠️ (compat paths missing - P1)
  - Deterministic duplicate resolution within a scope ✅
  - Skill loading into runtime context ✅
  - Permission restrictions for skill usage ❌ (P2)

### 3.4 Interface Implementations (Phase 4)

#### FR-015: Desktop/Web/ACP Interface
- **Description:** Desktop app, web interface, ACP
- **Status:** ❌ Not Started (P0 blockers)
- **PRD Reference:** 13-desktop-web-interface.md
- **Components:**
  - Desktop app shell (WebView integration) ❌ (P0)
  - Web server mode (full web interface) ❌ (P0)
  - ACP transport (editor integration) ❌ (P0)
  - Auth protection ❌ (P0)
  - Session sharing between interfaces ❌ (P1)
  - Sharing modes (manual/auto/disabled) ⚠️ (partial - P1)
- **Current State:** Only stubs exist in `crates/cli/src/cmd/`
- **P0 Blockers:**
  - Desktop app shell not implemented
  - Web server mode not implemented
  - ACP transport not implemented
  - Auth protection not implemented

#### FR-016: GitHub Integration
- **Description:** GitHub App integration, workflow triggers, comment parsing
- **Status:** ❌ Not Started (P1 gaps)
- **PRD Reference:** 14-github-gitlab-integration.md
- **Components:**
  - GitHub workflow trigger parsing (`issue_comment`, `pull_request_review`) ❌ (P1)
  - Comment/PR trigger parsing (`/oc` or `/opencode` command) ❌ (P1)
  - CI secret loading for GitHub Actions ❌ (P1)
  - GitHub App installation flow ❌ (not started)
- **P1 Gaps:**
  - Workflow triggers not implemented
  - Comment/PR parsing not implemented
  - CI secret loading not implemented

#### FR-017: GitLab Integration
- **Description:** GitLab CI/CD integration
- **Status:** ❌ Not Started (P2)
- **PRD Reference:** 14-github-gitlab-integration.md
- **Components:**
  - GitLab CI component ❌ (P2)
  - GitLab Duo support ❌ (P3 - marked experimental)
- **Note:** Lower priority, depends on GitHub integration patterns

### 3.5 TUI System (Phase 2-3)

#### FR-018: TUI Core System
- **Description:** Terminal user interface components
- **Status:** ⚠️ Partial (P0/P1 gaps)
- **PRD Reference:** 09-tui-system.md
- **Components:**
  - Session view rendering ⚠️ (markdown, syntax highlighting, diff - partial P1)
  - Slash commands (`/compact`, `/connect`, `/help`, etc.) ❌ (P0 - incomplete)
  - Input model (multiline, history, autocomplete) ⚠️ (partial - P1)
  - File references (`@`) fuzzy search ⚠️ (partial - P1)
  - Shell prefix (`!`) execution ❌ (P2)
  - Keybinding system ⚠️ (leader key, categories - partial - P1)
  - Sidebar (file tree, MCP/LSP status, diagnostics) ⚠️ (partial - P1)
  - Home view (recent sessions, quick actions) ❌ (P2)
- **P0 Blockers:**
  - Slash commands not fully implemented

### 3.6 Test Infrastructure (Phase 5-6)

#### FR-019: Authority Document Tests
- **Description:** Tests for core authority documents (01, 06, 07)
- **Status:** ❌ Not Done
- **PRD Reference:** 16-test-plan.md
- **Requirements:**
  - Core ownership tree tests (unit + integration)
  - Config precedence merge tests
  - API route-group tests
  - Session/message lifecycle tests

#### FR-020: Runtime Architecture Tests
- **Description:** Tests for runtime systems (02, 03, 08, 15)
- **Status:** ❌ Not Done
- **Components:**
  - Agent primary invariant tests (exactly one active)
  - Subagent execution tests
  - Tool registry tests
  - Plugin hook order tests
  - TUI plugin lifecycle tests

#### FR-021: Subsystem Tests
- **Description:** Tests for infrastructure subsystems (04, 05, 10, 11, 12)
- **Status:** ❌ Not Done
- **Components:**
  - MCP integration tests
  - LSP integration tests
  - Provider/model tests
  - Skills discovery tests

#### FR-022: Interface Tests
- **Description:** Tests for desktop/web/ACP/git interfaces (13, 14)
- **Status:** ❌ Not Done
- **Components:**
  - Desktop/web smoke tests
  - ACP handshake tests
  - GitHub workflow tests
  - GitLab integration tests

#### FR-023: Compatibility Suite
- **Description:** Regression tests for legacy/interop behavior
- **Status:** ❌ Not Done
- **Components:**
  - `tools` alias regression suite
  - Skill path regression suite
  - Plugin ownership boundary suite

#### FR-024: Non-Functional Tests
- **Description:** Performance, security, reliability tests
- **Status:** ❌ Not Done
- **Components:**
  - Performance baselines
  - Security tests
  - Recovery tests
  - Crash recovery
  - Snapshot/revert durability

---

## 4. Convention Tests

### 4.1 Architecture Boundary Tests
- **Location:** `tests/src/conventions/architecture_boundaries.rs`
- **Status:** ✅ Implemented (5 tests passing)

### 4.2 Config Ownership Tests
- **Location:** `tests/src/conventions/config_ownership.rs`
- **Status:** ✅ Implemented (4 tests passing)

### 4.3 Route/Resource Group Tests
- **Location:** `tests/src/conventions/route_conventions.rs`
- **Status:** ✅ Implemented (4 tests passing)

### 4.4 Test Placement Tests
- **Location:** `tests/src/conventions/test_layout.rs`
- **Status:** ✅ Implemented (5 tests passing)

### 4.5 TUI Convention Tests
- **Location:** `tests/src/conventions/tui_conventions.rs`
- **Status:** ✅ Implemented (5 tests passing)
- **Requires:** `ratatui-testing/`

**Total Convention Tests:** 23 passing

---

## 5. Technical Debt

| ID | Item | Module | Severity | Remediation |
|----|------|--------|----------|-------------|
| TD-001 | Hook execution order non-deterministic | plugin | Critical | Replace HashMap with IndexMap |
| TD-002 | Custom tool format mismatch | tools | Critical | Change scanner from TOOL.md to .ts/.js |
| TD-003 | Discovery-registry gap | tools | Critical | Connect discovered tools to ToolRegistry |
| TD-004 | Config ownership boundary | config | Critical | Enforce opencode.json vs tui.json split |
| TD-005 | Non-deterministic plugin loading | plugin | High | Implement deterministic loading order |
| TD-006 | Missing MCP transport | mcp | Critical | Implement local/remote server connection |
| TD-007 | ACP stubs only | cli | Critical | Implement ACP protocol transport |
| TD-008 | Deprecated `mode` field | config | Medium | Remove in major version |
| TD-009 | Deprecated `tools` field | config | Medium | Remove after migration |
| TD-010 | Deprecated `keybinds` field | config | Low | Moved to tui.json |
| TD-011 | Deprecated `layout` field | config | Low | Always uses stretch |
| TD-012 | Hardcoded built-in skills | core | Medium | Consider externalization |
| TD-013 | Magic numbers in compaction | core | Low | Make configurable |
| TD-014 | SHA256 args hashing | storage | Low | Consider CAS |
| TD-015 | Custom JSONC parser | config | Medium | Use existing crate |
| TD-016 | `#[serde(other)]` in Part | core | Low | Explicit error handling |

---

## 6. P0/P1/P2 Issue Tracking

### P0 - Blocking Issues (20 items)

| ID | Issue | Module | FR Reference | Target Phase |
|----|-------|--------|---------------|--------------|
| P0-1 | Exactly one active primary agent invariant | agent | FR-005 | Phase 2 |
| P0-2 | Subagent execution (child context, result handoff) | agent | FR-005 | Phase 2 |
| P0-3 | Task/delegation mechanism | agent | FR-005 | Phase 2 |
| P0-4 | Custom tool registration with ToolRegistry | tools | FR-006, FR-007 | Phase 2 |
| P0-5 | Custom tool discovery format (.ts/.js) | tools | FR-007 | Phase 2 |
| P0-6 | Local MCP server connection | mcp | FR-010 | Phase 3 |
| P0-7 | Remote MCP server connection | mcp | FR-010 | Phase 3 |
| P0-8 | Tool discovery from MCP servers | mcp | FR-010 | Phase 3 |
| P0-9 | Built-in LSP server detection | lsp | FR-011 | Phase 3 |
| P0-10 | Diagnostics retrieval from LSP | lsp | FR-011 | Phase 3 |
| P0-11 | Config precedence enforcement | config | FR-003 | Phase 1 |
| P0-12 | Config ownership boundary | config | FR-003 | Phase 1 |
| P0-13 | Route registration by resource group | server | FR-004 | Phase 1 |
| P0-14 | Auth enforcement per endpoint | server | FR-004 | Phase 1 |
| P0-15 | Session/message lifecycle CRUD | server | FR-004 | Phase 1 |
| P0-16 | Plugin hook execution order | plugin | FR-008 | Phase 2 |
| P0-17 | Plugin-provided tool registration | plugin | FR-008 | Phase 2 |
| P0-18 | Plugin config separation | plugin | FR-008 | Phase 2 |
| P0-19 | TUI plugin `plugin_enabled` semantics | tui | FR-009 | Phase 2 |
| P0-20 | TUI plugin activate/deactivate | tui | FR-009 | Phase 2 |

### P1 - Important Issues (32 items)

| ID | Issue | Module | FR Reference | Target Phase |
|----|-------|--------|---------------|--------------|
| P1-1 | Ownership invariant automated tests | core | FR-001 | Phase 5 |
| P1-2 | Stable ID semantics tests | core | FR-001 | Phase 5 |
| P1-3 | Snapshot/Revert model tests | storage | FR-002 | Phase 5 |
| P1-4 | Hidden vs visible agent behavior | agent | FR-005 | Phase 2 |
| P1-5 | Permission inheritance (parent→subagent) | agent | FR-005 | Phase 2 |
| P1-6 | MCP tool qualification | tools | FR-006 | Phase 3 |
| P1-7 | JSONC parsing | config | FR-003 | Phase 1 |
| P1-8 | Variable expansion | config | FR-003 | Phase 1 |
| P1-9 | `tools` → `permission` alias tests | config | FR-003 | Phase 5 |
| P1-10 | Request validation | server | FR-004 | Phase 1 |
| P1-11 | Streaming endpoints | server | FR-004 | Phase 1 |
| P1-12 | Custom LSP registration | lsp | FR-011 | Phase 3 |
| P1-13 | LSP failure handling | lsp | FR-011 | Phase 3 |
| P1-14 | Slash commands | tui | FR-018 | Phase 2 |
| P1-15 | Input model (multiline, history, autocomplete) | tui | FR-018 | Phase 2 |
| P1-16 | File references (`@`) fuzzy search | tui | FR-018 | Phase 2 |
| P1-17 | Keybinding system | tui | FR-018 | Phase 2 |
| P1-18 | Sidebar sections | tui | FR-018 | Phase 2 |
| P1-19 | TUI plugin commands API | tui | FR-009 | Phase 2 |
| P1-20 | TUI plugin routes API | tui | FR-009 | Phase 2 |
| P1-21 | TUI plugin theme API | tui | FR-009 | Phase 2 |
| P1-22 | TUI plugin events API | tui | FR-009 | Phase 2 |
| P1-23 | TUI plugin state API | tui | FR-009 | Phase 2 |
| P1-24 | TUI plugin onDispose lifecycle | tui | FR-009 | Phase 2 |
| P1-25 | Provider abstraction | llm | FR-012 | Phase 3 |
| P1-26 | Default model selection precedence | llm | FR-012 | Phase 3 |
| P1-27 | Per-agent model override | llm | FR-012 | Phase 3 |
| P1-28 | Local model providers | llm | FR-012 | Phase 3 |
| P1-29 | Skills compat paths (Claude/Agent-style) | core | FR-014 | Phase 3 |
| P1-30 | Desktop/web session sharing | cli | FR-015 | Phase 4 |
| P1-31 | Desktop/web sharing modes | cli | FR-015 | Phase 4 |
| P1-32 | GitHub workflow triggers | git | FR-016 | Phase 4 |

### P2 - Nice to Have (16 items)

| ID | Issue | Module | FR Reference | Target Phase |
|----|-------|--------|---------------|--------------|
| P2-1 | Compaction with shareability verification | storage | FR-002 | Phase 5 |
| P2-2 | Workspace path validation | core | FR-001 | Phase 1 |
| P2-3 | Deterministic collision resolution | tools | FR-006 | Phase 3 |
| P2-4 | Result caching for safe tools | tools | FR-006 | Phase 3 |
| P2-5 | MCP per-server OAuth | mcp | FR-010 | Phase 3 |
| P2-6 | MCP timeout handling | mcp | FR-010 | Phase 3 |
| P2-7 | MCP context cost warnings | mcp | FR-010 | Phase 3 |
| P2-8 | Experimental LSP tool | lsp | FR-011 | Phase 3 |
| P2-9 | Config auth/secret storage | config | FR-003 | Phase 1 |
| P2-10 | Server API error shape consistency | server | FR-004 | Phase 1 |
| P2-11 | Plugin cleanup/unload | plugin | FR-008 | Phase 2 |
| P2-12 | Shell prefix (`!`) handling | tui | FR-018 | Phase 2 |
| P2-13 | Home view | tui | FR-018 | Phase 2 |
| P2-14 | LLM variant/reasoning budget | llm | FR-012 | Phase 3 |
| P2-15 | Skills permission restrictions | core | FR-014 | Phase 3 |
| P2-16 | GitLab CI component | git | FR-017 | Phase 4 |

---

## 7. Release Gates

| Gate | Criteria | Status |
|------|----------|--------|
| Phase 0 | Workspace builds, tests run, clippy clean | ✅ |
| Phase 1 | Authority tests green (01, 06, 07) | 🚧 |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | 🚧 |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | 🚧 |
| Phase 4 | Interface smoke workflows pass (13, 14) | ❌ |
| Phase 5a | Compatibility suite green | ❌ |
| Phase 5b | Conventions suite green | ✅ |
| Phase 6 | Non-functional baselines recorded | ❌ |

---

## 8. Crate Ownership Summary

| Crate | Phase | PRD | Status |
|-------|-------|-----|--------|
| `crates/core/` | 1 | `01`, `06` | ⚠️ Partial |
| `crates/storage/` | 1 | `01` | ⚠️ Partial |
| `crates/config/` | 1 | `06` | ⚠️ Partial (P0 gaps) |
| `crates/server/` | 1, 4 | `07`, `13` | ⚠️ Partial (P0 gaps) |
| `crates/agent/` | 2 | `02` | ⚠️ Partial (P0 gaps) |
| `crates/tools/` | 2, 3 | `03`, `11` | ⚠️ Partial (P0 gaps) |
| `crates/plugin/` | 2 | `08` | ⚠️ Partial (P0 gaps) |
| `crates/tui/` | 2, 3 | `09`, `15` | ⚠️ Partial (P0 gaps) |
| `crates/mcp/` | 3 | `04` | ❌ Stubs only |
| `crates/lsp/` | 3 | `05` | ❌ Stubs only |
| `crates/llm/` | 3 | `10` | ⚠️ Partial |
| `crates/git/` | 4 | `14` | ❌ Not started |
| `crates/sdk/` | 2 | `15` | ❌ Not started |
| `ratatui-testing/` | 2, 3 | `09`, `15` | ✅ |

---

## 9. Immediate Actions (Next Sprint)

Based on gap analysis P0 blockers, the following must be addressed:

1. **FR-005/FR-008:** Fix plugin hooks order (HashMap → IndexMap)
2. **FR-006/FR-007:** Connect custom tool discovery to ToolRegistry
3. **FR-005:** Implement subagent execution (child context, result handoff)
4. **FR-003:** Enforce config ownership boundary (opencode.json vs tui.json)
5. **FR-010:** Implement MCP local/remote server connection
6. **FR-011:** Implement LSP diagnostics retrieval
7. **FR-004:** Complete session/message lifecycle CRUD
8. **FR-009:** Implement TUI plugin activate/deactivate

---

## 10. Cross-References

- [PRD Test Plan](./gap-analysis.md#prd-test-plan)
- [Gap Analysis Report](./gap-analysis.md)
- [Iteration 1 Spec](../iteration-1/spec_v1.md)

---

## 11. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 4.0 | 2026-04-10 | Updated based on Iteration 4 gap analysis. Corrected implementation status (was ~85%, now ~35-40%). Added 8 new FRs (FR-015 to FR-024). Expanded P0 blockers to 20 items. Added detailed technical debt table. |
| 1.0 | 2026-04-09 | Initial version based on PRD and gap analysis |

---

*Document generated: 2026-04-10*
*Iteration: 4*
*Phase: Phase 1-2 of 6*
