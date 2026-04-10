# OpenCode Rust Port — Implementation Plan v4

**Version:** 4.0  
**Generated:** 2026-04-10  
**Based on:** Spec v4 & Gap Analysis (Iteration 4)  
**Status:** Draft

---

## 1. Executive Summary

This plan addresses the critical P0 blockers identified in the Gap Analysis for the OpenCode Rust port. With ~35-40% overall completion, the focus for this iteration is on:

1. **Unblocking Phase 2** — Agent invariants, tool registry, plugin hooks
2. **Completing Phase 1** — Config ownership, Server API CRUD
3. **Starting Phase 3** — MCP/LSP transport implementation

**Key Constraint:** P0 items must be resolved before dependent P1/P2 work can proceed.

---

## 2. P0 Blockers Summary (20 Items)

### Critical Path Analysis

```
Phase 1 Completeness (< 60% → 85%)
├── P0-11: Config precedence enforcement
├── P0-12: Config ownership boundary
├── P0-13: Route registration by resource group
├── P0-14: Auth enforcement per endpoint
└── P0-15: Session/message lifecycle CRUD

Phase 2 Completeness (< 50% → 75%)
├── P0-1:  Exactly one active primary agent invariant
├── P0-2:  Subagent execution (child context, result handoff)
├── P0-3:  Task/delegation mechanism
├── P0-4:  Custom tool registration with ToolRegistry
├── P0-5:  Custom tool discovery format (.ts/.js)
├── P0-16: Plugin hook execution order (HashMap → IndexMap)
├── P0-17: Plugin-provided tool registration
├── P0-18: Plugin config separation
├── P0-19: TUI plugin `plugin_enabled` semantics
└── P0-20: TUI plugin activate/deactivate

Phase 3 Start (0% → 20%)
├── P0-6:  Local MCP server connection
├── P0-7:  Remote MCP server connection
├── P0-8:  Tool discovery from MCP servers
├── P0-9:  Built-in LSP server detection
└── P0-10: Diagnostics retrieval from LSP
```

---

## 3. Sprint Breakdown

### Sprint 0: Critical Path Unblock (Week 1-2)

**Goal:** Fix the most blocking issues that prevent parallel work.

| ID | Task | Module | Dependencies | Effort |
|----|------|--------|--------------|--------|
| P0-16 | Fix plugin hooks order (HashMap → IndexMap) | plugin | None | 2h |
| P0-12 | Enforce config ownership boundary | config | None | 4h |
| P0-4  | Connect custom tool discovery to ToolRegistry | tools | P0-5 | 8h |
| P0-5  | Fix custom tool discovery format (.ts/.js) | tools | None | 4h |

**Rationale:** P0-16 is a simple fix (TD-001) that unblocks plugin testing. P0-12 is isolated. P0-4/5 are paired.

---

### Sprint 1: Phase 1 Completion (Week 2-4)

**Goal:** Complete authority contracts (Config, Server API).

| ID | Task | Module | Dependencies | Effort |
|----|------|--------|--------------|--------|
| P0-11 | Config precedence enforcement | config | None | 8h |
| P0-13 | Route registration by resource group | server | None | 8h |
| P0-14 | Auth enforcement per endpoint | server | P0-13 | 8h |
| P0-15 | Session/message lifecycle CRUD | server | P0-13 | 12h |

**Exit Criteria:** Phase 1 authority tests can run.

---

### Sprint 2: Phase 2 Core (Week 4-6)

**Goal:** Complete agent system and tool registry.

| ID | Task | Module | Dependencies | Effort |
|----|------|--------|--------------|--------|
| P0-1  | Exactly one active primary agent invariant | agent | None | 8h |
| P0-2  | Subagent execution (child context, result handoff) | agent | P0-1 | 16h |
| P0-3  | Task/delegation mechanism | agent | P0-2 | 12h |
| P0-17 | Plugin-provided tool registration | plugin | P0-16 | 8h |
| P0-18 | Plugin config separation | plugin | P0-12 | 4h |

**Exit Criteria:** Phase 2 runtime tests can run.

---

### Sprint 3: Phase 2 TUI Plugin (Week 6-8)

**Goal:** Complete TUI plugin API surface.

| ID | Task | Module | Dependencies | Effort |
|----|------|--------|--------------|--------|
| P0-19 | TUI plugin `plugin_enabled` semantics | tui | None | 8h |
| P0-20 | TUI plugin activate/deactivate | tui | P0-19 | 12h |

**Exit Criteria:** TUI plugin lifecycle tests can run.

---

### Sprint 4: Phase 3 Infrastructure (Week 8-12)

**Goal:** Implement MCP/LSP transport layer.

| ID | Task | Module | Dependencies | Effort |
|----|------|--------|--------------|--------|
| P0-6  | Local MCP server connection | mcp | None | 24h |
| P0-7  | Remote MCP server connection | mcp | P0-6 | 16h |
| P0-8  | Tool discovery from MCP servers | mcp | P0-6, P0-7 | 12h |
| P0-9  | Built-in LSP server detection | lsp | None | 16h |
| P0-10 | Diagnostics retrieval from LSP | lsp | P0-9 | 12h |

**Exit Criteria:** Phase 3 subsystem tests can run.

---

## 4. P1 Continuation (Parallel Tracks)

While P0 work is sequential on the critical path, these can proceed in parallel:

### P1-A: TUI System (No P0 dependencies)
- P1-14: Slash commands
- P1-15: Input model (multiline, history, autocomplete)
- P1-16: File references (`@`) fuzzy search
- P1-17: Keybinding system
- P1-18: Sidebar sections

### P1-B: LLM/Provider (No P0 dependencies)
- P1-25: Provider abstraction
- P1-26: Default model selection precedence
- P1-27: Per-agent model override
- P1-28: Local model providers

### P1-C: Skills System (No P0 dependencies)
- P1-29: Skills compat paths (Claude/Agent-style)

### P1-D: Server API (After P0-13,14,15)
- P1-10: Request validation
- P1-11: Streaming endpoints

---

## 5. Technical Debt Fixes (Embedded in P0)

| TD-ID | Fix | Embedded In | Status |
|-------|-----|-------------|--------|
| TD-001 | HashMap → IndexMap (hook order) | P0-16 | Sprint 0 |
| TD-002 | Scanner: TOOL.md → .ts/.js | P0-5 | Sprint 0 |
| TD-003 | Discovery → Registry connection | P0-4 | Sprint 0 |
| TD-004 | Config ownership boundary | P0-12 | Sprint 0 |
| TD-006 | MCP transport implementation | P0-6, P0-7 | Sprint 4 |

---

## 6. Release Gates

| Gate | Criteria | Blocking Items |
|------|----------|----------------|
| Phase 1 | Authority tests green (01, 06, 07) | P0-11, P0-12, P0-13, P0-14, P0-15 |
| Phase 2 | Runtime tests green (02, 03, 08, 15) | P0-1, P0-2, P0-3, P0-4, P0-5, P0-16, P0-17, P0-18, P0-19, P0-20 |
| Phase 3 | Subsystem tests green (04, 05, 10, 11, 12) | P0-6, P0-7, P0-8, P0-9, P0-10 |

---

## 7. Resource Allocation (Suggested)

| Sprint | Focus | Team |
|--------|-------|------|
| Sprint 0 | Critical Path Unblock | 1 engineer |
| Sprint 1 | Phase 1 Completion | 1 engineer |
| Sprint 2 | Phase 2 Core | 1-2 engineers |
| Sprint 3 | Phase 2 TUI Plugin | 1 engineer |
| Sprint 4 | Phase 3 Infrastructure | 1-2 engineers |

**Note:** P1 parallel tracks (A, B, C, D) can run concurrently with any sprint.

---

## 8. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| MCP transport complexity underestimated | High | High | Prototype spike in Sprint 4 week 1 |
| Plugin-provided tools design instability | Medium | Medium | Design review before P0-17 |
| LSP diagnostics API varies by server | High | Medium | Abstract behind trait |

---

## 9. Success Criteria

- [ ] All 20 P0 blockers resolved
- [ ] Phase 1 gate: Authority tests pass
- [ ] Phase 2 gate: Runtime tests pass  
- [ ] Phase 3 gate: Subsystem tests can run
- [ ] Zero new P0 items introduced

---

## 10. Cross-References

- [Spec v4](./spec_v4.md)
- [Gap Analysis](./gap-analysis.md)
- [Constitution Updates](./constitution_updates.md)

---

*Plan version: 4.0*  
*Iteration: 4*  
*Last updated: 2026-04-10*