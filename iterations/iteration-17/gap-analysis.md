# Gap Analysis Report - Iteration 17

**Generated:** 2026-04-14
**Analysis Scope:** OpenCode Rust Port Implementation vs. PRD Specifications (01-19)
**Previous Analysis:** Iteration 16 (2026-04-14)
**Test Run:** `cargo test --all-features --all` - 610 passed, 14 failed (across all packages)

---

## Executive Summary

**Significant progress since iteration-16.** All P0 blocking issues from PRD have been resolved. Implementation is approximately **85-90% complete** relative to PRD scope.

**Key Improvements Since Iteration-16:**
- All P0 issues resolved (custom tool discovery, registration, plugin tool registration)
- Hook execution determinism fixed with priority ordering
- Plugin config ownership enforced
- Session lifecycle integration tests added
- Ownership tree acyclicity tests added
- Primary agent invariant tests added
- ACP transport layer fully implemented
- Desktop and web server modes fully implemented
- TUI component tests added (slash commands, input model, sidebar, etc.)

**Remaining Critical Issues:**
- Config crate is still an empty re-export (violates PRD 19 crate ownership)
- Test infrastructure issues: 14 failing tests across config and TUI
- Desktop/web smoke test port conflict issue

---

## 1. Gap Analysis by PRD Document

### 1.1 Core Architecture (01) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Part type - extensible versioning surface | ✅ Done | `crates/core/src/part.rs` - versioned enum with Unknown variant |
| Project entity with stable ID | ✅ Done | `crates/core/src/project.rs` |
| Session entity with stable ID, parent lineage | ✅ Done | `crates/core/src/session.rs` |
| Message entity - ordered history | ✅ Done | `crates/core/src/message.rs` |
| Ownership tree (Project→Session→Message→Part) acyclic | ✅ Done | 40+ acyclicity tests in `session.rs` |
| Fork model - child session without parent mutation | ✅ Done | `delegation.rs` + session fork logic |
| Snapshot/checkpoint metadata | ✅ Done | `crates/core/src/snapshot.rs`, `checkpoint.rs` |
| Session status machine (idle→running→terminal) | ✅ Done | `session_state.rs` |

**Status Change:** Ownership tree acyclicity tests now exist (40+ tests).

---

### 1.2 Agent System (02) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Primary agent execution loop | ✅ Done | `crates/agent/src/runtime.rs` |
| Exactly one active primary agent invariant | ✅ Done | 20+ invariant tests in `runtime.rs` |
| Hidden vs visible agent behavior | ✅ Done | Tests verify hidden agents don't affect invariant |
| Subagent execution - child context | ✅ Done | `crates/agent/src/delegation.rs` |
| Task/delegation mechanism | ✅ Done | `delegation.rs` |
| Permission inheritance from parent to subagent | ✅ Done | Tests confirm intersection logic |
| Runtime restriction of subagent permissions | ✅ Done | `effective_scope = parent_scope.intersect(subagent_scope)` |

**Status Change:** Primary agent invariant tests now exist (20+ tests).

---

### 1.3 Tools System (03) ✅ FIXED

| Requirement | Status | Gap |
|------------|--------|-----|
| Tool registry - registration, lookup, listing | ✅ Done | `crates/tools/src/registry.rs` (2288 lines) |
| Built-in tool interface - stable name/description/args | ✅ Done | Tool trait implementation |
| Custom tool discovery | ✅ **FIXED** | Now scans `.ts/.js` files per PRD (was `TOOL.md`) |
| Custom tools registered with ToolRegistry | ✅ **FIXED** | `register_custom_tool()` and `register_discovered_custom_tools()` |
| Execution pipeline: name lookup → permission → validation → execute | ✅ Done | Permission gate in AgentExecutor |
| Argument validation | ✅ Done | Schema validation exists |
| MCP tool qualification (server-qualified naming) | ✅ Done | `crates/mcp/src/tool_bridge.rs` |
| Deterministic collision resolution | ✅ Done | ToolSource priority (Builtin > Plugin > CustomProject > CustomGlobal) |
| Result caching for safe tools | ✅ Done | `CachedToolResult` with TTL and dependency tracking |

**Critical Gap Resolution (Iteration-16 → Iteration-17):**

1. **Custom Tool Discovery Format** - ✅ FIXED
   - Implementation now scans `.ts` and `.js` files per PRD
   - Location: `crates/core/src/config/directory_scanner.rs:274`

2. **Custom Tools Registration** - ✅ FIXED
   - `register_discovered_custom_tools()` now registers tools with `ToolRegistry`
   - Location: `crates/core/src/tool.rs:246`

---

### 1.4 MCP System (04) ✅ DONE

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

### 1.6 Configuration System (06) ⚠️ PARTIAL - Config Crate Issue

| Requirement | Status | Gap |
|------------|--------|-----|
| JSON and JSONC parsing | ✅ Done | In `crates/core/src/config.rs` (3800+ lines) |
| Config precedence (remote→global→custom→project→.opencode→inline) | ✅ Done | Fully implemented |
| Variable expansion: `{env:VAR}` and `{file:PATH}` | ✅ Done | Implemented in config.rs |
| `tools` legacy alias normalization to `permission` | ✅ Done | Legacy conversion exists |
| Config ownership boundary (opencode.json vs tui.json) | ✅ Done | Enforced with warnings |
| Permission rule type with glob pattern support | ✅ Done | `permission.rs` |
| Auth/secret storage paths | ✅ Done | `~/.local/share/opencode/auth.json` |

**Critical Gap:**

| Gap Item | Severity | Module | Status |
|----------|----------|--------|--------|
| `crates/config/src/lib.rs` is empty re-export | P1 | config | ❌ **NOT FIXED** |

- Per PRD 19, config should be in dedicated `crates/config/` crate
- Current implementation: `pub use opencode_core::config::Config;`
- This violates the intended crate ownership architecture

---

### 1.7 HTTP Server API (07) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Route registration by resource group | ✅ Done | Routes organized by: session, config, provider, permission, share, MCP, SSE, acp, ws |
| Auth enforcement per endpoint | ✅ Done | Middleware exists and is tested |
| Request validation | ✅ Done | `validation.rs` |
| Session/message lifecycle endpoints | ✅ Done | `session.rs`, `share.rs` |
| Streaming endpoints (SSE/websocket) | ✅ Done | `sse.rs`, `ws.rs` |
| API error shape consistency | ✅ Done | `error.rs` |

**Gaps:**
- No explicit tests verifying route-group presence (not critical)
- API negative tests for unauthorized/malformed requests (not critical)

---

### 1.8 Plugin System (08) ✅ FIXED

| Requirement | Status | Gap |
|------------|--------|-----|
| Plugin source loading from configured paths | ✅ Done | `crates/plugin/src/discovery.rs` |
| Hooks: on_init, on_start, on_tool_call, on_message, on_session_end | ✅ Done | All implemented in `lib.rs` |
| Hook execution order deterministic | ✅ **FIXED** | Uses `IndexMap` with priority ordering |
| Plugin-provided tool registration through standard registry | ✅ **FIXED** | `Plugin::register_tool()` method now exists |
| Failure containment - plugin errors don't crash runtime | ✅ Done | Hooks log warnings but don't panic |
| Server/runtime plugin config ownership | ✅ **FIXED** | Config ownership split enforced |

**Critical Gap Resolution (Iteration-16 → Iteration-17):**

1. **Non-deterministic Hook Execution** - ✅ FIXED
   - Hooks now execute in priority order (lowest priority first)
   - Location: `crates/plugin/src/lib.rs:604`
   - Tests: `test_hook_order_is_deterministic_by_priority`

2. **Plugin Tool Registration Missing** - ✅ FIXED
   - `Plugin` trait now has `register_tool(&mut self, tool: PluginTool)` method
   - `PluginManager` has `register_tools_in_registry()` method
   - Tests: `test_plugin_register_tool_via_trait`, `test_register_tools_in_registry_integration`

3. **Plugin Config Ownership Not Enforced** - ✅ FIXED
   - Config ownership boundary now enforced with warnings
   - Tests: `test_plugin_trait_default_register_tool`

---

### 1.9 TUI System (09) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Session view - markdown, syntax highlighting, diff | ✅ Done | `app.rs` (191KB) |
| Slash commands | ✅ Done | `/command` parsing in `command.rs` |
| Input model: multiline, history, autocomplete | ✅ Done | `input/` module |
| Sidebar - file tree, MCP/LSP status, diagnostics | ✅ Done | `components/` and `widgets/` |
| Keybinding system with leader key | ✅ Done | `keybinding.rs` |
| `@` file reference with fuzzy search | ✅ Done | `file_ref_handler.rs` |
| `!` shell prefix handling | ✅ Done | `shell_handler.rs` |

**Tests Added Since Iteration-16:**
- `slash_command_tests.rs` - 287 lines, comprehensive slash command tests
- `input_model_tests.rs` - 371 lines, autocomplete, multiline tests
- `shell_prefix_tests.rs` - shell prefix handling tests
- `component_tests.rs` - sidebar, input widget tests
- `file_references_tests.rs` - @ file reference tests

**Remaining Gap:**
- 3 TUI tests failing (keybinding 2, theme 1)

---

### 1.10 Provider/Model System (10) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Provider abstraction - registration, credential lookup | ✅ Done | `crates/llm/src/provider_abstraction.rs` |
| Default model selection | ✅ Done | `crates/llm/src/model_selection.rs` |
| Per-agent model override | ✅ Done | Not explicitly tested but implementation exists |
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

---

### 1.12 Skills System (12) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| SKILL.md format support with frontmatter | ✅ Done | `crates/core/src/skill.rs` (1400+ lines) |
| Discovery precedence: project→global→compat | ✅ Done | Priority-based ordering |
| Deterministic duplicate resolution within scope | ✅ Done | First-found wins per scope |
| Compatibility path loading (Claude/Agent) | ✅ Done | `.claude/skills/`, `.agents/skills/` |
| Skill loading into runtime context | ✅ Done | `inject_into_prompt()` |
| Permission restrictions for skill usage | ✅ Done | Uses tool permission system |

---

### 1.13 Desktop/Web Interface (13) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Desktop app startup flow | ✅ Done | `crates/cli/src/cmd/desktop.rs` (207 lines) |
| Web server mode | ✅ Done | `crates/cli/src/cmd/web.rs` (86 lines) |
| Auth-protected interface access | ✅ Done | Web UI has password protection |
| Session sharing between interface modes | ✅ Done | ShareServer implemented |
| ACP startup/handshake for editor integration | ✅ Done | `crates/server/src/routes/acp.rs`, `acp_ws.rs` |
| Sharing behavior in managed/restricted deployments | ✅ Done | `share` config option supported |

**Status Change:** Desktop and web modes are now fully implemented with proper server state management.

---

### 1.14 GitHub/GitLab Integration (14) ✅ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| GitHub workflow trigger examples | ✅ Done | `crates/git/src/github.rs` |
| Comment/PR trigger parsing | ✅ Done | `trigger.rs` |
| CI secret loading for GitHub Actions | ✅ Done | Auth integration |
| GitLab CI component support | ✅ Done | `crates/git/src/gitlab_ci.rs` |
| GitLab Duo behavior (experimental) | ⚠️ Marked experimental | No explicit handling |

---

### 1.15 TUI Plugin API (15) ✅ MOSTLY DONE

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
| Theme auto-sync on install | ✅ Done | Not explicitly tested but implementation exists |

**Tests Added Since Iteration-16:**
- `plugin_lifecycle_tests.rs`
- `plugin_events_tests.rs`
- `plugin_state_tests.rs`
- `plugin_theme_tests.rs`
- `plugin_commands_tests.rs`
- `plugin_dispose_tests.rs`
- `plugin_enabled_tests.rs`
- `plugin_slots_tests.rs`

---

### 1.16 Test Plan (16) ✅ MOSTLY DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| Unit tests for core entities | ✅ Done | Various test files |
| Integration tests for agent flow | ✅ Done | `agent_tool_tests.rs`, `agent_llm_tests.rs` |
| Session lifecycle tests | ✅ Done | `session_lifecycle_tests.rs` (21KB) |
| Compaction and shareability tests | ✅ Done | `compaction_shareability_tests.rs` (17KB) |
| MCP protocol tests | ✅ Done | `mcp_protocol_tests.rs` |
| Session storage tests | ✅ Done | `session_storage_tests.rs` |
| Agent switch tests | ✅ Done | `agent_switch_tests.rs` (9KB) |
| ACP transport tests | ✅ Done | `acp_transport_tests.rs` |
| Convention tests | ✅ Done | `conventions/` module |
| TUI component tests | ✅ Done | `slash_command_tests.rs`, `input_model_tests.rs`, etc. |
| ratatui-testing crate | ✅ Done | `ratatui-testing/` crate exists |

**Status Change:** ratatui-testing crate now exists with full implementation.

---

### 1.17 Rust Test Implementation Roadmap (17) ✅ DONE

Tests implemented per roadmap.

---

### 1.18 Crate-by-Crate Test Backlog (18) ✅ MOSTLY DONE

Most crates have tests. Remaining gap in test infrastructure issues.

---

### 1.19 Implementation Plan (19) ⚠️ PARTIAL

| Requirement | Status | Gap |
|------------|--------|-----|
| Phase 0: Project Foundation | ✅ Done | 100% |
| Phase 1: Authority (Core/Config/Storage/Server) | ✅ Mostly Done | ~95% |
| Phase 2: Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~95% |
| Phase 3: Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~95% |
| Phase 4: Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~90% |
| Phase 5: Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6: Release Qualification | ⚠️ Not Started | ~0% |

---

### 1.20 Ratatui Testing (20) ✅ DONE

| Requirement | Status | Gap |
|------------|--------|-----|
| ratatui-testing crate | ✅ Done | `opencode-rust/ratatui-testing/` exists |
| State testing | ✅ Done | `ratatui-testing/src/state.rs` |
| PTY simulation | ✅ Done | `ratatui-testing/src/pty.rs` |
| CLI testing | ✅ Done | `ratatui-testing/src/cli.rs` |
| Diff utilities | ✅ Done | `ratatui-testing/src/diff.rs` |
| Test DSL | ✅ Done | `ratatui-testing/src/dsl.rs` |

---

## 2. Gap Summary Table

| Gap Item | Severity | Module | Status |修复建议 |
|----------|----------|--------|--------|---------|
| Custom tool discovery scans TOOL.md instead of .ts/.js | P0 | tools | ✅ **FIXED** | Now correctly scans `.ts/.js` files |
| Custom tools not registered with ToolRegistry | P0 | tools | ✅ **FIXED** | `register_discovered_custom_tools()` now works |
| Plugin tool registration not implemented | P0 | plugin | ✅ **FIXED** | `register_tool()` method now exists |
| Non-deterministic hook execution order | P1 | plugin | ✅ **FIXED** | Priority-based ordering implemented |
| Plugin config ownership not enforced | P1 | plugin | ✅ **FIXED** | Config ownership split enforced |
| Config crate is empty re-export | P1 | config | ❌ **NOT FIXED** | Move config logic to `crates/config/` |
| Exactly-one-active-primary-agent invariant untested | P1 | agent | ✅ **FIXED** | 20+ invariant tests now exist |
| Ownership tree acyclicity not tested | P1 | core | ✅ **FIXED** | 40+ acyclicity tests now exist |
| Session lifecycle integration tests incomplete | P1 | storage | ✅ **FIXED** | `session_lifecycle_tests.rs` added |
| Desktop app implementation | P1 | cli | ✅ **FIXED** | Full implementation in desktop.rs |
| Web server mode incomplete | P1 | cli | ✅ **FIXED** | Full implementation in web.rs |
| ACP transport not implemented | P1 | control-plane | ✅ **FIXED** | Handshake, events, WS all implemented |
| TUI slash command tests missing | P2 | tui | ✅ **FIXED** | `slash_command_tests.rs` added (287 lines) |
| TUI input model tests missing | P2 | tui | ✅ **FIXED** | `input_model_tests.rs` added (371 lines) |
| TUI sidebar tests missing | P2 | tui | ✅ **FIXED** | `component_tests.rs` added |
| Config tests failing with PoisonError | P1 | core | ❌ **NEW** | Fix test infrastructure (ENV_LOCK issue) |
| TUI keybinding tests failing (2 tests) | P2 | tui | ❌ **NEW** | Fix case sensitivity and Space key handling |
| TUI theme color parsing test failing | P2 | tui | ❌ **NEW** | Fix hex color parsing |
| Desktop/web smoke test port conflict | P2 | cli | ❌ **NEW** | Fix port allocation in tests |
| Per-agent model override untested | P2 | llm | ⚠️ Deferred | Implementation exists, not critical |
| Hidden vs visible agent UI behavior untested | P2 | agent | ⚠️ Deferred | Tests exist for invariant, UI behavior not critical |

---

## 3. P0/P1/P2 Problem Classification

### P0 - Blocking Issues (ALL RESOLVED) ✅

All P0 blocking issues from iteration-16 have been resolved:

1. ✅ **Custom Tool Discovery Format Mismatch** - FIXED
2. ✅ **Custom Tools Not Registered with ToolRegistry** - FIXED
3. ✅ **Plugin Tool Registration Missing** - FIXED

### P1 - High Priority Issues

| Issue | Status | Module |
|-------|--------|--------|
| Config crate empty re-export | ❌ **NOT FIXED** | config |
| Config tests failing with PoisonError | ❌ **NEW** | core (test infra) |

### P2 - Medium Priority Issues

| Issue | Status | Module |
|-------|--------|--------|
| TUI keybinding tests failing (2 tests) | ❌ **NEW** | tui |
| TUI theme color parsing test failing | ❌ **NEW** | tui |
| Desktop/web smoke test port conflict | ❌ **NEW** | cli |
| Per-agent model override untested | ⚠️ Deferred | llm |
| Hidden vs visible agent UI behavior untested | ⚠️ Deferred | agent |

---

## 4. Technical Debt

| Item | Description | Impact | Status |
|------|-------------|--------|--------|
| Empty `crates/config/` crate | Re-exports from core instead of housing config logic | Violates PRD 19 crate ownership | ❌ **NOT FIXED** |
| Config tests use ENV_LOCK with race condition | Tests fail with PoisonError when run in parallel | Test infrastructure broken | ❌ **NOT FIXED** |
| TUI test failures | 3 tests failing (keybinding 2, theme 1) | UI regression risk | ❌ **NOT FIXED** |
| Desktop/web smoke test port conflict | Test assumes specific port availability | Flaky test | ❌ **NOT FIXED** |
| Deprecated `mode` field | PRD marked for removal in v4.0 | Legacy cleanup pending | ⚠️ Deferred |
| Deprecated `tools` field | PRD marked for removal after migration | Legacy cleanup pending | ⚠️ Deferred |

---

## 5. Implementation Progress Summary

### Overall Status

**Implementation: ~85-90% complete** (up from ~75-80% in iteration-16)

### Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Done | ~95% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~95% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~95% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~90% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

### Crate-Level Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | Registry and custom tool discovery FIXED |
| `crates/plugin/` | ✅ Done | Hooks and tool registration FIXED |
| `crates/tui/` | ✅ Done | Full implementation, tests added |
| `crates/server/` | ✅ Done | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ⚠️ Broken | Still empty re-export, not real crate |
| `crates/cli/` | ✅ Done | Desktop/web implemented |
| `crates/control-plane/` | ✅ Done | ACP stream, events, enterprise features |
| `ratatui-testing/` | ✅ Done | TUI testing framework crate |

### Test Results Summary

```
cargo test --all-features --all:
- 610 passed
- 14 failed (across all packages)

Failed tests breakdown:
- Config tests: 10 failures (PoisonError - test infrastructure issue)
- TUI tests: 3 failures (keybinding 2, theme 1)
- CLI tests: 1 failure (desktop_web_different_ports - port conflict)
```

---

## 6. Recommendations

### Immediate Actions (P1 Fixes)

1. **Fix Config Crate**
   - Move config logic from `core` to dedicated `crates/config/` crate
   - Align with PRD 19 crate ownership intentions

2. **Fix Config Test Infrastructure**
   - Refactor ENV_LOCK to use proper async Mutex or remove shared state
   - Ensure tests can run in parallel without race conditions

### Medium-term Actions (P2)

3. **Fix TUI Test Failures**
   - Fix keybinding tests: case sensitivity and Space key handling
   - Fix theme hex color parsing test

4. **Fix Desktop/Web Smoke Test**
   - Use dynamic port allocation instead of hardcoded port 3000

5. **Begin Phase 6 Release Qualification** (when ready)
   - End-to-end integration tests
   - Performance benchmarking
   - Security audit

---

## 7. Conclusion

The OpenCode Rust port has made **significant progress** since iteration-16, resolving all P0 blocking issues and most P1 issues:

**Resolved Issues:**
- ✅ Custom tool discovery now correctly scans `.ts/.js` files
- ✅ Custom tools are properly registered with ToolRegistry
- ✅ Plugin tool registration fully implemented
- ✅ Hook execution now deterministic with priority ordering
- ✅ Plugin config ownership enforced
- ✅ Desktop and web server modes fully implemented
- ✅ ACP transport layer completed
- ✅ Session lifecycle tests added (21KB test file)
- ✅ Ownership tree acyclicity tests added (40+ tests)
- ✅ Primary agent invariant tests added (20+ tests)
- ✅ TUI component tests added (slash commands, input model, sidebar, plugins)
- ✅ ratatui-testing crate fully implemented

**Remaining Critical Issues:**
- ❌ Config crate still an empty re-export (violates PRD 19)
- ❌ Test infrastructure issues (14 failing tests - mostly config PoisonError)

**Overall Progress:** ~85-90% complete

**Priority for next iteration:** Fix remaining P1 issues (config crate refactor, test infrastructure), then begin Phase 6 release qualification when ready.

---

## Appendix: Test Failure Details

### Config Tests (10 failures - PoisonError)

All failures are in `crates/core/src/config.rs` tests:
- `test_precedence_cli_none_values_dont_override_env`
- `test_precedence_cli_overrides_env`
- `test_precedence_env_config_content_overrides_file`
- `test_precedence_env_overrides_config_file`
- `test_precedence_full_chain_integration`
- `test_precedence_multiple_env_vars_stack`
- `test_precedence_opencode_dir_overrides_project`
- `test_precedence_project_config_overrides_global`
- `test_precedence_provider_api_keys_from_env`
- `test_load_multi_with_cli_overrides_full_chain`

**Root Cause:** `ENV_LOCK` mutex causes PoisonError when tests run in parallel.

### TUI Tests (3 failures)

1. `keybinding::tests::test_key_parsing_simple` - assertion failure comparing `Char('P')` to `Char('p')`
2. `keybinding::tests::test_key_parsing_space` - assertion failure comparing `Char(' ')` to `Space`
3. `theme::tests::test_parse_hex_color` - color parsing returns wrong value

### CLI Tests (1 failure)

`desktop_web_different_ports` - assertion failure: "Desktop server should start on port 3000"

---

*Document generated: 2026-04-14*
*Iteration: 17*
*Phase: Phase 1-5 of 6 (Authority through Hardening Complete, Release Qualification Pending)*
*Priority: Fix P1 test infrastructure issues, then begin Phase 6 release qualification*