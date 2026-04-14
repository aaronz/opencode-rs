# Specification Document - Iteration 18

**Project:** OpenCode Rust Port  
**Iteration:** 18  
**Date:** 2026-04-14  
**Phase:** Phase 5 (Hardening) / Phase 6 (Release Qualification)  

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Implementation Status](#2-implementation-status)
3. [ratatui-testing Framework (FR-023)](#3-ratatui-testing-framework-fr-023)
4. [Remaining Gaps](#4-remaining-gaps)
5. [Acceptance Criteria](#5-acceptance-criteria)
6. [Technical Debt](#6-technical-debt)
7. [Recommendations](#7-recommendations)

---

## 1. Executive Summary

**Overall Implementation Status:** ~87-90% complete

### Progress Since Iteration-17

| Category | Change |
|----------|--------|
| **Resolved** | Duplicate `directory_scanner.rs` removed (P1-NEW-2) |
| **Still Pending** | Two ToolRegistry implementations diverge risk (P1-NEW-3) |
| **Still Pending** | Route-group tests, API negative tests, security tests |
| **Still Pending** | ratatui-testing components (BufferDiff, StateTester, TestDsl, CliTester) |

### Priority Classification

| Priority | Total | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| P0 | 3 | 3 | 0 | 100% |
| P1 | 11 | 10 | 1 | ~91% |
| P2 | 12 | 6 | 6 | 50% |

---

## 2. Implementation Status

### 2.1 Crate-Level Status

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

### 2.2 Phase Status

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

## 3. ratatui-testing Framework (FR-023)

**Feature ID:** FR-023  
**PRD Reference:** PRD: ratatui-testing  
**Implementation Path:** `opencode-rust/ratatui-testing/`

### 3.1 Component Status

| Component | Status | Implementation | Lines | Notes |
|-----------|--------|----------------|-------|-------|
| PtySimulator | ⚠️ Partial | `src/pty.rs` | 115 | PTY done, key/mouse injection stubs |
| BufferDiff | ❌ Stub | `src/diff.rs` | 19 | Returns empty string |
| StateTester | ❌ Stub | `src/state.rs` | 22 | Returns Ok(()) |
| TestDsl | ❌ Stub | `src/dsl.rs` | 19 | Returns Ok(()) |
| CliTester | ❌ Stub | `src/cli.rs` | 19 | Returns empty string |

### 3.2 PtySimulator - FR-023.1

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

**Dependencies:** `portable-pty = "0.8"`, `crossterm` (events feature)

---

### 3.3 BufferDiff - FR-023.2

**Status:** ❌ Stub Implementation

**Current Behavior:**
```rust
pub fn diff(&self, _expected: &str, _actual: &str) -> Result<String> {
    Ok(String::new())  // Always returns empty string
}
```

**Required Implementation:**

```rust
use ratatui::buffer::Buffer;

pub struct BufferDiff {
    ignore_fg: bool,
    ignore_bg: bool,
    ignore_attributes: bool,
}

impl BufferDiff {
    pub fn new() -> Self;
    pub fn ignore_fg(mut self, ignore: bool) -> Self;
    pub fn ignore_bg(mut self, ignore: bool) -> Self;
    pub fn ignore_attributes(mut self, ignore: bool) -> Self;
    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult;
    pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult;
}

pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
}

pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}
```

**Acceptance Criteria (FR-023.2):**
- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y of differences
- [ ] Supports ignoring foreground color
- [ ] Supports ignoring background color
- [ ] Supports ignoring attributes (bold, italic, etc.)
- [ ] Provides human-readable diff output

**Dependencies:** `ratatui` for `Buffer` and `Cell` types

---

### 3.4 StateTester - FR-023.3

**Status:** ❌ Stub Implementation

**Current Behavior:**
```rust
pub fn assert_state<S>(&self, _state: &S) -> Result<()>
where
    S: serde::Serialize,
{
    Ok(())  // Always returns Ok
}
```

**Required Implementation:**

```rust
pub struct StateTester {
    snapshot: Option<serde_json::Value>,
}

impl StateTester {
    pub fn new() -> Self;
    pub fn capture<S>(&mut self, state: &S) -> Result<()>
    where
        S: serde::Serialize;
    pub fn assert_state<S>(&self, state: &S) -> Result<()>
    where
        S: serde::Serialize;
    pub fn assert_state_matches(&self, expected: &serde_json::Value) -> Result<()>;
}
```

**Acceptance Criteria (FR-023.3):**
- [ ] Captures serializable state to JSON
- [ ] Compares current state to captured snapshot
- [ ] Reports mismatches with JSON diff

**Dependencies:** `serde`, `serde_json`

---

### 3.5 TestDsl - FR-023.4

**Status:** ❌ Stub Implementation

**Current Behavior:**
```rust
pub fn render(&self, _widget: impl std::fmt::Debug) -> Result<()> {
    Ok(())  // Always returns Ok
}
```

**Required Implementation:**

```rust
use ratatui::{buffer::Buffer, widgets::Widget};

pub struct TestDsl {
    pty: Option<PtySimulator>,
    buffer_diff: BufferDiff,
    state_tester: StateTester,
}

impl TestDsl {
    pub fn new() -> Self;
    pub fn with_pty(mut self) -> Result<Self>;
    pub fn pty_mut(&mut self) -> Option<&mut PtySimulator>;
    pub fn buffer_diff(&self) -> &BufferDiff;
    pub fn state_tester(&mut self) -> &mut StateTester;
    pub fn render(&self, widget: &impl Widget) -> Result<Buffer>;
    pub fn assert_buffer_eq(&self, expected: &Buffer, actual: &Buffer) -> Result<()>;
    pub fn send_keys(&mut self, keys: &str) -> Result<&mut Self>;
    pub fn wait_for(&mut self, timeout: Duration, predicate: impl Fn(&str) -> bool) -> Result<&mut Self>;
    pub fn capture_state<S>(&mut self, state: &S) -> &mut Self;
    pub fn assert_state<S: serde::Serialize>(&self, state: &S) -> Result<()>;
}
```

**Acceptance Criteria (FR-023.4):**
- [ ] Renders widget to Buffer
- [ ] Composes PTY, BufferDiff, StateTester
- [ ] Fluent API chains correctly
- [ ] Wait-for predicate support

---

### 3.6 CliTester - FR-023.5

**Status:** ❌ Stub Implementation

**Current Behavior:**
```rust
pub fn run(&self, _args: &[&str]) -> Result<String> {
    Ok(String::new())  // Always returns empty string
}
```

**Required Implementation:**

```rust
pub struct CliTester {
    temp_dir: Option<TempDir>,
}

impl CliTester {
    pub fn new() -> Self;
    pub fn with_temp_dir(mut self) -> Result<Self>;
    pub fn run(&self, args: &[&str]) -> Result<CliOutput>;
    pub fn capture_stdout(&mut self) -> &mut Self;
    pub fn capture_stderr(&mut self) -> &mut Self;
}

pub struct CliOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
```

**Acceptance Criteria (FR-023.5):**
- [ ] Spawns process with args
- [ ] Captures stdout/stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories

**Dependencies:** `tempfile`

---

## 4. Remaining Gaps

### 4.1 P1 - High Priority Issues

| ID | Issue | Module | Status | Fix Required |
|----|-------|--------|--------|--------------|
| P1-NEW-3 | Two `ToolRegistry` implementations | core/tools | NOT FIXED | Audit `core::ToolRegistry` usage, remove if dead code |

**Gap Detail (P1-NEW-3):**

| Location | Lines | Purpose |
|----------|-------|---------|
| `crates/core/src/tool.rs` | ~1025 | Simple HashMap-based (still exported from core) |
| `crates/tools/src/registry.rs` | ~2288 | Full-featured with caching, async, source tracking |

**Risk:** `core::ToolRegistry` is re-exported but not actively used in runtime. Potential confusion and maintenance burden.

**Fix Required:**
1. Audit all usages of `opencode_core::ToolRegistry`
2. Remove `core::ToolRegistry` if truly dead code
3. Update `crates/core/src/lib.rs` exports accordingly

---

### 4.2 P2 - Medium Priority Issues

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

### 4.3 Route-Group Tests Gap (P2-1)

| Route Group | Test Coverage | Status |
|-------------|--------------|--------|
| Session routes | ✅ Done | `server_integration_tests.rs:840-1158` |
| Permission routes | ✅ Done | `server_integration_tests.rs:67-130` |
| Auth middleware | ✅ Done | `server_integration_tests.rs:123-183, 1186-1285` |
| MCP routes | ❌ Missing | No explicit MCP route group tests |
| Config routes | ❌ Missing | No explicit config route group tests |
| Provider routes | ❌ Missing | No explicit provider route group tests |

---

### 4.4 API Negative Tests Gap (P2-2)

| Test Type | Status | Evidence |
|-----------|--------|----------|
| Unauthorized access (missing token) | ✅ Done | `server_integration_tests.rs:123-130` |
| Invalid auth token | ✅ Done | `server_integration_tests.rs:138-164` |
| Empty auth token | ✅ Done | `server_integration_tests.rs:191-198` |
| Malformed request bodies | ❌ Missing | No tests for invalid JSON, missing required fields |
| Invalid session/message IDs | ❌ Missing | No tests for non-existent session operations |
| SQL injection / path traversal | ❌ Missing | No security-focused negative tests |

---

### 4.5 Hook Determinism Test Gap (P2-3)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Deterministic hook execution | ✅ Implemented | `sorted_plugin_names()` with priority sorting |
| Explicit 100-iteration test | ❌ Missing | No test verifying consistent ordering |

**Gap Detail:** While `sorted_plugin_names()` implements deterministic ordering via explicit priority sorting (`plugin/src/lib.rs:606`), there is no explicit test that verifies hook execution produces consistent results across multiple invocations.

**Fix Required:** Add test in `plugin/src/lib.rs` that registers multiple plugins with different priorities and verifies `sorted_plugin_names()` returns consistent ordering across 100 iterations.

---

### 4.6 Security Tests Gap (P2-4)

| Test Type | Status | Evidence |
|-----------|--------|----------|
| SQL injection | ❌ Missing | No tests |
| Path traversal | ❌ Missing | No tests |
| Request smuggling | ❌ Missing | No tests |

---

## 5. Acceptance Criteria

### 5.1 ratatui-testing Framework (FR-023)

#### PtySimulator (FR-023.1)
- [x] Creates PTY master/slave pair on Unix
- [x] Writes strings to PTY slave
- [x] Reads output from PTY master with timeout
- [x] Resizes PTY window (cols/rows)
- [ ] Injects KeyEvent via crossterm
- [ ] Injects MouseEvent via crossterm

#### BufferDiff (FR-023.2)
- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y of differences
- [ ] Supports ignoring foreground color
- [ ] Supports ignoring background color
- [ ] Supports ignoring attributes (bold, italic, etc.)
- [ ] Provides human-readable diff output

#### StateTester (FR-023.3)
- [ ] Captures serializable state to JSON
- [ ] Compares current state to captured snapshot
- [ ] Reports mismatches with JSON diff

#### TestDsl (FR-023.4)
- [ ] Renders widget to Buffer
- [ ] Composes PTY, BufferDiff, StateTester
- [ ] Fluent API chains correctly
- [ ] Wait-for predicate support

#### CliTester (FR-023.5)
- [ ] Spawns process with args
- [ ] Captures stdout/stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories

#### Integration
- [ ] All modules compile together
- [ ] Integration tests pass
- [ ] Works with `cargo test`
- [ ] Cross-platform (Unix primary, Windows best-effort)

---

## 6. Technical Debt

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

## 7. Recommendations

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

## Appendix A: File Structure

```
opencode-rust/
├── iterations/
│   └── iteration-18/
│       └── spec_v18.md          # This document
├── opencode-rust/
│   └── ratatui-testing/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs           # 19 lines - exports
│       │   ├── pty.rs           # 115 lines - partial impl
│       │   ├── diff.rs          # 19 lines - STUB
│       │   ├── state.rs         # 22 lines - STUB
│       │   ├── dsl.rs           # 19 lines - STUB
│       │   └── cli.rs           # 19 lines - STUB
│       └── tests/
│           └── pty_tests.rs
```

---

## Appendix B: Feature Requirements Index

| ID | Feature | Component | Status |
|----|---------|-----------|--------|
| FR-023.1 | PtySimulator | ratatui-testing | ⚠️ Partial |
| FR-023.2 | BufferDiff | ratatui-testing | ❌ Not Implemented |
| FR-023.3 | StateTester | ratatui-testing | ❌ Not Implemented |
| FR-023.4 | TestDsl | ratatui-testing | ❌ Not Implemented |
| FR-023.5 | CliTester | ratatui-testing | ❌ Not Implemented |

---

## Appendix C: Iteration History

| Iteration | Date | Key Changes |
|-----------|------|-------------|
| 15 | 2026-04-13 | Initial PRD analysis, 3 P0 issues identified |
| 16 | 2026-04-14 | ACP E2E tests (1083 lines), Phase 6 tests |
| 17 | 2026-04-14 | P1 items progress, comprehensive spec |
| 18 | 2026-04-14 | Duplicate directory_scanner removed, ratatui-testing spec updated |

---

*Document generated: 2026-04-14*  
*Iteration: 18*  
*Phase: Phase 5-6 of 6 (Hardening, Release Qualification)*
