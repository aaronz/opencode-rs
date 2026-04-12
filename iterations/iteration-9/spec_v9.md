# OpenCode Rust Port — Specification Document v9

**Version:** 9.0
**Generated:** 2026-04-12
**Based on:** PRD specifications and gap analysis (Iteration 9)
**Status:** Draft

---

## 1. Overview

This document defines the specification for the OpenCode Rust port implementation. It is derived from the PRD specifications (`01`–`19`) and updated based on gap analysis between planned features and current implementation status.

**Overall Completion Estimate: ~90-92%**
**Phase Status:** Phase 5-6 of 6 (Hardening, Release Qualification)

---

## 2. Implementation Status Summary

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | ✅ Complete | ~99% |
| Phase 2 | Runtime Core | ✅ Complete | ~99% |
| Phase 3 | Infrastructure Subsystems | ✅ Complete | ~98% |
| Phase 4 | Interface Implementations | ✅ Complete | ~90% |
| Phase 5 | Hardening | 🚧 In Progress | ~85% |
| Phase 6 | Release Qualification | 🚧 Partial | ~70% |

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
- **Completed Gaps:**
  - Project VCS worktree root distinction (P2-1 - ✅ Done)
  - Workspace path validation (P2-2 - ✅ Done)
- **Test Coverage:** Unit tests for ownership tree acyclicity, serialization roundtrips ✅

#### FR-002: Storage Layer
- **Description:** Persistence and recovery for projects, sessions, messages
- **Status:** ✅ Implemented
- **Components:**
  - Project/session/message persistence ✅
  - Session recovery after restart ✅
  - Snapshot create/load ✅
  - Revert to checkpoint ✅
  - Compaction with preserved resumability/shareability ✅
- **Completed Gaps:**
  - Compaction shareability verification (P2-3 - ✅ Done)

### 3.2 Configuration System (Phase 1)

#### FR-003: Config System
- **Description:** Configuration precedence, normalization, ownership boundaries
- **Status:** ✅ Mostly Complete (P1 gaps remain)
- **Components:**
  - JSON and JSONC parsing ✅
  - Config precedence: remote → global → custom → project → `.opencode` → inline ✅
  - Variable expansion: `{env:VAR}` and `{file:PATH}` ✅
  - `tools` legacy alias normalization into `permission` ✅
  - Permission rule type with glob pattern support ✅
  - Auth/secret storage paths ✅
  - Config ownership boundary: `opencode.json` vs `tui.json` split ✅
  - Circular reference detection in variable expansion (P1-2 - ✅ Done)
- **P1 Gaps:**
  - JSONC error messages improved with source line display and caret (P1-1 - ✅ Done)
  - Circular variable expansion detection (P1-2 - ✅ Done)
  - Deprecated fields remaining: `mode`, `tools`, `theme`, `keybinds` (P1-3 - In Progress)

### 3.3 Server API (Phase 1)

#### FR-004: HTTP API Surface
- **Description:** Route groups, auth, request validation
- **Status:** ✅ Mostly Complete
- **Components:**
  - Route registration by canonical resource group ✅
  - Auth enforcement per endpoint ✅
  - Request validation ✅
  - Session/message lifecycle endpoints (CRUD) ✅
  - Streaming endpoints (SSE) ✅
  - API error shape consistency (P2-9 - ✅ Done)
- **Completed Gaps:**
  - Request validation edge cases (P1-11 - ✅ Done)
  - API error shape consistency (P2-9 - ✅ Done)

### 3.4 Agent System (Phase 2)

#### FR-005: Agent System
- **Description:** Primary/subagent model, permission boundaries
- **Status:** ✅ Implemented
- **Components:**
  - Primary agent execution loop ✅
  - Exactly one active primary agent per session invariant ✅
  - Hidden vs visible agent behavior ✅
  - Subagent execution — child context, result handoff ✅
  - Permission inheritance from parent to subagent (P1-10 - ✅ Done)
  - Runtime restriction of subagent permissions ✅
- **Agents Implemented:**
  - `build` — Full tool access (default, visible)
  - `plan` — Read-only analysis (visible)
  - `compaction` — Hidden, context compression
  - `title` — Hidden, title generation
  - `summary` — Hidden, session summarization
  - `general` — Full tool access for subagent (write, edit, bash, read, grep, glob, list)
  - `explore` — Read-only code exploration (read, grep, glob, list)
- **Completed Gaps:**
  - Permission inheritance edge cases (P1-10 - ✅ Done)

### 3.5 Tools System (Phase 2-3)

#### FR-006: Tools System
- **Description:** Registry, execution pipeline, permission gate
- **Status:** ✅ Implemented
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
- **Completed Gaps:**
  - Result caching invalidation (P2-5 - ✅ Done)
  - Deterministic collision resolution (P2-4 - ✅ Done)

#### FR-007: Custom Tool File Loader
- **Description:** File-based discovery and loading of custom tools
- **Status:** ✅ Implemented
- **PRD Reference:** 03-tools-system.md
- **Components:**
  - Project-level: `.opencode/tools/` directory ✅
  - Global-level: `~/.config/opencode/tools/` directory ✅
  - File-based tool registration to registry ✅
  - Tool definition format: TypeScript/JavaScript files ✅

### 3.6 Plugin System (Phase 2)

#### FR-008: Plugin System
- **Description:** Server/runtime plugin hooks and loading
- **Status:** ✅ Implemented
- **Components:**
  - Plugin source loading from configured paths ✅
  - Hooks: `on_init`, `on_start`, `on_tool_call`, `on_message`, `on_session_end` ✅
  - Failure containment — plugin errors do not crash runtime ✅
  - Plugin-provided tool registration ✅
  - Plugin cleanup/unload ✅
  - Deterministic plugin loading order (IndexMap) ✅

### 3.7 TUI Plugin API (Phase 2)

#### FR-009: TUI Plugin API
- **Description:** Plugin surface for terminal UI
- **Status:** ✅ Implemented
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

### 3.8 MCP System (Phase 3)

#### FR-010: MCP Integration
- **Description:** Local/remote MCP server integration
- **Status:** ✅ Implemented
- **PRD Reference:** 04-mcp-system.md
- **Components:**
  - Local MCP server connection (stdio transport + JSON-RPC) ✅
  - Remote MCP server connection (HTTP+SSE) ✅
  - Per-server OAuth configuration ✅
  - Tool discovery from MCP servers ✅
  - Tool naming with server qualification (`<servername>_<toolname>`) ✅
  - Permission gating for MCP tools ✅
  - Timeout and unavailable-server handling ✅
  - Context cost warnings ✅
- **Completed Gaps:**
  - Per-server OAuth token storage (P2-6 - ✅ Done)
  - Context cost warnings (P2-7 - ✅ Done)

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
  - Experimental LSP tool (`goToDefinition`, `findReferences`) ✅ (behind feature flag)
- **Completed Gaps:**
  - Experimental LSP tool testing (P2-8 - ✅ Done)

### 3.10 Provider/Model System (Phase 3)

#### FR-012: Provider/Model System
- **Description:** LLM provider abstraction and model selection
- **Status:** ✅ Mostly Implemented
- **Components:**
  - Provider abstraction ✅
  - Default model selection ✅
  - Per-agent model override ✅
  - Local model providers (Ollama, LM Studio) ✅
  - Variant/reasoning budget support (P2-13 - ✅ Done)
- **Providers Implemented:**
  - OpenAI, Anthropic, Google, Azure, Bedrock ✅
  - Ollama, LM Studio, Local models ✅
- **P1 Gap:**
  - Variant/reasoning budget (P1-10 - ✅ Done, marked experimental)

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
  - Permission restrictions for skill usage ✅

### 3.13 Desktop/Web/ACP Interface (Phase 4)

#### FR-015: Desktop/Web/ACP Interface
- **Description:** Desktop app, web interface, ACP
- **Status:** ✅ Mostly Complete
- **PRD Reference:** 13-desktop-web-interface.md
- **Components:**
  - Desktop app shell (WebView integration) ✅ (P0-new-2 - Done)
  - Web server mode (partial - HTTP server + browser open) ✅
  - **ACP HTTP+SSE transport ✅ (P0-new-3 - IMPLEMENTED)**
  - ACP CLI commands ✅
  - ACP handshake flow ✅
  - Auth protection ✅ (Complete auth middleware)
  - Session sharing between interfaces ✅ (P1-9 - Done)
  - Sharing modes (manual/auto/disabled) ✅
- **Completed Gaps:**
  - Desktop WebView integration (P0-new-2 - ✅ Done)
  - Session sharing between interfaces (P1-9 - ✅ Done)
  - ACP transport (P0-new-3 - ✅ Done)

### 3.14 GitHub/GitLab Integration (Phase 4)

#### FR-016: GitHub Integration
- **Description:** GitHub App integration, workflow triggers, comment parsing
- **Status:** ✅ Implemented
- **PRD Reference:** 14-github-gitlab-integration.md
- **Components:**
  - GitHub workflow trigger parsing (`issue_comment`, `pull_request_review`) ✅
  - Comment/PR trigger parsing (`/oc` or `/opencode` command) ✅
  - CI secret loading for GitHub Actions ✅
  - GitHub App installation flow ✅

#### FR-017: GitLab Integration
- **Description:** GitLab CI/CD integration
- **Status:** ✅ Implemented
- **PRD Reference:** 14-github-gitlab-integration.md
- **Components:**
  - GitLab CI component ✅
  - GitLab Duo support ✅ (Marked experimental - P2-14 - ✅ Done)

### 3.15 TUI System (Phase 2-3)

#### FR-018: TUI Core System
- **Description:** Terminal user interface components
- **Status:** ✅ Mostly Implemented
- **PRD Reference:** 09-tui-system.md
- **Components:**
  - Session view rendering ✅ (markdown, syntax highlighting, diff)
  - **Slash commands ✅ (All implemented: `/compact`, `/connect`, `/help`)**
  - **Multiline input ✅ (P1-5 - ✅ DONE)**
  - File references (`@`) fuzzy search ✅
  - Shell prefix (`!`) execution ✅
  - Keybinding system ✅ (leader key, categories)
  - Sidebar (file tree, MCP/LSP status, diagnostics) ✅
  - Home view ✅ (P2-12 - ✅ Done: recent sessions, quick actions)
- **Completed Gaps:**
  - Shell prefix (`!`) handler (P2-11 - ✅ Done)
  - Home view completion (P2-12 - ✅ Done)

### 3.16 Test Infrastructure (Phase 5-6)

#### FR-019: Authority Document Tests
- **Description:** Tests for core authority documents (01, 06, 07)
- **Status:** ✅ Implemented
- **PRD Reference:** 16-test-plan.md
- **Components:**
  - Core ownership tree tests (unit + integration) ✅
  - Config precedence merge tests ✅
  - API route-group tests ✅
  - Session/message lifecycle tests ✅
- **Test Suites:** 4 suites

#### FR-020: Runtime Architecture Tests
- **Description:** Tests for runtime systems (02, 03, 08, 15)
- **Status:** ✅ Implemented
- **Components:**
  - Agent primary invariant tests (exactly one active) ✅
  - Subagent execution tests ✅
  - Tool registry tests ✅
  - Plugin hook order tests ✅
  - TUI plugin lifecycle tests ✅
- **Test Suites:** 5 suites

#### FR-021: Subsystem Tests
- **Description:** Tests for infrastructure subsystems (04, 05, 10, 11, 12)
- **Status:** ✅ Implemented
- **Components:**
  - MCP integration tests ✅
  - LSP integration tests ✅
  - Provider/model tests ✅
  - Skills discovery tests ✅
- **Test Suites:** 4 suites

#### FR-022: Interface Tests
- **Description:** Tests for desktop/web/ACP/git interfaces (13, 14)
- **Status:** ✅ Implemented
- **Components:**
  - Desktop/web smoke tests ✅
  - ACP handshake tests ✅
  - GitHub workflow tests ✅
  - GitLab integration tests ✅
- **Test Suites:** 4 suites
- **Completed Gaps:**
  - CLI e2e prompt history tests (P2-15 - ✅ Fixed)

#### FR-023: Compatibility Suite
- **Description:** Regression tests for legacy/interop behavior
- **Status:** ✅ Implemented
- **Components:**
  - `tools` alias regression suite ✅
  - Skill path regression suite ✅
  - Plugin ownership boundary suite ✅
- **Test Suites:** 3 suites

#### FR-024: Non-Functional Tests
- **Description:** Performance, security, reliability tests
- **Status:** ✅ Implemented
- **Components:**
  - Performance baselines ✅
  - Security tests ✅
  - Recovery tests ✅
  - Crash recovery ✅
  - Snapshot/revert durability ✅
- **Test Suites:** 5 suites

### 3.17 Convention Tests

#### FR-025: Convention Tests
- **Description:** Architecture, config, route, layout, and TUI conventions
- **Status:** ✅ Implemented (23 tests)
- **Components:**
  - Architecture Boundary Tests ✅ (5 tests)
  - Config Ownership Tests ✅ (4 tests)
  - Route Conventions Tests ✅ (4 tests)
  - Test Layout Tests ✅ (5 tests)
  - TUI Convention Tests ✅ (5 tests)

### 3.18 Rust Test Roadmap (Phase 5-6)

#### FR-026: Per-Crate Test Implementation
- **Description:** Per-crate test coverage following Rust best practices
- **Status:** 🚧 In Progress (~75%)
- **PRD Reference:** 17-rust-test-roadmap.md
- **Components:**
  - Unit tests per crate ✅ (Most crates)
  - Integration tests per crate ✅
  - Mock helpers and fixtures ✅
- **Completed Gaps:**
  - Dead code cleanup (DC-1 through DC-10 - ✅ All Fixed)

### 3.19 Crate Test Backlog (Phase 5-6)

#### FR-027: Crate Test Backlog
- **Description:** Test coverage backlog for remaining gaps
- **Status:** 🚧 Partial (~70%)
- **PRD Reference:** 18-crate-test-backlog.md
- **Components:**
  - Gap-based test additions ✅ (Some addressed)

### 3.20 Implementation Plan Tracking

#### FR-028: Implementation Plan
- **Description:** Overall implementation tracking and phasing
- **Status:** ✅ Complete
- **PRD Reference:** 19-impl-plan.md
- **Components:**
  - Phase-by-phase progress tracking ✅
  - Iteration tracking ✅

---

## 4. Technical Debt

| ID | Item | Module | Severity | Remediation | Status |
|----|------|--------|----------|-------------|--------|
| ~~TD-001~~ | ~~Clippy unreachable pattern~~ | ~~permission~~ | ~~CRITICAL~~ | ~~Fixed in models.rs~~ | ✅ **RESOLVED** |
| ~~TD-002~~ | ~~Desktop WebView stub~~ | ~~cli~~ | ~~P0~~ | ~~Implemented~~ | ✅ **RESOLVED** |
| TD-003 | Deprecated `mode` field | config | Medium | Remove in v4.0 | 🚧 In Progress |
| TD-004 | Deprecated `tools` field | config | Medium | Remove after migration | Deferred |
| TD-005 | Deprecated `theme` field | config | Low | Moved to tui.json | Deferred |
| TD-006 | Deprecated `keybinds` field | config | Low | Moved to tui.json | Deferred |
| TD-007 | Magic numbers in compaction | core | Low | Make configurable | Deferred |
| TD-008 | Custom JSONC parser | config | Medium | Consider existing crate | Deferred |
| TD-009 | `#[serde(other)]` in Part | core | Low | Explicit error handling | Deferred |
| TD-016 | **Clippy errors (18)** | **core, ratatui-testing** | **HIGH** | **Fix all clippy errors** | **P0-9** |

---

## 5. P0/P1/P2 Issue Tracking

### P0 - Blocking Issues

| ID | Issue | Module | FR Reference | Status |
|----|-------|--------|---------------|--------|
| P0-1 through P0-20 | (From Iteration 4) | various | various | ✅ All Fixed |
| P0-new-1 | Git crate syntax error | git | n/a | ✅ **RESOLVED** |
| P0-8 | Clippy unreachable pattern | permission | n/a | ✅ **FIXED** |
| P0-new-2 | Desktop WebView integration | cli | FR-015 | ✅ **IMPLEMENTED** |
| P0-new-3 | ACP HTTP+SSE transport | cli/server | FR-015 | ✅ **IMPLEMENTED** |
| **P0-9** | **Clippy fails with `-D warnings` (18 errors)** | **core, ratatui-testing** | **n/a** | ❌ **NEW** |

**P0 Blockers Summary:** 1 remaining (P0-9: 18 clippy errors)

### P1 - Important Issues (1 remaining)

| ID | Issue | Module | FR Reference | Status |
|----|-------|--------|---------------|--------|
| P1-3 | Deprecated fields (mode, tools, theme, keybinds) | config | FR-003 | 🚧 In Progress |

**Completed P1 Issues (Iteration 9):**
| ID | Issue | Status |
|----|-------|--------|
| P1-2 | Circular variable expansion detection | ✅ Done |
| P1-9 | Session sharing between interfaces | ✅ Done |
| P1-10 | Variant/reasoning budget | ✅ Done |

**Completed P1 Issues (Iteration 8):**
| ID | Issue | Status |
|----|-------|--------|
| P1-5 | Multiline input terminal support | ✅ Done |
| P1-7 | TUI Plugin dialogs incomplete | ✅ Done |
| P1-8 | TUI Plugin slots system incomplete | ✅ Done |

**Completed P1 Issues (Iteration 7):**
| ID | Issue | Status |
|----|-------|--------|
| P1-5 | Multiline input terminal support | ✅ Done (Iteration 7) |
| P1-6 | File reference autocomplete improvement | ✅ Done |
| P1-10 | Permission inheritance edge cases | ✅ Done |
| P1-11 | Request validation edge cases | ✅ Done |

### P2 - Nice to Have (2 remaining)

| ID | Issue | Module | FR Reference | Status |
|----|-------|--------|---------------|--------|
| P2-16 | Remaining clippy warnings | various | n/a | Deferred |
| P2-17 | Per-crate test backlog | tests | FR-026/027 | Deferred |

**Completed P2 Issues (Iteration 9):**
| ID | Issue | Status |
|----|-------|--------|
| P2-1 | Project VCS worktree root distinction | ✅ Done |
| P2-2 | Workspace path validation | ✅ Done |
| P2-9 | API error shape consistency | ✅ Done |
| P2-12 | Home view completion | ✅ Done |
| P2-13 | LLM reasoning budget | ✅ Done |
| P2-14 | GitLab Duo marking | ✅ Done |
| P2-15 | Git test cleanup | ✅ Done |

**Completed P2 Issues (Iteration 7-8):**
| ID | Issue | Status |
|----|-------|--------|
| P2-4 | Deterministic collision resolution | ✅ Done |
| P2-5 | Result caching invalidation | ✅ Done |
| P2-6 | Per-server OAuth token storage | ✅ Done |
| P2-7 | Context cost warnings | ✅ Done |
| P2-8 | Experimental LSP tool testing | ✅ Done |
| P2-10 | Plugin cleanup/unload | ✅ Done |
| P2-11 | Shell prefix (`!`) handler | ✅ Done |

---

## 6. Code Quality Issues

### Critical Issues (Must Fix)

| ID | Item | File | Severity | Description |
|----|------|------|----------|-------------|
| **CQ-1** | **Clippy errors (18 total)** | **core, ratatui-testing** | **HIGH** | **Must fix for release with `-D warnings`** |

### Clippy Errors Detail (P0-9)

**ratatui-testing (1 error):**
| Error | File | Fix |
|-------|------|-----|
| `new_without_default` for `StateTester` | ratatui-testing/src/state.rs:6 | Add `impl Default for StateTester` |

**opencode-core (17 errors):**
| Error | File | Count |
|-------|------|-------|
| deprecated `AgentMode` enum | config.rs:436 | 2 |
| deprecated `AgentConfig::mode` field | command.rs:567, config.rs:2771 | 2 |
| `question_mark` (use `?`) | config.rs:1594 | 1 |
| `needless_borrows_for_generic_args` | config.rs:2068 | 1 |
| `redundant_closure` (use `ok_or`) | session_sharing.rs:323 | 1 |
| `map_entry` (use entry API) | session_sharing.rs:225 | 1 |
| `and_then` → `map` | crash_recovery.rs:241 | 1 |
| `very_complex_type` | skill.rs | 1 |
| `&PathBuf` → `&Path` | skill.rs:116 | 5 |

### Dead Code (Resolved in Iteration 9)

| ID | Item | Status |
|----|------|--------|
| DC-1 | Unused `Message` import | ✅ Fixed |
| DC-2 | Unused `SecretStorage` methods | ✅ Fixed |
| DC-3 | Unused `e` variable in lsp_tool | ✅ Fixed |
| DC-4 | Unused `body` variable in github | ✅ Fixed |
| DC-5 | `open_browser` function unused | ✅ Fixed |
| DC-6 | `format_time_elapsed` function unused | ✅ Fixed |
| DC-7 | Unused `complete` variable | ✅ Fixed |
| DC-8 | Unused `models_url` function | ✅ Fixed |
| DC-9 | Unused `ChatStreamChunk` struct | ✅ Fixed |
| DC-10 | Unused `role` field | ✅ Fixed |

### Deprecated Usage Warnings

| ID | Item | File | Severity | Description |
|----|------|------|----------|-------------|
| DEP-1 | `AgentMode` enum | config.rs:436 | Medium | Deprecated, use 'permission' field instead |
| DEP-2 | `AgentConfig::mode` field | config.rs, command.rs | Medium | Deprecated, use 'permission' field instead |
| DEP-3 | `AgentMode` in validation | config.rs:2771 | Medium | Warning added for v4.0 removal |

---

## 7. Build Status

### Release Build
```
Finished `release` profile [optimized] target(s) in 54.92s
```
All crates compile successfully with `cargo build`.

### Clippy Status (with `-D warnings`)

**FAILS** - 18 errors total:

**ratatui-testing (1 error):**
```
error: you should consider adding a `Default` implementation for `StateTester`
 --> ratatui-testing/src/state.rs:6:5
```

**opencode-core (17 errors):**
```
error: use of deprecated enum `config::AgentMode`
   --> crates/core/src/config.rs:436:22

error: use of deprecated field `config::AgentConfig::mode`
   --> crates/core/src/command.rs:567:30
   --> crates/core/src/config.rs:2771:20

error: this block may be rewritten with the `?` operator
   --> crates/core/src/config.rs:1594:17

error: the borrowed expression implements the required traits
   --> crates/core/src/config.rs:2068:41

error: unnecessary closure used to substitute value for `Option::None`
   --> crates/core/src/session_sharing.rs:323:9

error: usage of `contains_key` followed by `insert` on a `HashMap`
   --> crates/core/src/session_sharing.rs:225

error: using `Option.and_then(|x| Some(y))`
   --> crates/core/src/crash_recovery.rs:241:29

error: writing `&PathBuf` instead of `&Path` involves a new object
   --> crates/core/src/skill.rs:116:36

error: very complex type used. Consider factoring parts into `type` definitions
   --> crates/core/src/skill.rs
```

**Clippy passes** for: permission, agent, tools, mcp, lsp, plugin, server, cli, git, llm, storage

### Per-Crate Status

| Crate | Build | Tests | Clippy | Notes |
|-------|-------|-------|--------|-------|
| opencode-core | ✅ | ✅ | ❌ | 17 clippy errors |
| opencode-permission | ✅ | ✅ | ✅ | Fixed from iter-8 |
| opencode-agent | ✅ | ✅ | ✅ | Clean |
| opencode-tools | ✅ | ✅ | ✅ | Clean |
| opencode-mcp | ✅ | ✅ | ✅ | Clean |
| opencode-lsp | ✅ | ✅ | ✅ | Clean |
| opencode-plugin | ✅ | ✅ | ✅ | Clean |
| opencode-server | ✅ | ✅ | ✅ | Clean |
| opencode-cli | ✅ | ✅ | ✅ | Clean |
| opencode-git | ✅ | ✅ | ✅ | Clean |
| opencode-llm | ✅ | ✅ | ✅ | Clean |
| opencode-storage | ✅ | ✅ | ✅ | Clean |
| ratatui-testing | ✅ | ✅ | ❌ | 1 clippy error |

---

## 8. Test Coverage Status

| Test Suite | Status | Test Count | Notes |
|------------|--------|------------|-------|
| Authority Tests (FR-019) | ✅ Done | 4 suites | Core ownership, config, API, lifecycle |
| Runtime Tests (FR-020) | ✅ Done | 5 suites | Agent invariants, subagent, tool, plugin, TUI |
| Subsystem Tests (FR-021) | ✅ Done | 4 suites | MCP, LSP, provider, skills |
| Interface Tests (FR-022) | ✅ Done | 4 suites | Desktop/web, ACP, GitHub, GitLab |
| Compatibility Suite (FR-023) | ✅ Done | 3 suites | Legacy/interop regressions |
| Non-Functional Tests (FR-024) | ✅ Done | 5 suites | Performance, security, recovery |
| Convention Tests (FR-025) | ✅ Done | 23 tests | Architecture, config, route, layout, TUI |
| **Total** | ✅ Done | **~25 suites + 23 tests** | |

---

## 9. Release Gates

| Gate | Criteria | Status | Notes |
|------|----------|--------|-------|
| Phase 0 | Workspace builds, tests run, clippy clean | ❌ | Clippy fails (P0-9) |
| Phase 1 | Authority tests green (01, 06, 07) | ✅ | All 4 suites pass |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | ✅ | All 5 suites pass |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | ✅ | All 4 suites pass |
| Phase 4 | Interface smoke workflows pass (13, 14) | ✅ | Desktop WebView done |
| Phase 5a | Compatibility suite green | ✅ | All 3 suites pass |
| Phase 5b | Conventions suite green | ✅ | All 23 tests pass |
| Phase 6 | Non-functional baselines recorded | 🚧 | Partial - needs verification |

---

## 10. Crate Ownership Summary

| Crate | Phase | PRD | Status | P0/P1/P2 |
|-------|-------|-----|--------|-----------|
| `crates/core/` | 1 | `01`, `06` | ✅ Complete | P0-9 (clippy), TD-003-008 |
| `crates/storage/` | 1 | `01` | ✅ Complete | None |
| `crates/config/` | 1 | `06` | ✅ Complete | P1-3, TD-003-006, TD-008 |
| `crates/permission/` | 1 | `02` | ✅ Complete | Fixed P0-8 |
| `crates/server/` | 1, 4 | `07`, `13` | ✅ Complete | None |
| `crates/agent/` | 2 | `02` | ✅ Complete | None |
| `crates/tools/` | 2, 3 | `03`, `11` | ✅ Complete | None |
| `crates/plugin/` | 2 | `08` | ✅ Complete | None |
| `crates/tui/` | 2, 3 | `09`, `15` | ✅ Complete | None |
| `crates/mcp/` | 3 | `04` | ✅ Complete | None |
| `crates/lsp/` | 3 | `05` | ✅ Complete | None |
| `crates/llm/` | 3 | `10` | ✅ Complete | P1-10 |
| `crates/git/` | 4 | `14` | ✅ Complete | None |
| `ratatui-testing/` | 2, 3 | `09`, `15` | ✅ Complete | P0-9 (clippy) |

---

## 11. Immediate Actions

### Must Fix (Before Release) - P0

1. **Fix P0-9: Clippy errors (18 total)**
   - **ratatui-testing (1 error):**
     - Add `impl Default for StateTester` in `ratatui-testing/src/state.rs`
   
   - **opencode-core (17 errors):**
     - Fix deprecated `AgentMode` usage (2 errors)
     - Fix `question_mark` in config.rs:1594
     - Fix `needless_borrows_for_generic_args` in config.rs:2068
     - Fix `redundant_closure` in session_sharing.rs:323
     - Fix `map_entry` in session_sharing.rs:225
     - Fix `and_then` → `map` in crash_recovery.rs:241
     - Fix `very_complex_type` in skill.rs
     - Fix `&PathBuf` → `&Path` (5 occurrences) in skill.rs:116

### Should Fix (Before Release) - P1

2. **Plan P1-3: Deprecated fields removal**
   - The `mode` field is deprecated but still used
   - Plan complete removal in v4.0

3. **Plan P1-10: Variant/reasoning budget**
   - Mark as experimental in documentation

---

## 12. Iteration Progress

| Iteration | Date | Completion | Key Changes |
|-----------|------|------------|-------------|
| 1 | 2026-04-09 | ~20% | Initial gap analysis |
| 4 | 2026-04-10 | ~35-40% | Major P0 progress |
| 5 | 2026-04-11 | ~70-75% | Desktop/ACP gaps identified |
| 6 | 2026-04-12 | ~80-85% | ACP done, dialogs/slots done |
| 7 | 2026-04-12 | ~80-85% | Multiline done, P2-6/7/10/15 done |
| 8 | 2026-04-12 | ~85-90% | P0-8 clippy identified, 2 P0 blockers |
| 9 | 2026-04-12 | ~90-92% | P0-8, P0-new-2, P1-2, P1-9, P2-1/2/9/12/13/14/15 all fixed. 1 P0 remains (clippy) |

---

## 13. Progress Since Iteration 8

### Completed Items

| Item | Status | Commit | Notes |
|------|--------|--------|-------|
| P0-8 Clippy unreachable pattern | ✅ Fixed | 95c1c0c | Fixed in permission/models.rs |
| P0-new-2 Desktop WebView | ✅ Done | 131a17e | Basic WebView implementation |
| P1-2 Circular detection | ✅ Done | 2b43da4 | Detection algorithm added |
| P1-9 Session sharing | ✅ Done | 43e6564 | Cross-interface sync complete |
| P2-1 VCS worktree root | ✅ Done | 8e36b4e | worktree_root field added |
| P2-2 Workspace validation | ✅ Done | 6976c92 | validate_workspace() in detect() |
| P2-9 API error shape | ✅ Done | 4634293 | Consistent error format |
| P2-12 Home view | ✅ Done | 16b00c7 | Recent sessions, quick actions |
| P2-13 LLM reasoning budget | ✅ Done | 76d999b | Variant/reasoning support |
| P2-14 GitLab Duo marking | ✅ Done | 5292612 | Marked experimental |
| P2-15 Git test cleanup | ✅ Done | fced218 | DC-1 through DC-10 fixed |
| CLI test_prompt_history | ✅ Fixed | 5d8b024 | test_prompt_history_persistence passing |
| DC-1 through DC-10 | ✅ All Fixed | Multiple | Dead code cleanup complete |

### New Issues Identified

| Item | Status | Notes |
|------|--------|-------|
| P0-9 Clippy failures (18) | ❌ NEW | 17 in core, 1 in ratatui-testing |

---

## 14. Cross-References

| Topic | Document | Notes |
|-------|----------|-------|
| Core entities & session lifecycle | [01-core-architecture.md](./01-core-architecture.md) | Session ownership tree, lifecycle invariants |
| Agent system | [02-agent-system.md](./02-agent-system.md) | Primary/subagent model, permission boundaries |
| Tool implementation | [03-tools-system.md](./03-tools-system.md) | Built-in tool list, custom tool format, MCP integration |
| MCP system | [04-mcp-system.md](./04-mcp-system.md) | Local/remote transport implemented |
| LSP system | [05-lsp-system.md](./05-lsp-system.md) | Diagnostics pipeline complete |
| Configuration schema | [06-configuration-system.md](./06-configuration-system.md) | `AgentConfig` schema, `permission` rule type, precedence |
| Server API | [07-server-api.md](./07-server-api.md) | Route groups, auth, CRUD |
| Plugin system | [08-plugin-system.md](./08-plugin-system.md) | IndexMap for deterministic order |
| TUI system | [09-tui-system.md](./09-tui-system.md) | Slash commands, keybinds improved |
| Provider model | [10-provider-model.md](./10-provider-model.md) | Ollama, LM Studio support |
| Formatters | [11-formatters.md](./11-formatters.md) | FormatterEngine complete |
| Skills system | [12-skills-system.md](./12-skills-system.md) | SKILL.md, compat paths |
| Desktop/web interface | [13-desktop-web-interface.md](./13-desktop-web-interface.md) | ACP done, WebView implemented |
| GitHub/GitLab | [14-github-gitlab-integration.md](./14-github-gitlab-integration.md) | GitLab CI, GitHub workflows |
| TUI plugin API | [15-tui-plugin-api.md](./15-tui-plugin-api.md) | Dialogs and slots completed |
| Test plan | [16-test-plan.md](./16-test-plan.md) | Authority tests complete |
| Rust test roadmap | [17-rust-test-roadmap.md](./17-rust-test-roadmap.md) | Per-crate tests in progress |
| Crate test backlog | [18-crate-test-backlog.md](./18-crate-test-backlog.md) | Some backlog addressed |
| Implementation plan | [19-impl-plan.md](./19-impl-plan.md) | This document |
| Iteration 8 Spec | [spec_v8.md](./spec_v8.md) | Previous version |
| Iteration 7 Spec | [spec_v7.md](./spec_v7.md) | Earlier version |

---

## 15. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 9.0 | 2026-04-12 | Updated based on Iteration 9 gap analysis. Overall completion ~90-92%. Fixed P0-8 (clippy unreachable pattern), P0-new-2 (Desktop WebView), P1-2 (circular detection), P1-9 (session sharing). Fixed P2-1, P2-2, P2-9, P2-12, P2-13, P2-14, P2-15. All dead code DC-1 through DC-10 cleaned up. New P0-9: 18 clippy errors (17 in core, 1 in ratatui-testing). Phase 5 hardening ~85%. Phase 6 release qualification ~70%. TD-001 and TD-002 resolved. Added TD-016 for clippy errors. Updated Technical Debt, P0/P1/P2 tracking, Build Status, Test Coverage, and Release Gates sections. |
| 8.0 | 2026-04-12 | Updated based on Iteration 8 gap analysis. Overall completion ~85-90%. Added P0-8 clippy unreachable pattern blocker in permission/models.rs:28. Added new CLI e2e test failures (test_prompt_history_persistence, test_prompt_history_navigation). P2-11 (Shell prefix) completed. Phase 4 coverage improved to ~75%. Phase 5 hardening updated to ~80%. Added TD-001 as critical clippy issue. Updated Technical Debt table with TD-013, TD-014, TD-015. Added Code Quality Issues section with Critical, Dead Code, and Deprecated categories. Updated Build Status with clippy failure details. Updated iteration progress table. |
| 7.0 | 2026-04-12 | Updated based on Iteration 7 gap analysis. P1-5 multiline input completed. P2-6 (Per-server OAuth), P2-7 (Context cost warnings), P2-10 (Plugin cleanup) completed. P2-15 (Git test code bugs) identified as cleanup issue rather than feature gap. Added Code Quality Warnings section (CQ-1 through CQ-9). Updated build status with warning counts per crate. Overall completion ~80-85%. |
| 6.0 | 2026-04-12 | Updated based on Iteration 6 gap analysis. P0-new-1 resolved (git syntax fixed), P0-new-3 implemented (ACP transport complete). P1-7, P1-8, P1-10, P1-11 completed. P2-10 completed. P2-15 introduced (git test bugs). P0-new-2 remains as sole P0 blocker. Phase 4 coverage improved to ~70%. Build status: git compiles but tests error. Added FR-026, FR-027, FR-028 for new PRDs. Updated agent table to reflect built-in agents with visibility. |
| 5.0 | 2026-04-11 | Updated based on Iteration 5 gap analysis. Overall completion ~70-75%. Added 3 new P0 blockers (git syntax error, Desktop WebView, ACP transport). Added FR-025 for convention tests. Consolidated P0 trackers from 20 to 3. Reduced P1 from 32 to 11 items. Updated phase statuses to Phase 4-6. |
| 4.0 | 2026-04-10 | Updated based on Iteration 4 gap analysis. Corrected implementation status (was ~85%, now ~35-40%). Added 8 new FRs (FR-015 to FR-024). Expanded P0 blockers to 20 items. |
| 1.0 | 2026-04-09 | Initial version based on PRD and gap analysis |

---

*Document generated: 2026-04-12*
*Iteration: 9*
*Phase: Phase 5-6 of 6 (Hardening, Release Qualification)*
