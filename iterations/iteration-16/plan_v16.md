# Implementation Plan - Iteration 16

**Version:** 16.0  
**Generated:** 2026-04-14  
**Based on:** Spec Document v16 and Gap Analysis  

---

## 1. Priority Classification

### P0 - Blocking Issues: NONE ✅
All P0 blocking issues from iteration-15 have been resolved.

| ID | Issue | Module | Status |
|----|-------|--------|--------|
| ~~P0-1~~ | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | ✅ FIXED |
| ~~P0-2~~ | Custom tools not registered with ToolRegistry | tools | ✅ FIXED |
| ~~P0-3~~ | Plugin tool registration missing | plugin | ✅ FIXED |

### P1 - High Priority Issues
| ID | Issue | Module | FR Ref | Status |
|----|-------|--------|--------|--------|
| ~~P1-1~~ | Non-deterministic hook execution order | plugin | FR-008 | ✅ FIXED |
| ~~P1-2~~ | Plugin config ownership not enforced | plugin | FR-008 | ✅ FIXED |
| ~~P1-3~~ | Exactly-one-active-primary-agent invariant untested | agent | FR-005 | ✅ FIXED |
| ~~P1-4~~ | Ownership tree acyclicity not tested | core | FR-001 | ✅ FIXED |
| ~~P1-5~~ | Session lifecycle integration tests incomplete | storage | FR-002 | ✅ FIXED |
| ~~P1-6~~ | Desktop app not implemented | cli | FR-015 | ✅ FIXED |
| ~~P1-7~~ | Web server mode incomplete | cli | FR-015 | ✅ FIXED |
| ~~P1-8~~ | ACP transport E2E test missing | control-plane | FR-015 | ✅ FIXED |
| ~~P1-9~~ | Config crate is empty re-export | config | FR-003 | ✅ FIXED |
| **P1-NEW-1** | ACP E2E connection test missing | control-plane | FR-015 | ✅ DONE |
| **P1-NEW-2** | Duplicate `directory_scanner.rs` (832 lines) | config | FR-003 | ❌ TODO |
| **P1-NEW-3** | Two `ToolRegistry` implementations diverge risk | core/tools | FR-006 | ❌ TODO |

### P2 - Medium Priority Issues
| ID | Issue | Module | FR Ref | Status |
|----|-------|--------|--------|--------|
| ~~P2-1~~ | TUI slash command tests missing | tui | FR-018 | ✅ FIXED |
| ~~P2-2~~ | TUI input model tests missing | tui | FR-018 | ✅ FIXED |
| ~~P2-3~~ | TUI sidebar tests missing | tui | FR-018 | ✅ FIXED |
| ~~P2-4~~ | Per-agent model override untested | llm | FR-012 | ✅ FIXED |
| P2-5 | Route-group presence tests missing | server | FR-004 | ⚠️ PARTIAL |
| P2-6 | API negative tests missing | server | FR-004 | ⚠️ PARTIAL |
| ~~P2-7~~ | Hidden vs visible agent UI behavior untested | agent | FR-005 | ✅ FIXED |
| ~~P2-8~~ | Theme auto-sync on install not tested | tui | FR-009 | ✅ FIXED |
| **P2-NEW-1** | Route-group MCP/config/provider tests missing | server | FR-004 | ❌ TODO |
| **P2-NEW-2** | Malformed request body tests missing | server | FR-004 | ❌ TODO |
| **P2-NEW-3** | Hook determinism explicit test missing | plugin | FR-008 | ❌ TODO |
| **P2-NEW-4** | Security tests (injection, path traversal) | server | FR-004 | ❌ TODO |

---

## 2. Implementation Phases

### Phase A: P1 Critical Items (Remaining from Iteration-15)

#### A.1: Add ACP E2E Integration Test (P1-NEW-1)
- **Location:** `crates/control-plane/` or `tests/`
- **Current:** `transport.rs` (847 lines), `handshake.rs` (630 lines) exist; `acp_transport_tests.rs` (141 lines) only tests serialization
- **Required:** Full E2E test that establishes connection, completes handshake, exchanges messages
- **Action:** Add integration test that:
  1. Starts server with ACP enabled
  2. Creates `AcpTransportClient`
  3. Connects via TCP/WebSocket
  4. Completes handshake
  5. Sends/receives messages
- **Dependencies:** None
- **Verification:** `cargo test -p opencode-integration-tests -- acp_e2e`
- **Status:** ✅ DONE - 20 E2E tests added in `tests/src/acp_e2e_tests.rs`

#### A.2: Remove Duplicate `directory_scanner.rs` (P1-NEW-2)
- **Location:** `crates/core/src/config/directory_scanner.rs`
- **Current:** Identical file exists at `crates/config/src/directory_scanner.rs` (832 lines each)
- **Required:** Remove duplicate, use single source of truth
- **Action:**
  1. Delete `crates/core/src/config/directory_scanner.rs`
  2. Update `crates/core/src/lib.rs` exports to use `opencode_config::DirectoryScanner`
  3. Verify no remaining references to deleted file
- **Dependencies:** None
- **Verification:** `cargo build --all-features && cargo test -p opencode-core`

#### A.3: Audit Two ToolRegistry Implementations (P1-NEW-3)
- **Location:** `crates/core/src/tool.rs` vs `crates/tools/src/registry.rs`
- **Current:** Two separate `ToolRegistry` structs with different designs
- **Risk:** `core::ToolRegistry` used in agent runtime may miss features (caching, async)
- **Action:**
  1. Trace all usages of `core::ToolRegistry` in agent runtime
  2. Verify `opencode_tools::ToolRegistry` features are available
  3. Either consolidate to single registry or document intentional separation
- **Dependencies:** A.2
- **Verification:** `cargo test -p opencode-agent -- tool_registry`

### Phase B: P2 Test Coverage

#### B.1: Complete Route-Group Tests (P2-NEW-1)
- **Location:** `crates/server/` integration tests
- **Current:** Session/permission routes tested, MCP/config/provider not explicit
- **Action:**
  1. Add explicit MCP route group tests (`/api/mcp/servers`, `/api/mcp/tools`, etc.)
  2. Add config route group tests
  3. Add provider route group tests
- **Dependencies:** None
- **Verification:** `cargo test -p opencode-server -- route_group`

#### B.2: Complete API Negative Tests (P2-NEW-2)
- **Location:** `crates/server/` integration tests
- **Current:** Auth tests done, malformed request tests missing
- **Action:**
  1. Add malformed request body tests (invalid JSON, missing fields, wrong types)
  2. Add invalid session/message ID tests
- **Dependencies:** None
- **Verification:** `cargo test -p opencode-server -- negative`

#### B.3: Add Hook Determinism Test (P2-NEW-3)
- **Location:** `crates/plugin/src/lib.rs` tests
- **Current:** `sorted_plugin_names()` implements deterministic ordering, but no explicit test
- **Action:** Add 100-iteration test verifying `sorted_plugin_names()` returns consistent ordering
- **Dependencies:** None
- **Verification:** `cargo test -p opencode-plugin -- hook_determinism`

#### B.4: Add Security Tests (P2-NEW-4)
- **Location:** `crates/server/` integration tests
- **Current:** No security-focused tests
- **Action:**
  1. Add SQL injection tests
  2. Add path traversal tests
  3. Add request smuggling tests
- **Dependencies:** B.2
- **Verification:** `cargo test -p opencode-server -- security`

### Phase C: Phase 6 Release Qualification

#### C.1: Pre-Release Checklist
- Run full test suite: `cargo test --all-features`
- Run clippy: `cargo clippy --all -- -D warnings`
- Run formatting: `cargo fmt --all`
- Performance benchmarks
- Memory profiling
- Security audit
- Documentation completeness check

---

## 3. Technical Debt Remediation

| TD | Item | Module | Severity | Action | Status |
|----|------|--------|----------|--------|--------|
| TD-001 | Empty `crates/config/` crate | config | ✅ RESOLVED | N/A — now has real implementation | ✅ Fixed |
| TD-002 | `DirectoryScanner` discovery mismatch | tools | ✅ RESOLVED | N/A — now scans .ts/.js | ✅ Fixed |
| TD-003 | Custom tools not registered | tools | ✅ RESOLVED | N/A — registration implemented | ✅ Fixed |
| TD-004 | Non-deterministic hook execution | plugin | ✅ RESOLVED | N/A — priority sorting implemented | ✅ Fixed |
| TD-005 | Plugin `register_tool()` missing | plugin | ✅ RESOLVED | N/A — method implemented | ✅ Fixed |
| TD-006 | ACP transport layer missing | control-plane | **PARTIAL** | Add E2E integration tests | ⚠️ In Progress |
| TD-007 | Deprecated `mode` field | config | Medium | Remove in v4.0 | Deferred |
| TD-008 | Deprecated `tools` field | config | Medium | Remove after migration | Deferred |
| TD-009 | Deprecated `theme` field | config | ✅ RESOLVED | Moved to tui.json | ✅ Fixed |
| TD-010 | Deprecated `keybinds` field | config | ✅ RESOLVED | Moved to tui.json | ✅ Fixed |
| TD-NEW-1 | Duplicate `directory_scanner.rs` | config/core | **HIGH** | Remove duplicate from core/ | ❌ TODO |
| TD-NEW-2 | Two ToolRegistry impls | core/tools | **HIGH** | Audit and consolidate | ❌ TODO |

---

## 4. Phase Status Summary

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~95% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~90% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~90% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ⚠️ Partial | ~80% (desktop/web done, ACP E2E pending) |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~85% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 5. Implementation Order

1. **Immediate: P1 Items (Critical)**
   - A.1: Add ACP E2E integration test
   - A.2: Remove duplicate `directory_scanner.rs`
   - A.3: Audit two ToolRegistry implementations

2. **Short-term: P2 Test Coverage**
   - B.1: Complete route-group tests
   - B.2: Complete API negative tests
   - B.3: Add hook determinism test
   - B.4: Add security tests

3. **Medium-term: Phase 6 Release Qualification**
   - C.1: Pre-release checklist

---

## 6. Verification Checklist

- [ ] ACP E2E test establishes connection, completes handshake, exchanges messages
- [ ] Duplicate `directory_scanner.rs` removed from `crates/core/src/config/`
- [ ] `crates/core/src/lib.rs` exports `opencode_config::DirectoryScanner`
- [ ] ToolRegistry audit completed and documented
- [ ] MCP route group tests added
- [ ] Config route group tests added
- [ ] Provider route group tests added
- [ ] Malformed request body tests added
- [ ] Invalid session/message ID tests added
- [ ] Hook determinism test (100 iterations) added
- [ ] Security tests (injection, path traversal) added
- [ ] Full test suite passes
- [ ] Clippy passes
- [ ] Formatting correct

---

## 7. Progress Summary (Iteration-15 → Iteration-16)

| Priority | Items | Fixed | Remaining | Completion |
|----------|-------|-------|-----------|------------|
| P0 | 3 | 3 | 0 | 100% |
| P1 | 12 | 9 | 3 | 75% |
| P2 | 12 | 6 | 6 | 50% |

**Overall Implementation Status:** ~80-85% complete (up from ~65-70%)

---

*Document generated: 2026-04-14*
*Iteration: 16*
*Phase: Phase 4-5 of 6 (Interface Implementation, Hardening)*
