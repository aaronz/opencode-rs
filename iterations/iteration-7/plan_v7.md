# Implementation Plan v7

**Version:** 7.0  
**Generated:** 2026-04-12  
**Based on:** Spec v7 gap analysis (Iteration 7)  
**Status:** Draft

---

## 1. Current Status Summary

| Metric | Value | Notes |
|--------|-------|-------|
| Overall Completion | ~80-85% | Phase 4-6 of 6 |
| Phase Status | Phase 4-6 | Interface, Hardening, Release Qualification |
| P0 Blockers | 1 | Desktop WebView integration (P0-new-2) |
| P1 Issues | 5 | Deferred |
| P2 Issues | 14 | Including 1 bug (P2-15) |

---

## 2. Implementation Phases

### Phase 0: Project Foundation ✅ Complete (100%)
- Workspace structure
- Build system
- CI pipeline

### Phase 1: Authority Implementation ✅ Complete (~98%)
- Core entity model (Project, Session, Message, Part)
- Storage layer
- Configuration system
- Server API

### Phase 2: Runtime Core ✅ Complete (~98%)
- Agent system (primary/subagent)
- Tools system
- Plugin system
- TUI Plugin API

### Phase 3: Infrastructure Subsystems ✅ Complete (~95%)
- MCP integration
- LSP integration
- Provider/model system
- Formatters
- Skills system

### Phase 4: Interface Implementations 🚧 In Progress (~70%)
- Desktop/Web/ACP interface ⚠️ (WebView P0 blocker)
- GitHub/GitLab integration

### Phase 5: Hardening ✅ Complete (~95%)
- Compatibility suite
- Convention tests
- Non-functional tests

### Phase 6: Release Qualification 🚧 Partial (~70%)
- Final testing
- Documentation
- Release preparation

---

## 3. Priority Tasks

### P0 - Must Fix Before Release

| ID | Task | Module | Status | Action |
|----|------|--------|--------|--------|
| **P0-new-2** | **Desktop WebView integration** | **cli** | ❌ **STUB** | **Implement actual WebView component** |

**P0-new-2 Details:**
- Current `desktop.rs` uses `wry` for WebView but only spawns browser when `desktop` feature is off
- When `desktop` feature is enabled, `spawn_webview_thread` creates a WebView but doesn't properly integrate with the app lifecycle
- Need actual WebView component per PRD 13 that shares state with the TUI/server
- **This is the ONLY remaining P0 blocker**

### P1 - Should Fix Before Release

| ID | Task | Module | Status | Action |
|----|------|--------|--------|--------|
| P1-1 | JSONC error messages clarity | config | ✅ Done | Enhanced error formatting |
| P1-2 | Circular variable expansion detection | config | Deferred | Add detection algorithm |
| P1-3 | Deprecated fields (mode, tools, theme, keybinds) | config | ✅ **Done** | Added #[deprecated] attrs, check_deprecated_fields() |
| P1-5 | Multiline input terminal support | tui | ✅ **Done** | Shift+Enter for new line |
| P1-9 | Session sharing between interfaces | cli | Deferred | Cross-interface sync |

### P2 - Nice to Have

| ID | Task | Module | Status | Notes |
|----|------|--------|--------|-------|
| P2-1 | Project VCS worktree root distinction | core | Deferred | |
| P2-2 | Workspace path validation | core | ✅ Done | |
| P2-3 | Compaction shareability verification | storage | ✅ Done | |
| P2-4 | Deterministic collision resolution | tools | ✅ Done | |
| P2-5 | Result caching invalidation | tools | ✅ Done | |
| P2-6 | Per-server OAuth token storage | mcp | ✅ **Done** | Iteration 7 |
| P2-7 | Context cost warnings | mcp | ✅ **Done** | Iteration 7 (context_cost.rs) |
| P2-8 | Experimental LSP tool testing | lsp | Deferred | |
| P2-9 | API error shape consistency | server | Deferred | |
| P2-10 | Plugin cleanup/unload | plugin | ✅ **Done** | Iteration 7 |
| P2-11 | Shell prefix (`!`) handler | tui | ✅ **Done** | Iteration 7 |
| P2-12 | Home view completion | tui | Deferred | |
| P2-13 | LLM variant/reasoning budget | llm | Deferred | |
| P2-14 | GitLab Duo experimental marking | git | Deferred | Marked experimental in docs |
| **P2-15** | **Git test code bugs** | **git** | ❌ **BUG** | Cleanup issue, not feature gap |

---

## 4. Release Gates Status

| Gate | Criteria | Status |
|------|----------|--------|
| Phase 0 | Workspace builds, tests run, clippy clean | ✅ |
| Phase 1 | Authority tests green (01, 06, 07) | ✅ |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | ✅ |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | ✅ |
| Phase 4 | Interface smoke workflows pass (13, 14) | 🚧 Blocked by P0-new-2 |
| Phase 5a | Compatibility suite green | ✅ |
| Phase 5b | Conventions suite green | ✅ |
| Phase 6 | Non-functional baselines recorded | 🚧 Partial |

---

## 5. Technical Debt

| ID | Item | Severity | Status |
|----|------|----------|--------|
| TD-001 | Git test code bugs | HIGH | P2-15 (cleanup) |
| TD-002 | Desktop WebView stub | P0 | P0-new-2 |
| TD-003 | Deprecated `mode` field | Medium | Deferred |
| TD-004 | Deprecated `tools` field | Medium | Deferred |
| TD-005 | Deprecated `theme` field | Low | Deferred |
| TD-006 | Deprecated `keybinds` field | Low | Deferred |
| TD-007 | Magic numbers in compaction | Low | Deferred |
| TD-008 | Custom JSONC parser | Medium | Deferred |
| TD-009 | `#[serde(other)]` in Part | Low | Deferred |
| TD-010 | Unused `SecretStorage` methods | Low | Deferred |
| TD-011 | `unreachable_patterns` warning | Low | Deferred |
| TD-012 | Unused imports in core | Low | Deferred |
| TD-013 | Unused variable `e` in lsp_tool | Low | Deferred |

---

## 6. Code Quality Warnings (CQ)

| ID | Item | File | Severity | Fix |
|----|------|------|----------|-----|
| CQ-1 | Unused `Message` import | core/crash_recovery.rs:1 | Low | Remove unused import |
| CQ-2 | Unused `SecretStorage` methods | core/config/secret_storage.rs:36 | Low | Remove or use 6 methods |
| CQ-3 | Unused `e` variable | tools/lsp_tool.rs:311,526,626,783 | Low | Prefix with `_e` |
| CQ-4 | Unused `body` variable | git/github.rs:566 | Low | Prefix with `_body` |
| CQ-5 | Unused `next_port` function | git/gitlab_ci.rs:413 | Low | Remove or use |
| CQ-6 | Unused `GitLabMockServer` | git/gitlab_ci.rs:706 | Low | Remove or use |
| CQ-7 | Unused imports | cli/src/cmd/quick.rs:5-6 | Low | Remove unused |
| CQ-8 | Unused `save_session_records` | cli/src/cmd/session.rs:42 | Low | Remove or use |
| CQ-9 | Unused `complete` variable | cli/src/cmd/mcp_auth.rs:216 | Low | Prefix with `_complete` |

---

## 7. Immediate Actions (Before Release)

### Must Fix (P0)
1. **P0-new-2: Desktop WebView integration**
   - Implement actual WebView component per PRD 13
   - Connect WebView to desktop mode lifecycle
   - Share state with TUI/server
   - **This is the ONLY remaining P0 blocker**

### Should Fix (P1)
2. **P2-15: Git test code cleanup** (low priority - cleanup issue)
   - Remove unused `next_port()` function or use it
   - Remove or use `GitLabMockServer` struct
   - Clean up dead test code

3. **P1-9: Session sharing between interfaces**
   - Cross-interface session synchronization

4. **P1-2: Circular variable expansion detection**
   - Add detection algorithm for circular references

5. **P1-3: Deprecated fields removal planning**
   - Plan removal of `mode`, `tools`, `theme`, `keybinds` in v4.0

---

## 8. Iteration History

| Iteration | Date | Completion | Key Changes |
|-----------|------|------------|-------------|
| 1 | 2026-04-09 | ~20% | Initial gap analysis |
| 4 | 2026-04-10 | ~35-40% | Major P0 progress |
| 5 | 2026-04-11 | ~70-75% | Desktop/ACP gaps identified |
| 6 | 2026-04-12 | ~80-85% | ACP done, dialogs/slots done, 1 P0 remains |
| 7 | 2026-04-12 | ~80-85% | P1-5 multiline done, P2-6, P2-7, P2-10 done, P2-15 identified |

---

## 9. Changes Since v6

| Category | Change |
|----------|--------|
| P1 Completed | P1-5 Multiline input terminal support ✅ |
| P2 Completed | P2-6 Per-server OAuth token storage ✅ |
| P2 Completed | P2-7 Context cost warnings (context_cost.rs) ✅ |
| P2 Completed | P2-10 Plugin cleanup/unload ✅ |
| P2 Reclassified | P2-15 Git test code bugs → identified as cleanup issue, not feature gap |
| New | Added Code Quality Warnings section (CQ-1 through CQ-9) |
| Build | All crates compile, opencode-git tests have warnings not errors |

---

*Plan generated: 2026-04-12*
*Version: 7.0*
*Priority: P0-new-2 (Desktop WebView) is the ONLY remaining P0 blocker*