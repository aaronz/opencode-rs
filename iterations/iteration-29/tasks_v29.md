# Task List - Iteration 29
# OpenCode RS

**Date:** 2026-04-17
**Iteration:** 29

---

## P0 - Critical Tasks

### FR-017: Production unwrap() Elimination

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P0-001 | Fix `unwrap()` in `crates/tools/src/edit.rs:159` | edit.rs | ✅ Done |
| P0-002 | Fix `unwrap()` in `crates/tools/src/web_search.rs:70` | web_search.rs | Not Started |
| P0-003 | Audit all production code for `.unwrap()` occurrences | All crates | ✅ Done |
| P0-004 | Convert high-risk unwrap() locations to proper error propagation | routes/*.rs | ✅ Done |
| P0-005 | Verify zero unwrap() in production (run grep command) | All crates | Not Started |

### FR-029: Error Handling Standardization

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P0-006 | Convert `crates/server/src/routes/` String errors to thiserror | Route handlers | ✅ Done |
| P0-007 | Ensure all library crates use thiserror for typed errors | All lib crates | ✅ Done |
| P0-008 | Verify application crates may use anyhow for flexible context | app crates | ✅ Done |
| P0-009 | Run clippy to verify no error handling warnings | All crates | ✅ Done |

---

## P1 - High Priority Tasks

### FR-018: Test Coverage Enforcement

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P1-001 | Add `cargo-llvm-cov` to CI pipeline | CI config | ✅ Done |
| P1-002 | Set `--fail-under-lines 80` coverage threshold | CI config | Not Started |
| P1-003 | Increase `agent` crate coverage from 45% to 80%+ | agent/* | Not Started |
| P1-004 | Increase `server` crate coverage from 40% to 80%+ | server/* | Not Started |
| P1-005 | Increase `tools` crate coverage from 50% to 80%+ | tools/* | Done |
| P1-006 | Increase `cli` crate coverage from 50% to 80%+ | cli/* | Not Started |
| P1-007 | Increase `llm` crate coverage from 55% to 80%+ | llm/* | Not Started |
| P1-008 | Increase `core` crate coverage from 60% to 80%+ | core/* | ✅ Done |
| P1-009 | Increase `tui` crate coverage from 60% to 80%+ | tui/* | Not Started |
| P1-010 | Increase `storage` crate coverage from 70% to 80%+ | storage/* | Not Started |
| P1-011 | Increase `plugin` crate coverage from 70% to 80%+ | plugin/* | ✅ Done |
| P1-012 | Increase `config` crate coverage from 70% to 80%+ | config/* | ✅ Done |
| P1-013 | Increase `auth` crate coverage from 75% to 80%+ | auth/* | Not Started |
| P1-014 | Verify coverage threshold passes in CI | CI config | ✅ Done |

### FR-028: Visibility Boundary Audit

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P1-015 | Audit `core` crate pub declarations | core/src/* | Not Started |
| P1-016 | Audit `cli` crate pub declarations | cli/src/* | ✅ Done |
| P1-017 | Audit `llm` crate pub declarations | llm/src/* | Not Started |
| P1-018 | Audit `tools` crate pub declarations | tools/src/* | ✅ Done |
| P1-019 | Audit `agent` crate pub declarations | agent/src/* | Not Started |
| P1-020 | Audit `tui` crate pub declarations | tui/src/* | Not Started |
| P1-021 | Audit `lsp` crate pub declarations | lsp/src/* | ✅ Done |
| P1-022 | Audit `storage` crate pub declarations | storage/src/* | Not Started |
| P1-023 | Audit `server` crate pub declarations | server/src/* | ✅ Done |
| P1-024 | Audit `auth` crate pub declarations | auth/src/* | ✅ Done |
| P1-025 | Audit `permission` crate pub declarations | permission/src/* | ✅ Done |
| P1-026 | Audit `plugin` crate pub declarations | plugin/src/* | ✅ Done |
| P1-027 | Audit `git` crate pub declarations | git/src/* | ✅ Done |
| P1-028 | Audit `mcp` crate pub declarations | mcp/src/* | Not Started |
| P1-029 | Audit `sdk` crate pub declarations | sdk/src/* | Not Started |
| P1-030 | Apply `pub(crate)` for internal crate sharing | All crates | Not Started |
| P1-031 | Verify only necessary items marked `pub` | All crates | ✅ Done |

### FR-024: Plugin API Version Stability

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P1-032 | Define plugin ABI version scheme (major.minor.patch) | plugin/* | ✅ Done |
| P1-033 | Document breaking change policy for plugins | plugin/* | Not Started |
| P1-034 | Add version checks to plugin runtime | plugin/src/lib.rs | Not Started |
| P1-035 | Publish plugin API stability policy | documentation | ✅ Done |

### FR-025: WebSocket Streaming Verification

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P1-036 | Verify `routes/ws.rs` WebSocket capability | server/src/routes/ws.rs | Not Started |
| P1-037 | Compare ws module vs SSE functionality | server/src/routes/* | ✅ Done |
| P1-038 | Document WebSocket streaming capabilities | documentation | ✅ Done |

### FR-026: SDK Documentation CI

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P1-039 | Add `cargo doc --no-deps --all-features` to CI | CI config | Not Started |
| P1-040 | Verify SDK doc comments are complete | sdk/src/* | Not Started |
| P1-041 | Consider docs.rs publishing setup | documentation | Not Started |

---

## P2 - Medium Priority Tasks

### FR-023: Unsafe Code SAFETY Comments

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P2-001 | Add SAFETY comment to `crates/plugin/src/lib.rs:661` | plugin/src/lib.rs | ✅ Done |
| P2-002 | Add SAFETY comments to `crates/tui/src/app.rs:4677, 4690` | tui/src/app.rs | Not Started |
| P2-003 | Add SAFETY comments to `crates/server/src/routes/validation.rs:237, 256` | server/src/routes/validation.rs | ✅ Done |
| P2-004 | Audit all `unsafe` blocks for missing SAFETY comments | All crates | ✅ Done |
| P2-005 | Verify all unsafe blocks have proper documentation | All crates | ✅ Done |

### FR-019: Benchmark CI Integration

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P2-006 | Add `cargo bench` to CI pipeline | CI config | ✅ Done |
| P2-007 | Set up performance regression detection | CI config | ✅ Done |
| P2-008 | Document baseline performance metrics | documentation | Not Started |
| P2-009 | Verify benchmark suite runs in CI | CI config | Not Started |

### FR-027: TOML Config Migration

| Task ID | Description | Files | Status |
|---------|-------------|-------|--------|
| P2-010 | Create TOML to JSONC migration tooling | config/* | ✅ Done |
| P2-011 | Implement auto-convert TOML to JSONC on load | config/* | ✅ Done |
| P2-012 | Add deprecation warning for TOML format | config/* | ✅ Done |
| P2-013 | Document TOML to JSONC migration steps | documentation | ✅ Done |
| P2-014 | Consider removing TOML support after transition | config/* | Not Started |

---

## CI Pipeline Updates

| Task ID | Description | Status |
|---------|-------------|--------|
| CI-001 | Add `cargo fmt --check` to CI | ✅ Done |
| CI-002 | Fix clippy warnings in CI | Needed |
| CI-003 | Add `cargo test --lib` to CI | Needed |
| CI-004 | Add `cargo test --test '*'` to CI | Needed |
| CI-005 | Add `cargo build --release` to CI | Needed |
| CI-006 | Add `cargo llvm-cov --fail-under-lines 80` to CI | Needed |
| CI-007 | Add `cargo audit` to CI | Needed |
| CI-008 | Add `cargo deny check` to CI | Needed |
| CI-009 | Add `cargo bench` to CI | Needed |
| CI-010 | Add `cargo doc --no-deps` to CI | Needed |

---

## Summary

| Priority | Total Tasks | Completed | Not Started |
|----------|-------------|-----------|-------------|
| P0 | 9 | 0 | 9 |
| P1 | 41 | 0 | 41 |
| P2 | 19 | 0 | 19 |
| CI | 10 | 0 | 10 |
| **Total** | **79** | **0** | **79** |

---

## Progress Tracking

| Date | P0 | P1 | P2 | CI | Notes |
|------|----|----|----|----|-------|
| 2026-04-17 | 0/9 | 0/41 | 0/19 | 0/10 | Initial task list created |

---

**End of Task List**