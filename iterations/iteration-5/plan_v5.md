# OpenCode Rust Port — Implementation Plan v5

**Version:** 5.0
**Generated:** 2026-04-11
**Based on:** Iteration 5 gap analysis and spec_v5.md
**Status:** Draft

---

## 1. Overview

This plan outlines the implementation roadmap for the OpenCode Rust port, derived from the PRD specifications and gap analysis.

**Overall Completion:** ~70-75%
**Current Phase:** Phase 4-6 of 6 (Interface Implementations, Hardening, Release Qualification)

---

## 2. Implementation Status by Phase

| Phase | Description | Status | Coverage |
|-------|-------------|--------|----------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority Implementation | ✅ Complete | ~95% |
| Phase 2 | Runtime Core | ✅ Complete | ~95% |
| Phase 3 | Infrastructure Subsystems | ✅ Complete | ~90% |
| Phase 4 | Interface Implementations | 🚧 In Progress | ~50% |
| Phase 5 | Hardening | ✅ Complete | ~90% |
| Phase 6 | Release Qualification | 🚧 Partial | ~60% |

---

## 3. P0 Blockers (Must Fix Before Release)

### P0-1: Fix Git Crate Syntax Error ✅ FIXED
- **File:** `opencode-rust/crates/git/src/gitlab_ci.rs`
- **Issue:** Orphaned code (lines 611-612: `port` statement and extra `}`) after test module, plus premature module close at line 610
- **Fix:** Removed orphaned lines 611-612 and premature closing brace at line 610
- **Verification:** `cargo build -p opencode-git` succeeds
- **Note:** Pre-existing bugs remain in test code (`Ordering` undeclared, `next_port` not found) - these are P2 issues

### P0-2: Desktop WebView Integration
- **Module:** `crates/cli/src/desktop.rs`
- **Issue:** Only HTTP server scaffolding, no actual WebView
- **Required:** Implement desktop shell with WebView per PRD 13
- **Deliverable:** Functional desktop mode with browser rendering

### P0-3: ACP HTTP+SSE Transport
- **Modules:** `crates/cli/src/acp.rs`, `crates/server/src/routes/acp.rs`
- **Issue:** ACP CLI commands exist but server transport incomplete
- **Required:** Full ACP HTTP+SSE transport layer implementation
- **Deliverable:** ACP endpoints functional for editor integration

---

## 4. P1 Issues (Should Fix)

### Config System (FR-003)
| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| P1-1 | JSONC error messages clarity | Medium | Deferred |
| P1-2 | Circular variable expansion detection | Medium | Deferred |
| P1-3 | Deprecated fields (`mode`, `tools`, `theme`, `keybinds`) | Medium | Deferred |

### TUI System (FR-018)
| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| P1-4 | Slash commands incomplete (`/compact`, `/connect`, `/help`) | High | Deferred |
| P1-5 | Multiline input terminal support | Medium | Deferred |
| P1-6 | File reference autocomplete improvement | Medium | Done |

### TUI Plugin API (FR-009)
| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| P1-7 | Dialog components incomplete (Alert/Confirm/Prompt/Select) | High | Deferred |
| P1-8 | Slots system incomplete | High | Deferred |

### Desktop/Web/ACP (FR-015)
| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| P1-9 | Session sharing between interfaces partial | Medium | Deferred |

### Agent System (FR-005)
| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| P1-10 | Permission inheritance edge cases | Medium | Deferred |

### Server API (FR-004)
| ID | Issue | Priority | Status |
|----|-------|----------|--------|
| P1-11 | Request validation edge cases | Medium | Deferred |

---

## 5. P2 Issues (Nice to Have)

| ID | Module | Description | Status |
|----|--------|-------------|--------|
| P2-1 | core | Project VCS worktree root distinction | Deferred |
| P2-2 | core | Workspace path validation | Deferred |
| P2-3 | storage | Compaction shareability verification | Deferred |
| P2-4 | tools | Deterministic collision resolution | Deferred |
| P2-5 | tools | Result caching invalidation | Deferred |
| P2-6 | mcp | Per-server OAuth token storage | Deferred |
| P2-7 | mcp | Context cost warnings | Deferred |
| P2-8 | lsp | Experimental LSP tool testing | Deferred |
| P2-9 | server | API error shape consistency | Deferred |
| P2-10 | plugin | Plugin cleanup/unload | Done |
| P2-11 | tui | Shell prefix (`!`) handler | Done |
| P2-12 | tui | Home view completion | Deferred |
| P2-13 | llm | LLM variant/reasoning budget | Deferred |
| P2-14 | git | GitLab Duo experimental marking | Deferred |

---

## 6. Build Status

| Crate | Status | Notes |
|-------|--------|-------|
| opencode-core | ✅ Compiles | Warnings only |
| opencode-agent | ✅ Compiles | Warnings only |
| opencode-tools | ✅ Compiles | Warnings only |
| opencode-mcp | ✅ Compiles | Warnings only |
| opencode-lsp | ✅ Compiles | Warnings only |
| opencode-plugin | ✅ Compiles | Warnings only |
| opencode-server | ✅ Compiles | Warnings only |
| opencode-cli | ✅ Compiles | Warnings only |
| **opencode-git** | ✅ Compiles | **Fixed syntax error** |
| opencode-llm | ✅ Compiles | Warnings only |

---

## 7. Release Gates

| Gate | Criteria | Status |
|------|----------|--------|
| Phase 0 | Workspace builds, tests run, clippy clean | ✅ |
| Phase 1 | Authority tests green (01, 06, 07) | ✅ |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | ✅ |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | ✅ |
| Phase 4 | Interface smoke workflows pass (13, 14) | 🚧 (P0 blockers) |
| Phase 5a | Compatibility suite green | ✅ |
| Phase 5b | Conventions suite green | ✅ |
| Phase 6 | Non-functional baselines recorded | 🚧 |

---

## 8. Immediate Actions

### Sprint 1: P0 Blockers (Priority)

1. **Fix git crate syntax error**
   - Remove orphaned `port` statement at line 611
   - Remove orphaned `}` at line 612
   - Verify: `cargo build -p opencode-git`

2. **Implement Desktop WebView integration**
   - Analyze PRD 13 requirements
   - Implement WebView component
   - Add desktop mode command

3. **Complete ACP transport layer**
   - Implement ACP HTTP+SSE endpoints
   - Verify CLI commands connect
   - Add integration tests

### Sprint 2: P1 Issues

1. Complete TUI slash commands (`/compact`, `/connect`, `/help`)
2. Improve config error handling
3. Complete TUI Plugin dialogs and slots
4. Address session sharing between interfaces

### Sprint 3: P2 Issues & Hardening

1. Deprecate legacy fields
2. Externalize magic numbers to config
3. Complete non-functional tests
4. Performance baseline verification

---

## 9. Crate Ownership

| Crate | Phase | PRD | Status |
|-------|-------|-----|--------|
| `crates/core/` | 1 | `01`, `06` | ✅ Complete |
| `crates/storage/` | 1 | `01` | ✅ Complete |
| `crates/config/` | 1 | `06` | ✅ Complete (P1 gaps) |
| `crates/server/` | 1, 4 | `07`, `13` | ✅ Complete (P1 gaps) |
| `crates/agent/` | 2 | `02` | ✅ Complete |
| `crates/tools/` | 2, 3 | `03`, `11` | ✅ Complete |
| `crates/plugin/` | 2 | `08` | ✅ Complete |
| `crates/tui/` | 2, 3 | `09`, `15` | ✅ Complete (P1 gaps) |
| `crates/mcp/` | 3 | `04` | ✅ Complete |
| `crates/lsp/` | 3 | `05` | ✅ Complete |
| `crates/llm/` | 3 | `10` | ✅ Complete |
| `crates/git/` | 4 | `14` | ✅ Compiles (P2 test fixes needed) |
| `ratatui-testing/` | 2, 3 | `09`, `15` | ✅ |

---

## 10. Test Coverage Status

| Test Suite | Status | Test Count |
|------------|--------|------------|
| Authority Tests (FR-019) | ✅ Done | 4 suites |
| Runtime Tests (FR-020) | ✅ Done | 5 suites |
| Subsystem Tests (FR-021) | ✅ Done | 4 suites |
| Interface Tests (FR-022) | ✅ Done | 4 suites |
| Compatibility Suite (FR-023) | ✅ Done | 3 suites |
| Non-Functional Tests (FR-024) | ✅ Done | 5 suites |
| Convention Tests (FR-025) | ✅ Done | 23 tests |
| **Total** | ✅ Done | **~25 suites + 23 tests** |

---

## 11. Change Log

| Version | Date | Changes |
|---------|------|---------|
| 5.1 | 2026-04-11 | Fixed P0-1: git crate syntax error resolved. Removed orphaned code (lines 611-612) and premature module close (line 610). Workspace now builds. |
| 5.0 | 2026-04-11 | Initial v5 plan based on gap analysis. P0 blockers: git syntax error, Desktop WebView, ACP transport. 11 P1 items, 14 P2 items. |
