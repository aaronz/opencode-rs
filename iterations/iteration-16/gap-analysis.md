# Gap Analysis Report - Iteration 16

**Generated:** 2026-04-13  
**Analysis Scope:** OpenCode Rust Implementation vs. Iteration 15 Gap Analysis  
**Previous Analysis:** Iteration 15 (2026-04-13)

---

## Executive Summary

This report analyzes the implementation gap between the current OpenCode Rust codebase and the previous iteration's gap analysis (Iteration 15). Significant progress has been made in addressing previously identified P0 blocking issues, particularly in **custom tool discovery**, **custom tool registration**, and **plugin tool registration**. 

**Overall Implementation Status:** ~75-80% complete (up from ~65-70%)

**Key Changes from Iteration 15:**
- ✅ All 3 P0 blocking issues have been resolved
- ✅ Hook execution determinism implemented with priority-based ordering
- ✅ Ownership tree acyclicity tests added
- ⚠️ 1 P1 issue remains unresolved (empty config crate)
- ⚠️ Desktop/Web/ACP partially implemented but not fully qualified

---

## 1. Status of Previously Identified Issues

### 1.1 P0 Issues (Previously Blocked)

| Issue ID | Description | Status | Evidence |
|----------|-------------|--------|----------|
| **P0-1** | Custom tool discovery scans TOOL.md instead of .ts/.js | ✅ **RESOLVED** | `crates/tools/src/discovery.rs:104-108` - `is_tool_file()` now checks for `.js`, `.ts`, `.mjs`, `.cjs` |
| **P0-2** | Custom tools not registered with ToolRegistry | ✅ **RESOLVED** | `crates/tools/src/discovery.rs:230-254` - `register_custom_tools()` function properly registers tools with registry |
| **P0-3** | Plugin tool registration missing | ✅ **RESOLVED** | `crates/plugin/src/lib.rs:264-272` - `Plugin::register_tool()` method exists; `PluginManager::register_tools_in_registry()` at line 807 |

### 1.2 P1 Issues

| Issue ID | Description | Status | Evidence |
|----------|-------------|--------|----------|
| **P1-1** | Non-deterministic hook execution order | ✅ **RESOLVED** | `crates/plugin/src/lib.rs:601-619` - `get_plugins_by_priority()` sorts by priority; tests at lines 1589, 1687, 1790, etc. |
| **P1-2** | Plugin config ownership not enforced | ⚠️ **PARTIAL** | Needs verification - config validation may exist in config module |
| **P1-3** | Exactly-one-active-primary-agent invariant untested | ✅ **RESOLVED** | `crates/agent/src/runtime.rs:153` - invariant documented; `test_primary_invariant_no_active_primary_agent_error` at line 880 |
| **P1-4** | Ownership tree acyclicity not tested | ✅ **RESOLVED** | `crates/core/src/session.rs` - tests `test_ownership_tree_acyclicity_*` at lines 1999-2407 |
| **P1-5** | Session lifecycle integration tests incomplete | ⚠️ **PARTIAL** | `crates/storage/src/recovery_tests.rs` exists but may not cover full create→fork→share→compact→revert chain |
| **P1-6** | Desktop app not implemented | ⚠️ **PARTIAL** | `crates/cli/src/cmd/desktop.rs` exists (205 lines); `e2e_desktop_web_smoke.rs` exists |
| **P1-7** | Web server mode incomplete | ⚠️ **PARTIAL** | `crates/server/src/routes/web_ui.rs` exists; desktop mode includes web server |
| **P1-8** | ACP transport not implemented | ⚠️ **PARTIAL** | `crates/control-plane/src/acp_stream.rs` (177 lines); `crates/server/src/routes/acp.rs` (371 lines) with handshake, connect, ack, events |
| **P1-9** | Config crate is empty re-export | ❌ **NOT FIXED** | `crates/config/src/lib.rs` still only contains `pub use opencode_core::config::Config;` |

---

## 2. Gap Analysis by Module

### 2.1 Core Architecture (01) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Part type - extensible versioning surface | ✅ Done | `crates/core/src/part.rs` |
| Project entity with stable ID | ✅ Done | `crates/core/src/project.rs` |
| Session entity with stable ID, parent lineage | ✅ Done | Extensive tests for acyclicity exist |
| Message entity - ordered history | ✅ Done | `crates/core/src/message.rs` |
| Fork model - child session without parent mutation | ✅ Done | `delegation.rs` + session fork logic |
| Snapshot/checkpoint metadata | ✅ Done | `crates/core/src/snapshot.rs`, `checkpoint.rs` |
| Session status machine (idle→running→terminal) | ✅ Done | `session_state.rs` |

**Status:** No critical gaps. Acyclicity and ownership tree tests have been added.

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

---

### 2.3 Tools System (03) ✅ FIXED

| Requirement | Status | Gap |
|------------|--------|-----|
| Tool registry - registration, lookup, listing | ✅ Done | `crates/tools/src/registry.rs` (2288 lines) |
| Built-in tool interface | ✅ Done | Tool trait implementation |
| Custom tool discovery | ✅ **FIXED** | Now scans `.js`, `.ts`, `.mjs`, `.cjs` files |
| Custom tools registered with ToolRegistry | ✅ **FIXED** | `register_custom_tools()` properly registers |
| Execution pipeline: name lookup → permission → validation → execute | ✅ Done | Permission gate in AgentExecutor |
| MCP tool qualification | ✅ Done | `<servername>_<toolname>` format |
| Deterministic collision resolution | ✅ Done | ToolSource priority (Builtin > Plugin > CustomProject > CustomGlobal) |
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

### 2.8 Plugin System (08) ✅ FIXED

| Requirement | Status | Gap |
|------------|--------|-----|
| Plugin source loading from configured paths | ✅ Done | `crates/plugin/src/discovery.rs` |
| Hooks: on_init, on_start, on_tool_call, on_message, on_session_end | ✅ Done | All implemented |
| Hook execution order deterministic | ✅ **FIXED** | Priority-based ordering with `get_plugins_by_priority()` |
| Plugin-provided tool registration | ✅ **FIXED** | `Plugin::register_tool()` + `register_tools_in_registry()` |
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
| Desktop app shell | ⚠️ Partial | `crates/cli/src/cmd/desktop.rs` exists with 205 lines |
| Web server mode | ⚠️ Partial | `web_ui.rs` exists; desktop mode includes web |
| Auth-protected interface access | ⚠️ Partial | Auth middleware exists for API |
| ACP handshake and events | ⚠️ Partial | `acp_stream.rs` + `routes/acp.rs` with full API |

**Gap:** Desktop/Web/ACP are partially implemented but lack full integration testing and qualification

---

## 3. Gap Summary Table

| Gap Item | Severity | Module |修复建议 | Status |
|----------|----------|--------|---------|--------|
| Config crate empty re-export | P1 | config | Move config logic to `crates/config/` as real crate | ❌ Open |
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

---

## 4. P0/P1/P2 Problem Classification

### P0 - Blocking Issues (Must Fix Before Release)

**All P0 issues from Iteration 15 have been resolved.**

| ID | Issue | Module | Resolution |
|----|-------|--------|------------|
| P0-1 | Custom tool discovery format mismatch | tools | ✅ Fixed - now scans .js/.ts/.mjs/.cjs |
| P0-2 | Custom tools not registered with ToolRegistry | tools | ✅ Fixed - register_custom_tools() works |
| P0-3 | Plugin tool registration missing | plugin | ✅ Fixed - register_tool() and register_tools_in_registry() exist |

### P1 - High Priority Issues

| ID | Issue | Module |修复建议 | Status |
|----|-------|--------|---------|--------|
| P1-1 | Config crate is empty re-export | config | Move config logic from `core` to dedicated `config` crate | ❌ Open |
| P1-2 | Desktop app not fully qualified | cli | Complete desktop integration tests | ⚠️ Partial |
| P1-3 | Web server mode not fully qualified | cli | Complete web mode integration tests | ⚠️ Partial |
| P1-4 | ACP transport not fully qualified | control-plane | Verify complete ACP handshake flow | ⚠️ Partial |
| P1-5 | Session lifecycle integration tests incomplete | storage | Add create→fork→share→compact→revert test | ⚠️ Partial |

### P2 - Medium Priority Issues

| ID | Issue | Module |修复建议 | Status |
|----|-------|--------|---------|--------|
| P2-1 | TUI slash command tests missing | tui | Add automated tests for slash command execution | ⚠️ Missing |
| P2-2 | TUI input model tests missing | tui | Add tests for multiline, history, autocomplete | ⚠️ Missing |
| P2-3 | TUI sidebar tests missing | tui | Add tests for visibility and content | ⚠️ Missing |
| P2-4 | Per-agent model override untested | llm | Add test verifying agent-specific model selection | ⚠️ Missing |
| P2-5 | Hidden vs visible agent UI behavior untested | agent | Add tests for agent visibility in selection flows | ⚠️ Missing |
| P2-6 | Plugin config ownership not verified | plugin | Add validation for config ownership boundaries | ⚠️ Unclear |

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

## 7. Change Log (vs Iteration 15)

| Version | Date | Changes |
|---------|------|---------|
| 16.0 | 2026-04-13 | **Major progress.** All P0 blocking issues resolved. Hook execution determinism fixed. Ownership tree acyclicity tests added. Overall completion improved from ~65-70% to ~75-80%. P1 issue TD-001 (empty config crate) remains open. Desktop/Web/ACP partially implemented but not fully qualified. |
| 15.0 | 2026-04-13 | Initial gap analysis with 3 P0 issues identified |

---

## 8. Recommendations

### Immediate Actions (P1 Fixes)

1. **Refactor Config Crate (TD-001)**
   - Move config logic from `core` to dedicated `crates/config/` crate
   - Align with PRD 19 crate ownership intentions

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

---

## 9. Conclusion

The OpenCode Rust port has made substantial progress since Iteration 15, successfully resolving all three P0 blocking issues related to custom tool discovery, custom tool registration, and plugin tool registration. The implementation is now approximately **75-80% complete**.

**Key Achievements:**
- Custom tool system fully implemented with proper .js/.ts file scanning
- Plugin tool registration fully implemented with priority-based hook ordering
- Ownership tree acyclicity and agent invariant tests added
- Core infrastructure (MCP, LSP, agents, tools, storage) largely complete

**Remaining Critical Items:**
- Empty `crates/config/` crate needs refactoring (P1)
- Desktop/Web/ACP interfaces need full qualification (P1)
- Session lifecycle integration tests need completion (P2)
- TUI component tests need to be added (P2)

**Priority for next iteration:** Refactor config crate to proper structure, then proceed with interface qualification (Desktop/Web/ACP).