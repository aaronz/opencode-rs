# Task List - Iteration 19

**Generated:** 2026-04-14  
**Based on:** spec_v19.md, gap-analysis.md  
**Total Tasks:** 6  
**P0:** 0 | **P1:** 3 | **P2:** 3

---

## P1 - High Priority

### P1.1: Fix `desktop_web_different_ports` Test
**Severity:** P1  
**Module:** cli  
**Status:** ✅ Done

**Issue:** Test uses hardcoded port 3000 causing conflicts with other processes.

**Location:** `crates/cli/tests/e2e_desktop_web_smoke.rs:162`

**Fix Required:**
- Replace hardcoded port 3000 with dynamic port allocation
- Use `TcpListener::bind("127.0.0.1:0")` to get available port

**Verification:**
```bash
cargo test -p cli desktop_web_different_ports
```

---

### P1.2: Begin Phase 6 Release Qualification
**Severity:** P1  
**Module:** all  
**Status:** NOT STARTED

**Phase 6 Scope:**
1. End-to-end integration tests
2. Performance benchmarking
3. Security audit
4. Observability validation
5. Documentation review

**Deliverables:**
- Phase 6 test plan document
- E2E test infrastructure
- Benchmark results
- Security audit report
- Observability validation report

**Verification:**
```bash
# Run full test suite
cargo test --all-features --all

# Run benchmarks (when implemented)
cargo bench
```

---

### P1.3: Fix Bedrock Test Environment Pollution
**Severity:** P1  
**Module:** llm  
**Status:** NOT FIXED

**Issue:** `AWS_BEARER_TOKEN_BEDROCK` and `AWS_ACCESS_KEY_ID` env vars from other tests pollute this test when run with `--all-features`.

**Location:** `crates/llm/src/bedrock.rs:266`

**Fix Required:**
- Use `temp_env` pattern to isolate environment variables
- Or run test in separate process
- Or clear relevant env vars before test execution

**Verification:**
```bash
cargo test --all-features -p opencode-llm test_bedrock_credential_resolution_bearer_token_priority
```

---

## P2 - Medium Priority

### P2.1: Fix Trailing Whitespace
**Severity:** P2  
**Module:** storage  
**Status:** NOT FIXED

**Issue:** 5 lines with trailing whitespace in `storage/src/service.rs:317, 340, 363, 386, 391`.

**Fix Required:**
```bash
cargo fmt --all
```

**Verification:**
```bash
cargo fmt --all -- --check
```

---

### P2.2: Deprecated `mode` Field Cleanup
**Severity:** P2  
**Module:** config  
**Status:** Deferred

**Issue:** Legacy `mode` field marked for removal in v4.0.

**Action:** Schedule removal for v4.0 release.

---

### P2.3: Deprecated `tools` Field Cleanup
**Severity:** P2  
**Module:** config  
**Status:** Deferred

**Issue:** Legacy `tools` field marked for removal after migration.

**Action:** Schedule removal after migration complete.

---

## Task Summary

| ID | Task | Priority | Module | Status |估计工时 |
|----|------|----------|--------|--------|---------|
| P1.1 | Fix desktop_web test port conflict | P1 | cli | ✅ Done | 1h |
| P1.2 | Begin Phase 6 Release Qualification | P1 | all | NOT STARTED | 40h |
| P1.3 | Fix Bedrock test env pollution | P1 | llm | NOT FIXED | 2h |
| P2.1 | Fix trailing whitespace | P2 | storage | NOT FIXED | 5min |
| P2.2 | Deprecated mode field cleanup | P2 | config | Deferred | - |
| P2.3 | Deprecated tools field cleanup | P2 | config | Deferred | - |

---

## Quick Commands

```bash
# Verify all P1 fixes
cargo test -p cli desktop_web_different_ports
cargo test --all-features -p opencode-llm test_bedrock_credential_resolution_bearer_token_priority

# Fix P2.1
cargo fmt --all

# Run full test suite
cargo test --all-features --all
```

---

**Last Updated:** 2026-04-14
