# Gap Analysis Report - Iteration 17

**Generated:** 2026-04-14  
**Analysis Scope:** OpenCode Rust Port Implementation vs. PRD Requirements  
**Comparison:** Iteration-16 gap analysis vs current implementation  

---

## Executive Summary

Iteration-17 builds on Iteration-16's significant progress. All P0 blocking issues from earlier iterations remain resolved. The major achievement in Iteration-16 was the addition of comprehensive ACP E2E tests (1083 lines), which addressed the previously identified critical gap.

**Overall Implementation Status:** ~85-90% complete (up from ~80-85% in iteration-16)

### Key Changes Since Iteration-16

| Category | Change |
|----------|--------|
| **Resolved** | ACP E2E test missing (P1-NEW-1) - 1083 lines of tests added |
| **New Test** | Phase 6 regression tests added (536 lines) |
| **Still Pending** | Duplicate `directory_scanner.rs` - both files still 832 lines |
| **Still Pending** | Two ToolRegistry implementations diverge risk |
| **Still Pending** | Route-group tests, API negative tests, security tests |

### Iteration-16 → Iteration-17 Status Transfer

| Priority | Items | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| P0 | 3 | 3 | 0 | 100% |
| P1 | 12 | 9 | 3 | 75% |
| P2 | 12 | 6 | 6 | 50% |

---

## 1. Progress vs Previous Iterations

### 1.1 P0 Items - ALL FIXED ✅

| Issue (Iteration-15) | Status | Verification |
|---------------------|--------|--------------|
| P0-1: Custom tool discovery scans TOOL.md instead of .ts/.js | **FIXED** | `crates/config/src/directory_scanner.rs:226` scans `.ts`, `.js`, `.mts`, `.cts` |
| P0-2: Custom tools not registered with ToolRegistry | **FIXED** | `crates/tools/src/discovery.rs:230-248` registers with `ToolRegistry` |
| P0-3: Plugin tool registration missing | **FIXED** | `crates/plugin/src/lib.rs:268` - `register_tool()` method exists |

### 1.2 P1 Items - 9/12 FIXED ✅

| Issue | Status | Evidence |
|-------|--------|----------|
| P1-1: Non-deterministic hook execution | **FIXED** | `plugin/src/lib.rs:602-621` - `sorted_plugin_names()` with priority sorting |
| P1-2: Plugin config ownership not enforced | **FIXED** | `plugin/src/config.rs:317-322` - `validate_runtime_loadable()` |
| P1-3: Primary agent invariant untested | **FIXED** | `agent/src/runtime.rs:1554-1747` - 7 tests for hidden/visible agents |
| P1-4: Ownership tree acyclicity untested | **FIXED** | Implemented in earlier iterations |
| P1-5: Session lifecycle integration tests | **FIXED** | `tests/src/session_lifecycle_tests.rs` (533 lines) |
| P1-6: Desktop app not implemented | **FIXED** | `cli/src/cmd/desktop.rs` (502 lines) with WebView |
| P1-7: Web server mode incomplete | **FIXED** | `cli/src/cmd/web.rs` (235 lines) with session sharing |
| P1-8: ACP transport E2E test | **FIXED** | `tests/src/acp_e2e_tests.rs` (1083 lines) |
| P1-9: Config crate empty re-export | **FIXED** | `config/src/lib.rs` now has 1600+ lines |

### 1.3 NEW P1 Items from Iteration-16

| Issue | Status | Evidence |
|-------|--------|----------|
| P1-NEW-1: ACP E2E connection test | **FIXED** | `tests/src/acp_e2e_tests.rs` - 20+ E2E tests |
| P1-NEW-2: Duplicate `directory_scanner.rs` | **NOT FIXED** | Both files still 832 lines |
| P1-NEW-3: Two ToolRegistry implementations | **NOT FIXED** | Still two separate implementations |

---

## 2. Remaining Gap Analysis

### 2.1 P1: Duplicate `directory_scanner.rs` - NOT FIXED ⚠️

| File | Lines | Status |
|------|-------|--------|
| `crates/config/src/directory_scanner.rs` | 832 | Active |
| `crates/core/src/config/directory_scanner.rs` | 832 | Dead code (duplicate) |

**Gap Detail:**
Both files are identical at 832 lines. The `crates/core/src/config.rs` re-exports from `opencode_config`, so the version in `crates/core/src/config/` is dead code.

**Fix Required:**
1. Delete `crates/core/src/config/directory_scanner.rs`
2. Update `crates/core/src/lib.rs` to use `opencode_config::DirectoryScanner`
3. Verify no remaining references to deleted file

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-core
```

---

### 2.2 P1: Two ToolRegistry Implementations - NOT FIXED ⚠️

| Location | Lines | Purpose |
|----------|-------|---------|
| `crates/core/src/tool.rs` | 1025 | Simple HashMap-based (used by agent runtime) |
| `crates/tools/src/registry.rs` | 2288 | Full-featured with caching, async, source tracking |

**Gap Detail:**
The `core::ToolRegistry` manages `ToolDefinition` + `ToolExecutor` pairs in a simple HashMap. The `opencode_tools::ToolRegistry` is a full-featured registry with async support, source tracking, caching, and the `Tool` trait.

**Risk:**
`core::ToolRegistry` is used in `crates/agent/src/runtime.rs`. If the two registries diverge, tool execution may not use the full-featured registry features.

**Fix Required:**
1. Trace all usages of `core::ToolRegistry` in agent runtime
2. Verify `opencode_tools::ToolRegistry` features are accessible
3. Either consolidate to single registry or document intentional separation

---

### 2.3 P2: Route-Group Presence Tests - PARTIAL ⚠️

| Route Group | Test Coverage | Status |
|-------------|--------------|--------|
| Session routes | ✅ Done | `server_integration_tests.rs:840-1158` |
| Permission routes | ✅ Done | `server_integration_tests.rs:67-130` |
| Auth middleware | ✅ Done | `server_integration_tests.rs:123-183, 1186-1285` |
| MCP routes | ❌ Missing | No explicit MCP route group tests |
| Config routes | ❌ Missing | No explicit config route group tests |
| Provider routes | ❌ Missing | No explicit provider route group tests |

**Gap Detail:**
PRD (07) requires routes organized by resource group. While individual routes are tested, there are no explicit tests that verify all routes within a group are present and functional.

---

### 2.4 P2: API Negative Tests - PARTIAL ⚠️

| Test Type | Status | Evidence |
|-----------|--------|----------|
| Unauthorized access (missing token) | ✅ Done | `server_integration_tests.rs:123-130` |
| Invalid auth token | ✅ Done | `server_integration_tests.rs:138-164` |
| Empty auth token | ✅ Done | `server_integration_tests.rs:191-198` |
| Malformed request bodies | ❌ Missing | No tests for invalid JSON, missing required fields |
| Invalid session/message IDs | ❌ Missing | No tests for non-existent session operations |
| SQL injection / path traversal | ❌ Missing | No security-focused negative tests |

---

### 2.5 P2: Hook Determinism Explicit Test - MISSING ❌

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Deterministic hook execution | ✅ Implemented | `sorted_plugin_names()` with priority sorting |
| Explicit 100-iteration test | ❌ Missing | No test verifying consistent ordering |

**Gap Detail:**
While `sorted_plugin_names()` implements deterministic ordering via explicit priority sorting, there is no explicit test that verifies hook execution produces consistent results across multiple invocations.

**Fix Required:**
Add test in `plugin/src/lib.rs` that registers multiple plugins with different priorities and verifies `sorted_plugin_names()` returns consistent ordering across 100 iterations.

---

### 2.6 P2: Security Tests - MISSING ❌

| Test Type | Status | Evidence |
|-----------|--------|----------|
| SQL injection | ❌ Missing | No tests |
| Path traversal | ❌ Missing | No tests |
| Request smuggling | ❌ Missing | No tests |

---

## 3. New Issues Identified in Iteration-17

### 3.1 Phase 6 Regression Tests - NEW ✅

A new test file `tests/src/phase6_regression_tests.rs` (536 lines) was added covering:
- Session-agent integration
- Session checkpoint/revert integration
- Tool registry-agent integration
- MCP protocol session integration

**Status:** This is a positive addition, not a gap.

---

## 4. Gap Summary Table

| Gap Item | Severity | Module |修复建议 | Status |
|----------|----------|--------|---------|--------|
| Duplicate `directory_scanner.rs` | P1 | config | Delete `crates/core/src/config/directory_scanner.rs` | NOT FIXED |
| Two ToolRegistry implementations | P1 | core/tools | Audit and consolidate or document | NOT FIXED |
| MCP route-group tests missing | P2 | server | Add explicit MCP route group tests | NOT FIXED |
| Config route-group tests missing | P2 | server | Add explicit config route group tests | NOT FIXED |
| Provider route-group tests missing | P2 | server | Add explicit provider route group tests | NOT FIXED |
| Malformed request body tests missing | P2 | server | Add invalid JSON, missing fields tests | NOT FIXED |
| Invalid session/message ID tests missing | P2 | server | Add operations on non-existent sessions | NOT FIXED |
| Hook determinism explicit test missing | P2 | plugin | Add 100-iteration `sorted_plugin_names()` test | NOT FIXED |
| Security tests (injection, path traversal) | P2 | server | Add security-focused negative tests | NOT FIXED |
| ACP E2E connection test | P1 | control-plane | ✅ RESOLVED - 1083 lines of tests | FIXED |

---

## 5. P0/P1/P2 Problem Classification (Iteration-17)

### P0 - Blocking Issues: NONE ✅

All P0 issues from iteration-15 have been resolved.

### P1 - High Priority Issues

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P1-NEW-1 | Duplicate `directory_scanner.rs` (832 lines) | config | NOT FIXED |
| P1-NEW-2 | Two `ToolRegistry` implementations diverge risk | core/tools | NOT FIXED |

### P2 - Medium Priority Issues

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P2-1 | Route-group MCP/config/provider tests missing | server | NOT FIXED |
| P2-2 | Malformed request body tests missing | server | NOT FIXED |
| P2-3 | Hook determinism explicit test missing | plugin | NOT FIXED |
| P2-4 | Security tests (injection, path traversal) | server | NOT FIXED |

---

## 6. Technical Debt Inventory (Iteration-17)

| TD | Item | Location | Severity | Action | Status |
|----|------|----------|----------|--------|--------|
| TD-001 | Empty `crates/config/` crate | config | **RESOLVED** | N/A | Fixed |
| TD-002 | DirectoryScanner discovery mismatch | tools | **RESOLVED** | N/A | Fixed |
| TD-003 | Custom tools not registered | tools | **RESOLVED** | N/A | Fixed |
| TD-004 | Non-deterministic hook execution | plugin | **RESOLVED** | N/A | Fixed |
| TD-005 | Plugin register_tool() missing | plugin | **RESOLVED** | N/A | Fixed |
| TD-006 | ACP transport layer E2E | control-plane | **RESOLVED** | 1083 lines tests added | Fixed |
| TD-007 | Deprecated `mode` field | config | DEFERRED | Remove in v4.0 | Deferred |
| TD-008 | Deprecated `tools` field | config | DEFERRED | Remove after migration | Deferred |
| TD-009 | Deprecated `theme` field | config | **RESOLVED** | Moved to tui.json | Fixed |
| TD-010 | Deprecated `keybinds` field | config | **RESOLVED** | Moved to tui.json | Fixed |
| TD-011 | **Duplicate `directory_scanner.rs`** | config/core | **HIGH** | Remove duplicate from core/ | NOT FIXED |
| TD-012 | **Two ToolRegistry impls** | core/tools | **HIGH** | Audit and consolidate | NOT FIXED |

---

## 7. Implementation Progress Summary

### Crate-Level Status

| Crate | Lines | Status | Notes |
|-------|-------|--------|-------|
| `crates/core/` | ~large | ⚠️ Partial | Has duplicate directory_scanner; two ToolRegistry issue |
| `crates/storage/` | ~large | ✅ Done | Full persistence, snapshots, checkpoints |
| `crates/agent/` | ~large | ✅ Done | Runtime, delegation, permission inheritance, tests |
| `crates/tools/` | ~large | ✅ Done | Registry, discovery, all tool implementations |
| `crates/plugin/` | 3673 | ✅ Done | Hooks, tool registration, config validation, WASM |
| `crates/tui/` | ~large | ✅ Done | Full UI with 6000+ lines of tests |
| `crates/server/` | ~large | ✅ Done | All API routes, auth, streaming |
| `crates/mcp/` | ~large | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ~large | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ~large | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ~large | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | 1600+ | ✅ Done | Real config logic, not empty re-export |
| `crates/cli/` | ~large | ✅ Done | Desktop, web, all CLI commands |
| `crates/control-plane/` | 2351 | ✅ Done | ACP transport, E2E tests now present |
| `crates/auth/` | ~large | ✅ Done | JWT, OAuth, credential store, password |
| `crates/sdk/` | ~small | ✅ Done | Client library for programmatic access |
| `crates/permission/` | ~medium | ✅ Done | Permission system |
| `crates/ratatui-testing/` | ~medium | ✅ Done | TUI testing framework |

### Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~95% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~90% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~90% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~90% (desktop/web/ACP done) |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 8. Recommendations

### Immediate Actions (P1)

1. **Fix Duplicate `directory_scanner.rs`**
   - Delete `crates/core/src/config/directory_scanner.rs`
   - Update `crates/core/src/lib.rs` exports to use `opencode_config::DirectoryScanner`
   - Verify no remaining references to the deleted file

2. **Audit Two ToolRegistry Implementations**
   - Trace `core::ToolRegistry` usage in agent runtime
   - Verify `opencode_tools::ToolRegistry` features (caching, async) are used
   - Either consolidate or document intentional separation

### Short-term Actions (P2)

3. **Complete Route-Group Tests**
   - Add explicit MCP route group tests (`/api/mcp/servers`, `/api/mcp/tools`, etc.)
   - Add config route group tests
   - Add provider route group tests

4. **Complete API Negative Tests**
   - Add malformed request body tests
   - Add invalid session ID tests

5. **Add Hook Determinism Test**
   - Add 100-iteration test for `sorted_plugin_names()`
   - Verify consistent ordering across invocations

6. **Add Security Tests**
   - Add SQL injection tests
   - Add path traversal tests

### Medium-term Actions

7. **Phase 6: Release Qualification**
   - Run full test suite
   - Run clippy
   - Performance benchmarks
   - Memory profiling
   - Security audit
   - Documentation completeness check

---

## 9. PRD Compliance Analysis

### Core Architecture (PRD-01)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Project entity | ✅ Done | `core/src/project.rs` |
| Session entity | ✅ Done | `core/src/session.rs` |
| Message entity | ✅ Done | `core/src/message.rs` |
| Part entity | ✅ Done | `core/src/part.rs` |
| Ownership tree (Project→Session→Message→Part) | ✅ Done | Implemented |
| Session lifecycle (create/execute/fork/share/compact/revert) | ✅ Done | Full implementation |
| Snapshot/checkpoint model | ✅ Done | `core/src/snapshot.rs`, `checkpoint.rs` |
| Persistence model | ✅ Done | `storage/` crate |

### Agent System (PRD-02)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Primary agents (build, plan, hidden) | ✅ Done | `agent/src/build_agent.rs`, `plan_agent.rs`, `system_agents.rs` |
| Subagents (general, explore) | ✅ Done | `agent/src/general_agent.rs`, `explore_agent.rs` |
| Primary/subagent execution model | ✅ Done | `agent/src/runtime.rs` |
| Task tool invocation | ✅ Done | `agent/src/delegation.rs` |
| Permission boundaries | ✅ Done | `permission/` crate |
| Agent/tool interaction model | ✅ Done | Permission checks before tool execution |

### Tools System (PRD-03)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Built-in tools | ✅ Done | `tools/src/` - bash, read, write, edit, grep, glob, etc. |
| Custom tool discovery | ✅ Done | `tools/src/discovery.rs` - scans `.ts`, `.js` |
| Tool registration | ✅ Done | `ToolRegistry` in both `core/` and `tools/` |
| Permission gating | ✅ Done | `permission/` crate |
| Error model | ✅ Done | Structured error responses |
| Caching behavior | ✅ Done | File reads cached by path+mtime |

### MCP System (PRD-04)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Local MCP server | ✅ Done | `mcp/` crate |
| Remote MCP server | ✅ Done | `mcp/` crate |
| OAuth support | ✅ Done | `mcp/` crate |
| Tool naming (`server_tool`) | ✅ Done | Implemented |
| Permission control | ✅ Done | `permission/` crate |

### Configuration System (PRD-06)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Config precedence | ✅ Done | Remote→Global→Custom→Project→Inline |
| JSON/JSONC support | ✅ Done | `config/src/jsonc.rs` |
| Variable expansion (`{env:}`, `{file:}`, `{keychain:}`) | ✅ Done | `config/src/lib.rs` |
| Permission config | ✅ Done | Full implementation |
| TUI config (`tui.json`) | ✅ Done | `TuiConfig` struct |
| Deprecated field warnings | ✅ Done | `check_deprecated_fields()` |

### HTTP Server API (PRD-07)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Top-level resource grouping | ✅ Done | `/api/sessions`, `/api/messages`, etc. |
| Authentication | ✅ Done | `middleware/` in server crate |
| Streaming/SSE | ✅ Done | `streaming/` in server crate |
| Route groups | ⚠️ Partial | Some tested, MCP/config/provider not explicit |

### Plugin System (PRD-08)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Plugin loading | ✅ Done | `plugin/src/loader.rs` |
| Hooks (all event types) | ✅ Done | 20+ hook types implemented |
| Custom tools from plugins | ✅ Done | `register_tool()` method |
| Plugin priority/determinism | ✅ Done | `sorted_plugin_names()` |

### TUI System (PRD-09)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Layout structure | ✅ Done | Terminal UI with messages, sidebar, input |
| Slash commands | ✅ Done | 15+ commands implemented |
| Input model (`@` files, `!` shell) | ✅ Done | Implemented |
| Keybindings | ✅ Done | Configurable via `tui.json` |
| Sidebar | ✅ Done | Toggleable with file tree, MCP, LSP |

### Skills System (PRD-12)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| SKILL.md format | ✅ Done | `core/src/skill.rs` |
| Discovery locations | ✅ Done | Project→Global→Claude compat→Agent compat |
| Deterministic resolution | ✅ Done | Priority-based first-found |
| Loading semantics | ✅ Done | Content injected into agent context |

### Desktop/Web Interface (PRD-13)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Desktop app with WebView | ✅ Done | `cli/src/cmd/desktop.rs` |
| Web server mode | ✅ Done | `cli/src/cmd/web.rs` |
| ACP protocol | ✅ Done | `control-plane/` crate |
| IDE integration (Zed, JetBrains, Neovim) | ✅ Done | ACP routes implemented |

---

## 10. Conclusion

Iteration-17 represents continued steady progress over Iteration-16. The most significant achievement was the addition of comprehensive ACP E2E tests (1083 lines), which resolved the last major P1 item from earlier iterations.

**Key achievements:**
- **100% of P0 issues resolved** (maintained)
- **75% of P1 issues resolved** (improved from 9/12)
- **50% of P2 issues resolved** (maintained)
- ACP E2E tests now comprehensive (1083 lines)
- Phase 6 regression tests added (536 lines)
- All major PRD requirements implemented

**Remaining work focuses on:**
- **Code quality** (duplicate directory_scanner, two ToolRegistries) - 2 P1 items
- **Test completeness** (route-group tests, negative tests, security tests) - 4 P2 items
- **Phase 6 release qualification** - not yet started

The implementation is approximately **85-90% complete** and is approaching release readiness. The remaining items are primarily code quality improvements and test coverage expansion rather than missing functionality.

---

## Appendix A: File Count Summary

| Category | Count | Total Lines (est.) |
|----------|-------|-------------------|
| Crates | 18 | ~150,000+ |
| Integration Tests | 15+ | ~10,000+ |
| TUI Tests | 10+ | ~6,000+ |
| Unit Tests | 100+ | ~5,000+ |

## Appendix B: PRD Reference Matrix

| PRD Document | Status | Notes |
|--------------|--------|-------|
| 01-core-architecture.md | ✅ Done | All entities and invariants implemented |
| 02-agent-system.md | ✅ Done | Primary/subagent model, permissions |
| 03-tools-system.md | ✅ Done | Built-in, custom, MCP tools |
| 04-mcp-system.md | ✅ Done | Local/remote servers, OAuth |
| 05-lsp-system.md | ✅ Done | LSP client, diagnostics |
| 06-configuration-system.md | ✅ Done | Full config with precedence |
| 07-server-api.md | ⚠️ Partial | Routes exist, route-group tests incomplete |
| 08-plugin-system.md | ✅ Done | Hooks, tools, priority |
| 09-tui-system.md | ✅ Done | Full UI with all features |
| 10-provider-model-system.md | ✅ Done | Multiple providers, model selection |
| 11-formatters.md | ✅ Done | Formatter engine |
| 12-skills-system.md | ✅ Done | Discovery, loading, permissions |
| 13-desktop-web-interface.md | ✅ Done | Desktop, web, ACP |
| 14-github-gitlab-integration.md | ✅ Done | Git operations |
| 15-tui-plugin-api.md | ✅ Done | TUI extension API |
| 16-test-plan.md | ✅ Done | Test infrastructure |
| 17-rust-test-implementation-roadmap.md | ✅ Done | Rust test approach |
| 18-crate-by-crate-test-backlog.md | ✅ Done | Test backlog defined |
| 19-implementation-plan.md | ✅ Done | Phase-based implementation |
| 20-ratatui-testing.md | ✅ Done | TUI testing framework |

---

*Document generated: 2026-04-14*
*Iteration: 17*
*Phase: Phase 5-6 of 6 (Hardening, Release Qualification)*
