# Implementation Plan v6

**Version:** 6.0  
**Generated:** 2026-04-12  
**Based on:** Spec v6 gap analysis  
**Status:** Draft

---

## 1. Current Status Summary

| Metric | Value | Notes |
|--------|-------|-------|
| Overall Completion | ~80-85% | Phase 4-6 of 6 |
| Phase Status | Phase 4-6 | Interface, Hardening, Release Qualification |
| P0 Blockers | 1 | Desktop WebView integration |
| P1 Issues | 6 | Deferred |
| P2 Issues | 14 | Including 1 new (P2-15) |

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
| P0-new-2 | Desktop WebView integration | cli | ❌ Stub | Implement actual WebView component |

**P0-new-2 Details:**
- Current `desktop.rs` only starts HTTP server and opens browser
- Need actual WebView component per PRD 13
- **This is the ONLY remaining P0 blocker**

### P1 - Should Fix Before Release

| ID | Task | Module | Status | Action |
|----|------|--------|--------|--------|
| P1-1 | JSONC error messages clarity | config | ✅ Done | Enhanced error formatting |
| P1-2 | Circular variable expansion detection | config | Deferred | Add detection algorithm |
| P1-3 | Deprecated fields (mode, tools, theme, keybinds) | config | Deferred | Remove in v4.0 |
| P1-5 | Multiline input terminal support | tui | Deferred | Shift+Enter for new line |
| P1-9 | Session sharing between interfaces | cli | Deferred | Cross-interface sync |

### P2 - Nice to Have

| ID | Task | Module | Status | Notes |
|----|------|--------|--------|-------|
| P2-1 | Project VCS worktree root distinction | core | Deferred | |
| P2-2 | Workspace path validation | core | Deferred | |
| P2-3 | Compaction shareability verification | storage | ✅ Done | |
| P2-4 | Deterministic collision resolution | tools | ✅ Done | |
| P2-5 | Result caching invalidation | tools | Deferred | |
| P2-6 | Per-server OAuth verification | mcp | Deferred | |
| P2-7 | Context cost warnings | mcp | Deferred | |
| P2-8 | Experimental LSP tool testing | lsp | Deferred | |
| P2-9 | API error shape consistency | server | Deferred | |
| P2-11 | Shell prefix (`!`) handler | tui | Deferred | |
| P2-12 | Home view completion | tui | Deferred | |
| P2-13 | LLM variant/reasoning budget | llm | Deferred | |
| P2-14 | GitLab Duo experimental marking | git | Deferred | |
| **P2-15** | **Git test code bugs** | **git** | ❌ **NEW** | Fix `next_port` and `Ordering` |

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
| TD-001 | Git test code bugs | HIGH | P2-15 |
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

---

## 6. Immediate Actions (Before Release)

### Must Fix
1. **P0-new-2: Desktop WebView integration** - Implement actual WebView component
2. **P2-15: Git test code bugs** - Fix duplicate test names, add `Ordering` import, define `next_port()`

### Should Fix
3. **P1-5: Multiline input terminal support** - Shift+Enter for new line
4. **P1-9: Session sharing between interfaces** - Cross-interface session synchronization

---

## 7. Iteration History

| Iteration | Date | Completion | Key Changes |
|-----------|------|------------|-------------|
| 1 | 2026-04-09 | ~20% | Initial gap analysis |
| 4 | 2026-04-10 | ~35-40% | Major P0 progress |
| 5 | 2026-04-11 | ~70-75% | Desktop/ACP gaps identified |
| 6 | 2026-04-12 | ~80-85% | ACP done, dialogs/slots done, 1 P0 remains |

---

*Plan generated: 2026-04-12*
*Version: 6.0*
