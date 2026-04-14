# Constitution Updates - Iteration 20

**Generated:** 2026-04-14
**Based on Gap Analysis:** `iteration-20/gap-analysis.md`
**Previous Constitution:** `iteration-19/constitution_updates.md` (v3.0)
**Status:** Amendment Proposal

---

## Executive Summary

**Implementation is ~93-96% complete** (stable from iteration-19).

**Key Changes Since Iteration-19:**
- `desktop_web_different_ports` test FIXED (dynamic port allocation)
- PRD 20 (ratatui-testing) identified as primary implementation target for iteration-20

**Assessment:** Constitution v3.0 is **MOSTLY ADEQUATE** but has one constitutional gap: Section R.1 does not explicitly mandate TUI testing infrastructure as a prerequisite for Phase 6. PRD 20 (ratatui-testing) is entirely in stub form, blocking Phase 6, but this is an enforcement failure of existing Section R.1 rather than a new gap.

**Recommendation:** Minor amendment to Section R.1 to explicitly require TUI testing infrastructure (PRD 20) as a prerequisite for Phase 6 end-to-end testing.

---

## Article I: Gap Analysis Summary (Iteration 20)

### P0 Issues (PRD 20 Implementation)

| Gap ID | Description | Status | Constitutional Reference |
|--------|-------------|--------|------------------------|
| P0-20-1 | PtySimulator stub implementation | ❌ NOT STARTED | Implicit in Section R.1 |
| P0-20-2 | BufferDiff stub implementation | ❌ NOT STARTED | Implicit in Section R.1 |
| P0-20-3 | StateTester stub implementation | ❌ NOT STARTED | Implicit in Section R.1 |
| P0-20-4 | TestDsl stub implementation | ❌ NOT STARTED | Implicit in Section R.1 |
| P0-20-5 | CliTester stub implementation | ❌ NOT STARTED | Implicit in Section R.1 |
| P0-20-6 | Empty ratatui-testing tests/ | ❌ NOT STARTED | Implicit in Section R.1 |

### P1/P2 Issues

| Gap ID | Description | Severity | Constitutional Coverage |
|--------|-------------|----------|------------------------|
| P1-20-1 | Phase 6 Release Qualification not started | P1 | ✅ Covered by Section R.1 |
| P1-20-2 | Bedrock test environment pollution | P1 | ✅ Covered by Section S.1 |
| P2-20-1 | Trailing whitespace | P2 | Not constitutional |
| P2-20-2 | TestHarness unused methods | P2 | Not constitutional |
| P2-20-3 | Deprecated fields | P2 | Covered by Amend D §D.2 |

---

## Article II: Constitutional Coverage Analysis

### Constitution v3.0 Coverage for Iteration 20 Issues

| Constitution Reference | Mandate | Iteration 20 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1–3.6 | Tools/Plugin/Hooks | ✅ Verified |
| Art IV §4.1–4.2 | MCP/LSP | ✅ Verified |
| Art V §5.1–5.3 | Server API | ✅ Verified |
| Art VI §6.1–6.2 | Desktop/ACP | ✅ Verified |
| Amend A–Q | Various | ✅ Verified |
| **Section R.1** | Phase 6 Release Qualification | ❌ **NOT STARTED** |
| **Section S.1** | Test Infrastructure Standards | ⚠️ **PARTIALLY VIOLATED** (Bedrock test) |

### Constitutional Gap Analysis

**Section R.1 Gap:** Section R.1 mandates "End-to-End Integration Tests" for Phase 6 but does NOT explicitly require TUI testing infrastructure. Since the TUI is a core interface mode, TUI testing infrastructure is implicitly required, but this is not explicit.

**Section S.1 Status:** Bedrock test (`test_bedrock_credential_resolution_bearer_token_priority`) still violates Section S.1 environment isolation requirements.

---

## Article III: New Constitutional Requirement

### Section R.1 Amendment - TUI Testing Infrastructure Mandate

**Issue:** Section R.1 does not explicitly state that TUI testing infrastructure is required for Phase 6. The `ratatui-testing` crate (PRD 20) is entirely in stub form, blocking Phase 6 end-to-end testing of the TUI.

**Current Section R.1 Text:**
```rust
// Section R.1 - Phase 6 Release Qualification (PRD 19)
CONSTRAINT: Phase 6 MUST be completed before v1.0 release.
Phase 6 Release Qualification MUST include:
1. End-to-End Integration Tests
2. Performance Benchmarks
3. Security Audit
4. Observability Validation
```

**Proposed Amendment to Section R.1:**

```rust
// Section R.1 - Phase 6 Release Qualification (PRD 19) - AMENDED
CONSTRAINT: Phase 6 MUST be completed before v1.0 release.
Phase 6 Release Qualification MUST include:

1. End-to-End Integration Tests
   - Full session lifecycle across all interface modes (CLI, TUI, Desktop, Web)
   - Multi-agent coordination and delegation scenarios
   - Cross-crate integration (storage → agent → tools → server)
   - **TUI Testing Infrastructure:** Before TUI end-to-end tests can run, PRD 20
     (ratatui-testing) MUST be fully implemented with:
     * PtySimulator for PTY master/slave terminal simulation
     * BufferDiff for rendering regression detection
     * StateTester for application state verification
     * TestDsl for fluent test composition
     * CliTester for CLI process testing
   - **Prerequisite Chain:** Phase 6 TUI testing requires PRD 20 completion

2. Performance Benchmarks
   - Session creation latency (target: <100ms)
   - Tool execution throughput (target: >100 tools/sec)
   - Memory usage under sustained load
   - Startup time benchmarks

3. Security Audit
   - Permission boundary verification
   - Auth token handling review
   - Input validation across all entry points
   - Plugin isolation verification

4. Observability Validation
   - Logging subsystem verification
   - Error reporting and diagnostics
   - Metrics collection functionality
   - Tracing integration (if implemented)
```

**Rationale:** Making the TUI testing infrastructure prerequisite explicit prevents recurrence of the current situation where Phase 6 cannot begin because the testing infrastructure is not implemented.

---

## Article IV: P1/P2 Issue Constitutionality Assessment

### P0 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| PtySimulator stub | Implicit in Section R.1 | Amend Section R.1 explicitly |
| BufferDiff stub | Implicit in Section R.1 | Amend Section R.1 explicitly |
| StateTester stub | Implicit in Section R.1 | Amend Section R.1 explicitly |
| TestDsl stub | Implicit in Section R.1 | Amend Section R.1 explicitly |
| CliTester stub | Implicit in Section R.1 | Amend Section R.1 explicitly |
| Empty tests/ dir | Implicit in Section R.1 | Amend Section R.1 explicitly |

**Conclusion:** The P0 issues are implementation failures of an implicit constitutional requirement. Amendment to Section R.1 makes this explicit.

### P1 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Phase 6 not started | ✅ Covered by Section R.1 | Enforce existing mandate |
| Bedrock test pollution | ✅ Covered by Section S.1 | Enforce existing mandate |

**Conclusion:** P1 issues are enforcement failures of existing constitutional provisions.

### P2 Issues

| Issue | Constitutional Coverage | Recommendation |
|-------|------------------------|----------------|
| Trailing whitespace | Not constitutional | Run `cargo fmt` |
| TestHarness unused methods | Not constitutional | Clean up dead code |
| Deprecated fields | Covered by Amend D §D.2 | Deferred to v4.0 |

**Conclusion:** P2 issues are implementation/code quality issues, not constitutional gaps.

---

## Article V: Required Actions (Implementation, Not Constitutional)

### Immediate Actions (P0 - PRD 20 Implementation)

1. **Implement PtySimulator**
   - Add `portable-pty` dependency to `Cargo.toml`
   - Implement PTY master/slave creation on Unix
   - Implement `write_input()`, `read_output()` with timeout
   - Implement `resize()`, `inject_key_event()`, `inject_mouse_event()`

2. **Implement BufferDiff**
   - Add cell-by-cell comparison using `ratatui::Buffer`
   - Implement `DiffResult` and `CellDiff` structs
   - Add color/attribute ignore options

3. **Implement StateTester**
   - Add `capture()` method for JSON serialization
   - Implement `assert_state()` and `assert_state_matches()`

4. **Implement TestDsl**
   - Compose PtySimulator, BufferDiff, StateTester
   - Implement fluent API with `send_keys()`, `wait_for()`

5. **Implement CliTester**
   - Use `assert_cmd` for process spawning
   - Capture stdout/stderr, return exit code

6. **Add Integration Tests**
   - Create `tests/pty_tests.rs`, `tests/buffer_diff_tests.rs`, etc.
   - Ensure `cargo test --all-features -p ratatui-testing` passes

### Medium-term Actions (P1)

7. **Begin Phase 6 Release Qualification** (after PRD 20 complete)
   - Section R.1 mandates this must be completed before v1.0
   - Define end-to-end integration tests
   - Define performance benchmarks

8. **Fix Bedrock Test Environment Pollution**
   - Use `temp_env::set_var` for environment variable isolation
   - Violates Section S.1

### Short-term Actions (P2)

9. **Run `cargo fmt --all`** to fix trailing whitespace

10. **Clean up TestHarness dead code** in `crates/cli/tests/common.rs`

11. **Run `cargo fix --tests --all`** to fix clippy warnings

---

## Article VI: Updated Compliance Checklist

### Phase 6 Release Qualification (Section R.1) - NOT STARTED

- [ ] **PRD 20 (ratatui-testing) fully implemented** - NEW PREREQUISITE
  - [ ] PtySimulator with PTY master/slave
  - [ ] BufferDiff with cell-by-cell comparison
  - [ ] StateTester with JSON snapshot support
  - [ ] TestDsl with fluent API
  - [ ] CliTester with process spawning
  - [ ] Integration tests for all modules
- [ ] End-to-End Integration Tests defined
- [ ] Performance Benchmarks defined (session <100ms, tools >100/sec)
- [ ] Security Audit planned
- [ ] Observability Validation planned
- [ ] No P0/P1 issues remaining before Phase 6 start

### Test Infrastructure Standards (Section S.1) - PARTIALLY VIOLATED

- [ ] All environment-dependent tests marked with `#[ignore]`
- [ ] All feature-gated integration tests documented
- [ ] Mock implementations available for CI
- [ ] Default test suite passes in clean environment
- [x] `desktop_web_different_ports` - ✅ FIXED (dynamic port allocation)
- [ ] `test_bedrock_credential_resolution...` - **MUST FIX** (violates Section S.1)

### Build Quality Gate (Amendment A + J + M + O)

- [x] `cargo build --all` exits 0
- [x] `cargo test --all --no-run` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [x] No P0 gaps in implementation

---

## Appendix A: Gap → Constitution Mapping (Iteration 20)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-20-1 | PtySimulator stub | Section R.1 (implicit) | **ENFORCEMENT FAILURE** |
| P0-20-2 | BufferDiff stub | Section R.1 (implicit) | **ENFORCEMENT FAILURE** |
| P0-20-3 | StateTester stub | Section R.1 (implicit) | **ENFORCEMENT FAILURE** |
| P0-20-4 | TestDsl stub | Section R.1 (implicit) | **ENFORCEMENT FAILURE** |
| P0-20-5 | CliTester stub | Section R.1 (implicit) | **ENFORCEMENT FAILURE** |
| P0-20-6 | Empty tests/ dir | Section R.1 (implicit) | **ENFORCEMENT FAILURE** |
| P1-20-1 | Phase 6 not started | Section R.1 | **ENFORCEMENT FAILURE** |
| P1-20-2 | Bedrock test pollution | Section S.1 | **VIOLATION** |
| P2-20-1 | trailing whitespace | Not constitutional | Fix with fmt |
| P2-20-2 | TestHarness dead code | Not constitutional | Clean up |
| P2-20-3 | deprecated fields | Amend D §D.2 | Deferred |

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
| v3.0 | Iteration 19 | I–VII + A–S | No new amendments (enforcement required) |
| **v3.1** | **Iteration 20** | **I–VII + A–S** | **Section R.1 amendment: TUI testing infrastructure (PRD 20) explicit prerequisite** |

---

## Priority Summary for Iteration 20

| Priority | Item | Action Required |
|----------|------|----------------|
| **P0** | PRD 20 (ratatui-testing) | Full implementation per PRD 20 spec |
| **P1** | Phase 6 Release Qualification | Cannot start until PRD 20 complete |
| **P1** | Bedrock test pollution | Implement Section S.1 (env isolation) |
| P2 | Trailing whitespace | Run `cargo fmt` |
| P2 | Deprecated fields | Deferred to v4.0 |

**Constitutional additions in Iteration 20:** Section R.1 amendment to explicitly require TUI testing infrastructure (PRD 20) as Phase 6 prerequisite.

---

## Summary

**Overall Completion:** ~93-96% complete (stable from iteration-19)

**Constitutional Assessment: MOSTLY ADEQUATE with Amendment**

The Constitution v3.0 is **adequate** for iteration-20 issues but requires one minor amendment to Section R.1 to explicitly mandate TUI testing infrastructure (PRD 20) as a prerequisite for Phase 6 end-to-end testing.

**Key Findings:**
- All P0 blockers are implementation failures, not constitutional gaps
- Section R.1 implicitly required TUI testing infrastructure but didn't state it explicitly
- Section S.1 violations persist (Bedrock test pollution)
- Phase 6 remains blocked by PRD 20 non-implementation

**Key Achievements Since Iteration 19:**
- ✅ `desktop_web_different_ports` test now passes
- ✅ Constitution correctly identified gaps as enforcement failures

**Required Actions:**
1. Amend Section R.1 to explicitly require PRD 20 TUI testing infrastructure
2. Implement PRD 20 (ratatui-testing) - 6 modules currently stubs
3. Fix Bedrock test environment isolation (Section S.1 violation)
4. Begin Phase 6 after PRD 20 complete

---

*Constitution v3.1 — Iteration 20*
*Total constitutional articles: 7 (original) + 19 amendments (A–S)*
*P0 blockers constitutionally covered: All (implicit in Section R.1, now explicit)*
*New constitutional addition: Section R.1 amendment (TUI testing infrastructure prerequisite)*
*Report generated: 2026-04-14*