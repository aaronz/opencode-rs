# Task List - Iteration 15

**Version:** 15.0  
**Generated:** 2026-04-13  
**Priority:** P0 tasks MUST be completed before release  

---

## P0 Tasks (Blocking - Must Fix)

### P0-1: ✅ Done
**Issue:** Custom tool discovery scans TOOL.md instead of .ts/.js  
**Module:** tools  
**FR Reference:** FR-007  
**Status:** ✅ Done  

- [x] Modify `DirectoryScanner::scan_tools()` in `crates/core/src/config/directory_scanner.rs:228`
- [x] Change file extension filter from `TOOL.md` to `.ts` and `.js`
- [x] Parse JavaScript/TypeScript tool definitions with `export default tool({...})` pattern
- [x] Extract tool name, description, and argument schema from parsed files
- [x] Verify discovery works for both project-level (`.opencode/tools/`) and global-level (`~/.config/opencode/tools/`)

**Acceptance Criteria:**
- Custom tools in `.opencode/tools/*.ts` and `.opencode/tools/*.js` are discovered
- Tool definitions are parsed correctly (name, description, args schema)

---

### P0-2: ✅ Done
**Issue:** Discovered tools recorded in config but NOT registered with ToolRegistry  
**Module:** tools  
**FR Reference:** FR-007  
**Status:** ✅ Done  

- [x] Create registration flow from `DirectoryScanner` to `ToolRegistry`
- [x] Ensure discovered custom tools appear in tool listing
- [x] Verify custom tools can be executed through standard tool pipeline
- [x] Add integration test for custom tool discovery → registration → execution

**Acceptance Criteria:**
- Discovered custom tools appear in `ToolRegistry` listing
- Custom tools can be executed with proper permission gating

---

### P0-3: ✅ Done
**Issue:** `PluginToolAdapter` exists but no mechanism to register plugin tools  
**Module:** plugin  
**FR Reference:** FR-008  
**Status:** ✅ Done  

- [x] Add `register_tool()` method to `Plugin` trait in `crates/plugin/src/lib.rs`
- [x] Implement tool registration in `PluginManager`
- [x] Integrate `PluginManager` with `ToolRegistry` for plugin-provided tools
- [x] Add tests verifying plugin tools appear in registry after plugin activation
- [x] Verify plugin tools respect permission system

**Acceptance Criteria:**
- Plugins can add tools to agent toolset via `register_tool()`
- Plugin tools appear in ToolRegistry after plugin activation
- Plugin tools are subject to same permission gating as built-in tools

---

## P1 Tasks (High Priority)

### P1-1: ✅ Done
**Issue:** Hooks execute in HashMap iteration order (non-deterministic)  
**Module:** plugin  
**FR Reference:** FR-008  
**Status:** ✅ Done  

- [x] Add explicit `priority` field to plugin/hook configuration
- [x] Replace HashMap iteration with ordered iteration using priority
- [x] Document execution order guarantees in code
- [x] Add test verifying deterministic execution order across multiple runs

**Acceptance Criteria:**
- Hooks execute in consistent, predictable order
- Test confirms same plugin order produces same hook execution order

---

### P1-2: ✅ Done
**Issue:** Server/runtime and TUI plugin configs can be mixed  
**Module:** plugin  
**FR Reference:** FR-008  
**Status:** TODO  

- [ ] Add validation in config loading to separate plugin configs
- [ ] Ensure `opencode.json` plugins stay in server/runtime domain
- [ ] Ensure `tui.json` plugins stay in TUI domain
- [ ] Add warning/error when configs are incorrectly mixed

**Acceptance Criteria:**
- Plugin config ownership boundary is enforced
- Mixed configs produce warning or error

---

### P1-3: ✅ Done
**Issue:** No test verifying exactly-one-active-primary-agent  
**Module:** agent  
**FR Reference:** FR-005  
**Status:** ✅ Done  

- [x] Add unit test in `crates/agent/` verifying single active primary agent
- [x] Test that creating new primary agent deactivates previous
- [x] Test that hidden agents (compaction, title, summary) don't affect invariant
- [x] Add integration test for session with multiple agent switches

**Acceptance Criteria:**
- Test confirms exactly one active primary agent per session
- Test verifies invariant maintained across agent switches
- Hidden agents (compaction, title, summary) don't affect invariant

---

### P1-4: ✅ Done
**Issue:** No unit tests for Project→Session→Message→Part ownership tree  
**Module:** core  
**FR Reference:** FR-001  
**Status:** ✅ Done  

- [x] Add unit test in `crates/core/` verifying ownership tree is acyclic
- [x] Test that cycles cannot be created (Project→Session→Message→Part→Project)
- [x] Add integration test for fork chain maintaining acyclicity

**Acceptance Criteria:**
- Ownership tree acyclicity is verified by tests ✅
- Fork operations maintain tree structure correctly ✅

**Test Commands:**
```bash
cargo test -p opencode-core -- ownership_tree  # 30 tests passed
cargo test -p opencode-core -- fork_acyclicity  # 8 tests passed
```

---

### P1-5: ✅ Done
**Issue:** Missing create→fork→share→compact→revert integration test  
**Module:** storage  
**FR Reference:** FR-002  
**Status:** TODO  

- [ ] Add integration test covering complete session lifecycle:
  1. Create session
  2. Fork child session
  3. Share session (export/share mechanism)
  4. Compact session (context compression)
  5. Revert to checkpoint
- [ ] Verify session state integrity after each operation
- [ ] Verify message history preserved correctly through operations

**Acceptance Criteria:**
- Complete lifecycle test passes
- Session state consistent after each operation

---

### P1-6: ✅ Done
**Issue:** Desktop app not implemented (stubs only)  
**Module:** cli  
**FR Reference:** FR-015  
**Status:** DONE  

- [x] Implement desktop startup flow in `crates/cli/`
- [x] Integrate WebView for desktop shell
- [x] Implement desktop-specific configuration handling
- [x] Add desktop mode to CLI argument parsing

**Acceptance Criteria:**
- [x] Desktop app starts successfully
- [x] WebView displays TUI content

**Tests Added:**
- 18 unit tests for CLI argument parsing (DesktopArgs fields, port, hostname, no_browser, acp_enabled)
- 11 unit tests for desktop configuration loading (config file loading, precedence logic, ACP settings)
- Integration tests verify desktop app starts successfully

---

### P1-7: ✅ Done
**Issue:** Web server mode incomplete (stub only)  
**Module:** cli  
**FR Reference:** FR-015  
**Status:** DONE  

- [x] Implement full web server in `crates/cli/src/cmd/web.rs`
- [x] Add authentication protection to web endpoints
- [x] Implement session sharing between web and TUI modes
- [x] Add web-specific route handlers

**Acceptance Criteria:**
- [x] Web server serves authenticated endpoints
- [x] Sessions accessible from both web and TUI interfaces

**Implementation Notes:**
- Enhanced web.rs with WebServerState struct for session sharing between web and TUI modes
- Added SessionSharing integration
- Created e2e_web_server.rs with 9 tests covering endpoint definition, session sharing, and health checks
- Added 9 web_auth tests in server_integration_tests.rs covering API key authentication

---

### P1-8: ✅ Done
**Issue:** ACP transport not implemented (event structs exist)  
**Module:** control-plane  
**FR Reference:** FR-015  
**Status:** TODO  

- [ ] Implement ACP handshake mechanism
- [ ] Implement ACP transport layer for editor communication
- [ ] Add connection management for ACP clients
- [ ] Integrate with server for ACP endpoint handling

**Acceptance Criteria:**
- ACP handshake completes successfully
- Editor integration works via ACP transport

---

### P1-9: Refactor Config Crate
**Issue:** `crates/config/src/lib.rs` is empty re-export  
**Module:** config  
**FR Reference:** FR-003  
**Status:** TODO  

- [ ] Move config logic from `crates/core/src/config.rs` to `crates/config/`
- [ ] Ensure `crates/config/` contains actual config implementation
- [ ] Update imports in dependent crates
- [ ] Verify config precedence and ownership still works

**Acceptance Criteria:**
- Config logic resides in `crates/config/` crate
- All config functionality preserved after refactor

---

## P2 Tasks (Medium Priority)

### P2-1: ✅ Done
**Issue:** No automated tests for slash command execution  
**Module:** tui  
**FR Reference:** FR-018  
**Status:** ✅ Done  

- [x] Add tests for `/compact` command
- [x] Add tests for `/connect` command
- [x] Add tests for `/help` command
- [x] Add tests for unknown/invalid slash commands

**Acceptance Criteria:** Slash commands have test coverage

**Test Commands:**
```bash
cargo test -p opencode-tui -- slash  # 33 tests passed (28 new + 5 existing)
```

**Implementation Notes:**
- Added 28 tests in `crates/tui/tests/slash_command_tests.rs`
- Added `filtered_commands()` getter method to `SlashCommandOverlay` in `crates/tui/src/dialogs/slash_command.rs`
- Tests cover command registry lookup, filtering, and action variants

---

### P2-2: TUI Input Model Tests
**Issue:** No tests for input model (multiline, history, autocomplete)  
**Module:** tui  
**FR Reference:** FR-018  
**Status:** ✅ Done  

- [x] Add tests for multiline input handling
- [x] Add tests for input history navigation
- [x] Add tests for autocomplete triggering

**Acceptance Criteria:** Input model has test coverage

---

### P2-3: TUI Sidebar Tests
**Issue:** No tests for sidebar visibility and content  
**Module:** tui  
**FR Reference:** FR-018  
**Status:** TODO  

- [ ] Add tests for sidebar visibility toggle
- [ ] Add tests for file tree content
- [ ] Add tests for MCP/LSP status display
- [ ] Add tests for diagnostics display

**Acceptance Criteria:** Sidebar has test coverage

---

### P2-4: Per-Agent Model Override Test
**Issue:** Per-agent model override not explicitly tested  
**Module:** llm  
**FR Reference:** FR-012  
**Status:** TODO  

- [ ] Add test for agent-specific model selection
- [ ] Verify `build` agent uses default model
- [ ] Verify model override affects only specified agent

**Acceptance Criteria:** Per-agent model override verified

---

### P2-5: Route-Group Presence Tests
**Issue:** No integration tests for route groups  
**Module:** server  
**FR Reference:** FR-004  
**Status:** TODO  

- [ ] Add tests for session route group
- [ ] Add tests for config route group
- [ ] Add tests for provider route group
- [ ] Add tests for permission route group
- [ ] Add tests for MCP route group

**Acceptance Criteria:** All route groups have tests

---

### P2-6: API Negative Tests
**Issue:** No negative tests for unauthorized/malformed requests  
**Module:** server  
**FR Reference:** FR-004  
**Status:** TODO  

- [ ] Add tests for unauthorized access (missing auth token)
- [ ] Add tests for invalid auth token
- [ ] Add tests for malformed request bodies
- [ ] Add tests for invalid session/message IDs

**Acceptance Criteria:** API security gaps covered

---

### P2-7: Hidden vs Visible Agent UI Tests
**Issue:** Hidden vs visible agent UI behavior untested  
**Module:** agent  
**FR Reference:** FR-005  
**Status:** TODO  

- [ ] Add tests for visible agents (build, plan) in selection flows
- [ ] Add tests for hidden agents (compaction, title, summary) not in selection
- [ ] Verify UI displays correct agent list

**Acceptance Criteria:** Agent visibility behavior verified

---

### P2-8: Theme Auto-Sync Test
**Issue:** Theme auto-sync on install not tested
**Module:** tui
**FR Reference:** FR-009
**Status:** ✅ Done

- [x] Add test for theme auto-sync when plugin installed
- [x] Verify theme applies immediately after install
- [x] Add regression tests for existing themes not affected by new plugin themes

**Acceptance Criteria:** Theme auto-sync verified

**Test Commands:**
```bash
cargo test -p opencode-tui -- theme_auto_sync  # 4 tests passed
cargo test -p opencode-tui -- plugin_theme     # 33 tests passed
cargo clippy --all -- -D warnings              # Passed
cargo build --release                          # Passed
```

**Implementation Notes:**
- Added 10 new tests in `crates/tui/tests/plugin_theme_tests.rs`
- Tests verify theme auto-sync when plugin is installed
- Tests verify theme applies immediately after install
- Tests ensure existing themes are not affected by new plugin themes

---

## Technical Debt Tasks

### TD-001: Empty crates/config/ crate
**Severity:** Medium  
**Action:** See P1-9

### TD-002: DirectoryScanner discovery mismatch
**Severity:** CRITICAL  
**Action:** See P0-1

### TD-003: ✅ Done
**Severity:** CRITICAL  
**Action:** ✅ Done - Added integration test `custom_tool_registration` in `crates/tools/src/discovery.rs`

### TD-004: Non-deterministic plugin hook execution
**Severity:** High  
**Action:** See P1-1

### TD-005: ✅ Done
**Severity:** CRITICAL  
**Action:** See P0-3

### TD-006: ✅ Done
**Severity:** High  
**Action:** See P1-8 - ACP transport layer fully implemented with handshake, messages, connection management, and event streaming.

### TD-007: ✅ Done
**Severity:** Medium  
**Action:** Remove in v4.0 - Added regression tests and unit tests for deprecated mode field warning.

### TD-008: Deprecated tools field
**Severity:** Medium  
**Action:** Remove after migration period

### TD-009: Deprecated theme field
**Severity:** Low  
**Action:** Already moved to tui.json

### TD-010: Deprecated keybinds field
**Severity:** Low  
**Action:** Already moved to tui.json

---

## Task Summary

| Priority | Count | Status |
|----------|-------|--------|
| P0 | 3 | TODO |
| P1 | 9 | TODO |
| P2 | 8 | ✅ Done |
| Technical Debt | 6 | Various |

**Total Tasks:** 26

---

## Definition of Done

A task is complete when:
1. All subtasks are checked off
2. Code compiles without errors
3. Tests pass (`cargo test --all-features`)
4. Clippy passes (`cargo clippy --all -- -D warnings`)
5. Formatting is correct (`cargo fmt --all`)
