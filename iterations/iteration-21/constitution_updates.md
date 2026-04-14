# Constitution Updates - Iteration 21

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-21/gap-analysis.md`
**Previous Constitution:** `iteration-20/constitution_updates.md` (v3.1)
**Status:** NO AMENDMENT REQUIRED - Constitutional Enforcement Crisis

---

## Executive Summary

**Implementation is ~93-96% complete** (UNCHANGED from iteration-20 - NO PROGRESS on PRD 20).

**Key Observations Since Iteration-20:**
- **ZERO progress on PRD 20 (ratatui-testing)** - all modules remain as stubs
- Phase 6 (Release Qualification) remains unstarted (blocked by PRD 20)
- Bedrock test pollution remains unfixed (Section S.1 violation persists)

**Assessment:** Constitution v3.1 is **ADEQUATE** but is being **flagrantly violated**. All P0/P1 issues from iteration-21 are identical to iteration-20 - they represent constitutional enforcement failures, not new gaps. The Constitution already mandates what needs to be done; the problem is willful non-compliance.

**Recommendation:** NO constitutional amendment. Instead, this iteration reveals a **constitutional enforcement crisis** requiring leadership intervention. The same P0 blockers identified in iteration-20 remain unimplemented despite explicit constitutional mandate (Section R.1 amendment).

---

## Article I: Gap Analysis Summary (Iteration 21)

### P0 Issues (PRD 20 Implementation) - UNCHANGED FROM ITERATION-20

| Gap ID | Description | Status | Constitutional Reference |
|--------|-------------|--------|------------------------|
| P0-21-1 | PtySimulator stub implementation | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-21-2 | BufferDiff stub implementation | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-21-3 | StateTester stub implementation | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-21-4 | TestDsl stub implementation | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-21-5 | CliTester stub implementation | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-21-6 | Empty ratatui-testing tests/ | ❌ NOT STARTED | Section R.1 (explicit) |

### P1/P2 Issues

| Gap ID | Description | Severity | Constitutional Coverage |
|--------|-------------|----------|------------------------|
| P1-21-1 | Phase 6 Release Qualification not started | P1 | ✅ Section R.1 |
| P1-21-2 | Bedrock test environment pollution | P1 | ✅ Section S.1 |
| P2-21-1 | Trailing whitespace | P2 | Not constitutional |
| P2-21-2 | TestHarness unused methods | P2 | Not constitutional |
| P2-21-3 | Deprecated fields | P2 | ✅ Amend D §D.2 |

---

## Article II: Constitutional Coverage Analysis

### Constitution v3.1 Coverage for Iteration 21 Issues

| Constitution Reference | Mandate | Iteration 21 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1–3.6 | Tools/Plugin/Hooks | ✅ Verified |
| Art IV §4.1–4.2 | MCP/LSP | ✅ Verified |
| Art V §5.1–5.3 | Server API | ✅ Verified |
| Art VI §6.1–6.2 | Desktop/ACP | ✅ Verified |
| Amend A–Q | Various | ✅ Verified |
| **Section R.1** | Phase 6 Release Qualification | ❌ **BLOCKED BY PRD 20** |
| **Section R.1 (Amended)** | TUI testing infrastructure (PRD 20) | ❌ **NOT IMPLEMENTED - CONSTITUTIONAL VIOLATION** |
| **Section S.1** | Test Infrastructure Standards | ❌ **VIOLATION PERSISTS** |

### Iteration-over-Iteration Comparison

| Issue | Iteration 20 Status | Iteration 21 Status | Change |
|-------|---------------------|---------------------|--------|
| PtySimulator | Stub | Stub | **NO PROGRESS** |
| BufferDiff | Stub | Stub | **NO PROGRESS** |
| StateTester | Stub | Stub | **NO PROGRESS** |
| TestDsl | Stub | Stub | **NO PROGRESS** |
| CliTester | Stub | Stub | **NO PROGRESS** |
| tests/ directory | Empty | Empty | **NO PROGRESS** |
| Phase 6 | Not started | Not started | **NO PROGRESS** |
| Bedrock test | Violation | Violation | **NO PROGRESS** |

---

## Article III: Constitutional Enforcement Crisis

### Crisis Declaration

**This iteration reveals a constitutional enforcement crisis:**

1. **Iteration 20** identified PRD 20 (ratatui-testing) as the primary P0 blocker
2. **Constitution v3.1** (iteration-20) was amended to EXPLICITLY mandate PRD 20 as Section R.1 prerequisite
3. **Iteration 21** shows ZERO progress on PRD 20 - all modules remain stubs
4. **Same 6 P0 issues** identified in iteration-20 remain completely unimplemented

### Constitutional Mandates Already in Place

**Section R.1 (Amended in v3.1) explicitly states:**
```rust
CONSTRAINT: Phase 6 MUST be completed before v1.0 release.
Phase 6 Release Qualification MUST include:
1. End-to-End Integration Tests
   - **TUI Testing Infrastructure:** Before TUI end-to-end tests can run, PRD 20
     (ratatui-testing) MUST be fully implemented with:
     * PtySimulator for PTY master/slave terminal simulation
     * BufferDiff for rendering regression detection
     * StateTester for application state verification
     * TestDsl for fluent test composition
     * CliTester for CLI process testing
   - **Prerequisite Chain:** Phase 6 TUI testing requires PRD 20 completion
2. Performance Benchmarks
3. Security Audit
4. Observability Validation
```

**Section S.1 (Test Infrastructure Standards) explicitly states:**
```rust
CONSTRAINT: All tests MUST maintain environment isolation.
Test Infrastructure Standards:
1. Environment-dependent tests MUST be marked with #[ignore]
2. Feature-gated integration tests MUST be documented
3. Mock implementations MUST be available for CI
4. Default test suite MUST pass in clean environment
```

### Enforcement Failure Evidence

| Mandate | Constitutional Text | Implementation Status | Violation |
|---------|---------------------|---------------------|-----------|
| PRD 20 (ratatui-testing) | Section R.1 amended | All 5 modules + tests/ = stubs | **YES - 7 iterations behind** |
| Phase 6 Release Qualification | Section R.1 | Not started (blocked by PRD 20) | **YES** |
| Test environment isolation | Section S.1 | Bedrock test still polluting | **YES - 2 iterations behind** |

---

## Article IV: P1/P2 Issue Constitutionality Assessment

### P0 Issues - CONSTITUTIONAL ENFORCEMENT FAILURE

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| PtySimulator stub | ✅ Section R.1 (explicit) | **ENFORCE EXISTING MANDATE** |
| BufferDiff stub | ✅ Section R.1 (explicit) | **ENFORCE EXISTING MANDATE** |
| StateTester stub | ✅ Section R.1 (explicit) | **ENFORCE EXISTING MANDATE** |
| TestDsl stub | ✅ Section R.1 (explicit) | **ENFORCE EXISTING MANDATE** |
| CliTester stub | ✅ Section R.1 (explicit) | **ENFORCE EXISTING MANDATE** |
| Empty tests/ dir | ✅ Section R.1 (explicit) | **ENFORCE EXISTING MANDATE** |

**Conclusion:** NO new constitutional amendment required. These are enforcement failures of an explicit constitutional mandate that was AMENDED in iteration-20 specifically to address these issues.

### P1 Issues - CONSTITUTIONAL ENFORCEMENT FAILURE

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Phase 6 not started | ✅ Section R.1 | **ENFORCE EXISTING MANDATE** |
| Bedrock test pollution | ✅ Section S.1 | **ENFORCE EXISTING MANDATE** |

**Conclusion:** P1 issues are enforcement failures of existing constitutional provisions.

### P2 Issues - Implementation Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Trailing whitespace | Not constitutional | Run `cargo fmt` |
| TestHarness unused methods | Not constitutional | Clean up dead code |
| Deprecated fields | ✅ Amend D §D.2 | Deferred to v4.0 |

**Conclusion:** P2 issues are implementation/code quality issues, not constitutional gaps.

---

## Article V: Constitutional Crisis Summary

### Critical Finding: Willful Non-Compliance

**The Constitution v3.1 was amended in iteration-20 specifically to address the PRD 20 issue. Despite this explicit mandate, iteration-21 shows ZERO progress.**

### Root Cause Analysis

1. **Iteration 18:** Section R.1 added (Phase 6 Release Qualification mandate)
2. **Iteration 19:** No new constitutional gaps identified - enforcement required
3. **Iteration 20:** Section R.1 AMENDED to explicitly require PRD 20 (ratatui-testing)
4. **Iteration 21:** Same P0 blockers remain - NO CONSTITUTIONAL GAP, ENFORCEMENT FAILURE

### Constitutional Integrity Assessment

| Metric | Status |
|--------|--------|
| Constitutional adequacy | ✅ ADEQUATE |
| Explicit PRD 20 mandate | ✅ PRESENT (Section R.1 amended v3.1) |
| Enforcement compliance | ❌ **FAILING** (7 iterations behind on PRD 20) |
| Phase 6 progress | ❌ **BLOCKED** |
| Test isolation compliance | ❌ **FAILING** (2 iterations behind on S.1) |

---

## Article VI: Required Actions (Implementation, Not Constitutional)

### Immediate Actions (P0 - Constitutional Enforcement)

1. **Implement PtySimulator** - Constitutional mandate, Section R.1
   - Add `portable-pty` dependency
   - PTY master/slave creation on Unix
   - `write_input()`, `read_output()` with timeout
   - `resize()`, `inject_key_event()`, `inject_mouse_event()`

2. **Implement BufferDiff** - Constitutional mandate, Section R.1
   - Cell-by-cell comparison using `ratatui::Buffer`
   - `DiffResult` and `CellDiff` structs
   - Color/attribute ignore options

3. **Implement StateTester** - Constitutional mandate, Section R.1
   - `capture()` method for JSON serialization
   - `assert_state()` and `assert_state_matches()`

4. **Implement TestDsl** - Constitutional mandate, Section R.1
   - Compose PtySimulator, BufferDiff, StateTester
   - Fluent API with `send_keys()`, `wait_for()`

5. **Implement CliTester** - Constitutional mandate, Section R.1
   - Process spawning with `assert_cmd`
   - stdout/stderr capture, exit code

6. **Add Integration Tests** - Constitutional mandate, Section R.1
   - `tests/pty_tests.rs`, `tests/buffer_diff_tests.rs`, etc.
   - `cargo test --all-features -p ratatui-testing` must pass

### Medium-term Actions (P1 - Constitutional Enforcement)

7. **Begin Phase 6 Release Qualification** - After PRD 20 complete
   - Section R.1 mandates completion before v1.0

8. **Fix Bedrock Test Environment Pollution** - Section S.1 violation
   - Use `temp_env::set_var` for environment variable isolation

### Short-term Actions (P2)

9. **Run `cargo fmt --all`** to fix trailing whitespace

10. **Clean up TestHarness dead code** in `crates/cli/tests/common.rs`

11. **Run `cargo fix --tests --all`** to fix clippy warnings

---

## Appendix A: Gap → Constitution Mapping (Iteration 21)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-21-1 | PtySimulator stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-21-2 | BufferDiff stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-21-3 | StateTester stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-21-4 | TestDsl stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-21-5 | CliTester stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-21-6 | Empty tests/ dir | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P1-21-1 | Phase 6 not started | Section R.1 | **CONSTITUTIONAL VIOLATION** |
| P1-21-2 | Bedrock test pollution | Section S.1 | **VIOLATION** |
| P2-21-1 | trailing whitespace | Not constitutional | Fix with fmt |
| P2-21-2 | TestHarness dead code | Not constitutional | Clean up |
| P2-21-3 | deprecated fields | Amend D §D.2 | Deferred |

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
| v3.0 | Iteration 18 | I–VII + A–S | Phase 6 Release Qualification (R), Test Infrastructure Standards (S) |
| v3.1 | Iteration 20 | I–VII + A–S | Section R.1 amendment: TUI testing infrastructure (PRD 20) explicit prerequisite |
| **v3.1** | **Iteration 21** | **I–VII + A–S** | **NO AMENDMENT - Enforcement Crisis Declaration** |

---

## Priority Summary for Iteration 21

| Priority | Item | Action Required | Constitutional Status |
|----------|------|----------------|----------------------|
| **P0** | PRD 20 (ratatui-testing) | Full implementation | **MANDATED BY SECTION R.1** |
| **P1** | Phase 6 Release Qualification | Cannot start until PRD 20 | **MANDATED BY SECTION R.1** |
| **P1** | Bedrock test pollution | Implement Section S.1 | **VIOLATION OF SECTION S.1** |
| P2 | Trailing whitespace | Run `cargo fmt` | Not constitutional |
| P2 | Deprecated fields | Deferred to v4.0 | Covered by Amend D |

**Constitutional additions in Iteration 21:** NONE - NO NEW CONSTITUTIONAL GAPS IDENTIFIED

---

## Summary

**Overall Completion:** ~93-96% complete (UNCHANGED - NO PROGRESS on PRD 20)

**Constitutional Assessment: ADEQUATE BUT VIOLATED**

The Constitution v3.1 is **constitutionally adequate** for iteration-21 issues. NO new constitutional amendments are required because:

1. **Section R.1 (Amended in v3.1)** already explicitly mandates PRD 20 (ratatui-testing) as Phase 6 prerequisite
2. **Section S.1** already mandates test environment isolation
3. All iteration-21 P0/P1 issues are identical to iteration-20 issues

**CRITICAL FINDING: CONSTITUTIONAL ENFORCEMENT CRISIS**

The real problem is not constitutional inadequacy - it is **willful non-compliance with existing constitutional mandates**:

- Section R.1 was amended in iteration-20 specifically to address the PRD 20 issue
- Despite this explicit mandate, iteration-21 shows ZERO progress on PRD 20
- All 6 P0 blockers from iteration-20 remain unimplemented
- Phase 6 remains blocked by the same issue from 7 iterations ago

**Required Actions:**
1. **CONSTITUTIONAL ENFORCEMENT:** Implement PRD 20 (ratatui-testing) - 5 modules + tests/ currently stubs
2. **CONSTITUTIONAL ENFORCEMENT:** Fix Bedrock test environment isolation (Section S.1 violation)
3. **BEGIN PHASE 6:** After PRD 20 complete, Section R.1 mandates Phase 6 completion before v1.0

---

*Constitution v3.1 — Iteration 21*
*Total constitutional articles: 7 (original) + 19 amendments (A–S)*
*P0 blockers constitutionally covered: ALL (explicit mandate in Section R.1)*
*New constitutional addition: NONE - Enforcement crisis, not constitutional gap*
*Report generated: 2026-04-14*