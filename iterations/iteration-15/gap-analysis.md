# Gap Analysis Report - Iteration 15

**Generated:** 2026-04-13  
**Analysis Scope:** OpenCode Rust Port Implementation vs. PRD Specifications (01-19)

---

## Executive Summary

This report analyzes the implementation gap between the current OpenCode Rust codebase and the PRD specification documents. The implementation has made significant progress in core functionality, particularly in Phase 1 (authority contracts) and Phase 2 (runtime core). However, several critical gaps remain, particularly in **custom tool discovery registration**, **plugin tool registration**, **hook execution determinism**, **TUI plugin API completeness**, and **interface implementations** (desktop/web/ACP).

**Overall Implementation Status:** ~65-70% complete

---

## 1. Gap Analysis by PRD Document

### 1.1 Core Architecture (01) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Part type - extensible versioning surface | ✅ Done | `crates/core/src/part.rs` - versioned enum with Unknown variant |
| Project entity with stable ID | ✅ Done | `crates/core/src/project.rs` |
| Session entity with stable ID, parent lineage | ✅ Done | `crates/core/src/session.rs` |
| Message entity - ordered history | ✅ Done | `crates/core/src/message.rs` |
| Ownership tree (Project→Session→Message→Part) acyclic | ⚠️ Unverified | No explicit invariant tests |
| Fork model - child session without parent mutation | ✅ Done | `delegation.rs` + session fork logic |
| Snapshot/checkpoint metadata | ✅ Done | `crates/core/src/snapshot.rs`, `checkpoint.rs` |
| Session status machine (idle→running→terminal) | ✅ Done | `session_state.rs` |

**Gaps:**
- No explicit unit tests verifying ownership tree acyclicity
- No integration tests for complete session lifecycle: create→fork→share→compact→revert

---

### 1.2 Agent System (02) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Primary agent execution loop | ✅ Done | `crates/agent/src/runtime.rs` |
| Exactly one active primary agent invariant | ⚠️ Unverified | No explicit invariant test |
| Hidden vs visible agent behavior (build, plan, compaction, title, summary) | ⚠️ Partial | Hidden agents exist but UI integration unclear |
| Subagent execution - child context | ✅ Done | `crates/agent/src/delegation.rs` |
| Task/delegation mechanism | ✅ Done | `delegation.rs` |
| Permission inheritance from parent to subagent | ✅ Done | Tests confirm intersection logic |
| Runtime restriction of subagent permissions | ✅ Done | `effective_scope = parent_scope.intersect(subagent_scope)` |

**Gaps:**
- No tests for hidden vs visible agent behavior in selection flows
- No tests verifying exactly-one-active-primary-agent invariant

---

### 1.3 Tools System (03) ❌ CRITICAL GAPS

| Requirement | Status | Gap |
|------------|--------|-----|
| Tool registry - registration, lookup, listing | ✅ Done | `crates/tools/src/registry.rs` (2288 lines) |
| Built-in tool interface - stable name/description/args | ✅ Done | Tool trait implementation |
| Custom tool discovery | ❌ Broken | `DirectoryScanner::scan_tools()` scans TOOL.md, NOT .ts/.js per PRD |
| Custom tools registered with ToolRegistry | ❌ Missing | Discovered tools recorded in config but NOT registered |
| Execution pipeline: name lookup → permission → validation → execute | ✅ Done | Permission gate in AgentExecutor |
| Argument validation | ✅ Done | Schema validation exists |
| MCP tool qualification (server-qualified naming) | ✅ Done | `crates/mcp/src/tool_bridge.rs` |
| Deterministic collision resolution | ✅ Done | ToolSource priority (Builtin > Plugin > CustomProject > CustomGlobal) |
| Result caching for safe tools | ✅ Done | `CachedToolResult` with TTL and dependency tracking |

**Critical Gaps:**

1. **Custom Tool Discovery Format Mismatch**
   - PRD requires: TypeScript/JavaScript files with `export default tool({...})`
   - Implementation: Scans `TOOL.md` files
   - Location: `crates/core/src/config/directory_scanner.rs:228`

2. **Custom Tools Not Registered**
   - Discovered tools stored in config but never added to `ToolRegistry`
   - `DirectoryScanner` records tools in `tools_info` but `ToolRegistry` only has `register()` for programmatic registration

---

### 1.4 MCP System (04) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Local MCP server connection | ✅ Done | `crates/mcp/src/client.rs`, `server.rs` |
| Remote MCP server connection | ✅ Done | HTTP+SSE transport |
| Per-server OAuth configuration | ✅ Done | `crates/mcp/src/auth.rs` |
| Tool discovery from MCP servers | ✅ Done | `registry.rs` |
| Tool naming with server qualification | ✅ Done | `<servername>_<toolname>` format |
| Permission gating for MCP tools | ✅ Done | Via standard tool pipeline |
| Timeout and unavailable-server handling | ✅ Done | Configurable timeout, error handling |
| Context cost warnings | ✅ Done | `crates/mcp/src/context_cost.rs` |

---

### 1.5 LSP System (05) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Built-in LSP server detection | ✅ Done | `crates/lsp/src/builtin.rs` |
| Custom LSP server registration via config | ✅ Done | `crates/lsp/src/custom.rs` |
| Diagnostics retrieval and surfacing | ✅ Done | `crates/lsp/src/client.rs` |
| LSP failure handling | ✅ Done | `crates/lsp/src/failure_handling_tests.rs` |
| Experimental LSP tool behavior | ✅ Done | `crates/lsp/src/experimental.rs` |

---

### 1.6 Configuration System (06) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| JSON and JSONC parsing | ✅ Done | In `crates/core/src/config.rs` (3800+ lines) |
| Config precedence (remote→global→custom→project→.opencode→inline) | ✅ Done | Fully implemented |
| Variable expansion: `{env:VAR}` and `{file:PATH}` | ✅ Done | Implemented in config.rs |
| `tools` legacy alias normalization to `permission` | ✅ Done | Legacy conversion exists |
| Config ownership boundary (opencode.json vs tui.json) | ✅ Done | Enforced with warnings |
| Permission rule type with glob pattern support | ✅ Done | `permission.rs` |
| Auth/secret storage paths | ✅ Done | `~/.local/share/opencode/auth.json` |

**Gaps:**
- `crates/config/src/lib.rs` is nearly empty (just re-exports core). Per PRD 19, config should be in dedicated `crates/config/` crate

---

### 1.7 HTTP Server API (07) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Route registration by resource group | ✅ Done | Routes organized by: session, config, provider, permission, share, MCP, SSE, acp, ws |
| Auth enforcement per endpoint | ⚠️ Partial | Middleware exists but needs verification |
| Request validation | ✅ Done | `validation.rs` |
| Session/message lifecycle endpoints | ✅ Done | `session.rs`, `share.rs` |
| Streaming endpoints (SSE/websocket) | ✅ Done | `sse.rs`, `ws.rs` |
| API error shape consistency | ✅ Done | `error.rs` |

**Gaps:**
- No explicit tests verifying route-group presence
- No negative tests for unauthorized/malformed requests

---

### 1.8 Plugin System (08) ❌ CRITICAL GAPS

| Requirement | Status | Gap |
|------------|--------|-----|
| Plugin source loading from configured paths | ✅ Done | `crates/plugin/src/discovery.rs` |
| Hooks: on_init, on_start, on_tool_call, on_message, on_session_end | ✅ Done | All implemented in `lib.rs` |
| Hook execution order deterministic | ❌ **BROKEN** | Hooks execute in `HashMap` iteration order (non-deterministic) |
| Plugin-provided tool registration through standard registry | ❌ **NOT STARTED** | `Plugin` trait has no `register_tool()` method |
| Failure containment - plugin errors don't crash runtime | ✅ Done | Hooks log warnings but don't panic |
| Server/runtime plugin config ownership | ❌ **NOT ENFORCED** | Config ownership split not verified |

**Critical Gaps:**

1. **Non-deterministic Hook Execution**
   - Location: `crates/plugin/src/lib.rs:358-369` (on_tool_call_all iterates `self.plugins`)
   - `IndexMap` insertion order is preserved BUT plugin registration order depends on discovery order
   - PRD requires deterministic execution order

2. **Plugin Tool Registration Missing**
   - `PluginToolAdapter` exists and implements `Tool` trait
   - But no mechanism to register plugin tools with `ToolRegistry`
   - `Plugin::register_tool()` method does not exist

---

### 1.9 TUI System (09) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Session view - markdown, syntax highlighting, diff | ✅ Done | `app.rs` (191KB) |
| Slash commands | ✅ Done | `/command` parsing in `command.rs` |
| Input model: multiline, history, autocomplete | ✅ Done | `input/` module |
| Sidebar - file tree, MCP/LSP status, diagnostics | ✅ Done | `components/` and `widgets/` |
| Keybinding system with leader key | ✅ Done | `keybinding.rs` |
| `@` file reference with fuzzy search | ✅ Done | `file_ref_handler.rs` |
| `!` shell prefix handling | ✅ Done | `shell_handler.rs` |

**Gaps:**
- No automated tests for slash command execution
- No tests for input model (multiline, history, autocomplete)
- No tests for sidebar visibility and content

---

### 1.10 Provider/Model System (10) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Provider abstraction - registration, credential lookup | ✅ Done | `crates/llm/src/provider_abstraction.rs` |
| Default model selection | ✅ Done | `crates/llm/src/model_selection.rs` |
| Per-agent model override | ⚠️ Unverified | Not explicitly tested |
| Provider credential resolution (env, file, secret store) | ✅ Done | `auth.rs`, layered auth |
| Local model provider (Ollama, LM Studio) | ✅ Done | `crates/llm/src/ollama.rs`, `lm_studio.rs` |
| Variant / reasoning budget handling | ✅ Done | `budget.rs` |

---

### 1.11 Formatters (11) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Formatter detection by file type | ✅ Done | `FormatterEngine::match_formatters()` |
| Project config-based formatter selection | ✅ Done | Config integration |
| Disable-all and per-formatter disable | ✅ Done | `FormatterConfig::Disabled` |
| Custom formatter command invocation | ✅ Done | `Command` execution with env vars |
| Formatter absence/error handling | ✅ Done | Non-fatal, logs warnings |

**Gap:**
- Automatic formatting on write/edit not explicitly in formatter crate (likely in TUI)

---

### 1.12 Skills System (12) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| SKILL.md format support with frontmatter | ✅ Done | `crates/core/src/skill.rs` (1400+ lines) |
| Discovery precedence: project→global→compat | ✅ Done | Priority-based ordering |
| Deterministic duplicate resolution within scope | ✅ Done | First-found wins per scope |
| Compatibility path loading (Claude/Agent) | ✅ Done | `.claude/skills/`, `.agents/skills/` |
| Skill loading into runtime context | ✅ Done | `inject_into_prompt()` |
| Permission restrictions for skill usage | ⚠️ Not explicitly tested | Uses tool permission system |

---

### 1.13 Desktop/Web Interface (13) ❌ NOT STARTED

| Requirement | Status | Gap |
|------------|--------|-----|
| Desktop app startup flow | ❌ NOT STARTED | Stubs in `crates/cli/` |
| Web server mode | ❌ NOT STARTED | Stub in `crates/cli/src/cmd/web.rs` |
| Auth-protected interface access | ❌ NOT STARTED | No implementation |
| Session sharing between interface modes | ❌ NOT STARTED | No mechanism |
| ACP startup/handshake for editor integration | ⚠️ Partial | `AcpAgentEvent` structs exist but no transport |
| Sharing behavior in managed/restricted deployments | ❌ NOT STARTED | No implementation |

---

### 1.14 GitHub/GitLab Integration (14) ⚠️ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| GitHub workflow trigger examples | ✅ Done | `crates/git/src/github.rs` |
| Comment/PR trigger parsing | ✅ Done | `trigger.rs` |
| CI secret loading for GitHub Actions | ✅ Done | Auth integration |
| GitLab CI component support | ⚠️ Partial | `crates/git/src/gitlab_ci.rs` exists |
| GitLab Duo behavior (experimental) | ⚠️ Marked experimental | No explicit handling |

---

### 1.15 TUI Plugin API (15) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| `tui.json` plugin configuration ownership | ✅ Done | Recognized in config system |
| Plugin identity - runtime ID resolution, file vs npm | ✅ Done | In TUI plugin system |
| Plugin deduplication before activation | ✅ Done | Deduplication logic exists |
| `plugin_enabled` semantics | ✅ Done | Per-plugin enable/disable |
| Commands, routes, dialogs, slots registration | ✅ Done | `plugin_api.rs` (54KB) |
| Theme install/set | ✅ Done | `theme.rs` |
| Events subscription | ✅ Done | `api.event.on()` |
| State get/set | ✅ Done | KV store + state |
| `onDispose` lifecycle | ✅ Done | Cleanup registration |
| Runtime `api.plugins.activate()`/`deactivate()` | ✅ Done | Plugin management |
| Bounded cleanup with AbortSignal | ✅ Done | AbortSignal enforcement |
| Theme auto-sync on install | ⚠️ Not verified | Not explicitly tested |

---

## 2. Gap Summary Table

| Gap Item | Severity | Module |修复建议 |
|----------|----------|--------|---------|
| Custom tool discovery scans TOOL.md instead of .ts/.js | P0 | tools | Implement TypeScript/JavaScript file discovery and dynamic import |
| Custom tools not registered with ToolRegistry | P0 | tools | Add registration flow from discovered tools to registry |
| Plugin tool registration not implemented | P0 | plugin | Add `register_tool()` method to Plugin trait, integrate with ToolRegistry |
| Non-deterministic hook execution order | P1 | plugin | Use ordered collection with explicit priority, document execution order |
| Plugin config ownership not enforced | P1 | plugin | Add validation separating server/runtime vs TUI plugin config |
| Exactly-one-active-primary-agent invariant untested | P1 | agent | Add invariant test verifying single active primary agent |
| Ownership tree acyclicity not tested | P1 | core | Add unit test verifying Project→Session→Message→Part is acyclic |
| Session lifecycle integration tests incomplete | P1 | storage | Add create→fork→share→compact→revert integration test |
| Desktop app not implemented | P1 | cli | Implement desktop startup flow with WebView |
| Web server mode incomplete | P1 | cli | Implement full web server with auth |
| ACP transport not implemented | P1 | control-plane | Implement ACP handshake and transport |
| TUI slash command tests missing | P2 | tui | Add automated tests for slash command execution |
| TUI input model tests missing | P2 | tui | Add tests for multiline, history, autocomplete |
| TUI sidebar tests missing | P2 | tui | Add tests for visibility and content |
| Per-agent model override untested | P2 | llm | Add test verifying agent-specific model selection |
| Route-group presence tests missing | P2 | server | Add integration tests for all route groups |
| API negative tests (auth, malformed requests) | P2 | server | Add unauthorized/malformed request tests |
| Config crate is empty re-export | P2 | config | Move config logic to `crates/config/` as intended |
| Hidden vs visible agent UI behavior untested | P2 | agent | Add tests for agent visibility in selection flows |

---

## 3. P0/P1/P2 Problem Classification

### P0 - Blocking Issues (Must Fix Before Release)

1. **Custom Tool Discovery Format Mismatch**
   - Custom tools cannot work because implementation scans `TOOL.md` but PRD requires `.ts/.js`
   - Impact: Users cannot define custom tools per PRD specification

2. **Custom Tools Not Registered with ToolRegistry**
   - Even if discovered, custom tools are not registered for execution
   - Impact: Custom tools defined in `.opencode/tools/` are non-functional

3. **Plugin Tool Registration Missing**
   - Plugins cannot add tools to the agent's toolset
   - Impact: Plugin system cannot extend agent capabilities per PRD

### P1 - High Priority Issues

4. **Non-deterministic Hook Execution Order**
   - Hooks execute in HashMap iteration order
   - Impact: Plugin behavior may be inconsistent between runs

5. **Plugin Config Ownership Not Enforced**
   - Server/runtime and TUI plugin configs can be mixed
   - Impact: Config ownership boundaries violated per PRD 06

6. **Desktop/Web/ACP Not Implemented**
   - Major interface features missing
   - Impact: Users cannot use desktop app or web interface

7. **Ownership Invariants Untested**
   - Project→Session→Message→Part acyclicity not verified
   - Impact: Core data integrity not guaranteed

### P2 - Medium Priority Issues

8. **Incomplete Test Coverage for Existing Features**
   - Many implemented features lack explicit tests
   - Impact: Regression risk, uncertainty about correctness

---

## 4. Technical Debt

| Item | Description | Impact |
|------|-------------|--------|
| Empty `crates/config/` crate | Re-exports from core instead of housing config logic | Violates PRD 19 crate ownership |
| `DirectoryScanner` discovery mismatch | Scans `TOOL.md` instead of `.ts/.js` per PRD 03 | Custom tools non-functional |
| Custom tools discovered but not registered | Gap between discovery and ToolRegistry | Custom tools non-functional |
| Non-deterministic plugin hook execution | Uses HashMap iteration instead of explicit ordering | Unpredictable plugin behavior |
| Plugin `register_tool()` method missing | `Plugin` trait cannot add tools | Plugin extensibility broken |
| ACP transport layer missing | Only has event structs, no actual transport | Editor integration incomplete |
| TUI tests use `ratatui-testing` | Convention tests exist but limited actual TUI tests | UI regression risk |

---

## 5. Implementation Progress Summary

### Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Done | ~90% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ⚠️ Partial | ~70% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Mostly Done | ~85% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ❌ Not Started | ~20% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~80% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

### Crate-Level Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ⚠️ Partial | Registry done, custom tool discovery broken |
| `crates/plugin/` | ⚠️ Partial | Hooks done, tool registration missing |
| `crates/tui/` | ✅ Done | Full implementation, needs tests |
| `crates/server/` | ✅ Done | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ❌ Broken | Empty re-export, not real crate |
| `crates/cli/` | ⚠️ Partial | Desktop/web stubs exist, not implemented |
| `crates/control-plane/` | ⚠️ Partial | ACP event structs exist, no transport |

---

## 6. Recommendations

### Immediate Actions (P0 Fixes)

1. **Fix Custom Tool Discovery**
   - Implement TypeScript/JavaScript file discovery in `DirectoryScanner`
   - Parse and execute tool definitions using dynamic import (or WASM runtime)
   - Register discovered tools with `ToolRegistry`

2. **Implement Plugin Tool Registration**
   - Add `register_tool()` method to `Plugin` trait
   - Integrate `PluginManager` with `ToolRegistry`
   - Add tests verifying plugin tools appear in registry

3. **Fix Hook Execution Determinism**
   - Add explicit `priority` field to plugins or hooks
   - Execute hooks in priority order
   - Document execution order guarantees

### Short-term Actions (P1)

4. **Complete Desktop/Web/ACP**
   - Implement desktop app shell with WebView
   - Implement web server mode with proper auth
   - Implement ACP transport layer

5. **Add Critical Invariant Tests**
   - Test exactly-one-active-primary-agent
   - Test ownership tree acyclicity
   - Test session lifecycle (create→fork→share→compact→revert)

### Medium-term Actions (P2)

6. **Complete Test Coverage**
   - Add tests for TUI components (slash commands, input, sidebar)
   - Add tests for API route groups
   - Add negative tests for permissions and auth

7. **Refactor Config Crate**
   - Move config logic from `core` to dedicated `config` crate
   - Align with PRD 19 crate ownership intentions

---

## 7. Conclusion

The OpenCode Rust port has made substantial progress, particularly in:
- Core entity models and persistence
- Agent runtime with subagent delegation
- Tool registry and execution pipeline
- MCP/LSP integration
- Configuration system

However, critical gaps remain in:
- **Custom tool discovery and registration** - blocks user extensibility
- **Plugin tool registration** - blocks plugin system extensibility
- **Desktop/Web/ACP interfaces** - blocks major user-facing features
- **Hook execution determinism** - affects reliability

The implementation is approximately **65-70% complete** relative to PRD scope. Phase 1 (authority) and Phase 2 (runtime core infrastructure) are substantially done but with the critical gaps noted above. Phase 4 (interfaces) has barely started. Phase 6 (release qualification) is not started.

**Priority for next iteration:** Fix P0 gaps in custom tool and plugin tool registration, then proceed with interface implementations.
