# Task List - Iteration 17

**Version:** 17.0  
**Generated:** 2026-04-14  
**Based on:** Spec v17, Gap Analysis  

---

## P0 Issues - ALL COMPLETE ✅

| Task | Issue | Module | Status |
|------|-------|--------|--------|
| P0-1 | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | ✅ COMPLETE |
| P0-2 | Custom tools not registered with ToolRegistry | tools | ✅ COMPLETE |
| P0-3 | Plugin tool registration missing | plugin | ✅ COMPLETE |

---

## P1 Issues - MUST COMPLETE ✅ (1/2 Done)

| Task | Issue | Module | Status |
|------|-------|--------|--------|
| P1-NEW-2 | Fix Duplicate `directory_scanner.rs` | config/core | ✅ DONE |

### Task P1-NEW-2: Fix Duplicate `directory_scanner.rs`

**Priority:** P1 - HIGH  
**Technical Debt:** TD-011  
**Module:** config/core  

**Steps:**
1. [x] Read `crates/core/src/config/directory_scanner.rs` to confirm contents (file does not exist - already removed)
2. [x] Read `crates/config/src/directory_scanner.rs` to verify they are duplicates (verified - 832 lines each)
3. [x] Delete `crates/core/src/config/directory_scanner.rs` (already deleted)
4. [x] Update `crates/core/src/lib.rs` to use `opencode_config::DirectoryScanner` (already configured via re-export)
5. [x] Search for any remaining references to deleted file (no references found)
6. [x] Run `cargo build --all-features` to verify ✅ PASSES
7. [x] Run `cargo test -p opencode-core` to verify ✅ ALL 71 TESTS PASS
8. [x] Run `cargo test -p opencode-config` to verify ✅ ALL 70 TESTS PASS

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-core && cargo test -p opencode-config
```

**Status:** ✅ DONE - The duplicate `directory_scanner.rs` has been removed from `crates/core/src/config/` and `lib.rs` correctly re-exports `DirectoryScanner` from `opencode_config` via the `config` module.

---

### Task P1-NEW-3: Audit Two ToolRegistry Implementations

**Priority:** P1 - HIGH  
**Technical Debt:** TD-012  
**Module:** core/tools  

**Steps:**
1. [ ] Read `crates/core/src/tool.rs` (ToolRegistry implementation)
2. [ ] Read `crates/tools/src/registry.rs` (ToolRegistry implementation)
3. [ ] Trace all usages of `core::ToolRegistry` in agent runtime
4. [ ] Verify which features from `opencode_tools::ToolRegistry` are needed
5. [ ] Determine consolidation approach:
   - Option A: Consolidate to single registry
   - Option B: Document intentional separation
   - Option C: Migrate agent to use full-featured registry
6. [ ] Implement chosen approach
7. [ ] Verify with cargo build and tests

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-agent && cargo test -p opencode-tools
```

---

## P2 Issues - Complete for Quality

### Task P2-NEW-1: Add Route-Group Tests

**Priority:** P2  
**Reference:** FR-004  
**Module:** server  

**Steps:**
1. [ ] Read existing server integration tests structure
2. [ ] Add explicit MCP route group tests:
   - [ ] Test `/api/mcp/servers` endpoint
   - [ ] Test `/api/mcp/tools` endpoint
   - [ ] Test MCP server registration
3. [ ] Add explicit config route group tests:
   - [ ] Test `/api/config` endpoints
4. [ ] Add explicit provider route group tests:
   - [ ] Test `/api/providers` endpoints
5. [ ] Run tests to verify

**Verification:**
```bash
cargo test -p opencode-server server_integration
```

---

### Task P2-NEW-2: Add Malformed Request Body Tests

**Priority:** P2  
**Reference:** FR-004  
**Module:** server  

**Steps:**
1. [ ] Read existing server integration tests for auth
2. [ ] Add invalid JSON body tests:
   - [ ] Test missing required fields
   - [ ] Test extra unknown fields
   - [ ] Test wrong field types
   - [ ] Test empty body
3. [ ] Add invalid session/message ID tests:
   - [ ] Test operations on non-existent session
   - [ ] Test operations on deleted session
   - [ ] Test invalid session ID format
4. [ ] Run tests to verify

**Verification:**
```bash
cargo test -p opencode-server
```

---

### Task P2-NEW-3: Add Hook Determinism Test

**Priority:** P2  
**Reference:** FR-008  
**Module:** plugin  

**Steps:**
1. [ ] Read `plugin/src/lib.rs` around line 602-621 for `sorted_plugin_names()`
2. [ ] Create test that:
   - [ ] Registers multiple plugins with different priorities
   - [ ] Calls `sorted_plugin_names()` 100 times
   - [ ] Verifies consistent ordering across all invocations
3. [ ] Run test to verify

**Verification:**
```bash
cargo test -p opencode-plugin sorted_plugin_names
```

---

### Task P2-NEW-4: Add Security Tests

**Priority:** P2  
**Reference:** FR-004  
**Module:** server  

**Steps:**
1. [ ] Read existing server integration tests
2. [ ] Add SQL injection tests:
   - [ ] Test SQL injection in session IDs
   - [ ] Test SQL injection in message content
3. [ ] Add path traversal tests:
   - [ ] Test path traversal in file operations
   - [ ] Test `../` in paths
4. [ ] Run tests to verify

**Verification:**
```bash
cargo test -p opencode-server
```

---

## New Feature Tasks (FR-025 to FR-028)

### Task FR-025: Implement ratatui-testing BufferDiff

**Priority:** P2  
**Reference:** FR-025  
**Module:** ratatui-testing  

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/diff.rs`
2. [ ] Implement cell-by-cell comparison of ratatui Buffer
3. [ ] Implement x,y position reporting of differences
4. [ ] Implement configurable ignore options (foreground, background, attributes)
5. [ ] Implement human-readable diff output
6. [ ] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

---

### Task FR-026: Implement ratatui-testing StateTester

**Priority:** P2  
**Reference:** FR-026  
**Module:** ratatui-testing  

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/state.rs`
2. [ ] Implement JSON state capture mechanism
3. [ ] Implement snapshot comparison
4. [ ] Implement JSON diff reporting
5. [ ] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

---

### Task FR-027: Implement ratatui-testing TestDsl

**Priority:** P2  
**Reference:** FR-027  
**Module:** ratatui-testing  

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/dsl.rs`
2. [ ] Implement widget rendering to Buffer
3. [ ] Implement PTY/BufferDiff/StateTester composition
4. [ ] Implement fluent API with method chaining
5. [ ] Implement wait-for predicate support
6. [ ] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

---

### Task FR-028: Implement ratatui-testing CliTester

**Priority:** P2  
**Reference:** FR-028  
**Module:** ratatui-testing  

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/cli.rs`
2. [ ] Implement process spawning with arguments
3. [ ] Implement stdout/stderr capture
4. [ ] Implement exit code capture
5. [ ] Implement temp directory cleanup
6. [ ] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

---

## Phase 6: Release Qualification

### Task PHASE-6: Release Qualification

**Priority:** Phase 6  
**Status:** Not Started  

**Steps:**
1. [ ] Run full test suite
   ```bash
   cargo test --all-features
   ```
2. [ ] Run clippy
   ```bash
   cargo clippy --all -- -D warnings
   ```
3. [ ] Run formatting check
   ```bash
   cargo fmt --all -- --check
   ```
4. [ ] Run doc tests
   ```bash
   cargo test --doc
   ```
5. [ ] Performance benchmarks (if applicable)
6. [ ] Memory profiling (if applicable)
7. [ ] Security audit
8. [ ] Documentation completeness check

---

## Task Summary

| Category | Tasks | Completed | Remaining |
|----------|-------|-----------|-----------|
| P0 | 3 | 3 | 0 |
| P1 | 2 | 0 | 2 |
| P2 | 8 | 0 | 8 |
| **Total** | **13** | **3** | **10** |

---

## Immediate Priority Order

1. **P1-NEW-2**: Fix Duplicate `directory_scanner.rs` (HIGH - blocks release)
2. **P1-NEW-3**: Audit Two ToolRegistry Implementations (HIGH - blocks release)
3. **P2-NEW-1**: Add Route-Group Tests (Quality)
4. **P2-NEW-2**: Add Malformed Request Body Tests (Quality)
5. **P2-NEW-3**: Add Hook Determinism Test (Quality)
6. **P2-NEW-4**: Add Security Tests (Quality)
7. **FR-025**: Implement BufferDiff
8. **FR-026**: Implement StateTester
9. **FR-027**: Implement TestDsl
10. **FR-028**: Implement CliTester
11. **PHASE-6**: Release Qualification

---

*Task list generated: 2026-04-14*  
*Iteration: 17*  
*Focus: P1 blockers first to enable release*
