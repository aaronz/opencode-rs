# Task List v28: Rust Conventions Compliance

**Date:** 2026-04-17  
**Iteration:** 28  
**Total Tasks:** 38  
**P0:** 11 | **P1:** 15 | **P2:** 12

---

## P0 Critical Tasks (Blocker - Must Complete First)

### unwrap() Elimination

| Task ID | Description | File | Line | Estimated | Status |
|---------|-------------|------|------|-----------|--------|
| T-001 | Fix `unwrap()` on Option - index lookup | `crates/tools/src/edit.rs` | 159 | 30 min | ✅ Done |
| T-002 | Fix `unwrap()` on Option - API key | `crates/tools/src/web_search.rs` | 70 | 30 min | ✅ Done |
| T-003 | Audit all production `.unwrap()` calls | All crates | - | 4 hours | ✅ Done |

### Server Route Error Refactoring

| Task ID | Description | File | Estimated | Status |
|---------|-------------|------|-----------|--------|
| T-004 | Convert String errors to thiserror enums | `crates/server/src/routes/*.rs` | 8 hours | ✅ Done |
| T-005 | Add SAFETY comments to unsafe blocks | `crates/server/src/routes/validation.rs:237,256` | 30 min | ✅ Done |

### Integration Test Fixes

| Task ID | Description | File | Estimated | Status |
|---------|-------------|------|-----------|--------|
| T-006 | Fix tool registry read tool test | `tests/src/agent_tool_tests.rs` | 1 hour | ✅ Done |
| T-007 | Fix tool registry write tool test | `tests/src/agent_tool_tests.rs` | 1 hour | ✅ Done |
| T-008 | Fix path normalization traversal test | `tests/src/security_tests.rs` | 1 hour | ✅ Done |
| T-009 | Fix message content sanitization test | `tests/src/security_tests.rs` | 1 hour | Failing |
| T-010 | Fix XSS prevention test | `tests/src/security_tests.rs` | 1 hour | ✅ Done |
| T-011 | Fix write tool path validation test | `tests/src/tool_registry_audit_tests.rs` | 1 hour | ✅ Done |

---

## P1 High Priority Tasks

### Visibility Audit

| Task ID | Description | File | Estimated | Status |
|---------|-------------|------|-----------|--------|
| T-012 | Audit pub visibility in core | `crates/core/src/` | 4 hours | ✅ Done |
| T-013 | Reduce pub to pub(crate) in core | `crates/core/src/` | 8 hours | ✅ Done |
| T-014 | Audit pub visibility in tools | `crates/tools/src/` | 2 hours | Not Started |
| T-015 | Reduce pub to pub(crate) in tools | `crates/tools/src/` | 4 hours | Not Started |
| T-016 | Audit pub visibility in agent | `crates/agent/src/` | 4 hours | Not Started |

### Test Coverage

| Task ID | Description | Target | Delta | Estimated | Status |
|---------|-------------|--------|-------|-----------|--------|
| T-017 | Increase core crate coverage | 80% | +20% | 2 days | ✅ Done |
| T-018 | Increase tools crate coverage | 80% | +30% | 3 days | Not Started |
| T-019 | Increase agent crate coverage | 80% | +35% | 3 days | Not Started |
| T-020 | Increase server crate coverage | 80% | +40% | 3 days | Not Started |
| T-021 | Increase llm crate coverage | 80% | +25% | 2 days | Not Started |
| T-022 | Add cargo-llvm-cov CI gate | 80% threshold | - | 4 hours | Not Started |

### Error Handling

| Task ID | Description | Estimated | Status |
|---------|-------------|-----------|--------|
| T-023 | Migrate legacy error variants to typed errors | 2 days | Partial |

---

## P2 Medium Priority Tasks

### Pattern Adoption

| Task ID | Description | Estimated | Status |
|---------|-------------|-----------|--------|
| T-024 | Add mockall dependency for trait mocking | 2 days | Not Started |
| T-025 | Add service layer to server routes | 1 week | Not Started |
| T-026 | Expand builder pattern to server routes | 1 week | Not Started |
| T-027 | Add newtypes: SessionId, UserId, ProjectId | 2 days | Not Started |
| T-028 | Audit enum matching for exhaustiveness | 2 days | Not Started |
| T-029 | Add rstest for parameterized tests | 2 days | Not Started |
| T-030 | Seal additional traits | 1 day | Not Started |
| T-031 | Extend repository pattern to all data access | 1 week | Not Started |

### Code Quality

| Task ID | Description | Estimated | Status |
|---------|-------------|-----------|--------|
| T-032 | Audit let mut usage in session.rs | 1 day | Not Started |
| T-033 | Add SAFETY comments to unsafe blocks | `crates/plugin/src/lib.rs:661` | 30 min | Not Started |
| T-034 | Add SAFETY comments to unsafe blocks | `crates/tui/src/app.rs:4677,4690` | 30 min | Not Started |

---

## Task Summary by Priority

```
P0 Tasks: 11 (Blocker - Must Fix First)
  - unwrap() fixes: 3
  - Server errors: 2
  - Integration tests: 6

P1 Tasks: 15 (High Priority)
  - Visibility audit: 5
  - Test coverage: 6
  - Error handling: 1
  - CI gate: 1
  - Repository pattern: 2

P2 Tasks: 12 (Medium Priority)
  - Pattern adoption: 8
  - Code quality: 4
```

---

## Progress Tracking

| Week | Focus | Target Tasks |
|------|-------|--------------|
| Week 1-2 | P0 Fixes | T-001 to T-011 |
| Week 3-4 | P1 Foundation | T-012 to T-023 |
| Week 5-6 | Coverage Expansion | T-017 to T-022 |
| Week 7-8 | P2 Patterns | T-024 to T-031 |
| Week 9+ | Polish | T-032 to T-034, remaining P1/P2 |

---

## Definition of Done

- [ ] All P0 tasks completed
- [ ] `cargo test --all` passes (136+ tests)
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo llvm-cov --fail-under-lines 80` passes
- [ ] Zero `.unwrap()` in production code
- [ ] All unsafe blocks have SAFETY comments
