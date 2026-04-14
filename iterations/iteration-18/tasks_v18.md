# Task List - Iteration 18

**Version:** 18.0  
**Generated:** 2026-04-14  
**Based on:** Spec v18, Gap Analysis  

---

## P0 Issues - ALL COMPLETE ✅

| Task | Issue | Module | Status |
|------|-------|--------|--------|
| P0-1 | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | ✅ COMPLETE |
| P0-2 | Custom tools not registered with ToolRegistry | tools | ✅ COMPLETE |
| P0-3 | Plugin tool registration missing | plugin | ✅ COMPLETE |

---

## P1 Issues - MUST COMPLETE (1/1 Remaining)

| Task | Issue | Module | Status |
|------|-------|--------|--------|
| P1-NEW-2 | Fix Duplicate `directory_scanner.rs` | config/core | ✅ DONE |
| P1-NEW-3 | Audit Two ToolRegistry Implementations | core/tools | ✅ DONE |

### Task P1-NEW-3: Audit Two ToolRegistry Implementations

**Priority:** P1 - HIGH  
**Technical Debt:** TD-012  
**Module:** core/tools  

**Steps:**
1. [x] Read `crates/core/src/tool.rs` (ToolRegistry implementation - ~1025 lines)
2. [x] Read `crates/tools/src/registry.rs` (ToolRegistry implementation - ~2288 lines)
3. [x] Search for all usages of `opencode_core::ToolRegistry`:
   ```bash
   grep -r "opencode_core::ToolRegistry\|core::ToolRegistry\|crate::ToolRegistry" --include="*.rs"
   ```
4. [x] Verify agent runtime uses `opencode_tools::ToolRegistry`
5. [x] Determine if `core::ToolRegistry` is dead code
6. [x] Decision: `core::ToolRegistry` is NOT dead code - used by MCP crate for tool bridging
7. [x] Run `cargo build --all-features` to verify
8. [x] Run `cargo test -p opencode-core` to verify

**Verification:**
```bash
cargo build --all-features && cargo test -p opencode-core
```

---

## P2 Issues - Complete for Quality

### Task P2-1: Add Route-Group Tests

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

### Task P2-2: Add Malformed Request Body Tests

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

### Task P2-3: Add Hook Determinism Test

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

### Task P2-4: Add Security Tests

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

## ratatui-testing Framework Tasks (P2-5 to P2-8)

### Task P2-5: Implement ratatui-testing BufferDiff (FR-023.2)

**Priority:** P2  
**Reference:** FR-023.2  
**Module:** ratatui-testing  

**Acceptance Criteria:**
- [ ] Compares two Buffers cell-by-cell
- [ ] Reports exact x,y of differences
- [ ] Supports ignoring foreground color
- [ ] Supports ignoring background color
- [ ] Supports ignoring attributes (bold, italic, etc.)
- [ ] Provides human-readable diff output

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/diff.rs`
2. [ ] Implement `BufferDiff` struct with ignore options
3. [ ] Implement `diff()` method for Buffer comparison
4. [ ] Implement `diff_str()` method for string comparison
5. [ ] Implement `DiffResult` and `CellDiff` types
6. [ ] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

---

### Task P2-6: Implement ratatui-testing StateTester (FR-023.3)

**Priority:** P2  
**Reference:** FR-023.3  
**Module:** ratatui-testing  
**Status:** ✅ Done

**Acceptance Criteria:**
- [x] Captures serializable state to JSON
- [x] Compares current state to captured snapshot
- [x] Reports mismatches with JSON diff

**Steps:**
1. [x] Read current stub in `ratatui-testing/src/state.rs`
2. [x] Implement `StateTester` struct
3. [x] Implement `capture()` method for state serialization
4. [x] Implement `assert_state()` method for comparison
5. [x] Implement `assert_state_matches()` method
6. [x] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

**Implementation Details:**
- Added `serde_json` dependency to Cargo.toml
- Implemented `StateSnapshot` struct for storing captured state
- Implemented `StateDiff` and `StateDiffEntry` for diff reporting
- Implemented `StateTester` with:
  - `capture_state()` - capture state to snapshot
  - `compare()` - compare current vs snapshot
  - `assert_state()` - assert current matches named snapshot
  - `assert_state_matches()` - compare two arbitrary values
  - `diff_to_string()` - human-readable diff output
- Added 18 comprehensive unit tests

---

### Task P2-7: Implement ratatui-testing TestDsl (FR-023.4)

**Priority:** P2  
**Reference:** FR-023.4  
**Module:** ratatui-testing  

**Acceptance Criteria:**
- [ ] Renders widget to Buffer
- [ ] Composes PTY, BufferDiff, StateTester
- [ ] Fluent API chains correctly
- [ ] Wait-for predicate support

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/dsl.rs`
2. [ ] Implement `TestDsl` struct composing PtySimulator, BufferDiff, StateTester
3. [ ] Implement `new()`, `with_pty()`, `pty_mut()`, `buffer_diff()`, `state_tester()`
4. [ ] Implement `render()` method for widget rendering
5. [ ] Implement `assert_buffer_eq()` method
6. [ ] Implement `send_keys()` and `wait_for()` methods
7. [ ] Implement `capture_state()` and `assert_state()` methods
8. [ ] Add tests

**Verification:**
```bash
cargo test -p ratatui-testing
```

---

### Task P2-8: Implement ratatui-testing CliTester (FR-023.5)

**Priority:** P2  
**Reference:** FR-023.5  
**Module:** ratatui-testing  

**Acceptance Criteria:**
- [ ] Spawns process with args
- [ ] Captures stdout/stderr
- [ ] Returns exit code
- [ ] Cleans up temp directories

**Steps:**
1. [ ] Read current stub in `ratatui-testing/src/cli.rs`
2. [ ] Implement `CliTester` struct with temp_dir
3. [ ] Implement `new()`, `with_temp_dir()` methods
4. [ ] Implement `run()` method for process spawning
5. [ ] Implement `capture_stdout()`, `capture_stderr()` methods
6. [ ] Implement `CliOutput` struct
7. [ ] Add tests

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
| P1 | 2 | 1 | 1 |
| P2 | 10 | 1 | 9 |
| **Total** | **15** | **5** | **10** |

---

## Immediate Priority Order

1. **P1-NEW-3**: Audit Two ToolRegistry Implementations (HIGH - blocks release)
2. **P2-1**: Add Route-Group Tests (Quality)
3. **P2-2**: Add Malformed Request Body Tests (Quality)
4. **P2-3**: Add Hook Determinism Test (Quality)
5. **P2-4**: Add Security Tests (Quality)
6. **P2-5**: Implement BufferDiff
7. **P2-6**: Implement StateTester
8. **P2-7**: Implement TestDsl
9. **P2-8**: Implement CliTester
10. **PHASE-6**: Release Qualification

---

## Key Changes from Iteration-17

| Item | Iteration-17 | Iteration-18 |
|------|--------------|--------------|
| P1-NEW-2 (directory_scanner) | NOT DONE | ✅ DONE |
| P1-NEW-3 (ToolRegistry) | NOT DONE | ✅ DONE |
| P2 items remaining | 4 | 6 (added 4 ratatui-testing) |
| ratatui-testing | 4 stubs | 4 stubs (unchanged) |

---

*Task list generated: 2026-04-14*  
*Iteration: 18*  
*Focus: P1 blockers first to enable release*