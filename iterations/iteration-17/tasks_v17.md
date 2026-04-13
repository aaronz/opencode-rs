# Task List - Iteration 17

**Version:** 17
**Date:** 2026-04-14
**Status:** In Development
**Implementation Progress:** ~85-90% Complete

---

## P0 - Blocking Issues (ALL RESOLVED ✅)

All P0 blocking issues from iteration-16 have been resolved:

| # | Task | Status | Module | Notes |
|---|------|--------|--------|-------|
| P0-1 | Custom tool discovery scans .ts/.js instead of TOOL.md | ✅ FIXED | tools | Now scans `.ts/.js` per PRD |
| P0-2 | Custom tools not registered with ToolRegistry | ✅ FIXED | tools | `register_discovered_custom_tools()` works |
| P0-3 | Plugin tool registration not implemented | ✅ FIXED | plugin | `register_tool()` method now exists |

---

## P1 - High Priority (Must Fix Before Phase 6)

| # | Task | Status | Module | Files | Effort |
|---|------|--------|--------|-------|--------|
| P1-1 | Fix config crate empty re-export | ✅ FIXED | config | `crates/config/src/lib.rs` | Medium |
| P1-2 | Fix config tests PoisonError in parallel runs | ❌ NOT FIXED | test infra | `crates/core/src/config.rs` | Medium |

### P1-1: Fix Config Crate Empty Re-export

**Description:** `crates/config/src/lib.rs` is an empty re-export that violates PRD 19 crate ownership architecture. Config logic resides in `crates/core/src/config.rs` but should be in dedicated `crates/config/` crate.

**Requirements:**
- Move config parsing logic from `core` to dedicated `crates/config/` crate
- Maintain all existing functionality (JSON/JSONC parsing, variable expansion, precedence)
- Update all imports across the codebase
- Ensure backward compatibility

**Files Affected:**
- `crates/config/src/lib.rs` (currently empty re-export)
- `crates/core/src/config.rs` (config logic to be moved)
- All files importing from config

**Tests:**
- All config precedence tests must pass
- All variable expansion tests must pass

---

### P1-2: Fix Config Tests PoisonError

**Description:** 10 config tests fail with PoisonError when run in parallel due to shared `ENV_LOCK` mutex state.

**Root Cause:** Tests use a shared mutex that causes PoisonError when tests panic in parallel execution.

**Requirements:**
- Refactor ENV_LOCK to use proper async Mutex or remove shared state
- Ensure tests can run in parallel without race conditions
- All 10 failing tests must pass:
  - `test_precedence_cli_none_values_dont_override_env`
  - `test_precedence_cli_overrides_env`
  - `test_precedence_env_config_content_overrides_file`
  - `test_precedence_env_overrides_config_file`
  - `test_precedence_full_chain_integration`
  - `test_precedence_multiple_env_vars_stack`
  - `test_precedence_opencode_dir_overrides_project`
  - `test_precedence_project_config_overrides_global`
  - `test_precedence_provider_api_keys_from_env`
  - `test_load_multi_with_cli_overrides_full_chain`

**Files Affected:**
- `crates/core/src/config.rs` (ENV_LOCK usage)

---

## P2 - Medium Priority (Should Fix Before Phase 6)

| # | Task | Status | Module | Files | Effort |
|---|------|--------|--------|-------|--------|
| P2-1 | Fix TUI keybinding case sensitivity test | ❌ NOT FIXED | tui | `crates/tui/src/keybinding.rs` | Low |
| P2-2 | Fix TUI keybinding Space key test | ❌ NOT FIXED | tui | `crates/tui/src/keybinding.rs` | Low |
| P2-3 | Fix TUI theme hex color parsing test | ❌ NOT FIXED | tui | `crates/tui/src/theme.rs` | Low |
| P2-4 | Fix desktop/web smoke test port conflict | ❌ NOT FIXED | cli | `crates/cli/src/cmd/desktop.rs` | Low |

### P2-1: Fix TUI Keybinding Case Sensitivity Test

**Test:** `keybinding::tests::test_key_parsing_simple`
**Issue:** Assertion failure comparing `Char('P')` to `Char('p')`

**Requirements:**
- Fix case sensitivity handling in key parsing
- Ensure test passes

---

### P2-2: Fix TUI Keybinding Space Key Test

**Test:** `keybinding::tests::test_key_parsing_space`
**Issue:** Assertion failure comparing `Char(' ')` to `Space`

**Requirements:**
- Fix Space vs Char(' ') handling in key parsing
- Ensure test passes

---

### P2-3: Fix TUI Theme Hex Color Parsing Test

**Test:** `theme::tests::test_parse_hex_color`
**Issue:** Color parsing returns wrong value

**Requirements:**
- Fix hex color parsing in theme module
- Ensure correct RGB value is returned
- Ensure test passes

---

### P2-4: Fix Desktop/Web Smoke Test Port Conflict

**Test:** `desktop_web_different_ports`
**Issue:** Test assumes specific port 3000 availability, fails when port is in use

**Requirements:**
- Use dynamic port allocation instead of hardcoded port
- Test should find available port dynamically
- Ensure test passes reliably

---

## Deferred Tasks (Not Critical for Phase 6)

| # | Task | Status | Module | Notes |
|---|------|--------|--------|-------|
| D-1 | Per-agent model override testing | ⚠️ Deferred | llm | Implementation exists, not explicitly tested |
| D-2 | Hidden vs visible agent UI behavior testing | ⚠️ Deferred | agent | Tests exist for invariant, UI not critical |
| D-3 | Remove deprecated `mode` field | ⚠️ Deferred | core | Marked for removal in v4.0 |
| D-4 | Remove deprecated `tools` field | ⚠️ Deferred | core | Marked for removal after migration |

---

## Phase 6 Release Qualification Tasks

**Prerequisites:** All P1 and P2 issues must be resolved.

| # | Task | Status | Notes |
|---|------|--------|-------|
| RQ-1 | End-to-end integration tests | ❌ Pending | Full session lifecycle testing |
| RQ-2 | Performance benchmarking | ❌ Pending | Baseline metrics, regression detection |
| RQ-3 | Security audit | ❌ Pending | Auth flow, permission validation |
| RQ-4 | API error handling verification | ❌ Pending | Consistent error shapes |
| RQ-5 | Cross-module integration | ❌ Pending | MCP, LSP, Tools, Agent integration |

---

## Task Dependencies

```
P1-1 (Config Crate) ─────┬──► RQ-1 (E2E Tests)
                         │
P1-2 (Test Infra) ───────┤
                         │
P2-1 (Keybinding) ───────┤──► Phase 6 Release
P2-2 (Space Key) ────────┤
P2-3 (Theme Color) ──────┤
                         │
P2-4 (Port Conflict) ───┘
```

---

## Test Results Target

**Current:** 610 passed, 14 failed

**Target:** 624 passed, 0 failed

**Tests to Fix:**
- Config tests: 10 failures (P1-2)
- TUI keybinding: 2 failures (P2-1, P2-2)
- TUI theme: 1 failure (P2-3)
- CLI desktop/web: 1 failure (P2-4)

---

## Summary

| Priority | Count | Completed | Remaining |
|----------|-------|-----------|-----------|
| P0 | 3 | 3 | 0 |
| P1 | 2 | 1 | 1 |
| P2 | 4 | 0 | 4 |
| Deferred | 4 | 0 | 4 |
| Phase 6 | 5 | 0 | 5 |

---

*Document Version: 17*
*Last Updated: 2026-04-14*
*Next Action: Fix P1-2 (config tests PoisonError)*
