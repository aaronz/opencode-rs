# OpenCode Rust Port — Task List v4

**Version:** 4.0  
**Generated:** 2026-04-10  
**Based on:** Spec v4 & Gap Analysis (Iteration 4)  
**Status:** Draft

---

## P0 Blockers (20 Items) — Must Fix

### Phase 1: Authority Implementation

| Task ID | Title | Module | FR | Sprint | Status | Dependencies |
|---------|-------|--------|-----|--------|--------|--------------|
| P0-11 | Config precedence enforcement | config | FR-003 | Sprint 1 | Done | None |
| P0-12 | Config ownership boundary (opencode.json vs tui.json) | config | FR-003 | Sprint 0 | Done | None |
| P0-13 | Route registration by resource group | server | FR-004 | Sprint 1 | Done | None |
| P0-14 | Auth enforcement per endpoint | server | FR-004 | Sprint 1 | Done | P0-13 |
| P0-15 | Session/message lifecycle CRUD | server | FR-004 | Sprint 1 | Pending | P0-13 |

### Phase 2: Runtime Core

| Task ID | Title | Module | FR | Sprint | Status | Dependencies |
|---------|-------|--------|-----|--------|--------|--------------|
| P0-1 | Exactly one active primary agent invariant | agent | FR-005 | Sprint 2 | Done | None |
| P0-2 | Subagent execution (child context, result handoff) | agent | FR-005 | Sprint 2 | Pending | P0-1 |
| P0-3 | Task/delegation mechanism | agent | FR-005 | Sprint 2 | Pending | P0-2 |
| P0-4 | Custom tool registration with ToolRegistry | tools | FR-006, FR-007 | Sprint 0 | Pending | P0-5 |
| P0-5 | Custom tool discovery format (.ts/.js) | tools | FR-007 | Sprint 0 | Pending | None |
| P0-16 | Plugin hook execution order (HashMap → IndexMap) | plugin | FR-008 | Sprint 0 | Pending | None |
| P0-17 | Plugin-provided tool registration | plugin | FR-008 | Sprint 2 | Pending | P0-16 |
| P0-18 | Plugin config separation | plugin | FR-008 | Sprint 2 | Pending | P0-12 |
| P0-19 | TUI plugin `plugin_enabled` semantics | tui | FR-009 | Sprint 3 | Pending | None |
| P0-20 | TUI plugin activate/deactivate | tui | FR-009 | Sprint 3 | Pending | P0-19 |

### Phase 3: Infrastructure Subsystems

| Task ID | Title | Module | FR | Sprint | Status | Dependencies |
|---------|-------|--------|-----|--------|--------|--------------|
| P0-6 | Local MCP server connection | mcp | FR-010 | Sprint 4 | Pending | None |
| P0-7 | Remote MCP server connection | mcp | FR-010 | Sprint 4 | Pending | P0-6 |
| P0-8 | Tool discovery from MCP servers | mcp | FR-010 | Sprint 4 | Pending | P0-6, P0-7 |
| P0-9 | Built-in LSP server detection | lsp | FR-011 | Sprint 4 | Pending | None |
| P0-10 | Diagnostics retrieval from LSP | lsp | FR-011 | Sprint 4 | Pending | P0-9 |

---

## P1 Issues (32 Items) — Should Fix

### Core Architecture (4 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-1 | Ownership invariant automated tests | core | FR-001 | Phase 5 | Pending | FR-001 complete |
| P1-2 | Stable ID semantics tests | core | FR-001 | Phase 5 | Pending | FR-001 complete |
| P1-3 | Snapshot/Revert model tests | storage | FR-002 | Phase 5 | Pending | FR-002 complete |

### Agent System (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-4 | Hidden vs visible agent behavior | agent | FR-005 | Phase 2 | Pending | P0-1 |
| P1-5 | Permission inheritance (parent→subagent) | agent | FR-005 | Phase 2 | Pending | P0-2 |

### Tools System (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-6 | MCP tool qualification | tools | FR-006 | Phase 3 | Pending | P0-8 |

### Configuration System (4 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-7 | JSONC parsing | config | FR-003 | Phase 1 | Pending | None |
| P1-8 | Variable expansion | config | FR-003 | Phase 1 | Pending | None |
| P1-9 | `tools` → `permission` alias tests | config | FR-003 | Phase 5 | Pending | P0-11 |

### Server API (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-10 | Request validation | server | FR-004 | Phase 1 | Pending | P0-15 |
| P1-11 | Streaming endpoints | server | FR-004 | Phase 1 | Pending | P0-15 |

### LSP System (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-12 | Custom LSP registration | lsp | FR-011 | Phase 3 | Pending | P0-9 |
| P1-13 | LSP failure handling | lsp | FR-011 | Phase 3 | Pending | P0-10 |

### TUI System (5 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-14 | Slash commands | tui | FR-018 | Phase 2 | Pending | None |
| P1-15 | Input model (multiline, history, autocomplete) | tui | FR-018 | Phase 2 | Pending | None |
| P1-16 | File references (`@`) fuzzy search | tui | FR-018 | Phase 2 | Pending | None |
| P1-17 | Keybinding system | tui | FR-018 | Phase 2 | Pending | None |
| P1-18 | Sidebar sections | tui | FR-018 | Phase 2 | Pending | None |

### TUI Plugin API (6 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-19 | TUI plugin commands API | tui | FR-009 | Phase 2 | Pending | P0-20 |
| P1-20 | TUI plugin routes API | tui | FR-009 | Phase 2 | Pending | P0-20 |
| P1-21 | TUI plugin theme API | tui | FR-009 | Phase 2 | Pending | P0-20 |
| P1-22 | TUI plugin events API | tui | FR-009 | Phase 2 | Pending | P0-20 |
| P1-23 | TUI plugin state API | tui | FR-009 | Phase 2 | Pending | P0-20 |
| P1-24 | TUI plugin onDispose lifecycle | tui | FR-009 | Phase 2 | Pending | P0-20 |

### LLM/Provider System (4 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-25 | Provider abstraction | llm | FR-012 | Phase 3 | Pending | None |
| P1-26 | Default model selection precedence | llm | FR-012 | Phase 3 | Pending | P1-25 |
| P1-27 | Per-agent model override | llm | FR-012 | Phase 3 | Pending | P1-25 |
| P1-28 | Local model providers | llm | FR-012 | Phase 3 | Pending | P1-25 |

### Skills System (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-29 | Skills compat paths (Claude/Agent-style) | core | FR-014 | Phase 3 | Pending | None |

### Desktop/Web/ACP (3 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-30 | Desktop/web session sharing | cli | FR-015 | Phase 4 | Pending | Phase 4 P0s |
| P1-31 | Desktop/web sharing modes | cli | FR-015 | Phase 4 | Pending | P1-30 |

### GitHub Integration (3 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P1-32 | GitHub workflow triggers | git | FR-016 | Phase 4 | Pending | Phase 4 start |

---

## P2 Issues (16 Items) — Nice to Have

### Core Architecture (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-1 | Compaction with shareability verification | storage | FR-002 | Phase 5 | Pending | FR-002 complete |
| P2-2 | Workspace path validation | core | FR-001 | Phase 1 | Pending | None |

### Tools System (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-3 | Deterministic collision resolution | tools | FR-006 | Phase 3 | Pending | P0-4 |
| P2-4 | Result caching for safe tools | tools | FR-006 | Phase 3 | Pending | P0-4 |

### MCP System (3 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-5 | MCP per-server OAuth | mcp | FR-010 | Phase 3 | Pending | P0-8 |
| P2-6 | MCP timeout handling | mcp | FR-010 | Phase 3 | Pending | P0-6 |
| P2-7 | MCP context cost warnings | mcp | FR-010 | Phase 3 | Pending | P0-8 |

### LSP System (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-8 | Experimental LSP tool | lsp | FR-011 | Phase 3 | Pending | P0-10 |

### Configuration System (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-9 | Config auth/secret storage | config | FR-003 | Phase 1 | Pending | P0-12 |

### Server API (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-10 | Server API error shape consistency | server | FR-004 | Phase 1 | Pending | P0-15 |

### Plugin System (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-11 | Plugin cleanup/unload | plugin | FR-008 | Phase 2 | Pending | P0-17 |

### TUI System (2 items)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-12 | Shell prefix (`!`) handling | tui | FR-018 | Phase 2 | Pending | P1-14 |
| P2-13 | Home view | tui | FR-018 | Phase 2 | Pending | None |

### LLM/Provider System (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-14 | LLM variant/reasoning budget | llm | FR-012 | Phase 3 | Pending | P1-27 |

### Skills System (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-15 | Skills permission restrictions | core | FR-014 | Phase 3 | Pending | P1-29 |

### GitLab Integration (1 item)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| P2-16 | GitLab CI component | git | FR-017 | Phase 4 | Pending | FR-016 complete |

---

## Test Infrastructure (Phase 5-6)

### Authority Document Tests (FR-019)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| T-019-1 | Core ownership tree tests (unit + integration) | core | FR-019 | Phase 5 | Pending | P1-1 |
| T-019-2 | Config precedence merge tests | config | FR-019 | Phase 5 | Pending | P0-11 |
| T-019-3 | API route-group tests | server | FR-019 | Phase 5 | Pending | P0-13 |
| T-019-4 | Session/message lifecycle tests | server | FR-019 | Phase 5 | Pending | P0-15 |

### Runtime Architecture Tests (FR-020)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| T-020-1 | Agent primary invariant tests (exactly one active) | agent | FR-020 | Phase 5 | Pending | P0-1 |
| T-020-2 | Subagent execution tests | agent | FR-020 | Phase 5 | Pending | P0-2 |
| T-020-3 | Tool registry tests | tools | FR-020 | Phase 5 | Pending | P0-4 |
| T-020-4 | Plugin hook order tests | plugin | FR-020 | Phase 5 | Pending | P0-16 |
| T-020-5 | TUI plugin lifecycle tests | tui | FR-020 | Phase 5 | Pending | P0-20 |

### Subsystem Tests (FR-021)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| T-021-1 | MCP integration tests | mcp | FR-021 | Phase 5 | Pending | P0-8 |
| T-021-2 | LSP integration tests | lsp | FR-021 | Phase 5 | Pending | P0-10 |
| T-021-3 | Provider/model tests | llm | FR-021 | Phase 5 | Pending | P1-28 |
| T-021-4 | Skills discovery tests | core | FR-021 | Phase 5 | Pending | P1-29 |

### Interface Tests (FR-022)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| T-022-1 | Desktop/web smoke tests | cli | FR-022 | Phase 6 | Pending | Phase 4 P0s |
| T-022-2 | ACP handshake tests | cli | FR-022 | Phase 6 | Pending | ACP transport |
| T-022-3 | GitHub workflow tests | git | FR-022 | Phase 6 | Pending | P1-32 |
| T-022-4 | GitLab integration tests | git | FR-022 | Phase 6 | Pending | P2-16 |

### Compatibility Suite (FR-023)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| T-023-1 | `tools` alias regression suite | config | FR-023 | Phase 5 | Pending | P1-9 |
| T-023-2 | Skill path regression suite | core | FR-023 | Phase 5 | Pending | P1-29 |
| T-023-3 | Plugin ownership boundary suite | plugin | FR-023 | Phase 5 | Pending | P0-18 |

### Non-Functional Tests (FR-024)

| Task ID | Title | Module | FR | Phase | Status | Dependencies |
|---------|-------|--------|-----|-------|--------|--------------|
| T-024-1 | Performance baselines | all | FR-024 | Phase 6 | Pending | All P0s |
| T-024-2 | Security tests | all | FR-024 | Phase 6 | Pending | Phase 5 |
| T-024-3 | Recovery tests | storage | FR-024 | Phase 6 | Pending | T-019-4 |
| T-024-4 | Crash recovery | storage | FR-024 | Phase 6 | Pending | T-024-3 |
| T-024-5 | Snapshot/revert durability | storage | FR-024 | Phase 6 | Pending | P1-3 |

---

## Summary Statistics

| Category | Count | Completed | Pending |
|----------|-------|-----------|---------|
| P0 Blockers | 20 | 0 | 20 |
| P1 Issues | 32 | 0 | 32 |
| P2 Issues | 16 | 0 | 16 |
| Test Infrastructure | 22 | 0 | 22 |
| **Total** | **90** | **0** | **90** |

---

## Sprint Assignment Summary

| Sprint | P0 Tasks | Key Deliverables |
|--------|----------|------------------|
| Sprint 0 | P0-12, P0-16, P0-4, P0-5 | Config boundary, Hook order, Tool registry gap |
| Sprint 1 | P0-11, P0-13, P0-14, P0-15 | Config precedence, Server API CRUD |
| Sprint 2 | P0-1, P0-2, P0-3, P0-17, P0-18 | Agent invariants, Subagent execution |
| Sprint 3 | P0-19, P0-20 | TUI plugin lifecycle |
| Sprint 4 | P0-6, P0-7, P0-8, P0-9, P0-10 | MCP/LSP transport |

---

*Task list version: 4.0*  
*Iteration: 4*  
*Last updated: 2026-04-10*