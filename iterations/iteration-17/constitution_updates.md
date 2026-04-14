# Constitution Updates - Iteration 17

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-17/gap-analysis.md`
**Previous Constitution:** `iteration-15/constitution_updates.md` (v2.8)
**Status:** Amendment Proposal

---

## Executive Summary

Iteration 17 gap analysis reveals that **all P0 blockers from Iteration 15 have been RESOLVED**:

1. ✅ Custom tool discovery now correctly scans `.ts/.js` files
2. ✅ Custom tools are properly registered with ToolRegistry
3. ✅ Plugin tool registration fully implemented via ToolProvider integration
4. ✅ Hook execution now deterministic with priority ordering
5. ✅ Plugin config ownership enforced

**Remaining issues are P1/P2 level** — primarily:
- Config crate empty re-export violates PRD 19 crate ownership architecture
- Test infrastructure issues (PoisonError in parallel tests)
- TUI test failures (keybinding, theme parsing)

**Assessment:** Constitution v2.8 is **MOSTLY ADEQUATE** — P0s are resolved. One new constitutional addition recommended for crate ownership architecture.

---

## Article I: Gap Analysis Summary (Iteration 17)

### P0 Resolution Status

| Gap ID | Description | Status | Constitutional Reference |
|--------|-------------|--------|------------------------|
| P0-15-1 | Custom tool discovery format | ✅ **FIXED** | Amend P §P.1 |
| P0-15-2 | Custom tools not registered | ✅ **FIXED** | Art III §3.4 |
| P0-15-3 | Plugin tool registration | ✅ **FIXED** | Art III §3.5 |
| P0-15-4 | Hook execution non-deterministic | ✅ **FIXED** | Art III §3.6 |

### Remaining Issues (P1/P2)

| Gap ID | Description | Severity | Constitutional Coverage |
|--------|-------------|----------|------------------------|
| P1-17-1 | Config crate empty re-export (violates PRD 19) | P1 | ❌ Not covered |
| P1-17-2 | Config tests failing with PoisonError | P1 | ⚠️ Test infrastructure, not constitutional |
| P2-17-1 | TUI keybinding tests failing (2 tests) | P2 | Not constitutional |
| P2-17-2 | TUI theme color parsing test failing | P2 | Not constitutional |
| P2-17-3 | Desktop/web smoke test port conflict | P2 | Not constitutional |

---

## Article II: Constitutional Coverage Analysis

### Constitution v2.8 Coverage for Iteration 17 Issues

| Constitution Reference | Mandate | Iteration 17 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified (20+ tests) |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1 | Deterministic hook order | ✅ **FIXED** (priority sorting) |
| Art III §3.2 | Plugin tool registration | ✅ **FIXED** (ToolProvider integrated) |
| Art III §3.3 | Config ownership boundary | ✅ Verified |
| Art III §3.4 | Custom tool registration | ✅ **FIXED** (.ts/.js scanning) |
| Art III §3.5 | Plugin tool registration | ✅ **FIXED** (registry integration) |
| Art III §3.6 | Hook execution determinism | ✅ **FIXED** (priority sorting) |
| Art IV §4.1 | MCP transport | ✅ Verified |
| Art IV §4.2 | LSP diagnostics pipeline | ✅ Verified |
| Art V §5.1–5.3 | Server API hardening | ✅ Verified |
| Art VI §6.1 | Desktop WebView | ✅ Verified |
| Art VI §6.2 | ACP HTTP+SSE transport | ✅ Verified |
| Amend A §A.1 | Build integrity gate | ✅ Verified |
| Amend J §J.1 | Clippy linting gate | ✅ Verified |
| Amend K §K.1 | CLI test quality gate | ⚠️ 1 port conflict failure |
| Amend O §O.1 | CI Gate Enforcement | ✅ Verified |
| Amend P §P.1 | Custom tool discovery | ✅ **FIXED** (.ts/.js format) |

---

## Article III: New Constitutional Requirement

### Section Q.1 - Crate Ownership Architecture (PRD 19)

**Issue:** `crates/config/src/lib.rs` is an empty re-export of `opencode_core::config::Config`. This violates PRD 19's intended crate ownership architecture.

**Requirement:** Each crate MUST contain real implementation code, not just re-exports from other crates.

```
# Current (VIOLATES PRD 19):
crates/config/src/lib.rs:
    pub use opencode_core::config::Config;

# Required (PRD 19 compliant):
crates/config/src/lib.rs:
    pub mod config;  // Real implementation lives here
    pub use config::Config;
```

**CONSTRAINT:** Crate ownership boundaries MUST be enforced:
1. `crates/core/` — Core entities, session, message, part models
2. `crates/config/` — Configuration parsing, precedence, variable expansion
3. `crates/storage/` — Persistence, snapshots, recovery
4. `crates/agent/` — Agent runtime, delegation
5. `crates/tools/` — Tool registry, discovery, execution
6. `crates/plugin/` — Plugin system, hooks
7. `crates/tui/` — Terminal UI components
8. `crates/server/` — HTTP API server
9. `crates/mcp/` — MCP client and transport
10. `crates/lsp/` — LSP client and diagnostics
11. `crates/llm/` — LLM providers, model selection
12. `crates/git/` — GitHub/GitLab integration
13. `crates/cli/` — CLI commands
14. `crates/control-plane/` — ACP stream, enterprise features

**CONSTRAINT:** No crate shall be a pure re-export of another crate's internals. Cross-crate public API re-exports are allowed; internal re-exports are not.

**P1 Gap Addressed:**
- "Config crate is empty re-export (violates PRD 19 crate ownership)"

---

## Article IV: P1/P2 Issue Constitutionality Assessment

### P1 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Config crate empty re-export | ❌ Not covered | Add Section Q.1 |
| Config tests PoisonError | ⚠️ Test infra (not constitutional) | Fix test implementation |

### P2 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| TUI keybinding tests failing | Not constitutional | Fix test implementation |
| TUI theme parsing test failing | Not constitutional | Fix test implementation |
| Desktop/web port conflict | Not constitutional | Fix test implementation |

**Conclusion:** Only one P1 issue requires constitutional coverage. The rest are implementation bugs, not design gaps.

---

## Article V: Updated Compliance Checklist

### Crate Architecture (NEW — Section Q.1)

- [ ] `crates/config/` contains real config implementation
- [ ] No crate is a pure re-export of another crate's internals
- [ ] Cross-crate public API re-exports documented
- [ ] PRD 19 crate ownership boundaries respected

### Build Quality Gate (Amendment A + J + M + O)
- [x] `cargo build --all` exits 0
- [x] `cargo test --all --no-run` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [x] No P0 gaps in tools system

### Tools System (Amendment P + Art III §3.4-3.6)
- [x] Custom tool discovery scans `.ts/.js` files
- [x] Discovered custom tools registered with `ToolRegistry`
- [x] Custom tool format follows ES module `export default tool({...})`
- [x] `PluginManager::register_plugin_tools()` implemented
- [x] Plugin tools appear in `ToolRegistry::list_tools()`
- [x] Hook execution sorted by `hook_priority()` (not just IndexMap)
- [x] Tool collision priority enforced: Built-in > Plugin > Custom

---

## Appendix A: Gap → Constitution Mapping (Iteration 17)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-15-1 | Custom tool discovery format | Amend P §P.1 | ✅ Fixed |
| P0-15-2 | Custom tools not registered | Art III §3.4 | ✅ Fixed |
| P0-15-3 | Plugin tool registration | Art III §3.5 | ✅ Fixed |
| P0-15-4 | Hook execution non-deterministic | Art III §3.6 | ✅ Fixed |
| P1-17-1 | Config crate empty re-export | Section Q.1 | **NEW** |
| P1-17-2 | Config tests PoisonError | Test infra | Fix implementation |
| P2-17-1 | TUI test failures | Test infra | Fix implementation |
| P2-17-3 | Port conflict test | Test infra | Fix implementation |

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
| **v2.9** | **Iteration 17** | **I–VII + A–Q** | **Crate ownership architecture (Q)** |

---

## Priority Summary for Iteration 17

| Priority | Item | Action Required |
|----------|------|-----------------|
| **P1** | Config crate empty re-export | Move config logic to `crates/config/` |
| P1 | Config tests PoisonError | Refactor ENV_LOCK to async Mutex |
| P2 | TUI test failures | Fix keybinding and theme tests |
| P2 | Port conflict test | Use dynamic port allocation |

**Constitutional additions in Iteration 17:** Section Q.1 (Crate Ownership Architecture)

---

## Summary

**Overall Completion:** ~85-90% complete (up from ~65-70% in Iteration 15)

**Constitutional Assessment: MOSTLY ADEQUATE**

The Constitution v2.8 is **adequate** for all P0 blockers which have now been resolved. One new constitutional section is recommended:

1. **Section Q.1:** Crate Ownership Architecture — mandates real implementation in each crate per PRD 19

**Key Achievements Since Iteration 15:**
- ✅ All 3 P0 blockers from Iteration 15 resolved
- ✅ Custom tool discovery/registration fully implemented
- ✅ Plugin tool registration integrated with ToolRegistry
- ✅ Hook execution made deterministic with priority sorting
- ✅ 610 tests passing, 14 failing (down from more failures)

**Remaining Work:**
- Move config logic from core to `crates/config/` crate
- Fix test infrastructure issues (PoisonError)
- Fix 3 TUI test failures
- Begin Phase 6 Release Qualification when ready

---

*Constitution v2.9 — Iteration 17*
*Total constitutional articles: 7 (original) + 17 amendments (A–Q)*
*P0 blockers constitutionally covered: 4 (all resolved)*
*New constitutional addition: Section Q.1 (Crate Ownership Architecture)*
*Report generated: 2026-04-14*
