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
**Status:** TODO  

- [ ] Create registration flow from `DirectoryScanner` to `ToolRegistry`
- [ ] Ensure discovered custom tools appear in tool listing
- [ ] Verify custom tools can be executed through standard tool pipeline
- [ ] Add integration test for custom tool discovery → registration → execution

**Acceptance Criteria:**
- Discovered custom tools appear in `ToolRegistry` listing
- Custom tools can be executed with proper permission gating

---

### P0-3: ✅ Done
**Issue:** `PluginToolAdapter` exists but no mechanism to register plugin tools  
**Module:** plugin  
**FR Reference:** FR-008  
**Status:** TODO  

- [ ] Add `register_tool()` method to `Plugin` trait in `crates/plugin/src/lib.rs`
- [ ] Implement tool registration in `PluginManager`
- [ ] Integrate `PluginManager` with `ToolRegistry` for plugin-provided tools
- [ ] Add tests verifying plugin tools appear in registry after plugin activation
- [ ] Verify plugin tools respect permission system

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

### P1-3: Test Primary Agent Invariant
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

### P1-4: Test Ownership Tree Acyclicity
**Issue:** No unit tests for Project→Session→Message→Part ownership tree  
**Module:** core  
**FR Reference:** FR-001  
**Status:** TODO  

- [ ] Add unit test in `crates/core/` verifying ownership tree is acyclic
- [ ] Test that cycles cannot be created (Project→Session→Message→Part→Project)
- [ ] Add integration test for fork chain maintaining acyclicity

**Acceptance Criteria:**
- Ownership tree acyclicity is verified by tests
- Fork operations maintain tree structure correctly

---

### P1-5: Complete Session Lifecycle Integration Tests
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

### P1-6: Implement Desktop App
**Issue:** Desktop app not implemented (stubs only)  
**Module:** cli  
**FR Reference:** FR-015  
**Status:** TODO  

- [ ] Implement desktop startup flow in `crates/cli/`
- [ ] Integrate WebView for desktop shell
- [ ] Implement desktop-specific configuration handling
- [ ] Add desktop mode to CLI argument parsing

**Acceptance Criteria:**
- Desktop app starts successfully
- WebView displays TUI content

---

### P1-7: Implement Web Server Mode
**Issue:** Web server mode incomplete (stub only)  
**Module:** cli  
**FR Reference:** FR-015  
**Status:** TODO  

- [ ] Implement full web server in `crates/cli/src/cmd/web.rs`
- [ ] Add authentication protection to web endpoints
- [ ] Implement session sharing between web and TUI modes
- [ ] Add web-specific route handlers

**Acceptance Criteria:**
- Web server serves authenticated endpoints
- Sessions accessible from both web and TUI interfaces

---

### P1-8: Implement ACP Transport
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

### P2-1: TUI Slash Command Tests
**Issue:** No automated tests for slash command execution  
**Module:** tui  
**FR Reference:** FR-018  
**Status:** TODO  

- [ ] Add tests for `/compact` command
- [ ] Add tests for `/connect` command
- [ ] Add tests for `/help` command
- [ ] Add tests for unknown/invalid slash commands

**Acceptance Criteria:** Slash commands have test coverage

---

### P2-2: TUI Input Model Tests
**Issue:** No tests for input model (multiline, history, autocomplete)  
**Module:** tui  
**FR Reference:** FR-018  
**Status:** TODO  

- [ ] Add tests for multiline input handling
- [ ] Add tests for input history navigation
- [ ] Add tests for autocomplete triggering

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
**Status:** TODO  

- [ ] Add test for theme auto-sync when plugin installed
- [ ] Verify theme applies immediately after install

**Acceptance Criteria:** Theme auto-sync verified

---

## Technical Debt Tasks

### TD-001: Empty crates/config/ crate
**Severity:** Medium  
**Action:** See P1-9

### TD-002: DirectoryScanner discovery mismatch
**Severity:** CRITICAL  
**Action:** See P0-1

### TD-003: Custom tools discovered but not registered
**Severity:** CRITICAL  
**Action:** See P0-2

### TD-004: Non-deterministic plugin hook execution
**Severity:** High  
**Action:** See P1-1

### TD-005: Plugin register_tool() method missing
**Severity:** CRITICAL  
**Action:** See P0-3

### TD-006: ACP transport layer missing
**Severity:** High  
**Action:** See P1-8

### TD-007: Deprecated mode field
**Severity:** Medium  
**Action:** Remove in v4.0

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
| P2 | 8 | TODO |
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
