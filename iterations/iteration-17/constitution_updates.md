# Constitution Updates - Iteration 17

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-17/gap-analysis.md`
**Previous Constitution:** `iteration-16/constitution_updates.md` (v2.9)
**Status:** Amendment Verification

---

## Executive Summary

Iteration-17 gap analysis shows **significant progress** — ACP E2E tests added (1083 lines), Phase 6 regression tests added (536 lines). However, **2 P1 issues** and **4 P2 issues** from Constitution v2.9 remain **NOT FIXED**.

**Resolved in Iteration-17:**
- ✅ **P1-NEW-1 (ACP E2E Test):** 1083 lines of E2E tests added in `tests/src/acp_e2e_tests.rs`

**Still Pending (from v2.9 mandates):**
- ❌ **Art III §3.7:** Duplicate `directory_scanner.rs` — both files still 832 lines
- ❌ **Art III §3.8:** Two `ToolRegistry` implementations — still separate implementations
- ❌ **Art IV §4.2:** Route-group MCP/config/provider tests — still missing
- ❌ **Art IV §4.3:** API negative tests (malformed requests, security) — still missing
- ❌ **Art IV §4.4:** Hook determinism explicit test — still missing

**Assessment:** Constitution v2.9 is **PARTIALLY ADEQUATE** — one P1 mandate (ACP E2E) was successfully addressed, but the code quality mandates (deduplication, registry consolidation) and test completeness mandates remain unfulfilled.

---

## Article I: Gap Analysis Summary (Iteration 17)

### Iteration-16 → 17 Status Transfer

| Gap ID | Description | Constitution Reference | Iteration 17 Status |
|--------|-------------|------------------------|---------------------|
| P1-NEW-1 | ACP E2E connection test | Art IV §4.1 | ✅ **FIXED** — 1083 lines of E2E tests |
| P1-NEW-2 | Duplicate `directory_scanner.rs` | Art III §3.7 | ❌ **NOT FIXED** — Both files still 832 lines |
| P1-NEW-3 | Two `ToolRegistry` implementations | Art III §3.8 | ❌ **NOT FIXED** — Still two separate implementations |
| P2-NEW-1 | Route-group MCP/config/provider tests | Art IV §4.2 | ❌ **NOT FIXED** |
| P2-NEW-2 | Malformed request body tests | Art IV §4.3 | ❌ **NOT FIXED** |
| P2-NEW-3 | Hook determinism explicit test | Art IV §4.4 | ✅ **FIXED** — 9 tests in plugin/src/lib.rs:3763-3945 |
| P2-NEW-4 | Security tests (injection, traversal) | Art IV §4.3 | ❌ **NOT FIXED** |

### Constitution v2.9 Mandate Verification

| Constitution Reference | Mandate | Status |
|------------------------|---------|--------|
| Art III §3.7 | Code deduplication (DirectoryScanner) | ❌ UNFIXED — Both files still 832 lines |
| Art III §3.8 | Registry consolidation or documentation | ❌ UNFIXED — Two implementations still separate |
| Art IV §4.1 | ACP E2E integration test | ✅ FIXED — `tests/src/acp_e2e_tests.rs` (1083 lines) |
| Art IV §4.2 | Route-group enumeration tests | ❌ UNFIXED — MCP/config/provider still missing |
| Art IV §4.3 | API negative tests | ❌ UNFIXED — Malformed requests, security tests missing |
| Art IV §4.4 | Hook determinism explicit test | ✅ FIXED — 9 tests for sorted_plugin_names() determinism (lines 3763-3945) |

---

## Article II: P1 Issues Requiring Immediate Action

### Issue 1: Duplicate `directory_scanner.rs` — Art III §3.7 UNFIXED

**Gap Detail:**
| File | Lines | Status |
|------|-------|--------|
| `crates/config/src/directory_scanner.rs` | 832 | Active (source of truth) |
| `crates/core/src/config/directory_scanner.rs` | 832 | Dead code (duplicate) |

**Constitutional Mandate (v2.9 Art III §3.7):**
> Duplicate code across crates MUST be eliminated. When identical or near-identical implementations exist, one must be removed with imports redirected.

**Required Action:**
```bash
# 1. Delete duplicate file
rm crates/core/src/config/directory_scanner.rs

# 2. Update crates/core/src/lib.rs to re-export from opencode_config
# Add or update export line:
pub use opencode_config::DirectoryScanner;

# 3. Verify no remaining references
cargo build --all-features && cargo test -p opencode-core
```

**Verification Checklist:**
- [ ] `crates/core/src/config/directory_scanner.rs` deleted
- [ ] `crates/core/src/lib.rs` re-exports `opencode_config::DirectoryScanner`
- [ ] `crates/core/src/config.rs` updated if necessary
- [ ] `cargo build --all-features` succeeds
- [ ] `cargo test -p opencode-core` passes

---

### Issue 2: Two `ToolRegistry` Implementations — Art III §3.8 UNFIXED

**Gap Detail:**
| Location | Lines | Purpose |
|----------|-------|---------|
| `crates/core/src/tool.rs` | 1025 | Simple HashMap-based (used by agent runtime) |
| `crates/tools/src/registry.rs` | 2288 | Full-featured with caching, async, source tracking |

**Constitutional Mandate (v2.9 Art III §3.8):**
> When multiple implementations of the same concept exist, the project MUST either:
> 1. Consolidate to a single implementation, OR
> 2. Document the intentional separation with explicit boundaries

**Required Action — Choose Option A or B:**

**Option A: Consolidate**
```rust
// In crates/core/src/tool.rs:
// Replace ToolRegistry with re-export from opencode_tools
pub use opencode_tools::ToolRegistry;
```

**Option B: Document Separation**
```rust
// In crates/core/src/tool.rs, add documentation:
/// Core tool registry for agent runtime.
/// 
/// **Design Note:** This registry is intentionally lightweight for fast agent 
/// initialization. For full-featured registry with caching, async support, 
/// and source tracking, use `opencode_tools::ToolRegistry`.
```

**Verification Checklist:**
- [ ] Decision made: consolidate (A) or document (B)
- [ ] If Option A: `core::ToolRegistry` re-exports from `opencode_tools`
- [ ] If Option B: `crates/core/src/tool.rs` contains separation documentation
- [ ] All usages in `crates/agent/src/runtime.rs` verified compatible
- [ ] `cargo build --all-features` succeeds

---

## Article III: P2 Issues Requiring Test Coverage

### Issue 3: Route-Group Enumeration Tests — Art IV §4.2 UNFIXED

**Gap Detail:**
| Route Group | Test Coverage | Status |
|-------------|--------------|--------|
| Session routes | ✅ Done | `server_integration_tests.rs:840-1158` |
| Permission routes | ✅ Done | `server_integration_tests.rs:67-130` |
| Auth middleware | ✅ Done | `server_integration_tests.rs:123-183, 1186-1285` |
| MCP routes | ❌ Missing | No explicit MCP route group tests |
| Config routes | ❌ Missing | No explicit config route group tests |
| Provider routes | ❌ Missing | No explicit provider route group tests |

**Constitutional Mandate (v2.9 Art IV §4.2):**
> Server integration tests MUST enumerate all routes in each route group and verify: route exists, authentication works, authorization boundaries enforced.

**Required Tests:**
```rust
// tests/src/server_integration_tests.rs or new file

// MCP Route Group
const MCP_ROUTES: &[&str] = &[
    "/api/mcp/servers",
    "/api/mcp/tools", 
    "/api/mcp/execute",
    "/api/mcp/connect",
    "/api/mcp/disconnect",
];

// Config Route Group  
const CONFIG_ROUTES: &[&str] = &[
    "/api/config",
    "/api/config/*",
];

// Provider Route Group
const PROVIDER_ROUTES: &[&str] = &[
    "/api/providers",
    "/api/providers/*",
];
```

**Verification Checklist:**
- [ ] MCP route group tests added
- [ ] Config route group tests added
- [ ] Provider route group tests added
- [ ] All routes verified present and functional

---

### Issue 4: API Negative Tests — Art IV §4.3 UNFIXED

**Gap Detail:**
| Test Type | Status | Evidence |
|-----------|--------|----------|
| Unauthorized access (missing token) | ✅ Done | `server_integration_tests.rs:123-130` |
| Invalid auth token | ✅ Done | `server_integration_tests.rs:138-164` |
| Empty auth token | ✅ Done | `server_integration_tests.rs:191-198` |
| Malformed request bodies | ❌ Missing | No tests for invalid JSON, missing required fields |
| Invalid session/message IDs | ❌ Missing | No tests for non-existent session operations |
| SQL injection / path traversal | ❌ Missing | No security-focused negative tests |

**Constitutional Mandate (v2.9 Art IV §4.3):**
> Server API tests MUST include negative test cases: malformed request bodies, invalid resource IDs, security tests.

**Required Tests:**
```rust
// Security and error test requirements:

// 1. Malformed request bodies
test_invalid_json()
test_missing_required_fields()
test_wrong_field_types()
test_oversized_payload()

// 2. Invalid resource IDs  
test_nonexistent_session_operations()
test_invalid_message_ids()
test_invalid_project_ids()

// 3. Security tests
test_sql_injection_prevention()
test_path_traversal_prevention()
test_xss_prevention()
```

---

### Issue 5: Hook Determinism Explicit Test — Art IV §4.4 UNFIXED

**Gap Detail:**
| Requirement | Status | Evidence |
|-------------|--------|----------|
| Deterministic hook execution | ✅ Implemented | `sorted_plugin_names()` with priority sorting |
| Explicit 100-iteration test | ❌ Missing | No test verifying consistent ordering |

**Constitutional Mandate (v2.9 Art IV §4.4):**
> Hook execution determinism MUST have an explicit test verifying consistent ordering across multiple invocations.

**Required Test:**
```rust
// In plugin/src/lib.rs or tests/

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

---

## Article IV: Positive Additions (Not Gaps)

### Phase 6 Regression Tests — NEW ✅

A new test file `tests/src/phase6_regression_tests.rs` (536 lines) was added covering:
- Session-agent integration
- Session checkpoint/revert integration
- Tool registry-agent integration
- MCP protocol session integration

**Status:** This is a positive addition and does not require constitutional mandate changes.

---

## Article V: Updated Compliance Checklist

### Code Quality Gate (Amendment Q + Art III §3.7-3.8)

- [ ] No duplicate source files across crates (>80% similarity = duplicate)
- [ ] `crates/core/src/config/directory_scanner.rs` deleted or re-export only
- [ ] ToolRegistry implementations documented or consolidated
- [ ] **ACP E2E integration test exists and passes** ✅ (Art IV §4.1 FIXED)
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

### Transport Layer (Art IV §4.1)
- [x] ACP E2E integration test exists (1083 lines) ✅ **FIXED IN ITER 17**

---

## Appendix A: Gap → Constitution Mapping (Iteration 17)

| Gap ID | Description | Constitution Reference | Iteration 17 Status |
|--------|-------------|----------------------|--------|
| P0-15-1 | Custom tool discovery format | Amend P §P.1 | ✅ VERIFIED FIXED |
| P0-15-2 | Custom tools not registered | Art III §3.4 | ✅ VERIFIED FIXED |
| P0-15-3 | Plugin tool registration | Art III §3.5 | ✅ VERIFIED FIXED |
| P1-15-4 | Non-deterministic hook execution | Art III §3.6 | ⚠️ Impl fixed, test missing |
| P1-NEW-1 | ACP E2E connection test | Art IV §4.1 | ✅ **FIXED IN ITER 17** |
| **P1-NEW-2** | Duplicate `directory_scanner.rs` | Art III §3.7 | ❌ **NOT FIXED** |
| **P1-NEW-3** | Two `ToolRegistry` implementations | Art III §3.8 | ❌ **NOT FIXED** |
| P2-NEW-1 | Route-group tests missing | Art IV §4.2 | ❌ NOT FIXED |
| P2-NEW-2 | Malformed request tests missing | Art IV §4.3 | ❌ NOT FIXED |
| P2-NEW-3 | Hook determinism test missing | Art IV §4.4 | ✅ FIXED |
| P2-NEW-4 | Security tests missing | Art IV §4.3 | ❌ NOT FIXED |

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
| v2.9 | Iteration 16 | I–VII + A–Q | Code deduplication (3.7), Registry consolidation (3.8), E2E tests (4.1-4.4) |
| **v2.10** | **Iteration 17** | **I–VII + A–Q** | **Verify Art IV §4.1 fixed, Art III §3.7-3.8 still pending** |

---

## Priority Summary for Iteration 17

| Priority | Item | Action Required | Constitutional Reference | Status |
|----------|------|-----------------|-------------------------|--------|
| **P1** | Duplicate `directory_scanner.rs` | Delete `core/src/config/directory_scanner.rs` | Art III §3.7 | ❌ NOT FIXED |
| **P1** | Two `ToolRegistry` implementations | Audit and document or consolidate | Art III §3.8 | ❌ NOT FIXED |
| ~~P1~~ | ~~ACP E2E integration test~~ | ~~Add integration test for ACP transport~~ | ~~Art IV §4.1~~ | ✅ **FIXED** |
| **P2** | Route-group MCP/config/provider tests | Add MCP, config, provider route tests | Art IV §4.2 | ❌ NOT FIXED |
| **P2** | API negative tests | Add malformed request, security tests | Art IV §4.3 | ❌ NOT FIXED |
| **P2** | Hook determinism test | Add 100-iteration determinism test | Art IV §4.4 | ❌ NOT FIXED |

**Constitutional additions in Iteration 17:** None required — v2.9 mandates remain valid and partially unmet.

---

## Summary

**Overall Completion:** ~85-90% (up from ~80-85% in Iteration 16)

**Constitutional Assessment: PARTIALLY ADEQUATE**

Constitution v2.9 is still **PARTIALLY ADEQUATE**. One P1 mandate (ACP E2E test) was successfully resolved, but:

**Remaining Mandates (Unfulfilled):**
1. **Art III §3.7:** Code deduplication — duplicate `directory_scanner.rs` still exists
2. **Art III §3.8:** Registry consolidation — two `ToolRegistry` still separate
3. **Art IV §4.2:** Route-group tests — MCP/config/provider still missing
4. **Art IV §4.3:** API negative tests — malformed requests, security tests missing
5. **Art IV §4.4:** Hook determinism test — explicit test still missing

**Recommendation:** No new constitutional amendments required. The existing mandates in v2.9 are adequate; focus should be on **fulfilling existing commitments** rather than adding new requirements.

---

*Constitution v2.10 — Iteration 17*
*Total constitutional articles: 7 (original) + 17 amendments (A–Q)*
*P1 mandates from v2.9: 1 fixed (ACP E2E), 2 still pending*
*P2 mandates from v2.9: 4 still pending*
*Report generated: 2026-04-14*
