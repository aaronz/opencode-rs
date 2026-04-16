# Task List - Iteration 22

**Project:** OpenCode RS
**Iteration:** 22
**Phase:** HTTP API Execution Gap Resolution
**Last Updated:** 2026-04-16
**Priority:** P0 > P1 > P2

---

## P0 Tasks (Critical - Must Complete Before Release)

### Task 1: Implement Session Execute Endpoint Handler

**FR-ID:** FR-031
**Priority:** P0
**Status:** ✅ Done

**Description:**
Create `POST /api/sessions/{id}/execute` endpoint handler with full agent execution and tool access.

**Files to Create:**
- `crates/server/src/routes/execute/mod.rs` - Main handler implementation (exists)
- `crates/server/src/routes/execute/types.rs` - ExecuteEvent types (exists)
- `crates/server/src/routes/execute/stream.rs` - SSE streaming (exists)
- `crates/server/src/routes/execute/integration.rs` - Agent execution loop (exists)

**Files to Modify:**
- `crates/server/src/routes/session.rs` - Add route registration
- `crates/server/src/routes/execute/types.rs` - Already exists with types

**Steps:**
1. Create `execute.rs` with `execute_session` handler
2. Integrate `AgentExecutor` for agent lifecycle
3. Integrate `ToolRegistry` for tool discovery
4. Implement SSE streaming for ExecuteEvent types (tool_call, tool_result, message, complete, error)
5. Register route: `cfg.route("/{id}/execute", web::post().to(execute_session))`
6. Add authentication middleware

**Dependencies:**
- `crates/agent/src/lib.rs` - AgentExecutor
- `crates/tools/src/lib.rs` - ToolRegistry

**Acceptance Criteria:**
- [ ] `POST /api/sessions/{session_id}/execute` returns 200 for valid session
- [ ] Tools from ToolRegistry are discovered and executed
- [ ] Tool execution results appear in response
- [ ] Invalid session ID returns 404
- [ ] Unauthenticated request returns 401
- [ ] SSE streaming works with `Accept: text/event-stream`

---

### Task 2: Refactor Run Endpoint to Use AgentExecutor

**FR-ID:** FR-032
**Priority:** P0
**Status:** Not Started

**Description:**
Refactor `run_prompt` in `run.rs` to integrate with AgentExecutor and ToolRegistry for full tool-aware execution.

**Files to Modify:**
- `crates/server/src/routes/run.rs`

**Steps:**
1. Refactor `run_prompt` to use `AgentExecutor` instead of basic LLM chat
2. Integrate `ToolRegistry` for tool discovery and execution
3. Enable streaming response with SSE for progressive token output
4. Propagate tool errors properly to response

**Dependencies:**
- `crates/agent/src/lib.rs` - AgentExecutor
- `crates/tools/src/lib.rs` - ToolRegistry

**Acceptance Criteria:**
- [ ] `POST /api/run` executes tools via AgentExecutor
- [ ] Tool results appear in response
- [ ] Streaming works for LLM tokens
- [ ] Tool errors are properly propagated

---

## P1 Tasks (High Priority - Next Sprint)

### Task 3: Implement Server Status Endpoint

**FR-ID:** FR-033
**Priority:** P1
**Status:** ✅ Done

**Description:**
Add `GET /api/status` endpoint for server health and status information.

**Files to Create:**
- `crates/server/src/routes/status.rs`

**Steps:**
1. Create `status.rs` with status handler
2. Implement response with version, status, uptime_seconds, session stats
3. Include provider status array
4. Include plugin list
5. No authentication required

**Route:** `GET /api/status`

**Response:**
```json
{
  "version": "1.0.0",
  "status": "running",
  "uptime_seconds": 3600,
  "active_sessions": 5,
  "total_sessions": 142,
  "providers": [{"name": "openai", "status": "ready", "model": "gpt-4"}],
  "plugins": [{"name": "example-plugin", "version": "1.0.0", "status": "loaded"}]
}
```

**Acceptance Criteria:**
- [ ] `GET /api/status` returns 200
- [ ] Response contains all specified fields
- [ ] Endpoint does not require authentication
- [ ] Response time < 100ms

---

### Task 4: Integrate WebSocket with Agent Runtime

**FR-ID:** FR-034
**Priority:** P1
**Status:** ✅ Done

**Description:**
Connect WebSocket endpoints to agent runtime for real-time streaming of agent execution output.

**Files to Modify:**
- `crates/server/src/routes/ws.rs`
- `crates/server/src/routes/acpws.rs`

**Steps:**
1. Create event emitter in agent runtime
2. Broadcast AgentEvent to connected WebSocket clients
3. Support multiple concurrent connections per session
4. Handle client disconnection gracefully

**Event Types:**
```rust
enum AgentEvent {
    ToolCall { tool: String, params: Value },
    ToolResult { tool: String, result: Value },
    Token { content: String },
    Message { role: String, content: String },
    Error { error: String },
    Complete { summary: String },
}
```

**Acceptance Criteria:**
- [ ] WebSocket receives tool call events in real-time
- [ ] WebSocket receives token-by-token LLM output
- [ ] Multiple clients can connect to same session
- [ ] Client disconnection doesn't crash server

---

### Task 5: Connect Permission Reply to Permission System

**FR-ID:** FR-035
**Priority:** P1
**Status:** Partial (handler exists, not connected)

**Description:**
Connect `permission_reply` handler to actual permission system to process user approval/denial decisions.

**Files to Modify:**
- `crates/server/src/routes/permission.rs`

**Steps:**
1. Connect `permission_reply` to `PermissionManager`
2. Update `ApprovalQueue` with user decisions
3. Trigger re-evaluation of pending requests after approval
4. Log decisions to audit trail

**Acceptance Criteria:**
- [ ] User approval triggers tool execution
- [ ] User denial returns PermissionDenied error
- [ ] Audit log records decision

---

### Task 6: Implement SSE Streaming Response

**FR-ID:** FR-036
**Priority:** P1
**Status:** ✅ Done

**Description:**
Implement chunked/streaming response for run and execute endpoints.

**Files to Modify:**
- `crates/server/src/routes/run.rs`
- `crates/server/src/routes/execute.rs`

**Steps:**
1. Detect `Accept: text/event-stream` header
2. Stream LLM tokens as they arrive using SSE format
3. Include proper SSE formatting (`data: {...}\n\n`)
4. Handle connection interruption gracefully

**Acceptance Criteria:**
- [ ] `curl -H "Accept: text/event-stream" -d '{"prompt":"hi"}' /api/run` streams
- [ ] Each token arrives as separate SSE event
- [ ] Connection close terminates cleanly

---

## P2 Tasks (Medium Priority - Future)

### Task 7: Complete SDK Documentation

**FR-ID:** FR-037
**Priority:** P2
**Status:** Incomplete

**Description:**
Complete SDK documentation for publishing to crates.io.

**Files to Modify:**
- `crates/sdk/src/**/*.rs`

**Steps:**
1. Add `cargo doc` comments to all public items
2. Create working examples for all public APIs
3. Prepare README with installation instructions

**Acceptance Criteria:**
- [ ] `cargo doc` generates complete documentation
- [ ] All public items have doc comments
- [ ] Examples compile and run

---

### Task 8: Document LSP Integration

**FR-ID:** FR-038
**Priority:** P2
**Status:** Incomplete

**Description:**
Document IDE extension support for LSP integration.

**Files to Create:**
- Documentation in `crates/lsp/`

**Steps:**
1. Document VSCode LSP client setup
2. Document Neovim LSP setup
3. List supported LSP features

**Acceptance Criteria:**
- [ ] VSCode LSP setup documented
- [ ] Neovim LSP setup documented
- [ ] List of supported LSP features

---

## Task Summary

| Task | FR-ID | Priority | Status |
|------|-------|----------|--------|
| 1. Session Execute Endpoint Handler | FR-031 | P0 | ✅ Done |
| 2. Run Endpoint Tool Integration | FR-032 | P0 | Not Started |
| 3. Server Status Endpoint | FR-033 | P1 | ✅ Done |
| 4. WebSocket Agent Streaming | FR-034 | P1 | ✅ Done |
| 5. Permission Reply Integration | FR-035 | P1 | Partial |
| 6. SSE Streaming Response | FR-036 | P1 | ✅ Done |
| 7. SDK Documentation | FR-037 | P2 | ✅ Done |
| 8. LSP Integration Documentation | FR-038 | P2 | Incomplete |

---

## Technical Debt Items

| ID | Item | Severity | Module | Status |
|----|------|----------|--------|--------|
| TD-020 | Execute endpoint types unused | Medium | server | Not fixed |
| TD-021 | Run endpoint tool integration | High | server | Not fixed |
| TD-022 | Permission reply not connected | Medium | permission | Not fixed |
| TD-023 | WebSocket agent integration | High | server | Not fixed |
| TD-024 | Streaming response | Medium | server | Not fixed |

*Tasks generated from spec_v22.md (Iteration 22)*
