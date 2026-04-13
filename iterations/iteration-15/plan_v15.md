# Implementation Plan - Iteration 15

**Version:** 15.0  
**Generated:** 2026-04-13  
**Based on:** Spec Document v15 and Gap Analysis  

---

## 1. Priority Classification

### P0 - Blocking Issues (Must Fix Before Release)
| ID | Issue | Module | FR Ref |
|----|-------|--------|--------|
| **P0-1** | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | FR-007 |
| **P0-2** | Custom tools not registered with ToolRegistry | tools | FR-007 |
| **P0-3** | Plugin tool registration missing | plugin | FR-008 |

### P1 - High Priority Issues
| ID | Issue | Module | FR Ref |
|----|-------|--------|--------|
| P1-1 | Non-deterministic hook execution order | plugin | FR-008 |
| P1-2 | Plugin config ownership not enforced | plugin | FR-008 |
| P1-3 | Exactly-one-active-primary-agent invariant untested | agent | FR-005 |
| P1-4 | Ownership tree acyclicity not tested | core | FR-001 |
| P1-5 | Session lifecycle integration tests incomplete | storage | FR-002 |
| P1-6 | Desktop app not implemented | cli | FR-015 |
| P1-7 | Web server mode incomplete | cli | FR-015 |
| P1-8 | ACP transport not implemented | control-plane | FR-015 |
| P1-9 | Config crate is empty re-export | config | FR-003 |

### P2 - Medium Priority Issues
| ID | Issue | Module | FR Ref |
|----|-------|--------|--------|
| P2-1 | TUI slash command tests missing | tui | FR-018 |
| P2-2 | TUI input model tests missing | tui | FR-018 |
| P2-3 | TUI sidebar tests missing | tui | FR-018 |
| P2-4 | Per-agent model override untested | llm | FR-012 |
| P2-5 | Route-group presence tests missing | server | FR-004 |
| P2-6 | API negative tests (auth, malformed) missing | server | FR-004 |
| P2-7 | Hidden vs visible agent UI behavior untested | agent | FR-005 |
| P2-8 | Theme auto-sync on install not tested | tui | FR-009 |

---

## 2. Implementation Phases

### Phase A: P0 Critical Fixes (Custom Tools & Plugin System)
**Objective:** Fix blocking issues in custom tool discovery/registration and plugin tool registration

#### A.1: Fix Custom Tool Discovery (P0-1)
- **Location:** `crates/core/src/config/directory_scanner.rs:228`
- **Current:** Scans `TOOL.md` files
- **Required:** Scan `.ts` and `.js` files with `export default tool({...})`
- **Action:** Implement TypeScript/JavaScript file discovery and parsing
- **Dependencies:** None

#### A.2: Register Custom Tools with ToolRegistry (P0-2)
- **Location:** `crates/tools/src/registry.rs`
- **Current:** `DirectoryScanner` records tools in `tools_info` but not registered
- **Required:** Discovered tools must be registered for execution
- **Action:** Add registration flow from discovered tools to ToolRegistry
- **Dependencies:** A.1

#### A.3: Implement Plugin Tool Registration (P0-3)
- **Location:** `crates/plugin/src/lib.rs`
- **Current:** `PluginToolAdapter` exists but no `register_tool()` method
- **Required:** Plugins must be able to add tools to agent toolset
- **Action:** Add `register_tool()` method to Plugin trait, integrate with ToolRegistry
- **Dependencies:** None

### Phase B: P1 Priority Fixes
**Objective:** Address high-priority gaps in determinism, invariants, tests, and interfaces

#### B.1: Fix Hook Execution Determinism (P1-1)
- **Location:** `crates/plugin/src/lib.rs:358-369`
- **Current:** Uses HashMap iteration (non-deterministic)
- **Required:** Deterministic execution order per PRD
- **Action:** Add explicit `priority` field, use ordered execution

#### B.2: Enforce Plugin Config Ownership (P1-2)
- **Current:** Server/runtime and TUI plugin configs can be mixed
- **Required:** Config ownership boundary enforcement
- **Action:** Add validation separating server/runtime vs TUI plugin config

#### B.3: Add Primary Agent Invariant Tests (P1-3)
- **Location:** `crates/agent/`
- **Current:** No test for exactly-one-active-primary-agent
- **Action:** Add invariant test verifying single active primary agent

#### B.4: Add Ownership Tree Acyclicity Tests (P1-4)
- **Location:** `crates/core/`
- **Current:** No unit tests for ownership tree acyclicity
- **Action:** Add unit test verifying Project→Session→Message→Part is acyclic

#### B.5: Complete Session Lifecycle Integration Tests (P1-5)
- **Location:** `crates/storage/`
- **Current:** No complete lifecycle test
- **Action:** Add create→fork→share→compact→revert integration test

#### B.6: Implement Desktop App (P1-6)
- **Location:** `crates/cli/`
- **Current:** Stubs only
- **Required:** Desktop app shell with WebView
- **Action:** Implement desktop startup flow

#### B.7: Implement Web Server Mode (P1-7)
- **Location:** `crates/cli/src/cmd/web.rs`
- **Current:** Stub only
- **Required:** Full web server with auth
- **Action:** Implement web server mode

#### B.8: Implement ACP Transport (P1-8)
- **Location:** `crates/control-plane/`
- **Current:** Event structs exist, no transport
- **Required:** ACP handshake and transport for editor integration
- **Action:** Implement ACP transport layer

#### B.9: Refactor Config Crate (P1-9)
- **Location:** `crates/config/`
- **Current:** Empty re-export from core
- **Required:** Config logic in dedicated crate per PRD 19
- **Action:** Move config logic from core to dedicated crate

### Phase C: P2 Medium Priority
**Objective:** Complete test coverage and minor improvements

#### C.1: TUI Test Coverage
- P2-1: Add tests for slash command execution
- P2-2: Add tests for input model (multiline, history, autocomplete)
- P2-3: Add tests for sidebar visibility and content

#### C.2: LLM/Provider Test Coverage
- P2-4: Add test for per-agent model override

#### C.3: Server Test Coverage
- P2-5: Add integration tests for route groups
- P2-6: Add negative tests for auth and malformed requests

#### C.4: Agent UI Behavior Tests
- P2-7: Add tests for hidden vs visible agent in selection flows

#### C.5: Theme Auto-Sync Tests
- P2-8: Add test for theme auto-sync on install

---

## 3. Technical Debt Remediation

| TD | Item | Module | Severity | Action |
|----|------|--------|----------|--------|
| TD-001 | Empty `crates/config/` crate | config | Medium | Move config logic to dedicated crate |
| TD-002 | `DirectoryScanner` discovery mismatch | tools | **CRITICAL** | Implement TypeScript/JavaScript discovery (see A.1) |
| TD-003 | Custom tools not registered | tools | **CRITICAL** | Add registration flow (see A.2) |
| TD-004 | Non-deterministic hook execution | plugin | High | Add explicit priority ordering (see B.1) |
| TD-005 | Plugin `register_tool()` missing | plugin | **CRITICAL** | Add method to Plugin trait (see A.3) |
| TD-006 | ACP transport layer missing | control-plane | High | Implement ACP transport (see B.8) |
| TD-007 | Deprecated `mode` field | config | Medium | Remove in v4.0 |
| TD-008 | Deprecated `tools` field | config | Medium | Remove after migration |
| TD-009 | Deprecated `theme` field | config | Low | Moved to tui.json |
| TD-010 | Deprecated `keybinds` field | config | Low | Moved to tui.json |

---

## 4. Phase Status Summary

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Complete | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Complete | ~90% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ⚠️ Partial | ~70% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Mostly Complete | ~85% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ❌ Not Started | ~20% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Mostly Complete | ~80% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 5. Implementation Order

1. **Week 1-2: P0 Critical Fixes**
   - A.1: Fix custom tool discovery (.ts/.js)
   - A.2: Register custom tools with ToolRegistry
   - A.3: Implement plugin tool registration

2. **Week 3-4: P1 Determinism & Invariants**
   - B.1: Fix hook execution determinism
   - B.2: Enforce plugin config ownership
   - B.3: Add primary agent invariant tests
   - B.4: Add ownership tree acyclicity tests
   - B.5: Complete session lifecycle tests

3. **Week 5-6: P1 Interface Implementation**
   - B.6: Implement desktop app
   - B.7: Implement web server mode
   - B.8: Implement ACP transport
   - B.9: Refactor config crate

4. **Week 7-8: P2 Test Coverage**
   - C.1: TUI test coverage
   - C.2: LLM test coverage
   - C.3: Server test coverage
   - C.4-C.5: Agent UI and theme tests

---

## 6. Verification Checklist

- [ ] Custom tools can be discovered from `.ts/.js` files
- [ ] Custom tools are registered with ToolRegistry and executable
- [ ] Plugin tools can be registered and appear in agent toolset
- [ ] Hooks execute in deterministic order
- [ ] Plugin config ownership is enforced
- [ ] Primary agent invariant verified by tests
- [ ] Ownership tree acyclicity verified by tests
- [ ] Session lifecycle (create→fork→share→compact→revert) works
- [ ] Desktop app starts and displays WebView
- [ ] Web server mode serves authenticated endpoints
- [ ] ACP transport establishes editor connections
- [ ] Config crate contains proper config logic
- [ ] TUI components have test coverage
- [ ] API negative tests exist
