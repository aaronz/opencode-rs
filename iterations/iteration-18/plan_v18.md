# Implementation Plan - Iteration 18

**Version:** 18.0  
**Generated:** 2026-04-14  
**Based on:** Spec v18, Gap Analysis  
**Status:** Draft  

---

## 1. Overview

**Overall Completion:** ~87-90%  
**Phase Status:** Phase 5-6 of 6 (Hardening, Release Qualification)  
**P0 Status:** ALL FIXED ✅  
**P1 Remaining:** 1 item (ToolRegistry divergence)  
**P2 Remaining:** 6 items (route-group tests, malformed requests, hook determinism, security tests, ratatui-testing components)  

---

## 2. Priority Matrix

| Priority | Items | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| **P0** | 3 | 3 | 0 | **100%** ✅ |
| **P1** | 11 | 10 | 1 | ~91% |
| **P2** | 12 | 6 | 6 | 50% |

---

## 3. P0 Issues - ALL RESOLVED ✅

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| P0-1 | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | ✅ FIXED |
| P0-2 | Custom tools not registered with ToolRegistry | tools | ✅ FIXED |
| P0-3 | Plugin tool registration missing | plugin | ✅ FIXED |

---

## 4. P1 Issues - MUST FIX

### 4.1 P1-NEW-3: Two ToolRegistry Implementations (TD-012)

**Severity:** HIGH  
**Module:** core/tools  
**Files:**
- `crates/core/src/tool.rs` (~1025 lines) - Simple HashMap-based
- `crates/tools/src/registry.rs` (~2288 lines) - Full-featured with caching, async

**Root Cause:** `opencode_core::ToolRegistry` is re-exported but not actively used in runtime. The agent runtime primarily uses `opencode_tools::ToolRegistry`.

**Risk:** 
- Confusion about which registry to use
- Maintenance burden of keeping two implementations in sync

**Fix Required:**
1. Audit all usages of `opencode_core::ToolRegistry`
2. If truly dead code, remove from `crates/core/src/tool.rs`
3. Update `crates/core/src/lib.rs` exports
4. Verify with `cargo build --all-features && cargo test -p opencode-core`

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-core
```

---

## 5. P2 Issues - Short-term

### 5.1 P2-1: Route-group MCP/config/provider Tests Missing

**Module:** server  
**Reference:** FR-004  
**Current Coverage:** Session, permission, auth routes tested  
**Missing:**
- MCP route group tests (`/api/mcp/servers`, `/api/mcp/tools`, etc.)
- Config route group tests
- Provider route group tests

**Fix Required:**
Add explicit route-group enumeration tests verifying all routes within each group are present and functional.

---

### 5.2 P2-2: Malformed Request Body Tests Missing

**Module:** server  
**Reference:** FR-004  
**Current Coverage:** Auth tests done  
**Missing:**
- Invalid JSON tests
- Missing required fields tests
- Invalid session/message ID tests

**Fix Required:**
Add negative tests for malformed requests, invalid IDs, and edge cases.

---

### 5.3 P2-3: Hook Determinism Explicit Test Missing

**Module:** plugin  
**Reference:** FR-008  
**Current Status:** `sorted_plugin_names()` with priority sorting implemented at `plugin/src/lib.rs:602-621`  
**Missing:** 100-iteration test verifying consistent ordering

**Fix Required:**
Add test in `plugin/src/lib.rs` that registers multiple plugins with different priorities and verifies `sorted_plugin_names()` returns consistent ordering across 100 iterations.

---

### 5.4 P2-4: Security Tests Missing

**Module:** server  
**Reference:** FR-004  
**Missing:**
- SQL injection tests
- Path traversal tests
- Request smuggling tests

**Fix Required:**
Add security-focused negative tests for injection attacks and path traversal.

---

### 5.5 P2-5 to P2-8: ratatui-testing Framework Components

| ID | Component | Status | Reference |
|----|-----------|--------|-----------|
| P2-5 | BufferDiff | Stub | FR-023.2 |
| P2-6 | StateTester | Stub | FR-023.3 |
| P2-7 | TestDsl | Stub | FR-023.4 |
| P2-8 | CliTester | ✅ Done | FR-023.5 |

---

## 6. ratatui-testing Framework Status (FR-023)

### 6.1 PtySimulator (FR-023.1) - Partial

**Status:** ⚠️ Partial Implementation

**Implemented:**
- `new(command: &[&str])` - Creates PTY master/slave pair on Unix
- `write_input(input: &str)` - Writes strings to PTY slave
- `read_output(timeout: Duration)` - Reads from PTY master with timeout
- `resize(cols: u16, rows: u16)` - Resizes PTY window
- `is_child_running()` - Checks child process status

**Not Implemented (Stubs):**
- `inject_key_event(event: KeyEvent)` - Returns `Ok(())` without injection
- `inject_mouse_event(event: MouseEvent)` - Returns `Ok(())` without injection

### 6.2 BufferDiff (FR-023.2) - Stub

**Status:** ❌ Stub Implementation

**Current Behavior:** Returns empty string

**Required:** Cell-by-cell Buffer comparison, x,y position reporting, configurable ignore options, human-readable diff

### 6.3 StateTester (FR-023.3) - Stub

**Status:** ❌ Stub Implementation

**Current Behavior:** Returns `Ok(())`

**Required:** JSON state capture, snapshot comparison, JSON diff reporting

### 6.4 TestDsl (FR-023.4) - Stub

**Status:** ❌ Stub Implementation

**Current Behavior:** Returns `Ok(())`

**Required:** Widget rendering to Buffer, composition of PTY/BufferDiff/StateTester, fluent API, wait-for predicate

### 6.5 CliTester (FR-023.5) - ✅ Done

**Status:** ✅ Full Implementation

**Implemented:**
- `new(command: &str)` - Creates CliTester with command
- `arg(arg: &str)` - Fluent builder for single argument
- `args(args: &[&str])` - Fluent builder for multiple arguments
- `env(key: &str, value: &str)` - Set environment variable
- `envs(vars: HashMap<&str, &str>)` - Set multiple environment variables
- `working_dir(dir: PathBuf)` - Set working directory
- `with_temp_dir()` - Create temp directory for process
- `run()` - Execute command, capture output
- `run_with_timeout(timeout: Duration)` - Execute with timeout

**CliOutput struct:**
- `stdout: String` - Captured stdout
- `stderr: String` - Captured stderr
- `exit_code: i32` - Process exit code
- `success: bool` - Whether exit code was 0

**Assertion methods:**
- `assert_success()` - Assert exit code is 0
- `assert_exit_code(expected: i32)` - Assert specific exit code
- `assert_stdout_contains(expected: &str)` - Assert stdout contains string
- `assert_stderr_contains(expected: &str)` - Assert stderr contains string

**Tests:** 13 tests pass (spawn, exit code, stdout/stderr capture, temp cleanup, env vars, assertions)

---

## 7. Implementation Order

### Immediate (P1 - Blockers)

1. **Audit and Remove Dead ToolRegistry** (P1-NEW-3)
   - Trace `opencode_core::ToolRegistry` usage
   - Verify dead code
   - Remove if appropriate
   - Update exports
   - Verify with cargo build/test

### Short-term (P2 - Quality)

2. **Add Route-Group Tests** (P2-1)
   - MCP route group tests
   - Config route group tests
   - Provider route group tests

3. **Add Malformed Request Tests** (P2-2)
   - Invalid JSON tests
   - Missing required fields tests
   - Invalid session/message ID tests

4. **Add Hook Determinism Test** (P2-3)
   - 100-iteration test for `sorted_plugin_names()`

5. **Add Security Tests** (P2-4)
   - SQL injection tests
   - Path traversal tests

### Medium-term (ratatui-testing)

6. **Implement BufferDiff** (P2-5, FR-023.2)
7. **Implement StateTester** (P2-6, FR-023.3)
8. **Implement TestDsl** (P2-7, FR-023.4)
9. **Implement CliTester** (P2-8, FR-023.5)

### Final (Phase 6)

10. **Release Qualification**
    - Full test suite
    - Clippy
    - Benchmarks
    - Security audit

---

## 8. Technical Debt Status

| ID | Item | Severity | Status |
|----|------|----------|--------|
| TD-001 | Empty `crates/config/` crate | RESOLVED | ✅ |
| TD-002 | DirectoryScanner discovery mismatch | RESOLVED | ✅ |
| TD-003 | Custom tools not registered | RESOLVED | ✅ |
| TD-004 | Non-deterministic hook execution | RESOLVED | ✅ |
| TD-005 | Plugin register_tool() missing | RESOLVED | ✅ |
| TD-006 | ACP transport layer E2E | RESOLVED | ✅ |
| TD-007 | Deprecated `mode` field | Deferred | ⏳ |
| TD-008 | Deprecated `tools` field | Deferred | ⏳ |
| TD-009 | Deprecated `theme` field | RESOLVED | ✅ |
| TD-010 | Deprecated `keybinds` field | RESOLVED | ✅ |
| TD-011 | Duplicate `directory_scanner.rs` | RESOLVED | ✅ |
| **TD-012** | Two ToolRegistry impls | HIGH | ❌ |

---

## 9. Crate-Level Status

| Crate | Phase | Status | Notes |
|-------|-------|--------|-------|
| `crates/core/` | 1 | ⚠️ Partial | Two ToolRegistry issue remains |
| `crates/storage/` | 1 | ✅ Done | Full persistence, snapshots, checkpoints |
| `crates/permission/` | 1 | ✅ Done | Permission system |
| `crates/server/` | 1, 4 | ✅ Done | All API routes, auth, streaming |
| `crates/agent/` | 2 | ✅ Done | Runtime, delegation, permission inheritance, tests |
| `crates/tools/` | 2, 3 | ✅ Done | Registry, discovery, all tool implementations |
| `crates/plugin/` | 2 | ✅ Done | Hooks, tool registration, config validation, WASM |
| `crates/tui/` | 2, 3 | ✅ Done | Full UI with 6000+ lines of tests |
| `crates/mcp/` | 3 | ✅ Done | Full MCP implementation |
| `crates/lsp/` | 3 | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | 3 | ✅ Done | Multiple providers, model selection |
| `crates/git/` | 4 | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | 1 | ✅ Done | Real config logic, not empty re-export |
| `crates/cli/` | 4 | ✅ Done | Desktop, web, all CLI commands |
| `crates/control-plane/` | 4 | ✅ Done | ACP transport, E2E tests present |
| `crates/auth/` | 1 | ✅ Done | JWT, OAuth, credential store, password |
| `crates/sdk/` | 4 | ✅ Done | Client library for programmatic access |
| `crates/permission/` | 1 | ✅ Done | Permission system |
| `ratatui-testing/` | 2, 3 | ⚠️ Partial | PtySimulator done; 4 stubs remaining |

---

## 10. Phase Status

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

*Plan generated: 2026-04-14*  
*Iteration: 18*  
*Priority: P1 blockers first, then P2 quality, then ratatui-testing, then release qualification*