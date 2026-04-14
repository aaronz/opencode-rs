# Implementation Plan - Iteration 19

**Version:** 1.0  
**Generated:** 2026-04-14  
**Based on:** spec_v19.md, gap-analysis.md

---

## 1. Priority Classification

### P0 - Blocking (All Resolved ✅)
All P0 blocking issues from prior iterations remain resolved.

### P1 - High Priority (3 items)

| # | Issue | Module | Status |
|---|-------|--------|--------|
| P1.1 | `desktop_web_different_ports` test failing - port conflict | cli | NOT FIXED |
| P1.2 | Phase 6 Release Qualification not started | all | NOT STARTED |
| P1.3 | `test_bedrock_credential_resolution_bearer_token_priority` fails with `--all-features` | llm | NOT FIXED |

### P2 - Medium Priority (3 items)

| # | Issue | Module | Status |
|---|-------|--------|--------|
| P2.1 | Trailing whitespace in `storage/src/service.rs` | storage | NOT FIXED |
| P2.2 | Deprecated `mode` field | config | Deferred |
| P2.3 | Deprecated `tools` field | config | Deferred |

---

## 2. P1 Implementation Tasks

### Task P1.1: Fix `desktop_web_different_ports` Test

**Location:** `crates/cli/tests/e2e_desktop_web_smoke.rs:162`

**Problem:** Test uses hardcoded port 3000 which conflicts with other processes.

**Solution:**
```rust
let listener = TcpListener::bind("127.0.0.1:0")?;
let port = listener.local_addr()?.port();
```

**Steps:**
1. Read test file to understand current implementation
2. Modify test to use dynamic port allocation via `TcpListener::bind("127.0.0.1:0")`
3. Verify fix by running test

---

### Task P1.2: Begin Phase 6 Release Qualification

**Phase 6 Scope:**
- End-to-end integration tests
- Performance benchmarking
- Security audit
- Observability validation
- Documentation review

**Steps:**
1. Create Phase 6 test plan document
2. Set up e2e test infrastructure
3. Define benchmarks for key operations
4. Create security audit checklist
5. Validate observability/tracing integration

---

### Task P1.3: Fix Bedrock Test Environment Pollution

**Location:** `crates/llm/src/bedrock.rs:266`

**Problem:** AWS env vars set by other tests pollute this test when run with `--all-features`.

**Solution Options:**
1. Use `temp_env` pattern for environment variable isolation
2. Run test in separate process
3. Clear env vars before test execution

**Steps:**
1. Read bedrock test file to understand current setup
2. Implement proper environment isolation using `temp_env::(predicate)`
3. Verify fix with `--all-features` flag

---

## 3. P2 Implementation Tasks

### Task P2.1: Fix Trailing Whitespace

**Location:** `crates/storage/src/service.rs:317, 340, 363, 386, 391`

**Solution:** Run `cargo fmt --all`

**Steps:**
1. Run `cargo fmt --all` on storage crate
2. Verify trailing whitespace is removed
3. Commit fix

---

## 4. Phase 6 Release Qualification Plan

### 6.1 End-to-End Integration Tests

| Test Area | Coverage Target | Status |
|-----------|-----------------|--------|
| Session lifecycle | Create, read, update, delete, fork | Pending |
| Tool execution | Registry, permission, validation, cache | Pending |
| MCP protocol | Local/remote server, OAuth, tool call | Pending |
| Agent runtime | Primary agent, subagent, permission inheritance | Pending |
| Config system | Load, precedence, variable expansion | Pending |

### 6.2 Performance Benchmarks

| Metric | Baseline | Target | Status |
|--------|----------|--------|--------|
| Tool registry lookup | TBD | < 1ms | Pending |
| Session creation | TBD | < 100ms | Pending |
| MCP tool call (local) | TBD | < 50ms | Pending |
| MCP tool call (remote) | TBD | < 500ms | Pending |

### 6.3 Security Audit Checklist

- [ ] Credential storage encryption validation
- [ ] Permission boundary enforcement
- [ ] MCP server authentication flow
- [ ] Input validation on all API endpoints
- [ ] SQL injection prevention (storage layer)
- [ ] Secrets exposure in logs

### 6.4 Observability Validation

- [ ] Tracing integration end-to-end
- [ ] Error reporting consistency
- [ ] Metrics collection and exposure

---

## 5. Timeline

| Week | Focus |
|------|-------|
| Current | P1.1: Fix desktop_web test |
| Current | P1.3: Fix Bedrock test isolation |
| Current | P2.1: Run cargo fmt |
| Next | P1.2: Begin Phase 6 qualification |

---

## 6. Dependencies

- P1.2 (Phase 6) depends on P1.1 and P1.3 being resolved first
- No external dependencies for P1.1, P1.3, P2.1

---

**Document Version:** 1.0  
**Last Updated:** 2026-04-14
