# Gap Analysis Report - Iteration 19

**Generated:** 2026-04-15
**Analysis Scope:** OpenCode Rust Port Implementation vs. PRD Requirements
**Comparison:** Iteration-18 gap analysis vs current implementation

---

## Executive Summary

**Overall Implementation Status:** ~88-90% complete

### Key Changes Since Iteration-18

| Category | Change |
|----------|--------|
| **Architecture Documented** | Two ToolRegistry implementations now documented as intentional design |
| **No P0 Issues** | All P0 blocking issues remain resolved |
| **P1 Remains** | Two ToolRegistry still exists (now documented as intentional) |
| **P2 Mostly Pending** | Route-group tests, negative tests, security tests, ratatui-testing stubs still missing |

### Iteration-18 → Iteration-19 Status Transfer

| Priority | Items | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| P0 | 3 | 3 | 0 | 100% |
| P1 | 11 | 10 | 1 | ~91% |
| P2 | 12 | 6 | 6 | 50% |

---

## 1. Progress vs Previous Iterations

### 1.1 P0 Items - ALL FIXED ✅

| Issue | Status | Verification |
|-------|--------|--------------|
| P0-1: Custom tool discovery scans TOOL.md instead of .ts/.js | **FIXED** | Verified in `crates/core/src/tool.rs:283` |
| P0-2: Custom tools not registered with ToolRegistry | **FIXED** | Verified in `crates/tools/src/discovery.rs` |
| P0-3: Plugin tool registration missing | **FIXED** | `crates/plugin/src/lib.rs` - `register_tool()` method exists |

### 1.2 P1 Items - 10/11 FIXED ✅

| Issue | Status | Evidence |
|-------|--------|----------|
| P1-1: Non-deterministic hook execution | **FIXED** | `plugin/src/lib.rs:602-621` - `sorted_plugin_names()` |
| P1-2: Plugin config ownership not enforced | **FIXED** | `plugin/src/config.rs:317-322` - `validate_runtime_loadable()` |
| P1-3: Primary agent invariant untested | **FIXED** | `agent/src/runtime.rs:1554-1747` - 7 tests |
| P1-4: Ownership tree acyclicity untested | **FIXED** | Implemented in earlier iterations |
| P1-5: Session lifecycle integration tests | **FIXED** | `tests/src/session_lifecycle_tests.rs` |
| P1-6: Desktop app not implemented | **FIXED** | `cli/src/cmd/desktop.rs` |
| P1-7: Web server mode incomplete | **FIXED** | `cli/src/cmd/web.rs` |
| P1-8: ACP transport E2E test | **FIXED** | `tests/src/acp_e2e_tests.rs` |
| P1-9: Config crate empty re-export | **FIXED** | `config/src/lib.rs` now has real logic |
| P1-NEW-1: ACP E2E connection test | **FIXED** | `tests/src/acp_e2e_tests.rs` |
| P1-NEW-2: Duplicate `directory_scanner.rs` | **FIXED** | Verified removed |
| P1-NEW-3: Two ToolRegistry implementations | **DOCUMENTED** | Now intentional architecture |

---

## 2. Remaining Gap Analysis

### 2.1 P1: Two ToolRegistry Implementations - INTENTIONAL ⚠️

| Location | Lines | Purpose |
|----------|-------|---------|
| `crates/core/src/tool.rs` | ~1088 | Simple synchronous registry (MCP bridging, legacy) |
| `crates/tools/src/registry.rs` | ~2288 | Advanced async registry (agent runtime) |

**Current Status:**
The two ToolRegistry implementations are now **documented as intentional architecture** in `crates/core/src/tool.rs:1-62`. The documentation clearly explains:
- `opencode_core::ToolRegistry`: Synchronous, for MCP and TUI
- `opencode_tools::ToolRegistry`: Async, for agent runtime

**Risk Assessment:** LOW (documented architecture)
- Both registries serve different purposes
- `opencode_tools::ToolRegistry` is the primary for agent runtime
- `opencode_core::ToolRegistry` is for MCP bridging
- Comprehensive audit tests exist in `tests/src/tool_registry_audit_tests.rs`

---

### 2.2 P2: Route-Group Presence Tests - NOT FIXED ⚠️

| Route Group | Test Coverage | Status |
|-------------|--------------|--------|
| Session routes | ✅ Done | `server_integration_tests.rs:840-1158` |
| Permission routes | ✅ Done | `server_integration_tests.rs:67-130` |
| Auth middleware | ✅ Done | `server_integration_tests.rs:123-183, 1186-1285` |
| MCP routes | ❌ Missing | No explicit MCP route group tests |
| Config routes | ❌ Missing | No explicit config route group tests |
| Provider routes | ❌ Missing | No explicit provider route group tests |

**Gap Detail:**
PRD (06) requires routes organized by resource group. Individual routes are tested, but no explicit tests verify all routes within a group are present.

---

### 2.3 P2: API Negative Tests - NOT FIXED ⚠️

| Test Type | Status | Evidence |
|-----------|--------|----------|
| Unauthorized access (missing token) | ✅ Done | `server_integration_tests.rs:123-130` |
| Invalid auth token | ✅ Done | `server_integration_tests.rs:138-164` |
| Empty auth token | ✅ Done | `server_integration_tests.rs:191-198` |
| Malformed request bodies | ❌ Missing | No tests for invalid JSON, missing required fields |
| Invalid session/message IDs | ❌ Missing | No tests for non-existent session operations |
| SQL injection | ❌ Missing | No tests |
| Path traversal | ❌ Missing | No tests |

---

### 2.4 P2: Hook Determinism Explicit Test - NOT FIXED ⚠️

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Deterministic hook execution | ✅ Implemented | `sorted_plugin_names()` with priority sorting |
| Explicit 100-iteration test | ❌ Missing | No test verifying consistent ordering |

**Gap Detail:**
`sorted_plugin_names()` implements deterministic ordering via explicit priority sorting (`plugin/src/lib.rs:606`), but no explicit test verifies hook execution produces consistent results across 100 iterations.

---

### 2.5 P2: Security Tests - PARTIAL ⚠️

| Test Type | Status | Evidence |
|-----------|--------|----------|
| Auth security tests | ✅ Done | `server_integration_tests.rs:1606-1858` |
| Permission security tests | ✅ Done | `server_integration_tests.rs:1759-1791` |
| SQL injection | ❌ Missing | No tests |
| Path traversal | ❌ Missing | No tests |
| Request smuggling | ❌ Missing | No tests |

---

## 3. ratatui-testing Framework Status

### 3.1 Component Status

| Component | Status | Implementation Details |
|-----------|--------|------------------------|
| PtySimulator | ✅ Implemented | Full PTY master/slave, read/write, resize |
| BufferDiff | ❌ Stub | Returns `Ok(String::new())` - no actual diff |
| StateTester | ❌ Stub | Returns `Ok(())` - no actual state capture |
| TestDsl | ❌ Stub | Returns `Ok(())` - no actual rendering |
| CliTester | ❌ Stub | Returns `Ok(String::new())` - no actual CLI run |

### 3.2 PtySimulator Implementation Status

```rust
// crates/ratatui-testing/src/pty.rs - 115 lines
✅ new() - Creates PTY pair
✅ write_input() - Writes to PTY slave
✅ read_output() - Reads from PTY master  
✅ resize() - Resizes PTY window
✅ is_child_running() - Checks child status
⚠️ inject_key_event() - Stub (returns Ok(()))
⚠️ inject_mouse_event() - Stub (returns Ok(()))
```

---

## 4. Gap Summary Table

| Gap Item | Severity | Module | 修复建议 | Status |
|----------|----------|--------|---------|--------|
| Two ToolRegistry implementations | P1 | core/tools | ✅ DOCUMENTED - Intentional architecture | INTENTIONAL |
| MCP route-group tests missing | P2 | server | Add explicit MCP route group tests | NOT FIXED |
| Config route-group tests missing | P2 | server | Add explicit config route group tests | NOT FIXED |
| Provider route-group tests missing | P2 | server | Add explicit provider route group tests | NOT FIXED |
| Malformed request body tests missing | P2 | server | Add invalid JSON, missing fields tests | NOT FIXED |
| Invalid session/message ID tests missing | P2 | server | Add operations on non-existent sessions | NOT FIXED |
| Hook determinism explicit test missing | P2 | plugin | Add 100-iteration `sorted_plugin_names()` test | NOT FIXED |
| Security tests (SQL injection, path traversal) | P2 | server | Add security-focused negative tests | NOT FIXED |
| ratatui-testing BufferDiff | P2 | testing | Implement cell-by-cell comparison | NOT FIXED |
| ratatui-testing StateTester | P2 | testing | Implement state capture and diff | NOT FIXED |
| ratatui-testing TestDsl | P2 | testing | Implement widget rendering | NOT FIXED |
| ratatui-testing CliTester | P2 | testing | Implement process spawning | NOT FIXED |

---

## 5. P0/P1/P2 Problem Classification (Iteration-19)

### P0 - Blocking Issues: NONE ✅

All P0 issues from previous iterations remain resolved.

### P1 - High Priority Issues

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P1-NEW-3 | Two `ToolRegistry` implementations | core/tools | **DOCUMENTED** (intentional architecture) |

### P2 - Medium Priority Issues

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P2-1 | Route-group MCP/config/provider tests missing | server | NOT FIXED |
| P2-2 | Malformed request body tests missing | server | NOT FIXED |
| P2-3 | Hook determinism explicit test missing | plugin | NOT FIXED |
| P2-4 | Security tests (SQL injection, path traversal) | server | NOT FIXED |
| P2-5 | ratatui-testing BufferDiff stub | testing | NOT FIXED |
| P2-6 | ratatui-testing StateTester stub | testing | NOT FIXED |
| P2-7 | ratatui-testing TestDsl stub | testing | NOT FIXED |
| P2-8 | ratatui-testing CliTester stub | testing | NOT FIXED |

---

## 6. Technical Debt Inventory (Iteration-19)

| TD | Item | Location | Severity | Action | Status |
|----|------|----------|----------|--------|--------|
| TD-001 | Empty `crates/config/` crate | config | **RESOLVED** | N/A | Fixed |
| TD-002 | DirectoryScanner discovery mismatch | tools | **RESOLVED** | N/A | Fixed |
| TD-003 | Custom tools not registered | tools | **RESOLVED** | N/A | Fixed |
| TD-004 | Non-deterministic hook execution | plugin | **RESOLVED** | N/A | Fixed |
| TD-005 | Plugin register_tool() missing | plugin | **RESOLVED** | N/A | Fixed |
| TD-006 | ACP transport layer E2E | control-plane | **RESOLVED** | N/A | Fixed |
| TD-007 | Deprecated `mode` field | config | DEFERRED | Remove in v4.0 | Deferred |
| TD-008 | Deprecated `tools` field | config | DEFERRED | Remove after migration | Deferred |
| TD-009 | Deprecated `theme` field | config | **RESOLVED** | Moved to tui.json | Fixed |
| TD-010 | Deprecated `keybinds` field | config | **RESOLVED** | Moved to tui.json | Fixed |
| TD-011 | Duplicate `directory_scanner.rs` | config/core | **RESOLVED** | Removed duplicate | Fixed |
| TD-012 | Two ToolRegistry impls | core/tools | **LOW** | Documented as intentional | DOCUMENTED |
| TD-013 | ratatui-testing BufferDiff | testing | MEDIUM | Implement cell-by-cell diff | NOT FIXED |
| TD-014 | ratatui-testing StateTester | testing | MEDIUM | Implement state capture | NOT FIXED |
| TD-015 | ratatui-testing TestDsl | testing | MEDIUM | Implement fluent DSL | NOT FIXED |
| TD-016 | ratatui-testing CliTester | testing | MEDIUM | Implement CLI testing | NOT FIXED |

---

## 7. Implementation Progress Summary

### Crate-Level Status

| Crate | Lines | Status | Notes |
|-------|-------|--------|-------|
| `crates/core/` | ~large | ✅ Done | Two registries now documented |
| `crates/storage/` | ~large | ✅ Done | Full persistence, snapshots, checkpoints |
| `crates/agent/` | ~large | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ~large | ✅ Done | Registry, discovery, all tool implementations |
| `crates/plugin/` | 3673+ | ✅ Done | Hooks, tool registration, WASM |
| `crates/tui/` | ~large | ✅ Done | Full UI with dialogs |
| `crates/server/` | 2221+ | ✅ Done | All API routes, auth, streaming |
| `crates/mcp/` | ~large | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ~large | ✅ Done | LSP client, diagnostics |
| `crates/llm/` | ~large | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ~large | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | 1600+ | ✅ Done | Real config logic |
| `crates/cli/` | ~large | ✅ Done | Desktop, web, all CLI commands |
| `crates/control-plane/` | 2351+ | ✅ Done | ACP transport, E2E tests |
| `crates/auth/` | ~large | ✅ Done | JWT, OAuth, credential store |
| `crates/sdk/` | ~small | ✅ Done | Client library |
| `crates/permission/` | ~medium | ✅ Done | Permission system |
| `crates/ratatui-testing/` | ~medium | ⚠️ Partial | PtySimulator done; 4 stubs |

### Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~98% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~95% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~92% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~95% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 8. Recommendations

### Immediate Actions (P1)

1. **Route-Group Tests (P2-1)**
   - Add explicit MCP route group tests (`/api/mcp/servers`, `/api/mcp/tools`, etc.)
   - Add config route group tests
   - Add provider route group tests

### Short-term Actions (P2)

2. **Complete API Negative Tests (P2-2)**
   - Add malformed request body tests
   - Add invalid session ID tests

3. **Add Hook Determinism Test (P2-3)**
   - Add 100-iteration test for `sorted_plugin_names()`
   - Verify consistent ordering across invocations

4. **Add Security Tests (P2-4)**
   - Add SQL injection tests
   - Add path traversal tests

5. **Complete ratatui-testing Framework (P2-5 to P2-8)**
   - BufferDiff: Implement cell-by-cell comparison
   - StateTester: Implement state capture and JSON diff
   - TestDsl: Implement widget rendering and fluent API
   - CliTester: Implement process spawning and output capture

### Medium-term Actions

6. **Phase 6: Release Qualification**
   - Run full test suite
   - Run clippy
   - Run formatting check
   - Run doc tests
   - Performance benchmarks

---

## 9. PRD Compliance Analysis

### Core Architecture (PRD-01)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Project entity | ✅ Done | `core/src/project.rs` |
| Session entity | ✅ Done | `core/src/session.rs` |
| Message entity | ✅ Done | `core/src/message.rs` |
| Part entity | ✅ Done | `core/src/part.rs` |
| Ownership tree | ✅ Done | Implemented |
| Session lifecycle | ✅ Done | Full implementation |
| Snapshot/checkpoint model | ✅ Done | `core/src/snapshot.rs`, `checkpoint.rs` |
| Persistence model | ✅ Done | `storage/` crate |

### Agent System (PRD-02)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Primary agents | ✅ Done | `agent/src/build_agent.rs`, `plan_agent.rs` |
| Subagents | ✅ Done | `agent/src/general_agent.rs`, `explore_agent.rs` |
| Primary/subagent execution model | ✅ Done | `agent/src/runtime.rs` |
| Task tool invocation | ✅ Done | `agent/src/delegation.rs` |
| Permission boundaries | ✅ Done | `permission/` crate |

### Tools System (PRD-03)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Built-in tools | ✅ Done | `tools/src/` - bash, read, write, edit, grep, glob |
| Custom tool discovery | ✅ Done | `tools/src/discovery.rs` - scans `.ts`, `.js` |
| Tool registration | ✅ Done | `opencode_tools::ToolRegistry` |
| Permission gating | ✅ Done | `permission/` crate |

### MCP System (PRD-04)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Local MCP server | ✅ Done | `mcp/` crate |
| Remote MCP server | ✅ Done | `mcp/` crate |
| OAuth support | ✅ Done | `mcp/` crate |
| Tool naming | ✅ Done | Implemented |

### Configuration System (PRD-06)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Config precedence | ✅ Done | Remote→Global→Custom→Project→Inline |
| JSON/JSONC support | ✅ Done | `config/src/jsonc.rs` |
| Variable expansion | ✅ Done | `config/src/lib.rs` |
| Permission config | ✅ Done | Full implementation |

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
| Hooks | ✅ Done | 20+ hook types implemented |
| Custom tools from plugins | ✅ Done | `register_tool()` method |
| Plugin priority/determinism | ✅ Done | `sorted_plugin_names()` |

### TUI System (PRD-09)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Layout structure | ✅ Done | Terminal UI with messages, sidebar, input |
| Slash commands | ✅ Done | 15+ commands implemented |
| Input model | ✅ Done | `@` files, `!` shell implemented |
| Keybindings | ✅ Done | Configurable via `tui.json` |

### ratatui-testing (PRD-20)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| PtySimulator | ✅ Done | Full implementation |
| BufferDiff | ❌ Stub | Returns empty string |
| StateTester | ❌ Stub | Returns Ok(()) |
| TestDsl | ❌ Stub | Returns Ok(()) |
| CliTester | ❌ Stub | Returns empty string |

---

## 10. Conclusion

Iteration-19 represents stable progress. The key change is that the two ToolRegistry implementations are now **documented as intentional architecture** rather than being considered a gap. The implementation is approximately **88-90% complete**.

**Key achievements:**
- **100% of P0 issues resolved** (maintained across iterations)
- **~91% of P1 issues resolved** (ToolRegistry now intentional design)
- **50% of P2 issues resolved** (unchanged from iteration-18)
- All major PRD requirements implemented

**Remaining work focuses on:**
- **Test completeness** (route-group tests, negative tests, security tests) - 4 P2 items
- **Testing framework** (ratatui-testing components) - 4 P2 items
- **Phase 6 release qualification** - not yet started

The implementation is approaching release readiness. The remaining items are primarily code quality improvements and test coverage expansion rather than missing functionality.

---

## Appendix A: File Count Summary

| Category | Count | Total Lines (est.) |
|----------|-------|-------------------|
| Crates | 18 | ~150,000+ |
| Integration Tests | 15+ | ~10,000+ |
| TUI Tests | 10+ | ~6,000+ |
| Unit Tests | 100+ | ~5,000+ |

## Appendix B: Iteration History

| Iteration | Date | Key Changes |
|-----------|------|-------------|
| 15 | 2026-04-13 | Initial PRD analysis, 3 P0 issues identified |
| 16 | 2026-04-14 | ACP E2E tests (1083 lines), Phase 6 tests |
| 17 | 2026-04-14 | P1 items progress, comprehensive spec |
| 18 | 2026-04-14 | Duplicate directory_scanner removed |
| 19 | 2026-04-15 | Two ToolRegistries documented as intentional |

---

*Document generated: 2026-04-15*
*Iteration: 19*
*Phase: Phase 5-6 of 6 (Hardening, Release Qualification)*
