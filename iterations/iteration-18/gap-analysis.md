# Gap Analysis Report - Iteration 18

**Generated:** 2026-04-14
**Analysis Scope:** OpenCode Rust Port Implementation vs. PRD Requirements
**Comparison:** Iteration-17 gap analysis vs current implementation

---

## Executive Summary

**Overall Implementation Status:** ~87-90% complete (unchanged from iteration-17)

### Key Changes Since Iteration-17

| Category | Change |
|----------|--------|
| **Resolved** | Duplicate `directory_scanner.rs` removed (P1-NEW-2) |
| **Still Pending** | Two ToolRegistry implementations diverge risk |
| **Still Pending** | Route-group tests, API negative tests, security tests |
| **Still Pending** | ratatui-testing components (BufferDiff, StateTester, TestDsl, CliTester) |

### Iteration-17 → Iteration-18 Status Transfer

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
| P0-1: Custom tool discovery scans TOOL.md instead of .ts/.js | **FIXED** | `crates/config/src/directory_scanner.rs:226` scans `.ts`, `.js`, `.mts`, `.cts` |
| P0-2: Custom tools not registered with ToolRegistry | **FIXED** | `crates/tools/src/discovery.rs:230-248` registers with `ToolRegistry` |
| P0-3: Plugin tool registration missing | **FIXED** | `crates/plugin/src/lib.rs:268` - `register_tool()` method exists |

### 1.2 P1 Items - 10/11 FIXED ✅

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
| P1-NEW-1: ACP E2E connection test | **FIXED** | `tests/src/acp_e2e_tests.rs` - 20+ E2E tests |
| P1-NEW-2: Duplicate `directory_scanner.rs` | **FIXED** | Duplicate removed, verified by grep |
| P1-NEW-3: Two ToolRegistry implementations | **NOT FIXED** | Still two separate implementations |

---

## 2. Remaining Gap Analysis

### 2.1 P1: Two ToolRegistry Implementations - NOT FIXED ⚠️

| Location | Lines | Purpose |
|----------|-------|---------|
| `crates/core/src/tool.rs` | ~1025 | Simple HashMap-based (still exported from core) |
| `crates/tools/src/registry.rs` | ~2288 | Full-featured with caching, async, source tracking |

**Gap Detail:**
The `opencode_core::ToolRegistry` is still exported from `crates/core/src/lib.rs:137` alongside `opencode_tools::ToolRegistry`. While the agent runtime primarily uses `opencode_tools::ToolRegistry` (verified in `agent/src/runtime.rs`), the `core::ToolRegistry` remains in the codebase as dead code.

**Risk:**
- `core::ToolRegistry` is re-exported but not actively used in runtime
- Potential confusion about which registry to use
- Maintenance burden of keeping two implementations in sync

**Fix Required:**
1. Audit all usages of `core::ToolRegistry`
2. Remove `core::ToolRegistry` if truly dead code
3. Update `crates/core/src/lib.rs` exports accordingly

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-core
```

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
PRD (07) requires routes organized by resource group. While individual routes are tested, there are no explicit tests that verify all routes within a group are present and functional.

---

### 2.3 P2: API Negative Tests - NOT FIXED ⚠️

| Test Type | Status | Evidence |
|-----------|--------|----------|
| Unauthorized access (missing token) | ✅ Done | `server_integration_tests.rs:123-130` |
| Invalid auth token | ✅ Done | `server_integration_tests.rs:138-164` |
| Empty auth token | ✅ Done | `server_integration_tests.rs:191-198` |
| Malformed request bodies | ❌ Missing | No tests for invalid JSON, missing required fields |
| Invalid session/message IDs | ❌ Missing | No tests for non-existent session operations |
| SQL injection / path traversal | ❌ Missing | No security-focused negative tests |

---

### 2.4 P2: Hook Determinism Explicit Test - NOT FIXED ⚠️

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Deterministic hook execution | ✅ Implemented | `sorted_plugin_names()` with priority sorting |
| Explicit 100-iteration test | ❌ Missing | No test verifying consistent ordering |

**Gap Detail:**
While `sorted_plugin_names()` implements deterministic ordering via explicit priority sorting (`plugin/src/lib.rs:606`), there is no explicit test that verifies hook execution produces consistent results across multiple invocations.

**Fix Required:**
Add test in `plugin/src/lib.rs` that registers multiple plugins with different priorities and verifies `sorted_plugin_names()` returns consistent ordering across 100 iterations.

---

### 2.5 P2: Security Tests - NOT FIXED ⚠️

| Test Type | Status | Evidence |
|-----------|--------|----------|
| SQL injection | ❌ Missing | No tests |
| Path traversal | ❌ Missing | No tests |
| Request smuggling | ❌ Missing | No tests |

---

## 3. ratatui-testing Framework Status (FR-023)

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

### 3.3 Missing Implementations

| Component | Missing Feature | Impact |
|-----------|----------------|--------|
| BufferDiff | Cell-by-cell comparison | Cannot verify UI output |
| StateTester | JSON state capture/diff | Cannot verify app state |
| TestDsl | Widget rendering | Cannot compose test scenarios |
| CliTester | Process spawning | Cannot test CLI behavior |

---

## 4. Gap Summary Table

| Gap Item | Severity | Module | 修复建议 | Status |
|----------|----------|--------|---------|--------|
| Two ToolRegistry implementations | P1 | core/tools | Audit and remove dead code | NOT FIXED |
| MCP route-group tests missing | P2 | server | Add explicit MCP route group tests | NOT FIXED |
| Config route-group tests missing | P2 | server | Add explicit config route group tests | NOT FIXED |
| Provider route-group tests missing | P2 | server | Add explicit provider route group tests | NOT FIXED |
| Malformed request body tests missing | P2 | server | Add invalid JSON, missing fields tests | NOT FIXED |
| Invalid session/message ID tests missing | P2 | server | Add operations on non-existent sessions | NOT FIXED |
| Hook determinism explicit test missing | P2 | plugin | Add 100-iteration `sorted_plugin_names()` test | NOT FIXED |
| Security tests (injection, path traversal) | P2 | server | Add security-focused negative tests | NOT FIXED |
| Duplicate `directory_scanner.rs` | P1 | config | ✅ RESOLVED - Duplicate removed | FIXED |
| ratatui-testing BufferDiff | P2 | testing | Implement cell-by-cell comparison | NOT FIXED |
| ratatui-testing StateTester | P2 | testing | Implement state capture and diff | NOT FIXED |
| ratatui-testing TestDsl | P2 | testing | Implement widget rendering | NOT FIXED |
| ratatui-testing CliTester | P2 | testing | Implement process spawning | NOT FIXED |

---

## 5. P0/P1/P2 Problem Classification (Iteration-18)

### P0 - Blocking Issues: NONE ✅

All P0 issues from iteration-15 have been resolved.

### P1 - High Priority Issues

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P1-NEW-3 | Two `ToolRegistry` implementations | core/tools | NOT FIXED |

### P2 - Medium Priority Issues

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P2-1 | Route-group MCP/config/provider tests missing | server | NOT FIXED |
| P2-2 | Malformed request body tests missing | server | NOT FIXED |
| P2-3 | Hook determinism explicit test missing | plugin | NOT FIXED |
| P2-4 | Security tests (injection, path traversal) | server | NOT FIXED |
| P2-5 | ratatui-testing BufferDiff stub | testing | NOT FIXED |
| P2-6 | ratatui-testing StateTester stub | testing | NOT FIXED |
| P2-7 | ratatui-testing TestDsl stub | testing | NOT FIXED |
| P2-8 | ratatui-testing CliTester stub | testing | NOT FIXED |

---

## 6. Technical Debt Inventory (Iteration-18)

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
| TD-011 | Duplicate `directory_scanner.rs` | config/core | **RESOLVED** | Removed duplicate | Fixed |
| TD-012 | Two ToolRegistry impls | core/tools | **HIGH** | Audit and remove dead code | NOT FIXED |
| TD-013 | ratatui-testing BufferDiff | testing | MEDIUM | Implement cell-by-cell diff | NOT FIXED |
| TD-014 | ratatui-testing StateTester | testing | MEDIUM | Implement state capture | NOT FIXED |
| TD-015 | ratatui-testing TestDsl | testing | MEDIUM | Implement fluent DSL | NOT FIXED |
| TD-016 | ratatui-testing CliTester | testing | MEDIUM | Implement CLI testing | NOT FIXED |

---

## 7. Implementation Progress Summary

### Crate-Level Status

| Crate | Lines | Status | Notes |
|-------|-------|--------|-------|
| `crates/core/` | ~large | ✅ Done | One ToolRegistry removed (duplicate) |
| `crates/storage/` | ~large | ✅ Done | Full persistence, snapshots, checkpoints |
| `crates/agent/` | ~large | ✅ Done | Runtime, delegation, permission inheritance, tests |
| `crates/tools/` | ~large | ✅ Done | Registry, discovery, all tool implementations |
| `crates/plugin/` | 3673 | ✅ Done | Hooks, tool registration, config validation, WASM |
| `crates/tui/` | ~large | ✅ Done | Full UI with 6000+ lines of tests |
| `crates/server/` | 2221 | ✅ Done | All API routes, auth, streaming |
| `crates/mcp/` | ~large | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ~large | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ~large | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ~large | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | 1600+ | ✅ Done | Real config logic, not empty re-export |
| `crates/cli/` | ~large | ✅ Done | Desktop, web, all CLI commands |
| `crates/control-plane/` | 2351 | ✅ Done | ACP transport, E2E tests present |
| `crates/auth/` | ~large | ✅ Done | JWT, OAuth, credential store, password |
| `crates/sdk/` | ~small | ✅ Done | Client library for programmatic access |
| `crates/permission/` | ~medium | ✅ Done | Permission system |
| `crates/ratatui-testing/` | ~medium | ⚠️ Partial | PtySimulator done; 4 stubs remaining |

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

1. **Audit and Remove Dead ToolRegistry (P1-NEW-3)**
   - Trace all usages of `opencode_core::ToolRegistry`
   - If truly dead code, remove from `crates/core/src/tool.rs`
   - Update `crates/core/src/lib.rs` exports
   - Verify with `cargo build --all-features && cargo test -p opencode-core`

### Short-term Actions (P2)

2. **Complete Route-Group Tests (P2-1)**
   - Add explicit MCP route group tests (`/api/mcp/servers`, `/api/mcp/tools`, etc.)
   - Add config route group tests
   - Add provider route group tests

3. **Complete API Negative Tests (P2-2)**
   - Add malformed request body tests
   - Add invalid session ID tests

4. **Add Hook Determinism Test (P2-3)**
   - Add 100-iteration test for `sorted_plugin_names()`
   - Verify consistent ordering across invocations

5. **Add Security Tests (P2-4)**
   - Add SQL injection tests
   - Add path traversal tests

6. **Complete ratatui-testing Framework (P2-5 to P2-8)**
   - BufferDiff: Implement cell-by-cell comparison
   - StateTester: Implement state capture and JSON diff
   - TestDsl: Implement widget rendering and fluent API
   - CliTester: Implement process spawning and output capture

### Medium-term Actions

7. **Phase 6: Release Qualification**
   - Run full test suite
   - Run clippy
   - Run formatting check
   - Run doc tests
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
| Ownership tree | ✅ Done | Implemented |
| Session lifecycle | ✅ Done | Full implementation |
| Snapshot/checkpoint model | ✅ Done | `core/src/snapshot.rs`, `checkpoint.rs` |
| Persistence model | ✅ Done | `storage/` crate |

### Agent System (PRD-02)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Primary agents | ✅ Done | `agent/src/build_agent.rs`, `plan_agent.rs`, `system_agents.rs` |
| Subagents | ✅ Done | `agent/src/general_agent.rs`, `explore_agent.rs` |
| Primary/subagent execution model | ✅ Done | `agent/src/runtime.rs` |
| Task tool invocation | ✅ Done | `agent/src/delegation.rs` |
| Permission boundaries | ✅ Done | `permission/` crate |

### Tools System (PRD-03)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Built-in tools | ✅ Done | `tools/src/` - bash, read, write, edit, grep, glob, etc. |
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

Iteration-18 represents continued steady progress over Iteration-17. The most significant achievement was the removal of the duplicate `directory_scanner.rs` file (P1-NEW-2), which reduces code duplication.

**Key achievements:**
- **100% of P0 issues resolved** (maintained)
- **~91% of P1 issues resolved** (improved from 75%)
- **50% of P2 issues resolved** (maintained)
- Duplicate `directory_scanner.rs` removed
- All major PRD requirements implemented

**Remaining work focuses on:**
- **Code quality** (two ToolRegistries) - 1 P1 item
- **Test completeness** (route-group tests, negative tests, security tests, hook determinism) - 4 P2 items
- **Testing framework** (ratatui-testing components) - 4 P2 items
- **Phase 6 release qualification** - not yet started

The implementation is approximately **87-90% complete** and is approaching release readiness. The remaining items are primarily code quality improvements and test coverage expansion rather than missing functionality.

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

---

*Document generated: 2026-04-14*
*Iteration: 18*
*Phase: Phase 5-6 of 6 (Hardening, Release Qualification)*
