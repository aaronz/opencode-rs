# Gap Analysis Report - Iteration 4

## Executive Summary

This report analyzes the gaps between the current implementation and the PRD specifications for the OpenCode Rust port. The implementation is in **Phase 1-2** of a 6-phase plan, with significant work remaining in core authority contracts, runtime systems, and interface implementations.

**Overall Completion Estimate: ~35-40%**

---

## 1. Implementation Progress Summary

### Phase 0 - Project Foundation ✅ COMPLETE
- [x] Workspace setup with all crates
- [x] Convention test suite (23 passing tests in `tests/conventions/`)

### Phase 1 - Authority Implementation 🚧 IN PROGRESS (~60%)
| Component | Status | Notes |
|-----------|--------|-------|
| `Project` type | ✅ Done | Stable ID, root path, VCS tracking |
| `Session` type | ✅ Done | Stable ID, parent lineage, status machine |
| `Message` type | ✅ Done | Ordered history, append-only model |
| `Part` type | ✅ Done | Extensible content parts |
| Storage/Snapshot/Revert | ⚠️ Partial | Not fully surveyed |
| Config system | ⚠️ Partial | Not fully surveyed |
| HTTP API | ⚠️ Partial | Not fully surveyed |

### Phase 2 - Runtime Core 🚧 IN PROGRESS (~50%)
| Component | Status | Notes |
|-----------|--------|-------|
| Agent runtime | ✅ Done | `crates/agent/src/runtime.rs` |
| Primary agent invariants | ❌ Missing | Exactly one active per session not enforced |
| Subagent execution | ❌ Missing | Child context, result handoff not implemented |
| Tool registry | ⚠️ Partial | Built-in done, custom tool discovery issues |
| Permission gate | ✅ Done | `AgentExecutor` in `crates/core/src/executor.rs` |
| Plugin hooks | ✅ Done | `on_init`, `on_start`, `on_tool_call`, etc. |
| Hook execution order | ❌ Issue | Non-deterministic (HashMap iteration) |
| Plugin-provided tools | ❌ NOT STARTED | No `register_tool()` method |
| TUI Plugin API | ❌ NOT STARTED | Full implementation missing |

### Phase 3 - Infrastructure Subsystems 🚧 IN PROGRESS (~40%)
| Component | Status | Notes |
|-----------|--------|-------|
| MCP integration | ❌ NOT STARTED | Local/remote servers, OAuth, tool discovery |
| LSP integration | ❌ NOT STARTED | Built-in server detection, diagnostics |
| Provider/Model | ⚠️ Partial | Not fully surveyed |
| Formatters | ✅ Done | `FormatterEngine` complete |
| Skills system | ✅ Done | SKILL.md parsing, discovery, loading |

### Phase 4 - Interface Implementations ❌ NOT STARTED (0%)
| Component | Status | Notes |
|-----------|--------|-------|
| Desktop app | ❌ NOT STARTED | Only stubs in `crates/cli/` |
| Web server mode | ❌ NOT STARTED | Stub in `crates/cli/src/cmd/web.rs` |
| ACP transport | ❌ NOT STARTED | Only structs exist in `acp_stream.rs` |
| GitHub integration | ❌ NOT STARTED | Workflow triggers, comment parsing |
| GitLab integration | ❌ NOT STARTED | CI component, Duo support |

### Phase 5 - Hardening 🚧 PARTIAL
| Component | Status | Notes |
|-----------|--------|-------|
| Compatibility suite | ❌ NOT STARTED | No regression tests |
| Convention tests | ✅ Done | 23 tests passing |

### Phase 6 - Release Qualification ❌ NOT STARTED (0%)
| Component | Status | Notes |
|-----------|--------|-------|
| Performance baselines | ❌ NOT STARTED | No benchmarks |
| Non-functional tests | ❌ NOT STARTED | No security/reliability tests |

---

## 2. Detailed Gap Analysis

### 2.1 Core Architecture (PRD 01)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Ownership invariant testing | P1 | core | No automated tests verifying `Project → Session → Message → Part` tree is acyclic | Add unit tests for ownership invariants |
| Stable ID semantics | P1 | core | Identity stability not fully verified in tests | Add serialization/deserialization roundtrip tests |
| Snapshot/Revert model | P1 | storage | Checkpoint creation and restoration not fully implemented | Implement and test checkpoint lifecycle |
| Compaction with shareability | P2 | storage | Compaction preserves resumability not verified | Add integration tests for compaction |
| Workspace path validation | P2 | core | Working directory boundary validation not surveyed | Ensure paths resolve within project/worktree |

### 2.2 Agent System (PRD 02)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Exactly one active primary agent | P0 | agent | No enforcement that session has exactly one active primary agent | Add invariant check in session state |
| Hidden vs visible agents | P1 | agent | build, plan, compaction, title, summary agent visibility not implemented | Implement agent visibility filtering |
| Subagent execution | P0 | agent | Child context creation, result handoff, parent history isolation missing | Implement subagent lifecycle |
| Task/delegation mechanism | P0 | agent | Task tool payload shape not defined/implemented | Define and implement `task` tool |
| Permission inheritance | P1 | agent | Parent → subagent permission scope not implemented | Implement permission inheritance model |

### 2.3 Tools System (PRD 03)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Custom tool discovery | P1 | tools | Scans `TOOL.md` not `.ts/.js` per PRD spec | Update scanner to match PRD format |
| Custom tool registration | P0 | tools | Discovered tools recorded in config but NOT registered with ToolRegistry | Connect discovery to registry |
| MCP tool qualification | P1 | tools | Server-qualified naming not implemented | Implement `<servername>_<toolname>` format |
| Deterministic collision resolution | P2 | tools | Custom vs built-in name collisions not handled | Implement priority-based resolution |
| Result caching | P2 | tools | Cache behavior for safe tools not implemented | Implement cache with invalidation |

### 2.4 MCP System (PRD 04)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Local MCP server connection | P0 | mcp | stdio transport with JSON-RPC not implemented | Implement local server lifecycle |
| Remote MCP server connection | P0 | mcp | HTTP+SSE transport not implemented | Implement remote server client |
| Per-server OAuth | P1 | mcp | OAuth configuration per server not implemented | Implement OAuth flow |
| Tool discovery from MCP | P0 | mcp | Dynamic tool discovery not implemented | Implement tools/list protocol |
| Timeout handling | P1 | mcp | Configurable timeout not implemented | Add timeout configuration |
| Context cost warnings | P2 | mcp | Warning when context usage high not implemented | Add context monitoring |

### 2.5 LSP System (PRD 05)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Built-in LSP server detection | P0 | lsp | Auto-detection by file extension not implemented | Implement language detection |
| Custom LSP registration | P1 | lsp | Custom server via config not implemented | Add config schema support |
| Diagnostics retrieval | P0 | lsp | LSP diagnostics not surfaced to runtime | Implement diagnostics pipeline |
| LSP failure handling | P1 | lsp | Graceful degradation not implemented | Add error handling |
| Experimental LSP tool | P2 | lsp | `goToDefinition`, `findReferences`, etc. not implemented | Gate behind feature flag |

### 2.6 Configuration System (PRD 06)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Config precedence | P0 | config | Remote → global → custom → project → inline not fully implemented | Verify and test merge order |
| JSONC parsing | P1 | config | JSON with comments support not surveyed | Implement JSONC parser |
| Variable expansion | P1 | config | `{env:VAR}` and `{file:PATH}` not fully implemented | Complete variable expansion |
| `tools` → `permission` alias | P1 | config | Legacy conversion not fully tested | Add normalization tests |
| Config ownership boundary | P0 | config | `opencode.json` vs `tui.json` split not enforced | Implement ownership checks |
| Auth/secret storage | P2 | config | `~/.local/share/opencode/auth.json` not surveyed | Implement secure storage |

### 2.7 Server API (PRD 07)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Route registration by resource group | P0 | server | Top-level grouping (Global, Project, Session, Message) not fully implemented | Implement resource groups |
| Auth enforcement | P0 | server | Per-endpoint auth not fully implemented | Implement auth middleware |
| Request validation | P1 | server | Schema validation for requests not fully surveyed | Add validation layer |
| Session/message lifecycle | P0 | server | CRUD operations not fully implemented | Complete session API |
| Streaming endpoints | P1 | server | SSE/websocket not fully implemented | Implement streaming |
| API error shape | P2 | server | Consistent error responses not enforced | Define error schema |

### 2.8 Plugin System (PRD 08)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Hook execution order | P0 | plugin | Non-deterministic HashMap iteration order | Use ordered map (IndexMap) |
| Plugin-provided tool registration | P0 | plugin | No `register_tool()` method in Plugin trait | Add tool registration API |
| Plugin config ownership | P0 | plugin | Not separated from TUI plugin config | Enforce config boundary |
| Plugin failure containment | ✅ Done | plugin | Errors don't crash runtime | Verify with tests |
| Plugin cleanup/unload | P2 | plugin | Cleanup on unload not implemented | Add lifecycle hooks |

### 2.9 TUI System (PRD 09)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Session view rendering | P1 | tui | Markdown, syntax highlighting, diff display partial | Complete rendering |
| Slash commands | P0 | tui | `/compact`, `/connect`, `/help`, etc. not fully implemented | Complete command registry |
| Input model | P1 | tui | Multiline, history, autocomplete partial | Complete input handling |
| File references (`@`) | P1 | tui | Fuzzy file search not fully implemented | Complete `@` handler |
| Shell prefix (`!`) | P2 | tui | Shell command execution not implemented | Add shell handler |
| Keybinding system | P1 | tui | Leader key, categories not fully implemented | Complete keybind system |
| Sidebar | P1 | tui | File tree, MCP/LSP status, diagnostics partial | Complete sidebar sections |
| Home view | P2 | tui | Recent sessions, quick actions not fully implemented | Complete home view |

### 2.10 TUI Plugin API (PRD 15)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| `tui.json` ownership | P0 | tui | Plugin configuration not fully isolated | Implement config boundary |
| Plugin identity/deduplication | P0 | tui | Runtime ID resolution, file vs npm not implemented | Implement deduplication |
| `plugin_enabled` semantics | P0 | tui | Enable/disable at runtime not implemented | Add runtime state |
| Commands registration | P1 | tui | `api.command.register()` not fully implemented | Complete command API |
| Routes registration | P1 | tui | `api.route.register()` not fully implemented | Complete route API |
| Dialogs | P2 | tui | Dialog components not fully implemented | Complete dialog system |
| Slots | P2 | tui | Slot registration not implemented | Add slot system |
| Theme API | P1 | tui | `install()`, `set()`, `mode()` partial | Complete theme API |
| Events | P1 | tui | Event subscription not fully implemented | Complete event system |
| State API | P1 | tui | KV store, live state partial | Complete state API |
| `onDispose` lifecycle | P1 | tui | Cleanup callbacks not fully implemented | Add disposal handling |
| Runtime activate/deactivate | P0 | tui | `api.plugins.activate/deactivate` not implemented | Implement plugin manager |

### 2.11 Provider/Model System (PRD 10)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Provider abstraction | P1 | llm | Registration, credential lookup partial | Complete abstraction |
| Default model selection | P1 | llm | Precedence (CLI → config → last → first) not fully implemented | Verify precedence |
| Per-agent model override | P1 | llm | Agent-specific models not implemented | Add agent config |
| Local model providers | P1 | llm | Ollama, LM Studio support partial | Complete local support |
| Variant/reasoning | P2 | llm | Thinking budget configuration not surveyed | Add variant support |

### 2.12 Skills System (PRD 12)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Compatibility paths | P1 | core | Claude/Agent-style directory loading not implemented | Add compat paths |
| Permission restrictions | P2 | core | Skill usage permission not implemented | Add permission check |

### 2.13 Desktop/Web/ACP (PRD 13)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| Desktop app shell | P0 | cli | WebView integration not implemented | Implement desktop runner |
| Web server mode | P0 | cli | Full web interface not implemented | Implement web server |
| Auth protection | P0 | cli | Password/auth not implemented | Add auth middleware |
| Session sharing | P1 | cli | Cross-interface session sharing not implemented | Implement session sync |
| ACP transport | P0 | cli | Editor integration transport not implemented | Implement ACP protocol |
| Sharing modes | P1 | cli | manual/auto/disabled not fully implemented | Complete sharing |

### 2.14 GitHub/GitLab (PRD 14)

| Gap Item | Severity | Module | Description |修复建议 |
|----------|----------|--------|-------------|---------|
| GitHub workflow triggers | P1 | git | `issue_comment`, `pull_request_review` parsing not implemented | Add trigger handlers |
| Comment/PR parsing | P1 | git | `/oc` or `/opencode` command detection not implemented | Add command parser |
| CI secret loading | P1 | git | GitHub Actions secrets not implemented | Add secret injection |
| GitLab CI component | P2 | git | CI/CD integration not implemented | Add GitLab support |
| GitLab Duo | P3 | git | Experimental, environment-dependent | Mark as experimental |

---

## 3. P0 Blockers (Must Fix)

These items block progress on dependent phases:

1. **Agent**: Exactly one active primary agent invariant not enforced
2. **Agent**: Subagent execution (child context, result handoff) not implemented
3. **Agent**: Task/delegation mechanism not implemented
4. **Tools**: Custom tool registration with ToolRegistry disconnected
5. **MCP**: Local/remote server connection not implemented
6. **MCP**: Tool discovery from MCP servers not implemented
7. **LSP**: Built-in server detection and diagnostics not implemented
8. **Config**: Config precedence not fully enforced
9. **Config**: Config ownership boundary (opencode.json vs tui.json) not enforced
10. **Server**: Route registration by resource group incomplete
11. **Server**: Auth enforcement incomplete
12. **Server**: Session/message lifecycle CRUD incomplete
13. **Plugin**: Hook execution order non-deterministic
14. **Plugin**: Plugin-provided tool registration not implemented
15. **Plugin**: Plugin config not separated from TUI plugin config
16. **TUI Plugin**: `plugin_enabled` runtime semantics not implemented
17. **TUI Plugin**: Runtime activate/deactivate not implemented
18. **Desktop**: Desktop app shell not implemented
19. **Desktop**: Web server mode not implemented
20. **Desktop**: ACP transport not implemented

---

## 4. P1 Issues (Should Fix)

1. **Core**: Ownership invariant automated tests missing
2. **Core**: Stable ID semantics tests missing
3. **Core**: Snapshot/Revert model not fully tested
4. **Agent**: Hidden vs visible agent behavior incomplete
5. **Agent**: Permission inheritance not implemented
6. **Tools**: Custom tool discovery format mismatch (TOOL.md vs .ts/.js)
7. **Tools**: MCP tool qualification not implemented
8. **Config**: JSONC parsing not fully implemented
9. **Config**: Variable expansion not fully implemented
10. **Config**: `tools` → `permission` alias not fully tested
11. **Server**: Request validation incomplete
12. **Server**: Streaming endpoints incomplete
13. **TUI**: Slash commands incomplete
14. **TUI**: Input model (multiline, history, autocomplete) partial
15. **TUI**: File references (`@`) fuzzy search partial
16. **TUI**: Keybinding system incomplete
17. **TUI**: Sidebar sections partial
18. **TUI Plugin**: Commands/routes/dialogs/slots incomplete
19. **TUI Plugin**: Theme API incomplete
20. **TUI Plugin**: Events and state API incomplete
21. **TUI Plugin**: `onDispose` lifecycle incomplete
22. **LLM**: Provider abstraction partial
23. **LLM**: Default model selection precedence incomplete
24. **LLM**: Per-agent model override not implemented
25. **LLM**: Local model providers incomplete
26. **Skills**: Compatibility paths not implemented
27. **Desktop**: Auth protection incomplete
28. **Desktop**: Session sharing between interfaces incomplete
29. **Desktop**: Sharing modes incomplete
30. **Git**: GitHub workflow triggers incomplete
31. **Git**: Comment/PR parsing incomplete
32. **Git**: CI secret loading incomplete

---

## 5. P2 Issues (Nice to Have)

1. **Core**: Compaction with shareability verification missing
2. **Core**: Workspace path validation incomplete
3. **Tools**: Deterministic collision resolution not implemented
4. **Tools**: Result caching not implemented
5. **MCP**: Per-server OAuth configuration incomplete
6. **MCP**: Timeout handling not fully implemented
7. **MCP**: Context cost warnings not implemented
8. **LSP**: Experimental LSP tool not implemented
9. **Config**: Auth/secret storage not fully surveyed
10. **Server**: API error shape consistency not enforced
11. **Plugin**: Plugin cleanup/unload not implemented
12. **TUI**: Shell prefix (`!`) handling not implemented
13. **TUI**: Home view incomplete
14. **LLM**: Variant/reasoning budget not fully implemented
15. **Skills**: Permission restrictions not implemented
16. **Git**: GitLab CI component not implemented

---

## 6. Technical Debt

| Item | Module | Description | Impact |
|------|--------|-------------|--------|
| Hook execution order | plugin | Using HashMap causes non-deterministic iteration | Debugging difficulty, potential security issues |
| Custom tool format | tools | Scanning TOOL.md instead of .ts/.js | Incompatible with PRD spec |
| Discovery-registry gap | tools | Discovered tools not registered | Custom tools don't work |
| Config ownership | config | No enforcement of opencode.json vs tui.json boundary | TUI config could pollute main config |
| Non-deterministic plugins | plugin | Plugin loading order not deterministic | Unpredictable behavior |
| Missing MCP transport | mcp | Only structs exist, no actual transport | MCP integration unusable |
| ACP stubs | cli | ACP structs exist but no transport | Editor integration unusable |

---

## 7. Test Coverage Gaps

Based on the test plan (PRD 16) and backlog (PRD 18):

### Authority Tests (Phase 1)
- ❌ Core ownership tree tests - NOT DONE
- ❌ Config precedence merge tests - NOT DONE  
- ❌ API route-group tests - NOT DONE

### Runtime Tests (Phase 2)
- ❌ Agent primary invariant tests - NOT DONE
- ❌ Subagent execution tests - NOT DONE
- ❌ Tool registry tests - PARTIAL
- ❌ Plugin hook order tests - NOT DONE
- ❌ TUI plugin lifecycle tests - NOT DONE

### Subsystem Tests (Phase 3)
- ❌ MCP integration tests - NOT DONE
- ❌ LSP integration tests - NOT DONE
- ❌ Provider/model tests - NOT DONE
- ❌ Skills discovery tests - PARTIAL

### Interface Tests (Phase 4)
- ❌ Desktop/web smoke tests - NOT DONE
- ❌ ACP handshake tests - NOT DONE
- ❌ GitHub workflow tests - NOT DONE

### Compatibility Tests (Phase 5)
- ❌ tools alias regression suite - NOT DONE
- ❌ Skill path regression suite - NOT DONE
- ❌ Plugin ownership boundary suite - NOT DONE

### Non-functional Tests (Phase 6)
- ❌ Performance baselines - NOT DONE
- ❌ Security tests - NOT DONE
- ❌ Recovery tests - NOT DONE

---

## 8. Recommendations

### Immediate Actions (Next Sprint)

1. **Fix tool registry gap**: Connect custom tool discovery to ToolRegistry
2. **Fix plugin hooks order**: Replace HashMap with IndexMap for deterministic iteration
3. **Implement subagent execution**: Add child context and result handoff
4. **Enforce config ownership**: Add boundary checks between opencode.json and tui.json
5. **Complete MCP transport**: Implement local/remote MCP server connection

### Short-term (2-4 sprints)

1. **Complete Phase 1 authority contracts**: Core entities, config, API
2. **Implement TUI plugin API**: Full PRD 15 compliance
3. **Add MCP/LSP integrations**: Infrastructure subsystems
4. **Start Phase 4 interfaces**: Desktop/web/ACP

### Medium-term

1. **Complete test coverage**: All phases per PRD 17-18
2. **Compatibility hardening**: Regression suites
3. **Performance baselines**: Non-functional testing

### Long-term

1. **Phase 6 release qualification**: Full non-functional testing
2. **GitHub/GitLab integration**: Complete PRD 14
3. **Polish**: UX, documentation, edge cases

---

## 9. Appendix: File Reference Map

| PRD Document | Implementation Location |
|--------------|------------------------|
| 01-core-architecture | `crates/core/src/{project,session,message,part}.rs` |
| 02-agent-system | `crates/agent/src/runtime.rs` |
| 03-tools-system | `crates/tools/src/registry.rs`, `crates/core/src/executor.rs` |
| 04-mcp-system | `crates/mcp/src/` (stubs only) |
| 05-lsp-system | `crates/lsp/src/` (stubs only) |
| 06-configuration-system | `crates/core/src/config.rs` |
| 07-server-api | `crates/server/src/routes/` |
| 08-plugin-system | `crates/plugin/src/lib.rs` |
| 09-tui-system | `crates/tui/src/` |
| 10-provider-model | `crates/llm/src/` |
| 11-formatters | `crates/core/src/formatter.rs` ✅ |
| 12-skills-system | `crates/core/src/skill.rs` ✅ |
| 13-desktop-web | `crates/cli/src/cmd/` (stubs) |
| 14-github-gitlab | `crates/git/src/` |
| 15-tui-plugin-api | `crates/tui/src/` (partial) |
| 16-test-plan | `tests/` |
| 17-rust-test-roadmap | `tests/conventions/` ✅ |
| 18-crate-test-backlog | Per-crate `tests/` directories |
| 19-impl-plan | This document |

---

*Report generated: 2026-04-10*
*Iteration: 4*
*Phase: Phase 1-2 of 6*
