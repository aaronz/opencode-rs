# Constitution Updates - Iteration 17

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-17/gap-analysis.md`
**Previous Constitution:** `iteration-15/constitution_updates.md` (v2.8)
**Status:** No Constitutional Amendments Required

---

## Executive Summary

Iteration 17 gap analysis confirms **all P0 blocking issues remain resolved** and no new P0 gaps have emerged. The Constitution v2.8 (from Iteration 15) remains **ADEQUATE** for the current implementation state.

**Key Findings:**
- ✅ All 3 P0 issues from Iteration 15 are RESOLVED (P0-1, P0-2, P0-3)
- ⚠️ P1 items (config crate, Desktop/Web/ACP) are structural issues, not constitutional gaps
- ⚠️ P2 items (TUI tests) are test coverage issues, not constitutional gaps
- No new P0 blockers identified

**Assessment:** Constitution v2.8 is **ADEQUATE**. No new amendments required.

---

## Article I: Gap Analysis Summary (Iteration 17)

### P0 Status (Previously Blocked - All Resolved)

| Gap ID | Description | Module | Status | Evidence |
|--------|-------------|--------|--------|----------|
| P0-1 | Custom tool discovery scans TOOL.md instead of .ts/.js | tools | ✅ RESOLVED | `crates/tools/src/discovery.rs:104-108` |
| P0-2 | Custom tools not registered with ToolRegistry | tools | ✅ RESOLVED | `crates/tools/src/discovery.rs:230-254` |
| P0-3 | Plugin tool registration missing | plugin | ✅ RESOLVED | `crates/plugin/src/lib.rs:264-272` |

### P1/P2 Items (Not P0 Blockers)

| Item | Description | Module | Status | Constitutional Coverage |
|------|-------------|--------|--------|-------------------------|
| P1-1 | Config crate empty re-export | config | ❌ Open | ⚠️ Not a constitutional issue (crate organization) |
| P1-2 | Desktop app not fully qualified | cli | ⚠️ Partial | Already covered in Art VI |
| P1-3 | Web server mode not fully qualified | cli | ⚠️ Partial | Already covered in Art VI |
| P1-4 | ACP transport not fully qualified | control-plane | ⚠️ Partial | Already covered in Art VI §6.2 |
| P2-1 | TUI slash command tests missing | tui | ⚠️ Missing | Already covered in Art VII checklist |
| P2-2 | TUI input model tests missing | tui | ⚠️ Missing | Already covered in Art VII checklist |
| P2-3 | TUI sidebar tests missing | tui | ⚠️ Missing | Already covered in Art VII checklist |
| P2-4 | Per-agent model override untested | llm | ⚠️ Untested | Not constitutionally required |
| P2-5 | Hidden vs visible agent UI untested | agent | ⚠️ Untested | Not constitutionally required |

---

## Article II: Constitutional Coverage Analysis

### Constitution v2.8 Coverage for Iteration 17

| Constitution Reference | Mandate | Iteration 17 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1 | Deterministic hook order | ✅ RESOLVED (priority-based) |
| Art III §3.2 | Plugin tool registration | ✅ RESOLVED (now integrated) |
| Art III §3.3 | Config ownership boundary | ✅ Verified |
| Art III §3.4 | Custom tool discovery | ✅ RESOLVED |
| Art III §3.5 | Plugin tool registration integration | ✅ RESOLVED |
| Art III §3.6 | Hook execution determinism | ✅ RESOLVED |
| Art IV §4.1 | MCP transport | ✅ Verified |
| Art IV §4.2 | LSP diagnostics pipeline | ✅ Verified |
| Art V §5.1–5.3 | Server API hardening | ✅ Verified |
| Art VI §6.1 | Desktop WebView | ⚠️ Partial (structural, not constitutional) |
| Art VI §6.2 | ACP HTTP+SSE transport | ⚠️ Partial (structural, not constitutional) |
| Amend P §P.1 | Custom tool discovery | ✅ Verified |
| Amend J §J.1 | Clippy linting gate | ✅ Verified |
| Amend O §O.1 | CI Gate Enforcement | ✅ Verified |

---

## Article III: P1 Analysis - Config Crate Issue

### Issue: P1-1 - Empty Config Crate

**Description:** `crates/config/src/lib.rs` only contains `pub use opencode_core::config::Config;` - it should contain actual config logic per PRD 19.

**Constitutional Status:** ⚠️ NOT a constitutional gap

**Rationale:**
- Constitution Article III §3.3 covers **config ownership boundaries** (opencode.json vs tui.json)
- Constitution does NOT mandate specific crate structure
- Crate organization is an implementation detail governed by Rust module design principles
- PRD 19 may specify crate intentions, but PRD compliance ≠ constitutional requirement

**Recommendation:** Track as TD-001 (Technical Debt), not as constitutional violation.

---

## Article IV: P2 Analysis - Test Coverage

### Issue: P2-1 through P2-5 - Missing Tests

**Description:** Various test coverage gaps identified (TUI components, per-agent model override, hidden agent UI).

**Constitutional Status:** ⚠️ NOT constitutional gaps

**Rationale:**
- Constitution Article VII provides a **compliance checklist** for tests
- The checklist is a guide, not a mandate that every item must be implemented before constitution is "adequate"
- Missing tests do not block release - they indicate technical debt
- Tests can be added incrementally without constitutional amendment

**Recommendation:** Track as P2 technical debt items, not constitutional violations.

---

## Article V: Updated Compliance Checklist

### Tools System (Amendment P + Art III §3.4-3.6)

- [x] Custom tool discovery scans `.ts/.js` files, not `TOOL.md`
- [x] Discovered custom tools registered with `ToolRegistry`
- [x] Custom tool format follows ES module `export default tool({...})`
- [x] `PluginManager::connect_to_registry()` implemented
- [x] `PluginManager::register_plugin_tools()` implemented
- [x] Plugin tools appear in `ToolRegistry::list_tools()`
- [x] Hook execution sorted by `hook_priority()` (not just IndexMap)
- [x] Tool collision priority enforced: Built-in > Plugin > Custom

### Agent System (Art II §2.1-2.3)

- [x] Primary agent invariant tested
- [x] Subagent context isolation verified
- [x] Task tool payload schema validated
- [x] Permission inheritance test coverage
- [ ] Hidden vs visible agent UI behavior tested (P2 - not blocking)

### TUI System (Art IX - from iteration to be added)

- [ ] Slash command execution tests (P2)
- [ ] Input model tests - multiline, history, autocomplete (P2)
- [ ] Sidebar visibility and content tests (P2)

### Desktop/Web/ACP (Art VI §6.1-6.2)

- [ ] Desktop e2e smoke tests (P1)
- [ ] Session sharing tests (P1)
- [ ] Web mode tests with auth flow (P1)
- [ ] ACP handshake/connect/ack end-to-end test (P1)

---

## Appendix A: Gap → Constitution Mapping (Iteration 17)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-15-1 | Custom tool discovery format | Amend P §P.1 | ✅ Verified |
| P0-15-2 | Custom tools not registered | Art III §3.4 | ✅ Verified |
| P0-15-3 | Plugin tool registration | Art III §3.5 | ✅ Verified |
| P0-15-4 | Hook execution non-deterministic | Art III §3.6 | ✅ Verified |
| P1-1 | Config crate empty re-export | N/A (structural) | ⚠️ Technical Debt |
| P1-2 | Desktop app not qualified | Art VI §6.1 | ⚠️ In Progress |
| P1-3 | Web mode not qualified | Art VI | ⚠️ In Progress |
| P1-4 | ACP not qualified | Art VI §6.2 | ⚠️ In Progress |
| P2-1 | TUI slash command tests | Art VII checklist | ⚠️ Missing |
| P2-2 | TUI input model tests | Art VII checklist | ⚠️ Missing |
| P2-3 | TUI sidebar tests | Art VII checklist | ⚠️ Missing |
| P2-4 | Per-agent model override | N/A | ⚠️ Untested |
| P2-5 | Hidden agent UI | N/A | ⚠️ Untested |

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
| **v2.8** | **Iteration 17** | **I–VII + A–P** | **No changes (still adequate)** |

---

## Appendix C: P0 Resolution Timeline

| Iteration | P0 Issues | Resolution |
|-----------|-----------|------------|
| Iteration 15 | 3 P0 blockers identified | Constitution v2.8 created |
| Iteration 16 | All 3 P0s in progress | Hook determinism fixed |
| Iteration 17 | All 3 P0s RESOLVED | No constitutional changes needed |

---

## Summary

**Overall Completion:** ~75-80%

**Constitutional Assessment: ADEQUATE (v2.8)**

The Constitution v2.8 remains adequate for the current implementation state:

1. **All P0 issues from Iteration 15 are resolved** - no new constitutional gaps
2. **P1 items are structural/organizational issues** - not constitutional gaps
3. **P2 items are test coverage issues** - not constitutional gaps

**Key Achievements:**
- Custom tool discovery ✅
- Custom tool registration ✅
- Plugin tool registration ✅
- Hook execution determinism ✅

**Remaining Work:**
- Config crate refactoring (P1 - TD-001)
- Desktop/Web/ACP qualification (P1)
- TUI component tests (P2)

**Recommendation:** No constitutional amendments required. Continue tracking P1/P2 items as technical debt.

---

*Constitution v2.8 — Iteration 17*
*Total constitutional articles: 7 (original) + 16 amendments (A–P)*
*P0 blockers constitutionally covered: All resolved*
*Constitutional amendments in Iteration 17: None required*
*Report generated: 2026-04-14*