# Task Checklist v9

**Version:** 9.0  
**Generated:** 2026-04-12  
**Based on:** Spec v9 and Gap Analysis Iteration 9  
**Status:** Active

---

## P0 Tasks (Blocking - Must Fix)

### P0-9: ✅ Done

#### ratatui-testing (1 error)
- [ ] Add `impl Default for StateTester` in `ratatui-testing/src/state.rs:6`

#### opencode-core (17 errors)

##### config.rs
- [x] Fix deprecated `AgentMode` enum usage (line 436) - 2 occurrences
- [x] Fix deprecated `AgentConfig::mode` field usage (line 2771)
- [ ] Fix `question_mark` - rewrite with `?` operator (line 1594)
- [ ] Fix `needless_borrows_for_generic_args` - remove unnecessary borrow (line 2068)

##### command.rs
- [x] Fix deprecated `AgentConfig::mode` field usage (line 567)

##### session_sharing.rs
- [ ] Fix `redundant_closure` - use `ok_or` instead (line 323)
- [ ] Fix `map_entry` - use entry API (line 225)

##### crash_recovery.rs
- [ ] Fix `and_then` → `map` (line 241)

##### skill.rs
- [ ] Fix `very_complex_type` - factor into type definitions
- [ ] Fix `&PathBuf` → `&Path` (5 occurrences at line 116)

---

## P1 Tasks (Important)

### P1-3: ✅ Done
- [x] Add deprecation warnings for `mode` field (config.rs, command.rs)
- [x] Document complete removal plan for v4.0
- [x] Track `tools`, `theme`, `keybinds` for future removal

### P1-10: ✅ Done
- [x] Mark as experimental in documentation
- [x] Add experimental flag to variant/reasoning budget feature

---

## P2 Tasks (Deferred)

### P2-16: Remaining Clippy Warnings
- [x] Review and address remaining clippy warnings (non-blocking)
  - Fixed test compilation errors in opencode-llm and opencode-lsp
  - Used correct module paths for importing re-exported types in test modules

### P2-17: Per-Crate Test Backlog
- [ ] Continue per-crate test implementation
- [ ] Address test coverage gaps

---

## Verification Tasks

### After P0-9 Fixes
- [ ] Run `cargo clippy --all -- -D warnings` - must pass
- [ ] Run `cargo build --release` - must compile
- [ ] Run `cargo test` - all tests must pass

### Phase 6 Release Qualification
- [ ] Verify non-functional baselines
- [ ] Complete Phase 6 release qualification

---

## Completed Tasks (Iteration 9)

### P0 Blockers Fixed
- [x] P0-8: Clippy unreachable pattern (permission/models.rs)
- [x] P0-new-1: Git crate syntax error
- [x] P0-new-2: Desktop WebView integration
- [x] P0-new-3: ACP HTTP+SSE transport

### P1 Issues Fixed
- [x] P1-2: Circular variable expansion detection
- [x] P1-5: Multiline input terminal support
- [x] P1-7: TUI Plugin dialogs incomplete
- [x] P1-8: TUI Plugin slots system incomplete
- [x] P1-9: Session sharing between interfaces
- [x] P1-10: Permission inheritance edge cases
- [x] P1-11: Request validation edge cases

### P2 Issues Fixed
- [x] P2-1: Project VCS worktree root distinction
- [x] P2-2: Workspace path validation
- [x] P2-4: Deterministic collision resolution
- [x] P2-5: Result caching invalidation
- [x] P2-6: Per-server OAuth token storage
- [x] P2-7: Context cost warnings
- [x] P2-8: Experimental LSP tool testing
- [x] P2-9: API error shape consistency
- [x] P2-10: Plugin cleanup/unload
- [x] P2-11: Shell prefix (`!`) handler
- [x] P2-12: Home view completion
- [x] P2-13: LLM reasoning budget
- [x] P2-14: GitLab Duo marking
- [x] P2-15: Git test cleanup

### Dead Code Cleanup (DC-1 through DC-10)
- [x] DC-1: Unused `Message` import
- [x] DC-2: Unused `SecretStorage` methods
- [x] DC-3: Unused `e` variable in lsp_tool
- [x] DC-4: Unused `body` variable in github
- [x] DC-5: `open_browser` function unused
- [x] DC-6: `format_time_elapsed` function unused
- [x] DC-7: Unused `complete` variable
- [x] DC-8: Unused `models_url` function
- [x] DC-9: Unused `ChatStreamChunk` struct
- [x] DC-10: Unused `role` field

---

## Summary

| Category | Total | Completed | Pending |
|----------|-------|-----------|---------|
| P0 Blockers | 1 | 0 | 1 |
| P1 Issues | 2 | 1 | 1 |
| P2 Issues | 2 | 0 | 2 |
| Verification | 5 | 0 | 5 |

**Remaining:** 10 tasks (all waiting on P0-9 fix)

---

*Task list generated: 2026-04-12*
*Iteration: 9*
