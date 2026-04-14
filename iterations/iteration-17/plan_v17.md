# Implementation Plan - Iteration 17

**Version:** 17
**Date:** 2026-04-14
**Status:** In Development (Phase 1-5 Complete, Phase 6 Pending)
**Implementation Progress:** ~85-90% Complete

---

## Executive Summary

Implementation is approximately **85-90% complete** relative to PRD scope. All P0 blocking issues from iteration-16 have been resolved. Phase 6 (Release Qualification) remains pending.

**Key Achievements in Iteration 17:**
- All P0 issues resolved (custom tool discovery, plugin tool registration, hook execution order)
- Session lifecycle integration tests added (21KB test file)
- Ownership tree acyclicity tests added (40+ tests)
- Primary agent invariant tests added (20+ tests)
- TUI component tests added (slash commands, input model, sidebar, plugins)
- Desktop and web server modes fully implemented
- ACP transport layer completed
- ratatui-testing crate fully implemented

---

## Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Mostly Done | ~95% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~95% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~95% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~90% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## Crate Implementation Status

| Crate | Status | Notes |
|-------|--------|-------|
| `crates/core/` | ✅ Done | Entity models, config, most functionality |
| `crates/storage/` | ✅ Done | Persistence, recovery, snapshots |
| `crates/agent/` | ✅ Done | Runtime, delegation, permission inheritance |
| `crates/tools/` | ✅ Done | Registry and custom tool discovery |
| `crates/plugin/` | ✅ Done | Hooks and tool registration |
| `crates/tui/` | ✅ Done | Full implementation, tests added |
| `crates/server/` | ✅ Done | API routes, auth, streaming |
| `crates/mcp/` | ✅ Done | Full MCP implementation |
| `crates/lsp/` | ✅ Done | LSP client, diagnostics, experimental tools |
| `crates/llm/` | ✅ Done | Multiple providers, model selection |
| `crates/git/` | ✅ Done | GitHub/GitLab integration |
| `crates/config/` | ⚠️ Broken | Empty re-export, not real crate |
| `crates/cli/` | ✅ Done | Desktop/web implemented |
| `crates/control-plane/` | ✅ Done | ACP stream, events, enterprise features |
| `ratatui-testing/` | ✅ Done | TUI testing framework crate |

---

## Remaining Issues

### P1 - High Priority (Must Fix Before Phase 6)

| Issue | Status | Module | Impact |
|-------|--------|--------|--------|
| Config crate empty re-export | ❌ NOT FIXED | config | Violates PRD 19 crate ownership architecture |
| Config tests failing with PoisonError | ❌ NOT FIXED | test infra | 10 tests fail in parallel runs |

### P2 - Medium Priority (Should Fix Before Phase 6)

| Issue | Status | Module | Impact |
|-------|--------|--------|--------|
| TUI keybinding tests failing (2 tests) | ❌ NOT FIXED | tui | Case sensitivity and Space key handling |
| TUI theme color parsing test failing | ❌ NOT FIXED | tui | Hex color parsing returns wrong value |
| Desktop/web smoke test port conflict | ❌ NOT FIXED | cli | Flaky test, assumes specific port |

### Deferred Items

| Issue | Status | Module | Notes |
|-------|--------|--------|-------|
| Per-agent model override testing | ⚠️ Deferred | llm | Implementation exists, not explicitly tested |
| Hidden vs visible agent UI behavior | ⚠️ Deferred | agent | Tests exist for invariant, UI behavior not critical |
| Deprecated `mode` field removal | ⚠️ Deferred | core | Marked for removal in v4.0 |
| Deprecated `tools` field removal | ⚠️ Deferred | core | Marked for removal after migration |

---

## Test Results Summary

```
cargo test --all-features --all:
- 610 passed
- 14 failed (across all packages)
```

### Failing Tests Breakdown

| Category | Failures | Root Cause |
|----------|----------|------------|
| Config tests | 10 | PoisonError from ENV_LOCK mutex in parallel tests |
| TUI tests | 3 | Keybinding (2), theme color parsing (1) |
| CLI tests | 1 | Port conflict in smoke test |

---

## Next Steps

### Immediate Actions (Phase 6 Preparation)

1. **Fix Config Crate** (P1)
   - Move config logic from `core` to dedicated `crates/config/` crate
   - Align with PRD 19 crate ownership intentions
   - Estimated effort: Medium

2. **Fix Test Infrastructure** (P1)
   - Refactor ENV_LOCK to use proper async Mutex or remove shared state
   - Ensure tests can run in parallel without race conditions
   - Estimated effort: Medium

### Medium-term Actions (Before Release)

3. **Fix TUI Test Failures** (P2)
   - Fix keybinding tests: case sensitivity and Space key handling
   - Fix theme hex color parsing test
   - Estimated effort: Low

4. **Fix Desktop/Web Smoke Test** (P2)
   - Use dynamic port allocation instead of hardcoded port 3000
   - Estimated effort: Low

### Phase 6 Release Qualification (When P1/P2 Issues Resolved)

5. **End-to-end Integration Tests**
   - Full session lifecycle testing
   - Cross-module integration verification

6. **Performance Benchmarking**
   - Baseline performance metrics
   - Regression detection setup

7. **Security Audit**
   - Auth flow verification
   - Permission system validation

---

## Technical Debt

| Item | Description | Status |
|------|-------------|--------|
| Empty `crates/config/` crate | Re-exports from core instead of housing config logic | ❌ NOT FIXED |
| Config tests use ENV_LOCK with race condition | Tests fail with PoisonError | ❌ NOT FIXED |
| TUI test failures | 3 tests failing (keybinding 2, theme 1) | ❌ NOT FIXED |
| Desktop/web smoke test port conflict | Test assumes specific port availability | ❌ NOT FIXED |
| Deprecated `mode` field | Marked for removal in v4.0 | ⚠️ Deferred |
| Deprecated `tools` field | Marked for removal after migration | ⚠️ Deferred |

---

## Success Criteria for Phase 6

- [ ] All 14 failing tests fixed
- [ ] Config crate properly separated per PRD 19
- [ ] All tests pass in parallel execution
- [ ] End-to-end integration tests pass
- [ ] Performance benchmarks meet baseline
- [ ] Security audit completed

---

*Document Version: 17*
*Last Updated: 2026-04-14*
*Next Action: Fix P1 issues (config crate refactor, test infrastructure)*
