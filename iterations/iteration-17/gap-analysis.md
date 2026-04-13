# Gap Analysis Report - Iteration 17

**Generated:** 2026-04-14  
**Analysis Scope:** OpenCode Rust Implementation vs. PRD Requirements  
**Previous Analysis:** Iteration 16 (2026-04-13)  
**Overall Implementation Status:** ~75-80% complete

---

## Executive Summary

This report analyzes the implementation gap between the current OpenCode Rust codebase and the PRD requirements. Progress since Iteration 16 is minimal (iteration-17 contains only a checkpoint file). All P0 blocking issues from previous iterations remain resolved. The primary outstanding work centers on **P1 infrastructure/integration items** and **P2 testing coverage**.

**Key Observations:**
- ✅ All P0 blocking issues from Iteration 15/16 remain resolved
- ⚠️ Config crate refactoring (P1) has NOT been completed
- ⚠️ Desktop/Web/ACP interfaces remain partially implemented
- ⚠️ TUI component tests remain missing
- No significant new functionality added since Iteration 16

---

## 1. Status of Previously Identified Issues

### 1.1 P0 Issues (Previously Blocked - All Resolved)

| Issue ID | Description | Status | Evidence |
|----------|-------------|--------|----------|
| **P0-1** | Custom tool discovery scans TOOL.md instead of .ts/.js | ✅ RESOLVED | `crates/tools/src/discovery.rs:104-108` |
| **P0-2** | Custom tools not registered with ToolRegistry | ✅ RESOLVED | `crates/tools/src/discovery.rs:230-254` |
| **P0-3** | Plugin tool registration missing | ✅ RESOLVED | `crates/plugin/src/lib.rs:264-272` |

### 1.2 P1 Issues (Progress Since Iteration 16)

| Issue ID | Description | Status | Evidence |
|----------|-------------|--------|----------|
| **P1-1** | Config crate is empty re-export | ❌ **NOT FIXED** | `crates/config/src/lib.rs` still only contains `pub use opencode_core::config::Config;` |
| **P1-2** | Desktop app not fully qualified | ⚠️ Partial | `crates/cli/src/cmd/desktop.rs` (205→6441 lines as of iter-16) |
| **P1-3** | Web server mode not fully qualified | ⚠️ Partial | `crates/cli/src/cmd/web.rs` exists |
| **P1-4** | ACP transport not fully qualified | ⚠️ Partial | `crates/control-plane/src/acp_stream.rs` exists |
| **P1-5** | Session lifecycle integration tests incomplete | ⚠️ Partial | `crates/storage/src/recovery_tests.rs` exists |
| **P1-6** | Hook execution determinism | ✅ RESOLVED | Priority-based ordering in `crates/plugin/src/lib.rs:601-619` |

---

## 2. Gap Analysis by Module

### 2.1 Core Architecture (01) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Project entity with stable ID | ✅ Done | `crates/core/src/project.rs` |
| Session entity with stable ID, parent lineage | ✅ Done | `crates/core/src/session.rs` |
| Message entity - ordered history | ✅ Done | `crates/core/src/message.rs` |
| Part type - extensible versioning surface | ✅ Done | `crates/core/src/part.rs` |
| Fork model - child session without parent mutation | ✅ Done | `crates/core/src/session.rs` delegation |
| Snapshot/checkpoint metadata | ✅ Done | `crates/core/src/snapshot.rs`, `checkpoint.rs` |
| Session status machine (idle→running→terminal) | ✅ Done | `crates/core/src/session_state.rs` |
| Ownership tree acyclicity | ✅ Tested | `test_ownership_tree_acyclicity_*` tests |

**Status:** No critical gaps.

---

### 2.2 Agent System (02) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|---------|
| Primary agent execution loop | ✅ Done | `crates/agent/src/runtime.rs` |
| Exactly one active primary agent invariant | ✅ Done | Documented at line 153, tested at line 880 |
| Hidden vs visible agent behavior | ⚠️ Partial | UI integration unclear |
| Subagent execution - child context | ✅ Done | `crates/agent/src/delegation.rs` |
| Permission inheritance from parent to subagent | ✅ Done | Intersection logic tested |
| Runtime restriction of subagent permissions | ✅ Done | `effective_scope = parent_scope.intersect(subagent_scope)` |

**Gaps:**
- Hidden vs visible agent UI behavior not explicitly tested
- Per-agent model override not explicitly tested

---

### 2.3 Tools System (03) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Tool registry - registration, lookup, listing | ✅ Done | `crates/tools/src/registry.rs` (2288 lines) |
| Built-in tool interface | ✅ Done | Tool trait implementation |
| Custom tool discovery | ✅ Done | Scans `.js`, `.ts`, `.mjs`, `.cjs` files |
| Custom tools registered with ToolRegistry | ✅ Done | `register_custom_tools()` properly registers |
| Execution pipeline: name lookup → permission → validation → execute | ✅ Done | Permission gate in AgentExecutor |
| MCP tool qualification | ✅ Done | `<servername>_<toolname>` format |
| Deterministic collision resolution | ✅ Done | ToolSource priority |
| Result caching for safe tools | ✅ Done | `CachedToolResult` with TTL |

---

### 2.4 MCP System (04) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Local MCP server connection | ✅ Done | `crates/mcp/src/client.rs`, `server.rs` |
| Remote MCP server connection | ✅ Done | HTTP+SSE transport |
| Per-server OAuth configuration | ✅ Done | `crates/mcp/src/auth.rs` |
| Tool discovery from MCP servers | ✅ Done | `registry.rs` |
| Context cost warnings | ✅ Done | `crates/mcp/src/context_cost.rs` |

---

### 2.5 LSP System (05) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Built-in LSP server detection | ✅ Done | `crates/lsp/src/builtin.rs` |
| Custom LSP server registration | ✅ Done | `crates/lsp/src/custom.rs` |
| Diagnostics retrieval and surfacing | ✅ Done | `crates/lsp/src/client.rs` |
| LSP failure handling | ✅ Done | `crates/lsp/src/failure_handling_tests.rs` |
| LSP experimental tool | ✅ Done | `crates/lsp/src/experimental_tool.rs` |

---

### 2.6 Configuration System (06) ⚠️ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| JSON and JSONC parsing | ✅ Done | `crates/core/src/config.rs` (3800+ lines) |
| Config precedence (remote→global→custom→project→.opencode→inline) | ✅ Done | Fully implemented |
| Variable expansion: `{env:VAR}` and `{file:PATH}` | ✅ Done | Implemented in config.rs |
| Permission rule type with glob pattern support | ✅ Done | `permission.rs` |
| Auth/secret storage paths | ✅ Done | `~/.local/share/opencode/auth.json` |

**Gap:**
- `crates/config/src/lib.rs` is nearly empty (just re-exports core). Per PRD 19, config should be in dedicated `crates/config/` crate

---

### 2.7 HTTP Server API (07) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Route registration by resource group | ✅ Done | 13 route groups defined |
| Auth enforcement per endpoint | ✅ Done | `middleware.rs` with API key validation |
| Request validation | ✅ Done | `validation.rs` |
| Session/message lifecycle endpoints | ✅ Done | `session.rs`, `share.rs` |
| Streaming endpoints (SSE/websocket) | ✅ Done | `sse.rs`, `ws.rs` |
| Route-group tests | ✅ Done | Tests at `server_integration_tests.rs:565-801` |

---

### 2.8 Plugin System (08) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Plugin source loading from configured paths | ✅ Done | `crates/plugin/src/discovery.rs` |
| Hooks: on_init, on_start, on_tool_call, on_message, on_session_end | ✅ Done | All implemented |
| Hook execution order deterministic | ✅ Done | Priority-based ordering with `get_plugins_by_priority()` |
| Plugin-provided tool registration | ✅ Done | `Plugin::register_tool()` + `register_tools_in_registry()` |
| Failure containment | ✅ Done | Hooks log warnings but don't panic |

---

### 2.9 TUI System (09) ⚠️ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Session view - markdown, syntax highlighting, diff | ✅ Done | `app.rs` (191KB) |
| Slash commands | ✅ Done | `/command` parsing in `command.rs` |
| Input model: multiline, history, autocomplete | ✅ Done | `input/` module |
| Sidebar - file tree, MCP/LSP status, diagnostics | ✅ Done | `components/` and `widgets/` |
| Keybinding system with leader key | ✅ Done | `keybinding.rs` |

**Gaps:**
- No automated tests for slash command execution
- No tests for input model (multiline, history, autocomplete)
- No tests for sidebar visibility and content

---

### 2.10 Provider/Model System (10) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Provider abstraction - registration, credential lookup | ✅ Done | `crates/llm/src/provider_abstraction.rs` |
| Default model selection | ✅ Done | `crates/llm/src/model_selection.rs` |
| Per-agent model override | ⚠️ Unverified | Not explicitly tested |
| Local model providers (Ollama, LM Studio) | ✅ Done | `crates/llm/src/ollama.rs`, `lm_studio.rs` |
| Variant/reasoning budget handling | ✅ Done | `budget.rs` |

---

### 2.11 Skills System (12) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| SKILL.md format support | ✅ Done | `crates/core/src/skill.rs` (1400+ lines) |
| Discovery precedence: project→global→compat | ✅ Done | Priority-based ordering |
| Skill loading into runtime context | ✅ Done | `inject_into_prompt()` |

---

### 2.12 Desktop/Web/ACP Interface (13) ⚠️ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Desktop app shell | ⚠️ Partial | `crates/cli/src/cmd/desktop.rs` exists (6441 lines) |
| Web server mode | ⚠️ Partial | `crates/cli/src/cmd/web.rs` exists |
| Auth-protected interface access | ⚠️ Partial | Auth middleware exists for API |
| ACP handshake and events | ⚠️ Partial | `acp_stream.rs` + `routes/acp.rs` with full API |

**Gap:** Desktop/Web/ACP are partially implemented but lack full integration testing and qualification

---

### 2.13 TUI Plugin API (15) ⚠️ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| TUI plugin config via tui.json | ⚠️ Partial | Schema exists but plugin system not fully implemented |
| Plugin entry formats (npm/file) | ⚠️ Partial | Not fully verified |
| TUI Plugin API surface | ⚠️ Partial | `api.command`, `api.route`, `api.ui`, `api.state`, `api.theme` etc. |
| Built-in plugins | ⚠️ Partial | `internal:*` plugins listed but implementation unclear |

**Gap:** TUI plugin API is partially defined in PRD 15 but implementation status is uncertain

---

## 3. Gap Summary Table

| Gap Item | Severity | Module | 修复建议 | Status |
|----------|----------|--------|---------|--------|
| Config crate empty re-export | P1 | config | Move config logic from `core` to dedicated `crates/config/` crate | ❌ Open |
| Desktop app integration testing | P1 | cli | Complete desktop smoke tests and session sharing tests | ⚠️ Partial |
| Web server mode qualification | P1 | cli | Complete web mode tests with auth | ⚠️ Partial |
| ACP transport full integration | P1 | control-plane | Verify ACP handshake/connect/ack/end-to-end | ⚠️ Partial |
| Session lifecycle full integration test | P2 | storage | Add create→fork→share→compact→revert test | ⚠️ Partial |
| TUI slash command tests | P2 | tui | Add automated tests for slash command execution | ⚠️ Missing |
| TUI input model tests | P2 | tui | Add tests for multiline, history, autocomplete | ⚠️ Missing |
| TUI sidebar tests | P2 | tui | Add tests for visibility and content | ⚠️ Missing |
| Per-agent model override tests | P2 | llm | Add test verifying agent-specific model selection | ⚠️ Missing |
| Hidden vs visible agent UI behavior | P2 | agent | Add tests for agent visibility in selection flows | ⚠️ Missing |
| Plugin config ownership enforcement | P2 | plugin | Add validation separating server/runtime vs TUI plugin config | ⚠️ Unclear |
| TUI plugin system qualification | P2 | tui | Verify TUI plugin API implementation against PRD 15 | ⚠️ Unclear |

---

## 4. P0/P1/P2 Problem Classification

### P0 - Blocking Issues (Must Fix Before Release)

**All P0 issues from previous iterations have been resolved.**

| ID | Issue | Module | Resolution |
|----|-------|--------|------------|
| P0-1 | Custom tool discovery format mismatch | tools | ✅ Fixed |
| P0-2 | Custom tools not registered with ToolRegistry | tools | ✅ Fixed |
| P0-3 | Plugin tool registration missing | plugin | ✅ Fixed |

### P1 - High Priority Issues

| ID | Issue | Module | 修复建议 | Status |
|----|-------|--------|---------|--------|
| P1-1 | Config crate is empty re-export | config | Move config logic from `core` to dedicated `config` crate | ❌ Open |
| P1-2 | Desktop app not fully qualified | cli | Complete desktop integration tests | ⚠️ Partial |
| P1-3 | Web server mode not fully qualified | cli | Complete web mode integration tests | ⚠️ Partial |
| P1-4 | ACP transport not fully qualified | control-plane | Verify complete ACP handshake flow | ⚠️ Partial |
| P1-5 | Session lifecycle integration tests incomplete | storage | Add create→fork→share→compact→revert test | ⚠️ Partial |

### P2 - Medium Priority Issues

| ID | Issue | Module | 修复建议 | Status |
|----|-------|--------|---------|--------|
| P2-1 | TUI slash command tests missing | tui | Add automated tests for slash command execution | ⚠️ Missing |
| P2-2 | TUI input model tests missing | tui | Add tests for multiline, history, autocomplete | ⚠️ Missing |
| P2-3 | TUI sidebar tests missing | tui | Add tests for visibility and content | ⚠️ Missing |
| P2-4 | Per-agent model override untested | llm | Add test verifying agent-specific model selection | ⚠️ Missing |
| P2-5 | Hidden vs visible agent UI behavior untested | agent | Add tests for agent visibility in selection flows | ⚠️ Missing |
| P2-6 | Plugin config ownership not verified | plugin | Add validation for config ownership boundaries | ⚠️ Unclear |
| P2-7 | TUI plugin system qualification | tui | Verify TUI plugin API implementation against PRD 15 | ⚠️ Unclear |

---

## 5. Technical Debt

| ID | Item | Module | Severity | Remediation | Status |
|----|------|--------|----------|-------------|--------|
| TD-001 | Empty `crates/config/` crate | config | Medium | Move config logic to dedicated crate | ❌ Open |
| TD-002 | Deprecated `mode` field | config | Low | Remove in v4.0 | Deferred |
| TD-003 | Deprecated `tools` field | config | Low | Remove after migration | Deferred |
| TD-004 | Deprecated `theme` field | config | Low | Moved to tui.json | Deferred |
| TD-005 | Deprecated `keybinds` field | config | Low | Moved to tui.json | Deferred |
| TD-006 | Desktop/Web/ACP partial implementation | cli | High | Complete integration and qualification | ⚠️ In Progress |

---

## 6. Implementation Progress Summary

### Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Done | ~95% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Mostly Done | ~90% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Mostly Done | ~90% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ⚠️ Partial | ~40% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Mostly Done | ~85% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

### Crate-Level Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/permission/` | ✅ Done | Permission system |
| `crates/server/` | ✅ Done | API routes, auth, streaming, tests |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | Registry, custom tool discovery/registration |
| `crates/plugin/` | ✅ Done | Hooks, tool registration, priority ordering |
| `crates/tui/` | ✅ Done | Full implementation, needs tests |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ⚠️ Partial | Empty re-export - needs refactor |
| `crates/cli/` | ⚠️ Partial | Desktop/web stubs exist, partial implementation |
| `crates/control-plane/` | ⚠️ Partial | ACP event stream and routes exist, partial integration |
| `ratatui-testing/` | ✅ Done | TUI testing framework |

---

## 7. PRD Compliance Matrix

### Core Architecture (PRD 01)
| Requirement | Status |
|------------|--------|
| Project entity with stable ID | ✅ |
| Session entity with stable ID, parent lineage | ✅ |
| Message entity - ordered history | ✅ |
| Part type - extensible versioning surface | ✅ |
| Fork model without parent mutation | ✅ |
| Snapshot/checkpoint metadata | ✅ |
| Session status machine | ✅ |
| Ownership tree acyclicity | ✅ |

### Agent System (PRD 02)
| Requirement | Status |
|------------|--------|
| Primary agent execution loop | ✅ |
| Exactly one active primary agent invariant | ✅ |
| Hidden vs visible agent behavior | ⚠️ Partial |
| Subagent execution - child context | ✅ |
| Permission inheritance | ✅ |
| Runtime permission restrictions | ✅ |

### Tools System (PRD 03)
| Requirement | Status |
|------------|--------|
| Tool registry | ✅ |
| Built-in tool interface | ✅ |
| Custom tool discovery (.js/.ts) | ✅ |
| Custom tool registration | ✅ |
| Execution pipeline | ✅ |
| MCP tool qualification | ✅ |
| Collision resolution | ✅ |
| Result caching | ✅ |

### MCP System (PRD 04)
| Requirement | Status |
|------------|--------|
| Local MCP server connection | ✅ |
| Remote MCP server connection | ✅ |
| Per-server OAuth configuration | ✅ |
| Tool discovery | ✅ |
| Context cost warnings | ✅ |

### LSP System (PRD 05)
| Requirement | Status |
|------------|--------|
| Built-in LSP server detection | ✅ |
| Custom LSP server registration | ✅ |
| Diagnostics retrieval | ✅ |
| LSP failure handling | ✅ |
| LSP experimental tool | ✅ |

### Configuration System (PRD 06)
| Requirement | Status |
|------------|--------|
| JSON/JSONC parsing | ✅ |
| Config precedence | ✅ |
| Variable expansion | ✅ |
| Permission rules with glob | ✅ |
| Auth/secret storage | ✅ |
| Config crate separation | ❌ Not Done |

### HTTP Server API (PRD 07)
| Requirement | Status |
|------------|--------|
| Route registration by resource group | ✅ |
| Auth enforcement per endpoint | ✅ |
| Request validation | ✅ |
| Session/message lifecycle | ✅ |
| Streaming endpoints | ✅ |
| Route-group tests | ✅ |

### Plugin System (PRD 08)
| Requirement | Status |
|------------|--------|
| Plugin source loading | ✅ |
| Event hooks | ✅ |
| Hook execution order deterministic | ✅ |
| Plugin-provided tool registration | ✅ |
| Failure containment | ✅ |

### TUI System (PRD 09)
| Requirement | Status |
|------------|--------|
| Session view with markdown/syntax/diff | ✅ |
| Slash commands | ✅ |
| Input model (multiline, history, autocomplete) | ✅ |
| Sidebar (file tree, MCP/LSP, diagnostics) | ✅ |
| Keybinding system with leader key | ✅ |
| Automated tests | ⚠️ Missing |

### Provider/Model System (PRD 10)
| Requirement | Status |
|------------|--------|
| Provider abstraction | ✅ |
| Default model selection | ✅ |
| Per-agent model override | ⚠️ Untested |
| Local model providers | ✅ |
| Variant/reasoning budget | ✅ |

### Skills System (PRD 12)
| Requirement | Status |
|------------|--------|
| SKILL.md format support | ✅ |
| Discovery precedence | ✅ |
| Skill loading | ✅ |

### Desktop/Web/ACP Interface (PRD 13)
| Requirement | Status |
|------------|--------|
| Desktop app shell | ⚠️ Partial |
| Web server mode | ⚠️ Partial |
| Auth-protected interface access | ⚠️ Partial |
| ACP handshake and events | ⚠️ Partial |

### TUI Plugin API (PRD 15)
| Requirement | Status |
|------------|--------|
| tui.json plugin config | ⚠️ Partial |
| npm/file plugin entry formats | ⚠️ Partial |
| TUI Plugin API surface | ⚠️ Partial |
| Built-in plugins | ⚠️ Partial |

---

## 8. Change Log (vs Iteration 16)

| Version | Date | Changes |
|---------|------|---------|
| 17.0 | 2026-04-14 | No significant changes. Iteration-17 is a fresh checkpoint. All P0 issues remain resolved. P1 issues (config crate, Desktop/Web/ACP) remain open. TUI tests remain missing. |
| 16.0 | 2026-04-13 | All P0 blocking issues resolved. Hook execution determinism fixed. Ownership tree acyclicity tests added. |
| 15.0 | 2026-04-13 | Initial gap analysis with 3 P0 issues identified |

---

## 9. Recommendations

### Immediate Actions (P1 Fixes)

1. **Refactor Config Crate (TD-001)**
   - Move config logic from `core` to dedicated `crates/config/` crate
   - Align with PRD 19 crate ownership intentions
   - This is a prerequisite for proper modularity

2. **Complete Desktop/Web/ACP Qualification**
   - Desktop: Complete e2e smoke tests and session sharing tests
   - Web: Complete web mode tests with auth flow
   - ACP: Verify complete handshake/connect/ack event flow

### Short-term Actions (P2)

3. **Add TUI Component Tests**
   - Add tests for slash command execution
   - Add tests for input model (multiline, history, autocomplete)
   - Add tests for sidebar visibility and content

4. **Add Missing Agent/Model Tests**
   - Add test for per-agent model override
   - Add tests for hidden vs visible agent UI behavior

5. **Verify TUI Plugin API Implementation**
   - Compare PRD 15 requirements against actual implementation
   - Document any gaps in TUI plugin system

---

## 10. Conclusion

The OpenCode Rust port maintains its ~75-80% completion status from Iteration 16. No significant new functionality has been added in this iteration.

**Key Achievements (from previous iterations):**
- All P0 blocking issues resolved
- Core infrastructure (MCP, LSP, agents, tools, storage) largely complete
- Custom tool system fully implemented
- Plugin tool registration fully implemented

**Remaining Critical Items:**
- Empty `crates/config/` crate needs refactoring (P1) - TD-001
- Desktop/Web/ACP interfaces need full qualification (P1)
- TUI component tests need to be added (P2)
- TUI plugin system needs verification against PRD 15 (P2)

**Priority for next iteration:** 
1. Refactor config crate to proper structure
2. Complete Desktop/Web/ACP integration testing
3. Add TUI component tests

---

## Appendix: PRD Document Index

| Document | Title | Status |
|---------|-------|--------|
| 01 | Core Architecture | ✅ Mostly Done |
| 02 | Agent System | ✅ Mostly Done |
| 03 | Tools System | ✅ Done |
| 04 | MCP System | ✅ Done |
| 05 | LSP System | ✅ Done |
| 06 | Configuration System | ⚠️ Partial (config crate issue) |
| 07 | HTTP Server API | ✅ Done |
| 08 | Plugin System | ✅ Done |
| 09 | TUI System | ⚠️ Partial (tests missing) |
| 10 | Provider/Model System | ✅ Done |
| 12 | Skills System | ✅ Done |
| 13 | Desktop/Web/ACP Interface | ⚠️ Partial |
| 15 | TUI Plugin API | ⚠️ Partial |

(End of file - total 498 lines)
