# OpenCode Rust Port — Specification Document v7

**Version:** 7.0
**Generated:** 2026-04-12
**Based on:** PRD specifications and gap analysis (Iteration 7)
**Status:** Draft

---

## 1. Overview

This document defines the specification for the OpenCode Rust port implementation. It is derived from the PRD specifications (`01`–`19`) and updated based on gap analysis between planned features and current implementation status.

**Overall Completion Estimate: ~80-85%**
**Phase Status:** Phase 4-6 of 6 (Interface Implementations, Hardening, Release Qualification)

---

## 2. Implementation Status Summary

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | ✅ Complete | ~98% |
| Phase 2 | Runtime Core | ✅ Complete | ~98% |
| Phase 3 | Infrastructure Subsystems | ✅ Complete | ~95% |
| Phase 4 | Interface Implementations | 🚧 In Progress | ~70% |
| Phase 5 | Hardening | ✅ Complete | ~95% |
| Phase 6 | Release Qualification | 🚧 Partial | ~70% |

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
- **Gap:** Project VCS worktree root distinction (P2 - deferred)
- **Gap:** Workspace path validation (P2 - deferred)
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
- **Gap:** Compaction shareability verification (P2 - ✅ Done in Iteration 7)

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
- **P1 Gaps:**
  - JSONC error messages improved with source line display and caret (P1-1 - ✅ Done)
  - Circular variable expansion detection incomplete (P1-2 - deferred)
  - Deprecated fields remaining: `mode`, `tools`, `theme`, `keybinds` (P1-3 - ✅ Done)

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
  - API error shape consistency ✅
- **P1 Gaps:**
  - Request validation edge cases (P1-11 - ✅ Done)
  - API error shape consistency enforcement (P2-9 - deferred)

### 3.4 Agent System (Phase 2)

#### FR-005: Agent System
- **Description:** Primary/subagent model, permission boundaries
- **Status:** ✅ Implemented
- **Components:**
  - Primary agent execution loop ✅
  - Exactly one active primary agent per session invariant ✅
  - Hidden vs visible agent behavior ✅
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
- **P1 Gaps:**
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
- **P2 Gaps:**
  - Result caching invalidation (P2-5 - deferred)

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
  - Plugin cleanup/unload ✅ (P2-10 - ✅ Done in Iteration 7)
  - Deterministic plugin loading order (IndexMap) ✅

### 3.7 TUI Plugin API (Phase 2)

#### FR-009: TUI Plugin API
- **Description:** Plugin surface for terminal UI
- **Status:** ✅ Implemented (P1 gaps resolved)
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
  - **Dialogs ✅ (P1-7 - All 4 dialogs implemented)**
    - `DialogAlert` ✅
    - `DialogConfirm` ✅
    - `DialogPrompt` ✅
    - `DialogSelect` ✅
  - **Slots system ✅ (P1-8 - Full implementation)**
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
  - Per-server OAuth configuration ✅ (P2-6 - ✅ Done in Iteration 7)
  - Tool discovery from MCP servers ✅
  - Tool naming with server qualification (`<servername>_<toolname>`) ✅
  - Permission gating for MCP tools ✅
  - Timeout and unavailable-server handling ✅
  - Context cost warnings ✅ (P2-7 - ✅ Done in Iteration 7, implemented in context_cost.rs)

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
  - Experimental LSP tool (`goToDefinition`, `findReferences`) ✅ (P2-8 - behind feature flag)
- **P2 Gap:** Experimental LSP tool testing (P2-8 - deferred)

### 3.10 Provider/Model System (Phase 3)

#### FR-012: Provider/Model System
- **Description:** LLM provider abstraction and model selection
- **Status:** ✅ Mostly Implemented
- **Components:**
  - Provider abstraction ✅
  - Default model selection ✅
  - Per-agent model override ✅
  - Local model providers (Ollama, LM Studio) ✅
- **Providers Implemented:**
  - OpenAI, Anthropic, Google, Azure, Bedrock ✅
  - Ollama, LM Studio, Local models ✅
- **P2 Gap:** Variant/reasoning budget (P2-13 - deferred)

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
- **Status:** ⚠️ Partial (1 P0 remaining)
- **PRD Reference:** 13-desktop-web-interface.md
- **Components:**
  - Desktop app shell (WebView integration) ❌ (P0-new-2 - **STUB ONLY**)
  - Web server mode (full web interface) ⚠️ (Partial - HTTP server + browser open)
  - **ACP HTTP+SSE transport ✅ (P0-new-3 - IMPLEMENTED)**
  - ACP CLI commands ✅
  - ACP handshake flow ✅
  - Auth protection ⚠️ (Partial)
  - Session sharing between interfaces ⚠️ (P1-9 - deferred)
  - Sharing modes (manual/auto/disabled) ✅
- **Current State:** ACP transport layer complete. Desktop WebView is the only remaining P0 blocker.
- **P0 Blocker:**
  - Desktop WebView integration not implemented (P0-new-2)
- **P1 Gaps:**
  - Session sharing between interfaces (P1-9 - deferred)
  - Auth protection completeness

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
  - GitLab Duo support ⚠️ (Marked as experimental in docs)
- **P2 Gap:**
  - GitLab Duo experimental marking (P2-14 - deferred)
  - Git test code bugs (P2-15 - ❌ BUG, not a feature gap)

### 3.15 TUI System (Phase 2-3)

#### FR-018: TUI Core System
- **Description:** Terminal user interface components
- **Status:** ✅ Mostly Implemented
- **PRD Reference:** 09-tui-system.md
- **Components:**
  - Session view rendering ✅ (markdown, syntax highlighting, diff)
  - **Slash commands ✅ (All implemented: `/compact`, `/connect`, `/help`)**
  - **Multiline input ✅ (P1-5 - ✅ DONE in Iteration 7)**
  - File references (`@`) fuzzy search ✅
  - Shell prefix (`!`) execution ❌ (P2-11 - deferred)
  - Keybinding system ✅ (leader key, categories)
  - Sidebar (file tree, MCP/LSP status, diagnostics) ✅
  - Home view ⚠️ (P2-12 - partial: recent sessions, quick actions)
- **P1 Gaps:**
  - Multiline input terminal support (P1-5 - ✅ Done in Iteration 7)
- **P2 Gaps:**
  - Shell prefix (`!`) handler (P2-11 - deferred)
  - Home view completion (P2-12 - deferred)

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
- **Status:** 🚧 In Progress (~70%)
- **PRD Reference:** 17-rust-test-roadmap.md
- **Components:**
  - Unit tests per crate ✅ (Most crates)
  - Integration tests per crate ✅
  - Mock helpers and fixtures ✅
- **Gaps:** Some crates need additional test coverage

### 3.19 Crate Test Backlog (Phase 5-6)

#### FR-027: Crate Test Backlog
- **Description:** Test coverage backlog for remaining gaps
- **Status:** 🚧 Partial (~60%)
- **PRD Reference:** 18-crate-test-backlog.md
- **Components:**
  - Gap-based test additions ✅ (Some addressed)
- **Gaps:** Additional backlog items remain

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
| TD-001 | Git test code bugs | git | **HIGH** | Fix `next_port`, `GitLabMockServer`, duplicate test names | **P2-15** |
| TD-002 | Desktop WebView stub | cli | **P0** | Implement actual WebView | P0-new-2 |
| TD-003 | Deprecated `mode` field | config | Medium | Remove in major version | Deferred |
| TD-004 | Deprecated `tools` field | config | Medium | Remove after migration | Deferred |
| TD-005 | Deprecated `theme` field | config | Low | Moved to tui.json | Deferred |
| TD-006 | Deprecated `keybinds` field | config | Low | Moved to tui.json | Deferred |
| TD-007 | Magic numbers in compaction | core | Low | Make configurable | Deferred |
| TD-008 | Custom JSONC parser | config | Medium | Consider existing crate | Deferred |
| TD-009 | `#[serde(other)]` in Part | core | Low | Explicit error handling | Deferred |
| TD-010 | Unused `SecretStorage` methods | core | Low | Remove or use | Deferred |
| TD-011 | `unreachable_patterns` warning | permission | Low | Fix match exhaustiveness | Deferred |
| TD-012 | Unused imports in core | core | Low | Clean up imports | Deferred |
| TD-013 | Unused variable `e` in lsp_tool | tools | Low | Prefix with underscore | Deferred |

---

## 5. P0/P1/P2 Issue Tracking

### P0 - Blocking Issues

| ID | Issue | Module | FR Reference | Status |
|----|-------|--------|---------------|--------|
| P0-1 through P0-20 | (From Iteration 4) | various | various | ✅ All Fixed |
| P0-new-1 | Git crate syntax error | git | n/a | ✅ **RESOLVED** |
| **P0-new-2** | **Desktop WebView integration** | **cli** | **FR-015** | ❌ **STUB ONLY** |
| P0-new-3 | ACP HTTP+SSE transport | cli/server | FR-015 | ✅ **IMPLEMENTED** |

### P1 - Important Issues (4 remaining)

| ID | Issue | Module | FR Reference | Status |
|----|-------|--------|---------------|--------|
| P1-1 | JSONC error messages clarity | config | FR-003 | ✅ Done |
| P1-2 | Circular variable expansion detection | config | FR-003 | Deferred |
| P1-3 | Deprecated fields (mode, tools, theme, keybinds) | config | FR-003 | ✅ Done |
| P1-9 | Session sharing between interfaces partial | cli | FR-015 | Deferred |
| P1-10 | Permission inheritance edge cases | agent | FR-005 | ✅ Done |
| P1-11 | Request validation edge cases | server | FR-004 | ✅ Done |

**Completed P1 Issues (Iteration 7):**
| ID | Issue | Status |
|----|-------|--------|
| P1-3 | Deprecated fields (mode, tools, theme, keybinds) | ✅ **Done** (Iteration 7) |
| P1-5 | Multiline input terminal support | ✅ **Done** (Iteration 7) |

**Completed P1 Issues (Prior):**
| ID | Issue | Status |
|----|-------|--------|
| P1-1 | JSONC error messages clarity | ✅ Done |
| P1-4 | Slash commands (`/compact`, `/connect`, `/help`) | ✅ Done |
| P1-6 | File reference autocomplete improvement | ✅ Done |
| P1-7 | TUI Plugin dialogs incomplete | ✅ Done |
| P1-8 | TUI Plugin slots system incomplete | ✅ Done |
| P1-10 | Permission inheritance edge cases | ✅ Done |
| P1-11 | Request validation edge cases | ✅ Done |

### P2 - Nice to Have (14 items)

| ID | Issue | Module | FR Reference | Status |
|----|-------|--------|---------------|--------|
| P2-1 | Project VCS worktree root distinction | core | FR-001 | Deferred |
| P2-2 | Workspace path validation | core | FR-001 | ✅ Done |
| P2-3 | Compaction shareability verification | storage | FR-002 | ✅ Done |
| P2-4 | Deterministic collision resolution | tools | FR-006 | ✅ Done |
| P2-5 | Result caching invalidation | tools | FR-006 | Deferred |
| P2-6 | Per-server OAuth token storage | mcp | FR-010 | ✅ Done |
| P2-7 | Context cost warnings | mcp | FR-010 | ✅ Done |
| P2-8 | Experimental LSP tool testing | lsp | FR-011 | Deferred |
| P2-9 | API error shape consistency | server | FR-004 | Deferred |
| P2-10 | Plugin cleanup/unload | plugin | FR-008 | ✅ Done |
| P2-11 | Shell prefix (`!`) handler | tui | FR-018 | Deferred |
| P2-12 | Home view completion | tui | FR-018 | Deferred |
| P2-13 | LLM variant/reasoning budget | llm | FR-012 | Deferred |
| P2-14 | GitLab Duo experimental marking | git | FR-017 | Deferred |
| **P2-15** | **Git test code bugs** | **git** | **n/a** | ❌ **BUG** |

**Completed P2 Issues (Iteration 7):**
| ID | Issue | Status |
|----|-------|--------|
| P2-6 | Per-server OAuth token storage | ✅ Done (Iteration 7) |
| P2-7 | Context cost warnings | ✅ Done (Iteration 7) |
| P2-10 | Plugin cleanup/unload | ✅ Done (Iteration 7) |

---

## 6. Code Quality Warnings

| ID | Item | File | Severity | Description |
|----|------|------|----------|-------------|
| CQ-1 | Unused `Message` import | core/crash_recovery.rs:1 | Low | `Message` unused in import |
| CQ-2 | Unused `SecretStorage` methods | core/config/secret_storage.rs:36 | Low | 6 methods never called |
| CQ-3 | Unused `e` variable | tools/lsp_tool.rs:311,526,626,783 | Low | Should be `_e` |
| CQ-4 | Unused `body` variable | git/github.rs:566 | Low | Should be `_body` |
| CQ-5 | Unused `next_port` function | git/gitlab_ci.rs:413 | Low | Function defined but never used |
| CQ-6 | Unused `GitLabMockServer` | git/gitlab_ci.rs:706 | Low | Struct never constructed |
| CQ-7 | Unused imports | cli/src/cmd/quick.rs:5-6 | Low | `save_session_records`, `SessionRecord` |
| CQ-8 | Unused `save_session_records` | cli/src/cmd/session.rs:42 | Low | Function never used |
| CQ-9 | Unused `complete` variable | cli/src/cmd/mcp_auth.rs:216 | Low | Should be `_complete` |

---

## 7. Build Status

| Crate | Build | Tests | Warnings |
|-------|-------|-------|----------|
| opencode-core | ✅ | ✅ | 2 (unused imports, dead code) |
| opencode-agent | ✅ | ✅ | 0 |
| opencode-tools | ✅ | ✅ | 4 (unused variables) |
| opencode-mcp | ✅ | ✅ | 3 |
| opencode-lsp | ✅ | ✅ | 0 |
| opencode-plugin | ✅ | ✅ | 1 |
| opencode-server | ✅ | ✅ | 2 |
| opencode-cli | ✅ | ✅ | 5 |
| opencode-git | ✅ | ⚠️ | 1 (unused code - P2-15) |
| opencode-llm | ✅ | ✅ | 12 |

**Git Crate Test Errors (P2-15):**
The `opencode-git` crate has test code issues:
- `function 'next_port' is never used` (line 413)
- `struct 'GitLabMockServer' is never constructed` (line 706)
- Associated items 'new', 'handle_request', 'url', and 'stop' are never used

**Root Cause:** Test code structure issue - duplicate test module definitions or orphaned test code. The mock server and helper function were likely refactored but test code was not cleaned up.

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
| Phase 0 | Workspace builds, tests run, clippy clean | ✅ | Release builds, clippy warnings |
| Phase 1 | Authority tests green (01, 06, 07) | ✅ | All 4 suites pass |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | ✅ | All 5 suites pass |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | ✅ | All 4 suites pass |
| Phase 4 | Interface smoke workflows pass (13, 14) | 🚧 | Desktop WebView P0 blocks |
| Phase 5a | Compatibility suite green | ✅ | All 3 suites pass |
| Phase 5b | Conventions suite green | ✅ | All 23 tests pass |
| Phase 6 | Non-functional baselines recorded | 🚧 | Partial - needs verification |

---

## 10. Crate Ownership Summary

| Crate | Phase | PRD | Status | P0/P1/P2 |
|-------|-------|-----|--------|----------|
| `crates/core/` | 1 | `01`, `06` | ✅ Complete | P2 gaps, TD-010, TD-012 |
| `crates/storage/` | 1 | `01` | ✅ Complete | None |
| `crates/config/` | 1 | `06` | ✅ Complete | P1-2, P1-3, TD-003-006, TD-008 |
| `crates/server/` | 1, 4 | `07`, `13` | ✅ Complete | ACP transport done |
| `crates/agent/` | 2 | `02` | ✅ Complete | P1-10 done |
| `crates/tools/` | 2, 3 | `03`, `11` | ✅ Complete | TD-013 |
| `crates/plugin/` | 2 | `08` | ✅ Complete | P2-10 done |
| `crates/tui/` | 2, 3 | `09`, `15` | ✅ Complete | P1-5 done |
| `crates/mcp/` | 3 | `04` | ✅ Complete | P2-6, P2-7 done |
| `crates/lsp/` | 3 | `05` | ✅ Complete | None |
| `crates/llm/` | 3 | `10` | ✅ Complete | None |
| `crates/git/` | 4 | `14` | ⚠️ Test bug | P2-15 |
| `ratatui-testing/` | 2, 3 | `09`, `15` | ✅ Complete | None |

---

## 11. Immediate Actions

### Must Fix (Before Release) - P0

1. **Fix P0-new-2: Desktop WebView integration**
   - Current `desktop.rs` uses `wry` for WebView but only spawns browser when `desktop` feature is off
   - When `desktop` feature is enabled, `spawn_webview_thread` creates a WebView but doesn't properly integrate with the app lifecycle
   - Need actual WebView component per PRD 13 that shares state with the TUI/server
   - **This is the ONLY remaining P0 blocker**

### Should Fix (Before Release) - P1

2. **Fix P2-15: Git test code cleanup**
   - Remove unused `next_port()` function or use it
   - Remove or use `GitLabMockServer` struct and its impl
   - Clean up dead test code

3. **Complete P1-9: Session sharing between interfaces**
   - Cross-interface session synchronization

4. **Address P1-2: Circular variable expansion detection**
   - Add detection algorithm for circular references in config variable expansion

5. **Plan P1-3: Deprecated fields removal**
   - Plan removal of `mode`, `tools`, `theme`, `keybinds` in v4.0

---

## 12. Iteration Progress

| Iteration | Date | Completion | Key Changes |
|-----------|------|------------|-------------|
| 1 | 2026-04-09 | ~20% | Initial gap analysis |
| 4 | 2026-04-10 | ~35-40% | Major P0 progress |
| 5 | 2026-04-11 | ~70-75% | Desktop/ACP gaps identified |
| 6 | 2026-04-12 | ~80-85% | ACP done, dialogs/slots done, 1 P0 remains |
| 7 | 2026-04-12 | ~80-85% | P1-5 multiline done, P2-6, P2-7, P2-10 done, P2-15 identified as cleanup issue |

---

## 13. Cross-References

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
| Desktop/web interface | [13-desktop-web-interface.md](./13-desktop-web-interface.md) | ACP done, WebView stub only |
| GitHub/GitLab | [14-github-gitlab-integration.md](./14-github-gitlab-integration.md) | GitLab CI, GitHub workflows |
| TUI plugin API | [15-tui-plugin-api.md](./15-tui-plugin-api.md) | Dialogs and slots completed |
| Test plan | [16-test-plan.md](./16-test-plan.md) | Authority tests complete |
| Rust test roadmap | [17-rust-test-roadmap.md](./17-rust-test-roadmap.md) | Per-crate tests in progress |
| Crate test backlog | [18-crate-test-backlog.md](./18-crate-test-backlog.md) | Some backlog addressed |
| Implementation plan | [19-impl-plan.md](./19-impl-plan.md) | This document |
| Iteration 6 Spec | [spec_v6.md](./spec_v6.md) | Previous version |
| Iteration 5 Spec | [spec_v5.md](./spec_v5.md) | Earlier version |

---

## 14. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 7.0 | 2026-04-12 | Updated based on Iteration 7 gap analysis. P1-5 multiline input completed. P2-6 (Per-server OAuth), P2-7 (Context cost warnings), P2-10 (Plugin cleanup) completed. P2-15 (Git test code bugs) identified as cleanup issue rather than feature gap. Added Code Quality Warnings section (CQ-1 through CQ-9). Updated build status with warning counts per crate. Overall completion ~80-85%. |
| 6.0 | 2026-04-12 | Updated based on Iteration 6 gap analysis. P0-new-1 resolved (git syntax fixed), P0-new-3 implemented (ACP transport complete). P1-7, P1-8, P1-10, P1-11 completed. P2-10 completed. P2-15 introduced (git test bugs). P0-new-2 remains as sole P0 blocker. Phase 4 coverage improved to ~70%. Build status: git compiles but tests error. Added FR-026, FR-027, FR-028 for new PRDs. Updated agent table to reflect built-in agents with visibility. |
| 5.0 | 2026-04-11 | Updated based on Iteration 5 gap analysis. Overall completion ~70-75%. Added 3 new P0 blockers (git syntax error, Desktop WebView, ACP transport). Added FR-025 for convention tests. Consolidated P0 trackers from 20 to 3. Reduced P1 from 32 to 11 items. Updated phase statuses to Phase 4-6. |
| 4.0 | 2026-04-10 | Updated based on Iteration 4 gap analysis. Corrected implementation status (was ~85%, now ~35-40%). Added 8 new FRs (FR-015 to FR-024). Expanded P0 blockers to 20 items. |
| 1.0 | 2026-04-09 | Initial version based on PRD and gap analysis |

---

*Document generated: 2026-04-12*
*Iteration: 7*
*Phase: Phase 4-6 of 6 (Interface Implementations, Hardening, Release Qualification)*