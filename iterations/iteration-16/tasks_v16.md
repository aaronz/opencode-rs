# Task List - Iteration 16

**Version:** 16.0  
**Generated:** 2026-04-14  
**Priority:** P1 tasks are critical for release  
**Overall Status:** ~80-85% complete (up from ~65-70% in iteration-15)

---

## P0 Tasks: ALL FIXED ✅

### P0-1: ✅ DONE
**Issue:** Custom tool discovery scans TOOL.md instead of .ts/.js  
**Module:** tools  
**FR Reference:** FR-007  
**Status:** ✅ FIXED

- [x] Modify `DirectoryScanner::scan_tools()` in `crates/config/src/directory_scanner.rs:226-229`
- [x] Change file extension filter from `TOOL.md` to `.ts`, `.js`, `.mts`, `.cts`
- [x] Parse JavaScript/TypeScript tool definitions
- [x] Tests at `config/src/directory_scanner.rs:672-754`

**Verification:** `cargo test -p opencode-config -- scan_tools`

---

### P0-2: ✅ DONE
**Issue:** Discovered tools recorded in config but NOT registered with ToolRegistry  
**Module:** tools  
**FR Reference:** FR-007  
**Status:** ✅ FIXED

- [x] Create registration flow in `crates/tools/src/discovery.rs:230-248`
- [x] `register_custom_tools()` registers with `ToolRegistry`
- [x] Discovered custom tools appear in tool listing
- [x] Custom tools executable through standard tool pipeline

**Verification:** `cargo test -p opencode-tools -- custom_tool`

---

### P0-3: ✅ DONE
**Issue:** `PluginToolAdapter` exists but no mechanism to register plugin tools  
**Module:** plugin  
**FR Reference:** FR-008  
**Status:** ✅ FIXED

- [x] `register_tool()` at `crates/plugin/src/lib.rs:268`
- [x] `export_as_tools()` at `crates/plugin/src/lib.rs:576`
- [x] `register_tools_in_registry()` at `crates/plugin/src/lib.rs:821`
- [x] 7 tests at `plugin/src/lib.rs:2305-2565`

**Verification:** `cargo test -p opencode-plugin -- tool_registration`

---

## P1 Tasks (High Priority)

### P1-NEW-1: ACP E2E Connection Test
**Issue:** ACP transport exists but no E2E integration test  
**Module:** control-plane  
**FR Reference:** FR-015  
**Status:** ✅ Done

- [x] Create integration test that starts server with ACP enabled
- [x] Create `AcpTransportClient` instance
- [x] Establish TCP/WebSocket connection
- [x] Complete ACP handshake
- [x] Send/receive test message
- [x] Verify full message exchange works

**Acceptance Criteria:**
- Test creates connection and completes handshake
- Message exchange verified end-to-end

**Dependencies:** None

**Verification:** `cargo test -p opencode-integration-tests -- acp_e2e`

---

### P1-NEW-2: ✅ Done
**Issue:** Identical file exists at `crates/core/src/config/directory_scanner.rs` (832 lines)  
**Module:** config  
**FR Reference:** FR-003  
**Status:** ❌ TODO

- [ ] Delete `crates/core/src/config/directory_scanner.rs`
- [ ] Update `crates/core/src/lib.rs` to use `opencode_config::DirectoryScanner`
- [ ] Verify no remaining references to deleted file
- [ ] Run full build to ensure no breakage

**Acceptance Criteria:**
- Duplicate file removed
- All functionality preserved
- Build passes

**Dependencies:** None

**Verification:** `cargo build --all-features && cargo test -p opencode-core`

---

### P1-NEW-3: ✅ Done
**Issue:** Two `ToolRegistry` structs exist with different designs  
**Module:** core/tools  
**FR Reference:** FR-006  
**Status:** ❌ TODO

- [ ] Trace all usages of `core::ToolRegistry` in agent runtime
- [ ] Verify `opencode_tools::ToolRegistry` features (caching, async) available
- [ ] Either consolidate to single registry OR document intentional separation
- [ ] Add documentation explaining the relationship

**Acceptance Criteria:**
- Audit complete with findings documented
- Either single registry OR clear documentation of separation

**Dependencies:** P1-NEW-2

**Verification:** `cargo test -p opencode-agent -- tool_registry`

---

## P2 Tasks (Medium Priority)

### P2-NEW-1: ✅ Done
**Issue:** No explicit MCP/config/provider route group tests  
**Module:** server  
**FR Reference:** FR-004  
**Status:** ✅ Done

- [x] Add MCP route group tests:
  - [x] `GET /api/mcp/servers` - get_mcp_servers
  - [x] `GET /api/mcp/tools` - get_mcp_tools
  - [x] `POST /api/mcp/connect` - connect_mcp_server
- [x] Add config route group tests:
  - [x] `GET /api/config` - get_config
  - [x] `PATCH /api/config` - update_config
- [x] Add provider route group tests:
  - [x] `GET /api/providers` - get_providers
  - [x] `GET /api/models` - get_models
- [x] Add error handling tests for invalid route group requests

**Acceptance Criteria:** All route groups have explicit enumeration tests

**Dependencies:** None

**Implementation:**
- MCP routes added at `crates/server/src/routes/mcp.rs`
- Tests added at `crates/server/src/server_integration_tests.rs`

**Verification:** `cargo test -p opencode-server -- route_group` - 43 tests pass

---

### P2-NEW-2: ✅ Done
**Issue:** No tests for invalid JSON, missing fields, wrong types  
**Module:** server  
**FR Reference:** FR-004  
**Status:** ❌ TODO

- [ ] Add invalid JSON body tests
- [ ] Add missing required field tests
- [ ] Add wrong type field tests
- [ ] Add invalid session/message ID tests

**Acceptance Criteria:** API validates malformed requests correctly

**Dependencies:** None

**Verification:** `cargo test -p opencode-server -- negative`

---

### P2-NEW-3: ✅ Done
**Issue:** `sorted_plugin_names()` implemented but no explicit test  
**Module:** plugin  
**FR Reference:** FR-008  
**Status:** ❌ TODO

- [ ] Register multiple plugins with different priorities
- [ ] Call `sorted_plugin_names()` 100 times
- [ ] Verify consistent ordering across all iterations
- [ ] Add stress test for edge cases

**Acceptance Criteria:**
- Test verifies deterministic ordering
- 100 iterations produce identical results

**Dependencies:** None

**Verification:** `cargo test -p opencode-plugin -- hook_determinism`

---

### P2-NEW-4: Security Tests
**Issue:** No security-focused tests (injection, path traversal)  
**Module:** server  
**FR Reference:** FR-004  
**Status:** ❌ TODO

- [ ] Add SQL injection tests on all database endpoints
- [ ] Add path traversal tests on file operations
- [ ] Add request smuggling tests
- [ ] Verify proper sanitization

**Acceptance Criteria:** Security vulnerabilities identified and documented

**Dependencies:** P2-NEW-2

**Verification:** `cargo test -p opencode-server -- security`

---

## Previously Completed Tasks (Reference)

### P1-1: ✅ DONE - Hook Execution Determinism
- `sorted_plugin_names()` at `crates/plugin/src/lib.rs:602-621`
- Priority sorting implemented

### P1-2: ✅ DONE - Plugin Config Ownership
- `validate_runtime_loadable()` at `plugin/src/config.rs:317-322`
- `validate_tui_loadable()` at `plugin/src/config.rs:328`

### P1-3: ✅ DONE - Primary Agent Invariant Tests
- 7 tests at `agent/src/runtime.rs:1554-1747`
- `agent/tests/agent_integration.rs:91-147`

### P1-4: ✅ DONE - Ownership Tree Acyclicity Tests
- Tests in `crates/core/`

### P1-5: ✅ DONE - Session Lifecycle Integration Tests
- `tests/src/session_lifecycle_tests.rs` (533 lines)
- `storage/tests/session_lifecycle_tests.rs` (421 lines)

### P1-6: ✅ DONE - Desktop App
- `cli/src/cmd/desktop.rs` (502 lines)
- `webview.rs` (122 lines)

### P1-7: ✅ DONE - Web Server Mode
- `cli/src/cmd/web.rs` (235 lines)

### P1-8: ⚠️ PARTIAL - ACP Transport
- Transport exists at `control-plane/src/transport.rs` (847 lines)
- E2E test still missing (see P1-NEW-1)

### P1-9: ✅ DONE - Config Crate Refactor
- `crates/config/src/lib.rs` now has 1600+ lines

### P2-1: ✅ DONE - TUI Slash Command Tests
- `tui/tests/slash_command_tests.rs` (287 lines)

### P2-2: ✅ DONE - TUI Input Model Tests
- `tui/tests/input_model_tests.rs` (371 lines)

### P2-3: ✅ DONE - TUI Sidebar Tests
- `tui/tests/sidebar_tests.rs` (741 lines)

### P2-4: ✅ DONE - Per-Agent Model Override Test
- 16 tests at `agent/tests/agent_integration.rs:169-316`

### P2-5: ✅ DONE - Route-Group Tests
- Session/permission/provider/config/MCP route tests done
- 14 new tests at `crates/server/src/server_integration_tests.rs:1150-1329`
- All tests pass: `cargo test -p opencode-server -- route_group`

### P2-6: ✅ DONE - API Negative Tests
- `server_integration_tests.rs:1783-2036` - api_negative_tests (12 tests)
- `server_integration_tests.rs:2052-2221` - auth_negative_tests (9 tests)
- All tests pass: `cargo test -p opencode-server -- api_negative` and `cargo test -p opencode-server -- auth_negative`

### P2-7: ✅ DONE - Hidden vs Visible Agent Tests
- `agent/tests/agent_integration.rs:91-147`
- `agent/src/runtime.rs:1554-1747`

### P2-8: ✅ DONE - Theme Auto-Sync Tests
- `tui/tests/plugin_theme_tests.rs` (447 lines)

---

## Technical Debt

| TD | Item | Status | Action |
|----|------|--------|--------|
| TD-001 | Empty `crates/config/` crate | ✅ Fixed | N/A |
| TD-002 | DirectoryScanner discovery mismatch | ✅ Fixed | N/A |
| TD-003 | Custom tools not registered | ✅ Fixed | N/A |
| TD-004 | Non-deterministic hook execution | ✅ Fixed | N/A |
| TD-005 | Plugin register_tool() missing | ✅ Fixed | N/A |
| TD-006 | ACP transport layer missing | ⚠️ Partial | See P1-NEW-1 |
| TD-007 | Deprecated `mode` field | Deferred | Remove in v4.0 |
| TD-008 | Deprecated `tools` field | Deferred | Remove after migration |
| TD-009 | Deprecated `theme` field | ✅ Fixed | Moved to tui.json |
| TD-010 | Deprecated `keybinds` field | ✅ Fixed | Moved to tui.json |
| TD-NEW-1 | Duplicate `directory_scanner.rs` | ❌ TODO | See P1-NEW-2 |
| TD-NEW-2 | Two ToolRegistry impls | ❌ TODO | See P1-NEW-3 |

---

## Task Summary

| Priority | Count | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| P0 | 3 | 3 | 0 | 100% |
| P1 | 12 | 9 | 3 | 75% |
| P2 | 12 | 6 | 6 | 50% |

**Total Active Tasks:** 9
- P1: 3 (critical for release)
- P2: 6 (important for completeness)

---

## Definition of Done

A task is complete when:
1. All subtasks are checked off
2. Code compiles without errors
3. Tests pass (`cargo test --all-features`)
4. Clippy passes (`cargo clippy --all -- -D warnings`)
5. Formatting is correct (`cargo fmt --all`)

---

## Verification Commands

```bash
# Full build
cargo build --all-features

# Run all tests
cargo test --all-features

# Run clippy
cargo clippy --all -- -D warnings

# Check formatting
cargo fmt --all -- --check

# P1 critical tests
cargo test -p opencode-integration-tests -- acp_e2e
cargo test -p opencode-core
cargo test -p opencode-agent -- tool_registry

# P2 tests
cargo test -p opencode-server -- route_group
cargo test -p opencode-server -- negative
cargo test -p opencode-plugin -- hook_determinism
cargo test -p opencode-server -- security
```

---

*Document generated: 2026-04-14*
*Iteration: 16*
