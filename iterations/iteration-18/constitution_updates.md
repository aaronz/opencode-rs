# Constitution Updates - Iteration 18

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-18/gap-analysis.md`
**Previous Constitution:** `iteration-17/constitution_updates.md` (v2.9)
**Status:** Amendment Proposal

---

## Executive Summary

Iteration 18 gap analysis reveals **ALL P0 blockers are now RESOLVED**:

1. ✅ Config crate has full implementation (1581+ lines) - PRD 19 compliant
2. ✅ Config test infrastructure (PoisonError) fixed
3. ✅ TUI keybinding tests fixed (2 tests)
4. ✅ TUI theme color parsing test fixed

**Implementation is ~90-95% complete** - only Phase 6 (Release Qualification) remains.

**Assessment:** Constitution v2.9 is **MOSTLY ADEQUATE** — no new P0 gaps introduced. Two minor additions recommended for Phase 6 guidance and test infrastructure standards.

---

## Article I: Gap Analysis Summary (Iteration 18)

### P0 Resolution Status (All Cleared)

| Gap ID | Description | Status | Constitutional Reference |
|--------|-------------|--------|------------------------|
| P0-17-1 | Config crate empty re-export | ✅ **FIXED** | Section Q.1 (now implemented) |
| P0-17-2 | Config tests PoisonError | ✅ **FIXED** | Test infrastructure (not constitutional) |
| P0-17-3 | TUI keybinding tests failing | ✅ **FIXED** | Test infrastructure (not constitutional) |
| P0-17-4 | TUI theme color parsing | ✅ **FIXED** | Test infrastructure (not constitutional) |

### Remaining Issues (P1/P2)

| Gap ID | Description | Severity | Constitutional Coverage |
|--------|-------------|----------|------------------------|
| P1-18-1 | Phase 6 Release Qualification not started | P1 | ⚠️ Not explicitly covered |
| P1-18-2 | GitLab CI integration tests failing (7 tests) | P1 | ⚠️ Test infrastructure, needs standard |
| P2-18-1 | Desktop/web smoke test port conflict | P2 | Not constitutional |
| P2-18-2 | Deprecated `mode` field | P2 | Covered by Amendment D §D.2 (deferred) |
| P2-18-3 | Deprecated `tools` field | P2 | Covered by Amendment D §D.2 (deferred) |

---

## Article II: Constitutional Coverage Analysis

### Constitution v2.9 Coverage for Iteration 18 Issues

| Constitution Reference | Mandate | Iteration 18 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified (20+ tests) |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1 | Deterministic hook order | ✅ Verified (priority sorting) |
| Art III §3.2 | Plugin tool registration | ✅ Verified |
| Art III §3.3 | Config ownership boundary | ✅ Verified |
| Art III §3.4 | Custom tool registration | ✅ Verified |
| Art III §3.5 | Plugin tool registration | ✅ Verified |
| Art III §3.6 | Hook execution determinism | ✅ Verified |
| Art IV §4.1 | MCP transport | ✅ Verified |
| Art IV §4.2 | LSP diagnostics pipeline | ✅ Verified |
| Art V §5.1–5.3 | Server API hardening | ✅ Verified |
| Art VI §6.1 | Desktop WebView | ✅ Verified |
| Art VI §6.2 | ACP HTTP+SSE transport | ✅ Verified |
| Amend A §A.1 | Build integrity gate | ✅ Verified |
| Amend J §J.1 | Clippy linting gate | ✅ Verified |
| Amend K §K.1 | CLI test quality gate | ⚠️ 1 port conflict failure |
| Amend O §O.1 | CI Gate Enforcement | ✅ Verified |
| Amend P §P.1 | Custom tool discovery | ✅ Verified |
| Section Q.1 | Crate ownership architecture | ✅ **FIXED** |

---

## Article III: New Constitutional Requirements

### Section R.1 - Phase 6 Release Qualification (PRD 19)

**Issue:** Phase 6 (Release Qualification) has never been started. The Constitution should provide guidance on what release qualification entails.

**Requirement:** Phase 6 Release Qualification MUST include:

1. **End-to-End Integration Tests**
   - Full session lifecycle across all interface modes (CLI, TUI, Desktop, Web)
   - Multi-agent coordination and delegation scenarios
   - Cross-crate integration (storage → agent → tools → server)

2. **Performance Benchmarks**
   - Session creation latency (target: <100ms)
   - Tool execution throughput (target: >100 tools/sec)
   - Memory usage under sustained load
   - Startup time benchmarks

3. **Security Audit**
   - Permission boundary verification
   - Auth token handling review
   - Input validation across all entry points
   - Plugin isolation verification

4. **Observability Validation**
   - Logging subsystem verification
   - Error reporting and diagnostics
   - Metrics collection functionality
   - Tracing integration (if implemented)

**CONSTRAINT:** Phase 6 MUST be completed before v1.0 release. No P0/P1 issues remaining is a necessary but not sufficient condition for release.

**Gap Addressed:**
- "Phase 6 Release Qualification not started"

### Section S.1 - Environment-Dependent Test Standards

**Issue:** 7 GitLab CI integration tests fail without a real GitLab server. These tests are not properly marked as environment-dependent.

**Requirement:** Integration tests that require external services MUST be properly gated:

```rust
// Environment-dependent integration test pattern
#[tokio::test]
#[ignore] // Requires real GitLab server at http://127.0.0.1:63182
async fn test_gitlab_ci_integration() {
    // Test implementation
}

// OR use feature gating for external dependencies
#[tokio::test]
#[cfg_attr(not(feature = "gitlab-integration"), ignore)]
async fn test_gitlab_ci_integration() {
    // Test implementation
}
```

**CONSTRAINT:** All integration tests that require external services MUST:
1. Be marked with `#[ignore]` and documented
2. OR be feature-gated with `#[cfg_attr(feature = "...", ignore)]`
3. OR use a mock implementation for CI environments
4. NEVER fail the default test suite in a clean environment

**Gap Addressed:**
- "GitLab CI integration tests failing (7 tests)"

---

## Article IV: P1/P2 Issue Constitutionality Assessment

### P1 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Phase 6 Release Qualification | ⚠️ Not explicitly covered | Add Section R.1 |
| GitLab CI tests failing | ⚠️ Test infrastructure standard needed | Add Section S.1 |

### P2 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Desktop/web port conflict | Not constitutional | Fix test implementation |
| Deprecated `mode` field | Covered by Amend D §D.2 | Deferred to v4.0 |
| Deprecated `tools` field | Covered by Amend D §D.2 | Deferred to v4.0 |

**Conclusion:** Two P1 issues require constitutional additions for proper guidance. The rest are implementation bugs or already covered.

---

## Article V: Updated Compliance Checklist

### Phase 6 Release Qualification (NEW — Section R.1)
- [ ] End-to-End Integration Tests defined and passing
- [ ] Performance Benchmarks defined and meeting targets
- [ ] Security Audit completed
- [ ] Observability Validation completed
- [ ] No P0/P1 issues remaining before Phase 6 start

### Test Infrastructure Standards (NEW — Section S.1)
- [ ] All environment-dependent tests marked with `#[ignore]`
- [ ] All feature-gated integration tests documented
- [ ] Mock implementations available for CI
- [ ] Default test suite passes in clean environment

### Build Quality Gate (Amendment A + J + M + O)
- [x] `cargo build --all` exits 0
- [x] `cargo test --all --no-run` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [x] No P0 gaps in implementation

### Tools System (Amendment P + Art III §3.4-3.6)
- [x] Custom tool discovery scans `.ts/.js` files
- [x] Discovered custom tools registered with `ToolRegistry`
- [x] Custom tool format follows ES module `export default tool({...})`
- [x] `PluginManager::register_plugin_tools()` implemented
- [x] Plugin tools appear in `ToolRegistry::list_tools()`
- [x] Hook execution sorted by `hook_priority()`
- [x] Tool collision priority enforced: Built-in > Plugin > Custom

### Crate Architecture (Section Q.1)
- [x] `crates/config/` contains real config implementation
- [x] No crate is a pure re-export of another crate's internals
- [x] Cross-crate public API re-exports documented
- [x] PRD 19 crate ownership boundaries respected

---

## Appendix A: Gap → Constitution Mapping (Iteration 18)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-17-1 | Config crate empty re-export | Section Q.1 | ✅ Fixed |
| P0-17-2 | Config tests PoisonError | Test infra | ✅ Fixed |
| P0-17-3 | TUI keybinding tests | Test infra | ✅ Fixed |
| P0-17-4 | TUI theme parsing | Test infra | ✅ Fixed |
| P1-18-1 | Phase 6 not started | Section R.1 | **NEW** |
| P1-18-2 | GitLab CI tests failing | Section S.1 | **NEW** |
| P2-18-1 | Port conflict test | Test infra | Fix implementation |
| P2-18-2 | Deprecated mode field | Amend D §D.2 | Deferred |

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
| v2.9 | Iteration 17 | I–VII + A–Q | Crate ownership architecture (Q) |
| **v3.0** | **Iteration 18** | **I–VII + A–S** | **Phase 6 Release Qualification (R), Test Infrastructure Standards (S)** |

---

## Priority Summary for Iteration 18

| Priority | Item | Action Required |
|----------|------|-----------------|
| **P1** | Phase 6 Release Qualification | Define scope, begin end-to-end testing |
| **P1** | GitLab CI tests | Mark with `#[ignore]` or feature gate |
| P2 | Port conflict test | Use dynamic port allocation |
| P2 | Deprecated fields | Deferred to v4.0 |

**Constitutional additions in Iteration 18:** Section R.1 (Phase 6 Release Qualification) + Section S.1 (Test Infrastructure Standards)

---

## Summary

**Overall Completion:** ~90-95% complete (up from ~85-90% in Iteration 17)

**Constitutional Assessment: MOSTLY ADEQUATE**

The Constitution v2.9 is **adequate** for all resolved P0 blockers. Two new constitutional sections are recommended:

1. **Section R.1:** Phase 6 Release Qualification — defines requirements for final release qualification
2. **Section S.1:** Test Infrastructure Standards — mandates proper marking of environment-dependent tests

**Key Achievements Since Iteration 17:**
- ✅ All P1 issues from Iteration 17 resolved
- ✅ Config crate now has full implementation (1581+ lines)
- ✅ Config test infrastructure fixed
- ✅ TUI tests all passing
- ✅ ~1020 tests passing, only 8 failing (environment-dependent)

**Remaining Work:**
- Define and begin Phase 6 Release Qualification
- Mark environment-dependent tests properly
- Fix desktop/web port conflict test (P2)
- Complete deprecated field removal for v4.0

---

*Constitution v3.0 — Iteration 18*
*Total constitutional articles: 7 (original) + 19 amendments (A–S)*
*P0 blockers constitutionally covered: All resolved*
*New constitutional additions: Section R.1 (Phase 6) + Section S.1 (Test Standards)*
*Report generated: 2026-04-14*
