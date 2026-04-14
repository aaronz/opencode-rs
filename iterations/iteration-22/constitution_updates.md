# Constitution Updates - Iteration 22

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-22/gap-analysis.md`
**Previous Constitution:** `iteration-21/constitution_updates.md` (v3.1)
**Status:** NO AMENDMENT REQUIRED - Sustained Constitutional Enforcement Crisis

---

## Executive Summary

**Implementation is ~93-96% complete** (UNCHANGED from iteration-21 and iteration-20 - ZERO PROGRESS on PRD 20).

**Key Observations Since Iteration-21:**
- **ZERO progress on PRD 20 (ratatui-testing)** - all 5 modules remain as stubs
- **ZERO progress on Phase 6** - remains blocked by PRD 20
- **ZERO progress on Bedrock test pollution** - Section S.1 violation persists
- ratatui-testing modules unchanged: pty.rs (23 lines), diff.rs (16 lines), state.rs (21 lines), dsl.rs (17 lines), cli.rs (16 lines)
- ratatui-testing tests/ directory remains **EMPTY**

**Assessment:** Constitution v3.1 is **ADEQUATE** but is being **willfully violated for the 3rd consecutive iteration**. All P0/P1 issues from iterations 20, 21, and 22 are identical - they represent systemic constitutional enforcement failures, not new gaps.

**Recommendation:** NO constitutional amendment. The Constitution already mandates what needs to be done. The problem is persistent non-compliance across 3 iterations (18, 19, 20, 21, 22 all identified same issues).

---

## Article I: Gap Analysis Summary (Iteration 22)

### P0 Issues (PRD 20 Implementation) - UNCHANGED FOR 3 ITERATIONS

| Gap ID | Description | Status | Constitutional Reference |
|--------|-------------|--------|------------------------|
| P0-22-1 | PtySimulator stub (23 lines, no actual PTY) | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-22-2 | BufferDiff stub (16 lines, returns empty) | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-22-3 | StateTester stub (21 lines, no capture) | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-22-4 | TestDsl stub (17 lines, no composition) | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-22-5 | CliTester stub (16 lines, no spawning) | ❌ NOT STARTED | Section R.1 (explicit) |
| P0-22-6 | ratatui-testing tests/ EMPTY | ❌ NOT STARTED | Section R.1 (explicit) |

### P1/P2 Issues

| Gap ID | Description | Severity | Constitutional Coverage |
|--------|-------------|----------|------------------------|
| P1-22-1 | Phase 6 Release Qualification not started | P1 | ✅ Section R.1 |
| P1-22-2 | Bedrock test environment pollution | P1 | ✅ Section S.1 |
| P2-22-1 | Trailing whitespace | P2 | Not constitutional |
| P2-22-2 | TestHarness unused methods | P2 | Not constitutional |

---

## Article II: Constitutional Coverage Analysis

### Constitution v3.1 Coverage for Iteration 22 Issues

| Constitution Reference | Mandate | Iteration 22 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1–3.6 | Tools/Plugin/Hooks | ✅ Verified |
| Art IV §4.1–4.2 | MCP/LSP | ✅ Verified |
| Art V §5.1–5.3 | Server API | ✅ Verified |
| Art VI §6.1–6.2 | Desktop/ACP | ✅ Verified |
| Amend A–Q | Various | ✅ Verified |
| **Section R.1 (Amended v3.1)** | TUI testing infrastructure (PRD 20) | ❌ **ZERO PROGRESS - 3 ITERATIONS BEHIND** |
| **Section R.1** | Phase 6 Release Qualification | ❌ **BLOCKED BY PRD 20 - 3 ITERATIONS BEHIND** |
| **Section S.1** | Test Infrastructure Standards | ❌ **VIOLATION PERSISTS - 3 ITERATIONS** |

### Iteration-over-Iteration Comparison

| Issue | Iter 20 | Iter 21 | Iter 22 | Change |
|-------|---------|---------|---------|--------|
| PtySimulator | Stub | Stub | Stub | **NO PROGRESS** |
| BufferDiff | Stub | Stub | Stub | **NO PROGRESS** |
| StateTester | Stub | Stub | Stub | **NO PROGRESS** |
| TestDsl | Stub | Stub | Stub | **NO PROGRESS** |
| CliTester | Stub | Stub | Stub | **NO PROGRESS** |
| tests/ directory | Empty | Empty | Empty | **NO PROGRESS** |
| Phase 6 | Not started | Not started | Not started | **NO PROGRESS** |
| Bedrock test | Violation | Violation | Violation | **NO PROGRESS** |

**Conclusion:** ZERO progress across 3 consecutive iterations. This is a systemic enforcement failure.

---

## Article III: Sustained Constitutional Enforcement Crisis

### Crisis Status: CHRONIC ENFORCEMENT FAILURE

**This iteration (iteration-22) reveals a CHRONIC constitutional enforcement crisis:**

1. **Iteration 18:** Section R.1 added (Phase 6 Release Qualification mandate)
2. **Iteration 19:** No new constitutional gaps - enforcement required
3. **Iteration 20:** Section R.1 AMENDED to explicitly require PRD 20 (ratatui-testing)
4. **Iteration 21:** Same P0 blockers remain - Crisis declared
5. **Iteration 22:** **SAME ISSUES REMAIN - ZERO PROGRESS**

### Constitutional Mandates Already in Place (v3.1)

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

### Enforcement Failure Evidence (Iteration 22)

| Mandate | Constitutional Text | Implementation Status | Violation Duration |
|---------|---------------------|---------------------|-------------------|
| PRD 20 (ratatui-testing) | Section R.1 amended | All 5 modules + tests/ = stubs | **5+ iterations behind** |
| Phase 6 Release Qualification | Section R.1 | Not started (blocked by PRD 20) | **5+ iterations behind** |
| Test environment isolation | Section S.1 | Bedrock test still polluting | **3 iterations behind** |

---

## Article IV: P0/P1/P2 Issue Constitutionality Assessment

### P0 Issues - CONSTITUTIONAL ENFORCEMENT FAILURE (3RD ITERATION)

| Issue | Constitutional Coverage | Status |
|-------|------------------------|--------|
| PtySimulator stub | ✅ Section R.1 (explicit) | **VIOLATION - NO PROGRESS** |
| BufferDiff stub | ✅ Section R.1 (explicit) | **VIOLATION - NO PROGRESS** |
| StateTester stub | ✅ Section R.1 (explicit) | **VIOLATION - NO PROGRESS** |
| TestDsl stub | ✅ Section R.1 (explicit) | **VIOLATION - NO PROGRESS** |
| CliTester stub | ✅ Section R.1 (explicit) | **VIOLATION - NO PROGRESS** |
| Empty tests/ dir | ✅ Section R.1 (explicit) | **VIOLATION - NO PROGRESS** |

**Conclusion:** NO new constitutional amendment required. These are enforcement failures of explicit constitutional mandates.

### P1 Issues - CONSTITUTIONAL ENFORCEMENT FAILURE

| Issue | Constitutional Coverage | Status |
|-------|------------------------|--------|
| Phase 6 not started | ✅ Section R.1 | **VIOLATION - BLOCKED BY PRD 20** |
| Bedrock test pollution | ✅ Section S.1 | **VIOLATION - 3 ITERATIONS** |

**Conclusion:** P1 issues are enforcement failures of existing constitutional provisions.

### P2 Issues - Implementation Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Trailing whitespace | Not constitutional | Run `cargo fmt` |
| TestHarness unused methods | Not constitutional | Clean up dead code |

**Conclusion:** P2 issues are implementation/code quality issues, not constitutional gaps.

---

## Article V: Iteration-22 Specific Findings

### Current Stub Implementations

| File | Lines | Status | Missing |
|------|-------|--------|---------|
| pty.rs | 23 | STUB | `portable-pty` imported but unused; `resize()`, `inject_key_event()`, `inject_mouse_event()` missing |
| diff.rs | 16 | STUB | `DiffResult`, `CellDiff` structs missing; `diff()` returns empty string |
| state.rs | 21 | STUB | `capture()` method missing |
| dsl.rs | 17 | STUB | PTY composition missing; `send_keys()`, `wait_for()` missing |
| cli.rs | 16 | STUB | Process spawning not implemented; `CliOutput` struct missing |
| tests/ | 0 | EMPTY | No test files exist |

### ratatui-testing Progress (Iteration 21 vs 22)

| File | Iter 21 | Iter 22 | Change |
|------|---------|---------|--------|
| lib.rs | 20 lines | 20 lines | No change |
| pty.rs | 23 lines | 23 lines | No change |
| diff.rs | 16 lines | 16 lines | No change |
| state.rs | 21 lines | 21 lines | No change |
| dsl.rs | 17 lines | 17 lines | No change |
| cli.rs | 16 lines | 16 lines | No change |
| tests/ | Empty | Empty | No change |

**Conclusion:** Iteration 22 shows ZERO functional progress. Only boilerplate `Default` impls added in earlier iterations.

---

## Article VI: Required Actions (Implementation, Not Constitutional)

### Immediate Actions (P0 - Constitutional Enforcement REQUIRED)

1. **Implement PtySimulator** - Constitutional mandate, Section R.1
   - Add `master: Option<Box<dyn MasterPty>>`, `child: Option<Box<dyn Child>>` fields
   - Implement `new(command: &[&str]) -> Result<Self>` creating PtyPair
   - Implement `resize(&mut self, cols: u16, rows: u16) -> Result<()>`
   - Implement `read_output(&mut self, timeout: Duration) -> Result<String>` with timeout
   - Implement `inject_key_event(&mut self, event: KeyEvent) -> Result<()>`
   - Implement `inject_mouse_event(&mut self, event: MouseEvent) -> Result<()>`

2. **Implement BufferDiff** - Constitutional mandate, Section R.1
   - Define `DiffResult` struct with `passed`, `expected`, `actual`, `differences`
   - Define `CellDiff` struct with `x`, `y`, `expected`, `actual`
   - Implement `ignore_fg()`, `ignore_bg()`, `ignore_attributes()` builder methods
   - Implement `diff_str(&self, expected: &str, actual: &str) -> DiffResult`

3. **Implement StateTester** - Constitutional mandate, Section R.1
   - Add `snapshot: Option<Value>`, `captured: Vec<Value>` fields
   - Implement `capture<S>(&mut self, state: &S) -> Result<()>` where S: Serialize
   - Implement `assert_state_matches(&self, expected: &Value) -> Result<()>`

4. **Implement TestDsl** - Constitutional mandate, Section R.1
   - Add `pty: Option<PtySimulator>`, `buffer_diff: BufferDiff`, `state_tester: StateTester` fields
   - Implement `with_pty(mut self, cmd: &[&str]) -> Result<Self>`
   - Implement `send_keys(&mut self, keys: &str) -> Result<&mut Self>`
   - Implement `wait_for<F>(&mut self, timeout: Duration, predicate: F) -> Result<&mut Self>`

5. **Implement CliTester** - Constitutional mandate, Section R.1
   - Define `CliOutput` struct with `exit_code`, `stdout`, `stderr`
   - Implement `with_temp_dir(mut self) -> Result<Self>`
   - Implement `run(&self, args: &[&str]) -> Result<CliOutput>`

6. **Add Integration Tests** - Constitutional mandate, Section R.1
   - Create `tests/pty_tests.rs`
   - Create `tests/buffer_diff_tests.rs`
   - Create `tests/state_tests.rs`
   - Create `tests/dsl_tests.rs`
   - Create `tests/cli_tests.rs`

### Medium-term Actions (P1 - Constitutional Enforcement REQUIRED)

7. **Begin Phase 6 Release Qualification** - After PRD 20 complete
   - Section R.1 mandates completion before v1.0

8. **Fix Bedrock Test Environment Pollution** - Section S.1 violation
   - Use `temp_env::set_var` for environment variable isolation

### Short-term Actions (P2)

9. **Run `cargo fmt --all`** to fix trailing whitespace

10. **Clean up TestHarness dead code** in `crates/cli/tests/common.rs`

---

## Appendix A: Gap → Constitution Mapping (Iteration 22)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-22-1 | PtySimulator stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-22-2 | BufferDiff stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-22-3 | StateTester stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-22-4 | TestDsl stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-22-5 | CliTester stub | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P0-22-6 | Empty tests/ dir | Section R.1 (explicit) | **CONSTITUTIONAL VIOLATION** |
| P1-22-1 | Phase 6 not started | Section R.1 | **CONSTITUTIONAL VIOLATION** |
| P1-22-2 | Bedrock test pollution | Section S.1 | **VIOLATION - 3 ITERATIONS** |
| P2-22-1 | trailing whitespace | Not constitutional | Fix with fmt |
| P2-22-2 | TestHarness dead code | Not constitutional | Clean up |

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
| v3.1 | Iteration 21 | I–VII + A–S | NO AMENDMENT - Crisis declared |
| **v3.1** | **Iteration 22** | **I–VII + A–S** | **NO AMENDMENT - Crisis sustained** |

---

## Priority Summary for Iteration 22

| Priority | Item | Action Required | Constitutional Status |
|----------|------|----------------|----------------------|
| **P0** | PRD 20 (ratatui-testing) | Full implementation of 5 modules + tests | **MANDATED BY SECTION R.1 (3 iterations behind)** |
| **P1** | Phase 6 Release Qualification | Cannot start until PRD 20 | **MANDATED BY SECTION R.1 (blocked)** |
| **P1** | Bedrock test pollution | Implement Section S.1 | **VIOLATION OF SECTION S.1 (3 iterations)** |
| P2 | Trailing whitespace | Run `cargo fmt` | Not constitutional |
| P2 | Deprecated fields | Deferred to v4.0 | Covered by Amend D |

**Constitutional additions in Iteration 22:** NONE - NO NEW CONSTITUTIONAL GAPS IDENTIFIED

---

## Summary

**Overall Completion:** ~93-96% complete (UNCHANGED - ZERO PROGRESS on PRD 20 for 3 iterations)

**Constitutional Assessment: ADEQUATE BUT CHRONICALLY VIOLATED**

The Constitution v3.1 is **constitutionally adequate** for iteration-22 issues. NO new constitutional amendments are required because:

1. **Section R.1 (Amended in v3.1)** already explicitly mandates PRD 20 (ratatui-testing) as Phase 6 prerequisite
2. **Section S.1** already mandates test environment isolation
3. All iteration-22 P0/P1 issues are identical to iterations 20 and 21 issues
4. There are **NO NEW constitutional gaps** - only sustained enforcement failures

**CRITICAL FINDING: CHRONIC CONSTITUTIONAL ENFORCEMENT CRISIS**

The problem is not constitutional inadequacy - it is **willful non-compliance with existing constitutional mandates**:

- Section R.1 was amended in iteration-20 specifically to address the PRD 20 issue
- Despite this explicit mandate, iterations 21 and 22 show ZERO progress on PRD 20
- All 6 P0 blockers from iterations 20, 21, and 22 remain unimplemented
- Phase 6 remains blocked by the same issue from 5+ iterations ago
- Bedrock test pollution persists for 3 iterations despite Section S.1 mandate

**Required Actions (Enforcement, Not Constitutional):**
1. **CONSTITUTIONAL ENFORCEMENT:** Implement PRD 20 (ratatui-testing) - 5 modules + tests/ currently stubs
2. **CONSTITUTIONAL ENFORCEMENT:** Fix Bedrock test environment isolation (Section S.1 violation)
3. **BEGIN PHASE 6:** After PRD 20 complete, Section R.1 mandates Phase 6 completion before v1.0

---

*Constitution v3.1 — Iteration 22*
*Total constitutional articles: 7 (original) + 19 amendments (A–S)*
*P0 blockers constitutionally covered: ALL (explicit mandate in Section R.1)*
*New constitutional addition: NONE - Sustained enforcement crisis, not constitutional gap*
*Report generated: 2026-04-14*
