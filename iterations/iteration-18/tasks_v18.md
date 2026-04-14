# Task List - Iteration 18

**Generated:** 2026-04-14  
**Priority:** P0 > P1 > P2  
**Status:** ~95% Complete, Phase 6 In Progress

---

## P0 Tasks (Blocking - ALL RESOLVED)

| # | Task | Module | Status | Notes |
|---|------|--------|--------|-------|
| P0-1 | Fix custom tool discovery format mismatch | tools | ✅ DONE | |
| P0-2 | Register custom tools with ToolRegistry | tools | ✅ DONE | |
| P0-3 | Implement plugin tool registration | plugin | ✅ DONE | |
| P0-4 | Fix non-deterministic hook execution order | plugin | ✅ DONE | Uses IndexMap with priority ordering |
| P0-5 | Enforce plugin config ownership | config | ✅ DONE | Server/runtime split enforced |
| P0-6 | Implement full config crate | config | ✅ DONE | 1581+ lines, PRD 19 compliant |

---

## P1 Tasks (High Priority)

### Phase 6 Release Qualification

| # | Task | Module | Status | Notes |
|---|------|--------|--------|-------|
| P1-1 | **Begin Phase 6 Release Qualification** | all | ✅ DONE | Added 3 new test files, enhanced MCP tests |
| P1-2 | Create end-to-end integration test suite | tests | ❌ NOT STARTED | |
| P1-3 | Run performance benchmarks | all | ❌ NOT STARTED | Startup time, latency, memory |
| P1-4 | Conduct security audit | all | ❌ NOT STARTED | Permission boundaries, auth, validation |
| P1-5 | Validate observability (logging/tracing) | all | ❌ NOT STARTED | |

### Test Infrastructure Fixes

| # | Task | Module | Status | Notes |
|---|------|--------|--------|-------|
| P1-6 | Fix GitLab CI integration tests | git | ❌ NOT STARTED | 7 tests require real GitLab server |
| P1-7 | Mark GitLab tests with `#[ignore]` | git | ❌ NOT STARTED | Document as requiring external GitLab |
| P1-8 | OR create mock GitLab server for CI | git | ❌ NOT STARTED | Alternative to ignoring tests |

---

## P2 Tasks (Medium Priority)

| # | Task | Module | Status | Notes |
|---|------|--------|--------|-------|
| P2-1 | Fix desktop_web_different_ports test | cli | ❌ NOT FIXED | Use dynamic port allocation |
| P2-2 | Remove deprecated `mode` field | config | ⚠️ Deferred | v4.0 cleanup |
| P2-3 | Remove deprecated `tools` field | config | ⚠️ Deferred | Post-migration cleanup |

---

## Task Details

### P1-1: Begin Phase 6 Release Qualification

**Description:** Start the final release qualification phase. This encompasses end-to-end testing, performance benchmarking, security audit, and observability validation.

**Owner:** all  
**Priority:** P1  
**Status:** ✅ DONE

**Subtasks:**
- [x] 6.1.1: Verify session creation and lifecycle
- [x] 6.1.2: Verify message processing flow
- [x] 6.1.3: Verify tool execution pipeline
- [x] 6.1.4: Verify MCP server connection and tool execution
- [x] 6.1.5: Verify LSP diagnostics flow
- [x] 6.1.6: Verify plugin hook execution

**Implementation Notes:**
- Added `lsp_diagnostics_tests.rs` with 6 new tests for LSP diagnostics flow
- Added `plugin_hook_tests.rs` with 17 new tests for plugin hook execution
- Added `phase6_regression_tests.rs` with 19 new regression tests
- Enhanced `mcp_protocol_tests.rs` with 20 new MCP server connection and tool execution tests
- All 181 integration tests now pass (excluding known GitLab CI failures)

---

### P1-6: Fix GitLab CI Integration Tests

**Description:** 7 integration tests in `crates/git/src/gitlab_ci.rs` require a real GitLab server at `http://127.0.0.1:63182`. These tests fail in normal CI.

**Owner:** git  
**Priority:** P1  
**Status:** ❌ NOT STARTED

**Affected Tests:**
- `gitlab_integration_tests::test_gitlab_ci_setup_and_trigger`
- `gitlab_integration_tests::test_gitlab_ci_template_end_to_end_with_component`
- `gitlab_integration_tests::test_gitlab_pipeline_status_monitoring`
- `gitlab_integration_tests::test_gitlab_pipeline_status_with_failed_pipeline`
- `gitlab_integration_tests::test_gitlab_pipeline_trigger`
- `gitlab_integration_tests::test_gitlab_pipeline_trigger_and_monitor_end_to_end`
- `gitlab_integration_tests::test_gitlab_pipeline_trigger_multiple_branches`

**Solution Options:**
1. Mark with `#[ignore]` and document requiring external GitLab
2. Create mock GitLab server for CI
3. Add proper feature gate for integration tests

---

### P2-1: Fix Desktop/Web Smoke Test Port Conflict

**Description:** `desktop_web_different_ports` test uses hardcoded port 3000 which may conflict with other processes.

**Owner:** cli  
**Priority:** P2  
**Status:** ❌ NOT FIXED

**Fix:** Use dynamic port allocation instead of hardcoded port 3000.

---

## Completed Tasks (Iteration 18)

| # | Task | Module | Status |
|---|------|--------|--------|
| ✅ | Config crate full implementation | config | DONE (1581+ lines) |
| ✅ | Fix config tests PoisonError | core | DONE |
| ✅ | Fix TUI keybinding tests (2 tests) | tui | DONE |
| ✅ | Fix TUI theme color parsing test | tui | DONE |
| ✅ | Resolve all P0 blocking issues | all | DONE |

---

## Test Failure Summary

```
cargo test --all-features --all:
- ~1020+ passed (including 181 integration tests)
- 8 failed

Failed tests breakdown:
- GitLab CI tests: 7 failures (integration tests require real GitLab server at http://127.0.0.1:63182)
- CLI tests: 1 failure (desktop_web_different_ports - port conflict with hardcoded port 3000)

Note: GitLab CI test failures are expected - these tests require an external GitLab server.
```

---

## Next Iteration Priorities

1. **Continue Phase 6 Release Qualification**
   - Complete end-to-end integration testing
   - Complete performance benchmarking
   - Address security findings
   - Validate observability

2. **Fix remaining test infrastructure issues**
   - GitLab CI tests proper handling
   - Desktop/web smoke test port

3. **Medium-term**: Legacy cleanup (v4.0)
   - Deprecated `mode` field removal
   - Deprecated `tools` field removal

---

**Document Version:** 1.8  
**Iteration:** 18  
**Last Updated:** 2026-04-14