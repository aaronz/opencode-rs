# Task List - Iteration 21
**Project:** OpenCode RS - Rust Implementation
**Iteration:** 21
**Date:** 2026-04-15
**Priority:** P0 tasks MUST be completed before P1, P1 before P2

---

## P0 Tasks (Critical - Blocking Release)

### FR-024: Session Execute API Implementation

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P0-024-01 | Create `server/src/routes/execute/types.rs` with ExecuteRequest, ExecuteEvent enums | ✅ Done | | execute/types.rs, execute/mod.rs, routes/mod.rs | None |
| P0-024-02 | Create `server/src/routes/execute/stream.rs` for SSE event formatting | TODO | | NEW | P0-024-01 |
| P0-024-03 | Create `server/src/routes/execute/integration.rs` bridging ToolRegistry to AgentExecutor | TODO | | NEW | P0-024-01, P0-024-02 |
| P0-024-04 | Create `server/src/routes/execute/mod.rs` route handler module | ✅ Done | | execute/mod.rs | P0-024-03 |
| P0-024-05 | Create `server/src/routes/execute.rs` re-export | ✅ Done | | execute.rs | P0-024-04 |
| P0-024-06 | Update `server/src/routes/mod.rs` to include execute routes | TODO | | routes/mod.rs | P0-024-05 |
| P0-024-07 | Update `server/src/lib.rs` to mount `/api/sessions/{id}/execute` route | ✅ Done | | routes/mod.rs | P0-024-06 |
| P0-024-08 | Add `ToolRegistry` discovery integration in execute endpoint | TODO | | execute/integration.rs | tools::ToolRegistry |
| P0-024-09 | Add `AgentExecutor` lifecycle management in execute endpoint | Done | | execute/integration.rs | agent::AgentExecutor |
| P0-024-10 | Add session state persistence after execution | ✅ Done | | execute/integration.rs | storage::SessionStore |
| P0-024-11 | Add request validation (session exists, auth) | TODO | | execute/mod.rs | auth middleware |
| P0-024-12 | Add streaming response with SSE formatting | TODO | | execute/stream.rs | P0-024-02 |
| P0-024-13 | Create integration test file `tests/src/agent_execute_api_tests.rs` | TODO | | NEW | P0-024-07 |
| P0-024-14 | Add test: valid session execute returns 200 | ✅ Done | | tests/src/agent_execute_api_tests.rs | P0-024-13 |
| P0-024-15 | Add test: invalid session returns 404 | ✅ Done | | tests/src/agent_execute_api_tests.rs | P0-024-14 |
| P0-024-16 | Add test: unauthenticated returns 401 | ✅ Done | | tests/src/agent_execute_api_tests.rs | P0-024-15 |
| P0-024-17 | Add test: tool execution results in response | ✅ Done | | tests/src/agent_execute_api_tests.rs | P0-024-16 |
| P0-024-18 | Verify build passes: `cargo build -p opencode-server` | ✅ Done | | | P0-024-17 |
| P0-024-19 | Verify tests pass: `cargo test -p opencode-integration-tests agent_execute` | TODO | | | P0-024-18 |

### FR-025: WebSocket Agent Streaming

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P0-025-01 | Create `crates/agent/src/events.rs` with AgentEvent enum | ✅ Done | | NEW (agent crate) | None |
| P0-025-02 | Implement `AgentEventEmitter` trait for AgentExecutor | ✅ Done | | agent/src/events.rs | P0-025-01 |
| P0-025-03 | Create `server/src/routes/ws/session_hub.rs` for event broadcast | ✅ Done | | NEW | None |
| P0-025-04 | Update `server/src/routes/ws.rs` to handle session-specific routing | ✅ Done | | routes/ws.rs | P0-025-03 |
| P0-025-05 | Integrate execute endpoint with session hub for event emission | TODO | | execute/integration.rs | P0-025-03, P0-024-03 |
| P0-025-06 | Add WebSocket connection lifecycle management | TODO | | routes/ws.rs | P0-025-04 |
| P0-025-07 | Add multiple client support per session (Arc<RwLock>) | TODO | | ws/session_hub.rs | P0-025-03 |
| P0-025-08 | Add client disconnect graceful handling | TODO | | routes/ws.rs | P0-025-06 |
| P0-025-09 | Create test file `tests/src/ws_streaming_tests.rs` | ✅ Done | | NEW | P0-025-05 |
| P0-025-10 | Add test: WebSocket connects successfully | ✅ Done | | tests/src/ws_streaming_tests.rs | P0-025-09 |
| P0-025-11 | Add test: tool call events stream in real-time | ✅ Done | | tests/src/ws_streaming_tests.rs | P0-025-10 |
| P0-025-12 | Add test: multiple clients receive same events | ✅ Done | | tests/src/ws_streaming_tests.rs | P0-025-11 |
| P0-025-13 | Add test: client disconnect doesn't crash server | TODO | | tests/src/ws_streaming_tests.rs | P0-025-12 |
| P0-025-14 | Verify build: `cargo build -p opencode-server` | ✅ Done | | | P0-025-13 |
| P0-025-15 | Verify tests pass: `cargo test -p opencode-integration-tests ws` | ✅ Done | | | P0-025-14 |

---

## P1 Tasks (High Priority)

### FR-026: Server Status Endpoint

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P1-026-01 | Create `server/src/routes/status.rs` | TODO | | NEW | None |
| P1-026-02 | Define `StatusResponse` struct with all fields | ✅ Done | | status.rs | None |
| P1-026-03 | Collect version info | ✅ Done | | status.rs | None |
| P1-026-04 | Collect uptime_seconds | ✅ Done | | status.rs | None |
| P1-026-05 | Collect active/total session counts | ✅ Done | | status.rs | storage |
| P1-026-06 | Collect provider status from LLM registry | TODO | | status.rs | llm |
| P1-026-07 | Collect plugin status from plugin system | TODO | | status.rs | plugin |
| P1-026-08 | Add no-auth route to App | ✅ Done | | lib.rs | P1-026-01 |
| P1-026-09 | Create test file `tests/src/status_endpoint_tests.rs` | TODO | | NEW | P1-026-08 |
| P1-026-10 | Add test: GET /api/status returns 200 | ✅ Done | | status_endpoint_tests.rs | P1-026-09 |
| P1-026-11 | Add test: response contains all required fields | ✅ Done | | status_endpoint_tests.rs | P1-026-10 |
| P1-026-12 | Add test: endpoint accessible without auth | ✅ Done | | status_endpoint_tests.rs | P1-026-11 |
| P1-026-13 | Verify build and tests pass | ✅ Done | | | P1-026-12 |

### FR-027: Streaming Response Support

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P1-027-01 | Modify `server/src/routes/run.rs` run_prompt function | TODO | | routes/run.rs | None |
| P1-027-02 | Add stream parameter to run_prompt | ✅ Done | | run.rs | None |
| P1-027-03 | Implement token-by-token SSE streaming | TODO | | run.rs | P1-027-02 |
| P1-027-04 | Add `Accept: text/event-stream` header detection | TODO | | run.rs | P1-027-03 |
| P1-027-05 | Handle connection interruption gracefully | ✅ Done | | run.rs | P1-027-04 |
| P1-027-06 | Create test file `tests/src/streaming_response_tests.rs` | TODO | | NEW | P1-027-05 |
| P1-027-07 | Add test: SSE streaming with curl equivalent | TODO | | streaming_response_tests.rs | P1-027-06 |
| P1-027-08 | Add test: tokens arrive progressively | TODO | | streaming_response_tests.rs | P1-027-07 |
| P1-027-09 | Add test: connection close terminates cleanly | TODO | | streaming_response_tests.rs | P1-027-08 |
| P1-027-10 | Verify build and tests pass | TODO | | | P1-027-09 |

### FR-028: Permission Reply Integration

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P1-028-01 | Review current `server/src/routes/permission.rs` handler | TODO | | routes/permission.rs | None |
| P1-028-02 | Connect permission_reply to PermissionManager | TODO | | permission.rs | permission::PermissionManager |
| P1-028-03 | Update ApprovalQueue on decision | TODO | | permission.rs | P1-028-02 |
| P1-028-04 | Trigger re-evaluation of pending requests | TODO | | permission.rs | P1-028-03 |
| P1-028-05 | Add audit logging for decisions | TODO | | permission.rs | P1-028-04 |
| P1-028-06 | Create test file `tests/src/permission_integration_tests.rs` | TODO | | NEW | P1-028-05 |
| P1-028-07 | Add test: approval triggers tool execution | TODO | | permission_integration_tests.rs | P1-028-06 |
| P1-028-08 | Add test: denial returns PermissionDenied error | TODO | | permission_integration_tests.rs | P1-028-07 |
| P1-028-09 | Add test: decision logged to audit trail | TODO | | permission_integration_tests.rs | P1-028-08 |
| P1-028-10 | Verify build and tests pass | TODO | | | P1-028-09 |

---

## P2 Tasks (Medium Priority)

### FR-023: ratatui-testing Framework Completion

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P2-023-01 | Review current PtySimulator implementation | TODO | | ratatui-testing/src/pty.rs | None |
| P2-023-02 | Complete PtySimulator event injection (KeyEvent, MouseEvent) | TODO | | ratatui-testing/src/pty.rs | P2-023-01 |
| P2-023-03 | Implement BufferDiff cell-by-cell comparison | TODO | | ratatui-testing/src/diff.rs | None |
| P2-023-04 | Implement StateTester state capture | TODO | | ratatui-testing/src/state.rs | None |
| P2-023-05 | Implement TestDsl fluent API | TODO | | ratatui-testing/src/dsl.rs | P2-023-02, P2-023-03, P2-023-04 |
| P2-023-06 | Implement CliTester process management | TODO | | ratatui-testing/src/cli.rs | None |
| P2-023-07 | Add tests for each component | TODO | | ratatui-testing/tests/ | P2-023-06 |
| P2-023-08 | Verify build: `cargo build -p ratatui-testing` | TODO | | | P2-023-07 |
| P2-023-09 | Verify tests: `cargo test -p ratatui-testing` | TODO | | | P2-023-08 |

### FR-029: Hook Determinism Test

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P2-029-01 | Add deterministic hook test in `crates/plugin/src/lib.rs` | TODO | | plugin/src/lib.rs | None |
| P2-029-02 | Test 100 iterations for consistent ordering | TODO | | plugin/src/lib.rs | P2-029-01 |
| P2-029-03 | Document expected ordering behavior in test | TODO | | plugin/src/lib.rs | P2-029-02 |
| P2-029-04 | Verify test passes: `cargo test -p opencode-plugin hook_determinism` | TODO | | | P2-029-03 |

### FR-030: Security Test Suite

| ID | Task | Status | Owner | Files | Dependencies |
|----|------|--------|-------|-------|--------------|
| P2-030-01 | Create `tests/src/security_tests.rs` | TODO | | NEW | None |
| P2-030-02 | Add SQL injection tests for session operations | TODO | | security_tests.rs | P2-030-01 |
| P2-030-03 | Add SQL injection tests for message operations | TODO | | security_tests.rs | P2-030-02 |
| P2-030-04 | Add path traversal tests for file operations | TODO | | security_tests.rs | P2-030-03 |
| P2-030-05 | Add request validation tests for all endpoints | TODO | | security_tests.rs | P2-030-04 |
| P2-030-06 | Verify all security tests pass | TODO | | | P2-030-05 |

---

## Verification & Release Tasks

| ID | Task | Status | Dependencies |
|----|------|--------|--------------|
| VR-01 | Run full test suite: `cargo test --all-features` | TODO | All P0, P1, P2 |
| VR-02 | Run clippy: `cargo clippy --all-targets --all-features -- -D warnings` | TODO | VR-01 |
| VR-03 | Run formatting: `cargo fmt --all -- --check` | TODO | VR-02 |
| VR-04 | Run doc tests: `cargo test --doc` | TODO | VR-03 |
| VR-05 | Run benchmarks: `cargo bench --all` | TODO | VR-04 |
| VR-06 | Memory profiling | TODO | VR-05 |
| VR-07 | Security audit | TODO | VR-06 |
| VR-08 | Documentation completeness check | TODO | VR-07 |

---

## Task Statistics

| Priority | Total Tasks | Completed | In Progress | TODO |
|----------|-------------|-----------|-------------|------|
| P0 | 34 | 0 | 0 | 34 |
| P1 | 33 | 0 | 0 | 33 |
| P2 | 20 | 0 | 0 | 20 |
| VR | 8 | 0 | 0 | 8 |
| **Total** | **95** | **0** | **0** | **95** |

---

## Progress Tracking

| Date | P0 Done | P1 Done | P2 Done | VR Done | Notes |
|------|---------|---------|---------|---------|-------|
| 2026-04-15 | 0/34 | 0/33 | 0/20 | 0/8 | Plan created |

---

*Task List Version: 21*
*Generated: 2026-04-15*
*Last Updated: 2026-04-15*
