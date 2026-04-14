# Constitution Updates - Iteration 16

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-16/gap-analysis.md`
**Previous Constitution:** `iteration-15/constitution_updates.md` (v2.8)
**Status:** Amendment Proposal

---

## Executive Summary

Iteration 16 gap analysis reveals **major progress** over Iteration 15 — all P0 blockers resolved, ~80-85% complete. However, **3 new P1 issues** and **4 new P2 issues** were identified that are NOT covered by Constitution v2.8:

**New P1 Issues (Not Covered):**
1. **Duplicate `directory_scanner.rs`** — 832-line code duplication between `config/` and `core/`
2. **Two `ToolRegistry` implementations** — `core::ToolRegistry` vs `opencode_tools::ToolRegistry` diverge risk
3. **ACP E2E integration test missing** — Transport exists but no end-to-end test

**New P2 Issues (Not Covered):**
4. Route-group tests missing (MCP, config, provider)
5. API negative tests missing (malformed requests, security)
6. Hook determinism test missing (implementation exists, test doesn't)
7. Security injection/traversal tests missing

**Assessment:** Constitution v2.8 is **PARTIALLY ADEQUATE** — covers resolved P0s but lacks mandates for code quality (deduplication), registry consolidation, and E2E test requirements.

**Net new constitutional content required:** 1 new amendment (Q) + 2 new sections

---

## Article I: Gap Analysis Summary (Iteration 16)

### New P1 Issues Identified

| Gap ID | Description | Module | Severity | Constitutional Coverage |
|--------|-------------|--------|----------|------------------------|
| **P1-NEW-1** | Duplicate `directory_scanner.rs` (832 lines) | config/core | P1 | ❌ Not covered |
| **P1-NEW-2** | Two `ToolRegistry` implementations diverge | core/tools | P1 | ❌ Not covered |
| **P1-NEW-3** | ACP E2E connection test missing | control-plane | P1 | ⚠️ Art VI §6.2 covers transport, not E2E testing |

### New P2 Issues Identified

| Gap ID | Description | Module | Severity | Constitutional Coverage |
|--------|-------------|--------|----------|------------------------|
| P2-NEW-1 | Route-group MCP/config/provider tests missing | server | P2 | ❌ Not covered |
| P2-NEW-2 | Malformed request body tests missing | server | P2 | ⚠️ Art V §5.x covers server, not explicit negative tests |
| P2-NEW-3 | Hook determinism no explicit test | plugin | P2 | ❌ Implementation exists, test mandate missing |
| P2-NEW-4 | Security tests (injection, path traversal) missing | server | P2 | ❌ Not covered |

---

## Article II: Iteration 15 → 16 Status Transfer

### Previously Blocked Items — Now Resolved

| Gap ID | Description | Constitution Reference | Iteration 16 Status |
|--------|-------------|------------------------|---------------------|
| P0-15-1 | Custom tool discovery scans TOOL.md | Amend P §P.1 | ✅ FIXED — scans `.ts/.js` |
| P0-15-2 | Custom tools not registered | Art III §3.4 | ✅ FIXED — `register_custom_tools()` |
| P0-15-3 | Plugin tool registration not integrated | Art III §3.5 | ✅ FIXED — `register_tool()`, `export_as_tools()` |
| P1-15-4 | Non-deterministic hook execution | Art III §3.6 | ✅ IMPLEMENTED — `sorted_plugin_names()` |

### Constitution v2.8 Coverage Verification

| Constitution Reference | Mandate | Iteration 16 Status |
|------------------------|---------|---------------------|
| Amend P §P.1 | Custom tool discovery | ✅ VERIFIED — `is_tool_file()` scans `.ts/.js/.mts/.cts` |
| Art III §3.4 | Custom tool registration | ✅ VERIFIED — `register_custom_tools()` at `discovery.rs:230-248` |
| Art III §3.5 | Plugin tool registration | ✅ VERIFIED — `register_tool()` at `lib.rs:268`, 7 tests |
| Art III §3.6 | Hook determinism | ⚠️ PARTIAL — `sorted_plugin_names()` implemented, no test |
| Art VI §6.2 | ACP transport | ✅ VERIFIED — `AcpTransportClient` (847 lines) |
| Art VI §6.2 | ACP E2E test | ❌ MISSING — No integration test exists |

---

## Article III: New P1 Requirements

### Section 3.7 - Code Deduplication Mandate

**Requirement:** Duplicate code across crates MUST be eliminated. When identical or near-identical implementations exist, one must be removed with imports redirected.

**CONSTRAINT:** The `DirectoryScanner` type:
- MUST exist in exactly ONE crate (`opencode_config`)
- MUST be re-exported from `opencode_core` if needed by core
- MUST NOT exist as duplicate files in multiple crates

**Example violation:**
```rust
// ❌ VIOLATION: Duplicate file
crates/config/src/directory_scanner.rs     // 832 lines
crates/core/src/config/directory_scanner.rs // 832 lines (DUPLICATE)

// ✅ COMPLIANT: Single source
crates/config/src/directory_scanner.rs     // 832 lines
// crates/core/src/lib.rs re-exports:
pub use opencode_config::DirectoryScanner;
```

**Verification checklist:**
- [ ] `crates/core/src/config/directory_scanner.rs` deleted or replaced with re-export
- [ ] No other duplicate file pairs exist (checked via line count + content hash)
- [ ] All imports reference single source of truth

**P1 Gap Addressed:** "Duplicate `directory_scanner.rs` (832 lines)"

### Section 3.8 - Registry Implementation Consolidation

**Requirement:** When multiple implementations of the same concept exist (e.g., `ToolRegistry`), the project MUST either:
1. Consolidate to a single implementation, OR
2. Document the intentional separation with explicit boundaries

**CONSTRAINT:** If multiple `ToolRegistry` implementations exist:
```rust
// Two implementations exist:
core::ToolRegistry          // Manages ToolDefinition + ToolExecutor pairs
opencode_tools::ToolRegistry // Full-featured registry with caching, async

// Resolution option A: Consolidate
// Move all tool registry functionality to opencode_tools::ToolRegistry
// Update crates/core/src/tool.rs to re-export from opencode_tools

// Resolution option B: Document separation
// In crates/core/src/tool.rs:
/// Core tool registry for agent runtime.
/// For full-featured registry with caching, use opencode_tools::ToolRegistry.
/// This registry is intentionally lightweight for fast agent initialization.
```

**Verification checklist:**
- [ ] Trace `core::ToolRegistry` usage in `crates/agent/src/runtime.rs`
- [ ] Verify `opencode_tools::ToolRegistry` features (caching, async) are available to agent
- [ ] Either consolidate or add `SEPARATION.md` documenting intentional split

**P1 Gap Addressed:** "Two `ToolRegistry` implementations diverge risk"

---

## Article IV: New P2 Requirements

### Section 4.1 - ACP Transport E2E Test Mandate

**Requirement:** ACP transport implementation MUST have an integration test that:
1. Starts a server with ACP enabled
2. Connects a client via `AcpTransportClient::connect()`
3. Completes handshake
4. Exchanges messages end-to-end

**CONSTRAINT:**
```rust
// MUST have test like:
#[tokio::test]
async fn test_acp_e2e_connection() {
    // 1. Start server
    let server = AcpServer::new().start().await;
    
    // 2. Connect client
    let client = AcpTransportClient::connect(server.addr()).await?;
    
    // 3. Complete handshake
    client.handshake().await?;
    
    // 4. Send and receive message
    client.send(Message::new("test")).await?;
    let recv = client.recv().await?;
    assert_eq!(recv.content(), "ack");
}
```

**P2 Gap Addressed:** "ACP E2E connection test missing"

### Section 4.2 - Route-Group Enumeration Tests

**Requirement:** Server integration tests MUST enumerate all routes in each route group and verify:
1. Route exists and responds
2. Authentication works correctly
3. Authorization boundaries enforced

**CONSTRAINT:**
```rust
// Route groups requiring enumeration tests:
const ROUTE_GROUPS: &[&str] = &["session", "permission", "auth", "mcp", "config", "provider"];

// For each group, test:
async fn test_mcp_route_group_complete() {
    let routes = discover_routes!("api/mcp/*");
    assert!(routes.contains("/api/mcp/servers"));
    assert!(routes.contains("/api/mcp/tools"));
    assert!(routes.contains("/api/mcp/execute"));
    // ... all MCP routes
}
```

**P2 Gap Addressed:** "Route-group MCP/config/provider tests missing"

### Section 4.3 - API Negative Test Requirements

**Requirement:** Server API tests MUST include negative test cases:

**CONSTRAINT:**
```rust
// Security and error test requirements:
// 1. Malformed request bodies
test_invalid_json()
test_missing_required_fields()
test_wrong_field_types()

// 2. Invalid resource IDs
test_nonexistent_session_operations()
test_invalid_message_ids()

// 3. Security tests
test_sql_injection_prevention()
test_path_traversal_prevention()
test_xss_prevention()
```

**P2 Gap Addressed:** "Malformed request body tests missing", "Security tests missing"

### Section 4.4 - Hook Determinism Explicit Test

**Requirement:** Hook execution determinism (implemented in Art III §3.6) MUST have an explicit test verifying consistent ordering across multiple invocations.

**CONSTRAINT:**
```rust
// MUST have test like:
#[test]
fn test_sorted_plugin_names_deterministic() {
    let manager = PluginManager::new();
    
    // Register plugins with various priorities
    manager.register_plugin("plugin_c", plugin_c, priority=10);
    manager.register_plugin("plugin_a", plugin_a, priority=0);
    manager.register_plugin("plugin_b", plugin_b, priority=5);
    
    // Verify consistent ordering across 100 iterations
    let expected = vec!["plugin_a", "plugin_b", "plugin_c"];
    for _ in 0..100 {
        let names = manager.sorted_plugin_names();
        assert_eq!(names, expected, "Hook order must be deterministic");
    }
}
```

**P2 Gap Addressed:** "Hook execution determinism no explicit test"

---

## Article V: Updated Compliance Checklist

### Code Quality Gate (NEW — Amendment Q)

- [ ] No duplicate source files across crates (>80% similarity = duplicate)
- [ ] `crates/core/src/config/directory_scanner.rs` deleted or re-export only
- [ ] ToolRegistry implementations documented or consolidated
- [ ] ACP E2E integration test exists and passes
- [ ] Route-group enumeration tests cover MCP, config, provider
- [ ] API negative tests include malformed requests, invalid IDs, security
- [ ] Hook determinism test exists with 100+ iterations

### Tools System (Amend P + Art III §3.4-3.8)
- [x] Custom tool discovery scans `.ts/.js` files
- [x] Discovered custom tools registered with `ToolRegistry`
- [x] Custom tool format follows ES module `export default tool({...})`
- [x] `PluginManager::register_tool()` implemented
- [x] Plugin tools appear in `ToolRegistry::list_tools()`
- [x] Hook execution sorted by priority (deterministic)
- [ ] Hook determinism explicit test (100 iterations)
- [ ] No duplicate tool-related code across crates

---

## Appendix A: Gap → Constitution Mapping (Iteration 16)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-15-1 | Custom tool discovery format | Amend P §P.1 | ✅ VERIFIED FIXED |
| P0-15-2 | Custom tools not registered | Art III §3.4 | ✅ VERIFIED FIXED |
| P0-15-3 | Plugin tool registration | Art III §3.5 | ✅ VERIFIED FIXED |
| P1-15-4 | Non-deterministic hook execution | Art III §3.6 | ⚠️ Impl fixed, test missing |
| **P1-NEW-1** | Duplicate `directory_scanner.rs` | **Art III §3.7 (NEW)** | **NEW** |
| **P1-NEW-2** | Two `ToolRegistry` implementations | **Art III §3.8 (NEW)** | **NEW** |
| **P1-NEW-3** | ACP E2E connection test missing | **Art IV §4.1 (NEW)** | **NEW** |
| P2-NEW-1 | Route-group tests missing | Art IV §4.2 (NEW) | NEW |
| P2-NEW-2 | Malformed request tests missing | Art IV §4.3 (NEW) | NEW |
| P2-NEW-3 | Hook determinism test missing | Art IV §4.4 (NEW) | NEW |
| P2-NEW-4 | Security tests missing | Art IV §4.3 (NEW) | NEW |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin, MCP/LSP, Server API |
| v2.1 | Iteration 5 | I–VII + A–D | Build gate, JSONC errors, slash commands |
| v2.2 | Iteration 6 | I–VII + A–F | Test code quality, ACP verification |
| v2.3 | Iteration 7 | I–VII + A–I | Desktop WebView, test enforcement |
| v2.4 | Iteration 8 | I–VII + A–L | Clippy hard gate, CLI tests |
| v2.5 | Iteration 9 | I–VII + A–N | Extended clippy coverage |
| v2.6 | Iteration 10 | I–VII + A–O | Clippy enforcement mechanism |
| v2.7 | Iteration 11 | I–VII + A–O | No changes (adequate) |
| v2.8 | Iteration 15 | I–VII + A–P | Custom tool discovery/registration (P), Hook determinism (3.6) |
| **v2.9** | **Iteration 16** | **I–VII + A–Q** | **Code deduplication (3.7), Registry consolidation (3.8), E2E tests (4.1-4.4)** |

---

## Priority Summary for Iteration 16

| Priority | Item | Action Required | Constitutional Reference |
|----------|------|-----------------|-------------------------|
| **P1** | Duplicate `directory_scanner.rs` | Delete `core/src/config/directory_scanner.rs` | Art III §3.7 |
| **P1** | Two `ToolRegistry` implementations | Audit and document or consolidate | Art III §3.8 |
| **P1** | ACP E2E integration test | Add integration test for ACP transport | Art IV §4.1 |
| **P2** | Route-group tests | Add MCP, config, provider route tests | Art IV §4.2 |
| **P2** | API negative tests | Add malformed request, security tests | Art IV §4.3 |
| **P2** | Hook determinism test | Add 100-iteration determinism test | Art IV §4.4 |
| **P3** | Security injection tests | Add SQL injection, path traversal tests | Art IV §4.3 |

**Constitutional additions in Iteration 16:** Amendment Q (Code Quality) + Art III §3.7-3.8 + Art IV §4.1-4.4

---

## Summary

**Overall Completion:** ~80-85% (up from ~65-70% in Iteration 15)

**Constitutional Assessment: PARTIALLY ADEQUATE**

Constitution v2.8 adequately covers the P0 blockers resolved in Iteration 16. However, **3 new P1 issues** and **4 new P2 issues** identified in Iteration 16 are NOT covered by existing constitutional mandates:

**New P1 Mandates Required:**
1. **Amendment Q + Art III §3.7:** Code deduplication — mandate single source of truth for shared code
2. **Art III §3.8:** Registry consolidation — mandate either consolidate or document multiple implementations
3. **Art IV §4.1:** ACP E2E test — mandate integration test for ACP transport layer

**New P2 Mandates Required:**
4. **Art IV §4.2:** Route-group enumeration tests — mandate explicit tests per route group
5. **Art IV §4.3:** API negative tests — mandate malformed request, invalid ID, security tests
6. **Art IV §4.4:** Hook determinism test — mandate explicit test for `sorted_plugin_names()`

**Implementation Status:**
- ✅ All P0 blockers from Iteration 15 resolved
- ✅ All P1 from Iteration 15 resolved (except hook determinism test)
- ⚠️ New P1/P2 issues from Iteration 16 require constitutional mandates

---

*Constitution v2.9 — Iteration 16*
*Total constitutional articles: 7 (original) + 17 amendments (A–Q)*
*P0 blockers constitutionally covered: 3 fixed (P0-15-1, P0-15-2, P0-15-3)*
*New P1 gaps requiring constitutional coverage: 3 (P1-NEW-1, P1-NEW-2, P1-NEW-3)*
*Constitutional verification: Art III §3.2 (fixed since Iter 15), Art III §3.6 (implementation fixed)*
*Report generated: 2026-04-14*
