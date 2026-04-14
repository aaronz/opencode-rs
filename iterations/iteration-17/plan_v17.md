# Implementation Plan - Iteration 17

**Version:** 17.0  
**Generated:** 2026-04-14  
**Based on:** Spec v17, Gap Analysis  
**Status:** Draft  

---

## 1. Overview

**Overall Completion:** ~85-90%  
**Phase Status:** Phase 5-6 of 6 (Hardening, Release Qualification)  
**P0 Status:** ALL FIXED ✅  
**P1 Remaining:** 2 items (duplicate directory_scanner, ToolRegistry divergence)  
**P2 Remaining:** 4 items (route-group tests, malformed requests, hook determinism, security tests)  

---

## 2. Priority Matrix

| Priority | Items | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| **P0** | 3 | 3 | 0 | **100%** ✅ |
| **P1** | 12 | 9 | 3 | 75% |
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

### 4.1 P1-NEW-2: Duplicate `directory_scanner.rs` (TD-011)

**Severity:** HIGH  
**Module:** config/core  
**Files Affected:**
- `crates/config/src/directory_scanner.rs` (832 lines - ACTIVE)
- `crates/core/src/config/directory_scanner.rs` (832 lines - DUPLICATE)

**Root Cause:** The `crates/core/src/config.rs` re-exports from `opencode_config`, so the version in `crates/core/src/config/` is dead code.

**Fix Required:**
1. Delete `crates/core/src/config/directory_scanner.rs`
2. Update `crates/core/src/lib.rs` exports to use `opencode_config::DirectoryScanner`
3. Verify no remaining references to deleted file
4. Run `cargo build --all-features && cargo test -p opencode-core`

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-core
```

---

### 4.2 P1-NEW-3: Two ToolRegistry Implementations (TD-012)

**Severity:** HIGH  
**Module:** core/tools  
**Files:**
- `crates/core/src/tool.rs` (1025 lines) - Simple HashMap-based
- `crates/tools/src/registry.rs` (2288 lines) - Full-featured with caching, async

**Risk:** `core::ToolRegistry` is used in `crates/agent/src/runtime.rs`. If the two registries diverge, tool execution may not use the full-featured registry features.

**Fix Required:**
1. Trace all usages of `core::ToolRegistry` in agent runtime
2. Verify `opencode_tools::ToolRegistry` features are accessible
3. Either consolidate to single registry or document intentional separation
4. Ensure agent runtime uses the full-featured registry

---

## 5. P2 Issues - Short-term

### 5.1 P2-NEW-1: Route-group MCP/config/provider Tests Missing

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

### 5.2 P2-NEW-2: Malformed Request Body Tests Missing

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

### 5.3 P2-NEW-3: Hook Determinism Explicit Test Missing

**Module:** plugin  
**Reference:** FR-008  
**Current Status:** `sorted_plugin_names()` with priority sorting implemented at `plugin/src/lib.rs:602-621`  
**Missing:** 100-iteration test verifying consistent ordering

**Fix Required:**
Add test in `plugin/src/lib.rs` that registers multiple plugins with different priorities and verifies `sorted_plugin_names()` returns consistent ordering across 100 iterations.

---

### 5.4 P2-NEW-4: Security Tests Missing

**Module:** server  
**Reference:** FR-004  
**Missing:**
- SQL injection tests
- Path traversal tests
- Request smuggling tests

**Fix Required:**
Add security-focused negative tests for injection attacks and path traversal.

---

## 6. New Feature Requirements (FR-025 to FR-028)

### FR-025: ratatui-testing BufferDiff Full Implementation

**Status:** ❌ Not Implemented (stub)  
**Components Required:**
- Cell-by-cell comparison of ratatui Buffer
- x,y position reporting of differences
- Configurable ignore options (foreground, background, attributes)
- Human-readable diff output

---

### FR-026: ratatui-testing StateTester Full Implementation

**Status:** ❌ Not Implemented (stub)  
**Components Required:**
- JSON state capture mechanism
- Snapshot comparison
- JSON diff reporting

---

### FR-027: ratatui-testing TestDsl Full Implementation

**Status:** ❌ Not Implemented (stub)  
**Components Required:**
- Widget rendering to Buffer
- PTY/BufferDiff/StateTester composition
- Fluent API with method chaining
- Wait-for predicate support

---

### FR-028: ratatui-testing CliTester Full Implementation

**Status:** ❌ Not Implemented (stub)  
**Components Required:**
- Process spawning with arguments
- stdout/stderr capture
- Exit code capture
- Temp directory cleanup

---

## 7. Phase 6: Release Qualification

**Status:** ❌ Not Started  

### Required Actions:
1. Run full test suite
2. Run clippy (`cargo clippy --all -- -D warnings`)
3. Run formatting check (`cargo fmt --all -- --check`)
4. Performance benchmarks
5. Memory profiling
6. Security audit
7. Documentation completeness check

---

## 8. Implementation Order

### Immediate (P1 - Blockers)

1. **Fix Duplicate `directory_scanner.rs`** (P1-NEW-2)
   - Delete `crates/core/src/config/directory_scanner.rs`
   - Update exports
   - Verify with cargo build/test

2. **Audit Two ToolRegistry Implementations** (P1-NEW-3)
   - Trace `core::ToolRegistry` usage
   - Verify feature parity
   - Consolidate or document

### Short-term (P2 - Quality)

3. **Add Route-Group Tests** (P2-NEW-1)
4. **Add Malformed Request Tests** (P2-NEW-2)
5. **Add Hook Determinism Test** (P2-NEW-3)
6. **Add Security Tests** (P2-NEW-4)

### Medium-term (New Features)

7. **Implement BufferDiff** (FR-025)
8. **Implement StateTester** (FR-026)
9. **Implement TestDsl** (FR-027)
10. **Implement CliTester** (FR-028)

### Final (Phase 6)

11. **Release Qualification**
    - Full test suite
    - Clippy
    - Benchmarks
    - Security audit

---

## 9. Technical Debt Status

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
| **TD-011** | Duplicate `directory_scanner.rs` | HIGH | ❌ |
| **TD-012** | Two ToolRegistry impls | HIGH | ❌ |

---

## 10. Crate-Level Status

| Crate | Phase | Status | Notes |
|-------|-------|--------|-------|
| `crates/core/` | 1 | ⚠️ Partial | Has duplicate directory_scanner; two ToolRegistry issue |
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
| `ratatui-testing/` | 2, 3 | ⚠️ Partial | PtySimulator implemented; others stubs |

---

*Plan generated: 2026-04-14*  
*Iteration: 17*  
*Priority: P1 blockers first, then P2 quality, then new features, then release qualification*
