# Implementation Plan - Iteration 21
**Project:** OpenCode RS - Rust Implementation
**Iteration:** 21
**Date:** 2026-04-15
**Phase:** Gap Resolution & P0 Implementation

---

## 1. Executive Summary

This plan addresses the critical gaps identified in Iteration-21 gap analysis. The implementation is prioritized with P0 (blocking) items first, followed by P1 (high priority) and P2 (medium priority).

### Priority Order:
1. **P0 (Critical):** FR-024 (Execute API), FR-025 (WebSocket Streaming)
2. **P1 (High):** FR-026 (Status Endpoint), FR-027 (Streaming Response), FR-028 (Permission Reply)
3. **P2 (Medium):** FR-023 (ratatui-testing), FR-029 (Hook Test), FR-030 (Security Tests)

---

## 2. P0 Implementation Plan

### 2.1 FR-024: Session Execute API

**Endpoint:** `POST /api/sessions/{id}/execute`

#### 2.1.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      POST /api/sessions/{id}/execute           │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Auth Middleware                              │
│                 (Validate JWT token)                            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Session Validation                           │
│            (Verify session exists, get mode)                   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Request Parsing                              │
│         { prompt, mode?: build|plan|general, stream?: bool }   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                 ToolRegistry Integration                         │
│           (Discover available tools for session)                │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                   AgentExecutor                                  │
│        (Create agent based on mode, setup tool registry)        │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
            ┌───────────────┐       ┌───────────────┐
            │  SSE Stream   │       │  JSON Response│
            │ (stream:true) │       │ (stream:false)│
            └───────────────┘       └───────────────┘
                    │                       │
                    ▼                       ▼
            ┌─────────────────────────────────────────────┐
            │        Server-Sent Events Format           │
            │  event: tool_call  → { tool, params }       │
            │  event: tool_result → { tool, result }      │
            │  event: message   → { role, content }       │
            │  event: complete  → { session_state }       │
            │  event: error     → { error }               │
            └─────────────────────────────────────────────┘
```

#### 2.1.2 File Structure Changes

```
crates/server/
├── src/
│   ├── routes/
│   │   ├── mod.rs           (add: mod execute;)
│   │   ├── execute.rs       (NEW: execute endpoint)
│   │   └── execute/
│   │       ├── mod.rs
│   │       ├── types.rs     (Request/Response types)
│   │       ├── stream.rs    (SSE event formatting)
│   │       └── integration.rs (ToolRegistry→AgentExecutor bridge)
│   └── lib.rs                (add execute routes to App)
```

#### 2.1.3 Implementation Steps

| Step | Action | Files Modified |
|------|--------|----------------|
| 1 | Create `server/src/routes/execute/types.rs` with request/response types | NEW |
| 2 | Create `server/src/routes/execute/stream.rs` for SSE formatting | NEW |
| 3 | Create `server/src/routes/execute/integration.rs` bridging ToolRegistry to AgentExecutor | NEW |
| 4 | Create `server/src/routes/execute/mod.rs` exposing route handler | NEW |
| 5 | Create `server/src/routes/execute.rs` re-exporting module | NEW |
| 6 | Update `server/src/routes/mod.rs` to include execute routes | routes/mod.rs |
| 7 | Update `server/src/lib.rs` to mount execute routes | lib.rs |
| 8 | Add integration test in `tests/src/agent_execute_api_tests.rs` | NEW |

#### 2.1.4 Request/Response Types

```rust
// Request
pub struct ExecuteRequest {
    pub prompt: String,
    pub mode: Option<SessionMode>,  // default: session's current mode
    pub stream: Option<bool>,      // default: true
}

// Response (SSE events)
pub enum ExecuteEvent {
    ToolCall { tool: String, id: String, params: Value },
    ToolResult { tool: String, id: String, result: Value },
    Message { role: String, content: String },
    Error { error: String },
    Complete { summary: String, session_state: Value },
}
```

#### 2.1.5 Integration Points

| Component | Integration Point | Method |
|-----------|------------------|--------|
| `crates/agent/src/executor.rs` | Create agent per mode | `AgentExecutor::new(mode, tool_registry)` |
| `crates/tools/src/registry.rs` | Discover tools | `ToolRegistry::list_tools()` |
| `crates/storage/src/` | Load session | `SessionStore::get_session(id)` |
| `crates/storage/src/` | Update session | `SessionStore::update_session(id, state)` |
| `crates/permission/src/` | Check permissions | `PermissionManager::check_permission(scope)` |

---

### 2.2 FR-025: WebSocket Agent Streaming

#### 2.2.1 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                   WebSocket Connection                          │
│                     /api/ws/{session_id}                        │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│               WebSocket Handler (actix_ws)                      │
│              - Connection lifecycle management                 │
│              - Message framing                                  │
│              - Client heartbeat                                 │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Session Hub                                   │
│            (Broadcast events to all session clients)           │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
            ┌───────────────┐       ┌───────────────┐
            │ Execute API   │       │  ACP Handler   │
            │ (FR-024)      │       │               │
            └───────────────┘       └───────────────┘
                    │                       │
                    └───────────┬───────────┘
                                ▼
                    ┌───────────────────────┐
                    │    Agent Runtime      │
                    │  - Tool execution      │
                    │  - LLM token stream   │
                    │  - State changes      │
                    └───────────────────────┘
```

#### 2.2.2 Implementation Steps

| Step | Action | Files Modified |
|------|--------|----------------|
| 1 | Create `server/src/routes/ws/session_hub.rs` for event broadcast | NEW |
| 2 | Update `server/src/routes/ws.rs` to handle session-specific WS | routes/ws.rs |
| 3 | Create event emitter trait in `crates/agent/src/events.rs` | NEW (agent crate) |
| 4 | Implement event emitter for `AgentExecutor` | agent crate |
| 5 | Update execute endpoint to emit events to session hub | routes/execute/ |
| 6 | Add WebSocket integration test | tests/src/ws_integration_tests.rs |

#### 2.2.3 Event Types

```rust
pub enum AgentEvent {
    ToolCall { tool: String, id: String, params: Value },
    ToolResult { tool: String, id: String, result: Value },
    Token { content: String },
    Message { role: String, content: String },
    Error { error: String },
    Complete { summary: String },
}
```

---

## 3. P1 Implementation Plan

### 3.1 FR-026: Server Status Endpoint

**Endpoint:** `GET /api/status`

```rust
#[derive(Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub status: ServerStatus,  // running | degraded | error
    pub uptime_seconds: u64,
    pub active_sessions: u32,
    pub total_sessions: u32,
    pub providers: Vec<ProviderStatus>,
    pub plugins: Vec<PluginStatus>,
}
```

| Step | Action |
|------|--------|
| 1 | Create `server/src/routes/status.rs` |
| 2 | Collect provider status from LLM registry |
| 3 | Collect plugin status from plugin system |
| 4 | Add no-auth route to App |
| 5 | Add integration test |

---

### 3.2 FR-027: Streaming Response Support

**Existing:** `POST /api/run` performs basic LLM chat, completes before responding

| Step | Action |
|------|--------|
| 1 | Modify `run_prompt` to accept `stream` param |
| 2 | Implement token-by-token SSE streaming |
| 3 | Add `Accept: text/event-stream` header detection |
| 4 | Test with `curl -H "Accept: text/event-stream"` |

---

### 3.3 FR-028: Permission Reply Integration

**Current:** `permission_reply` handler logs decision but doesn't update state

| Step | Action |
|------|--------|
| 1 | Connect handler to `PermissionManager` |
| 2 | Update `ApprovalQueue` on decision |
| 3 | Trigger re-evaluation of pending requests |
| 4 | Add audit logging |

---

## 4. P2 Implementation Plan

### 4.1 FR-023: ratatui-testing Framework

| Component | Status | Implementation |
|------------|--------|----------------|
| PtySimulator | Partial | Injects KeyEvent, MouseEvent to PTY |
| BufferDiff | Missing | Cell-by-cell buffer comparison |
| StateTester | Missing | State capture and comparison |
| TestDsl | Missing | Fluent API composition |
| CliTester | Missing | Process spawn and output capture |

---

### 4.2 FR-029: Hook Determinism Test

- Add 100-iteration test verifying consistent plugin ordering
- Location: `crates/plugin/src/lib.rs`

### 4.3 FR-030: Security Test Suite

| Test | Location | Coverage |
|------|----------|----------|
| SQL injection | `tests/src/security_tests.rs` | Session/message operations |
| Path traversal | `tests/src/security_tests.rs` | File operations |
| Request validation | `tests/src/security_tests.rs` | All endpoints |

---

## 5. Implementation Dependencies

### 5.1 Dependency Graph

```
FR-024 (Execute API)
    ├── FR-025 (WebSocket)     [shares session hub]
    └── needs:
        ├── agent::AgentExecutor
        ├── tools::ToolRegistry
        └── storage::SessionStore

FR-025 (WebSocket Streaming)
    └── needs:
        └── agent::events (NEW event emitter trait)

FR-026 (Status)
    └── needs:
        ├── llm providers status
        └── plugin status

FR-027 (Streaming Response)
    └── needs:
        └── run_prompt refactor

FR-028 (Permission Reply)
    └── needs:
        └── PermissionManager integration
```

---

## 6. Testing Strategy

### 6.1 Test Files to Create

| Test File | Coverage |
|-----------|----------|
| `tests/src/agent_execute_api_tests.rs` | FR-024 |
| `tests/src/ws_streaming_tests.rs` | FR-025 |
| `tests/src/status_endpoint_tests.rs` | FR-026 |
| `tests/src/streaming_response_tests.rs` | FR-027 |
| `tests/src/permission_integration_tests.rs` | FR-028 |

### 6.2 Test Infrastructure

- Use existing `tests/src/common/` helpers
- MockLLMProvider for agent tests
- TempProject for file operation tests

---

## 7. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| AgentExecutor not designed for streaming | High | Design event emitter first, then integrate |
| WebSocket concurrent connections | Medium | Use session hub with Arc<RwLock> |
| Breaking existing API users | Low | Add stream param defaulting to false |

---

## 8. Milestones

| Milestone | Items | Target |
|-----------|-------|--------|
| M1: P0 Complete | FR-024, FR-025 | Day 1 |
| M2: P1 Complete | FR-026, FR-027, FR-028 | Day 2 |
| M3: P2 Complete | FR-023, FR-029, FR-030 | Day 3 |
| M4: Integration | Full E2E tests, regression | Day 4 |

---

## 9. Acceptance Criteria Checklist

### FR-024 (Execute API)
- [ ] `POST /api/sessions/{id}/execute` responds 200
- [ ] Tools from ToolRegistry available during execution
- [ ] Tool results returned in response
- [ ] Invalid session returns 404
- [ ] Unauthenticated returns 401

### FR-025 (WebSocket)
- [ ] WebSocket connects to `/api/ws`
- [ ] Tool events stream in real-time
- [ ] LLM tokens stream progressively
- [ ] Multiple clients per session work
- [ ] Client disconnect doesn't crash server

### FR-026 (Status)
- [ ] `GET /api/status` returns 200
- [ ] All specified fields present
- [ ] No auth required
- [ ] Response time < 100ms

### FR-027 (Streaming)
- [ ] SSE streaming works
- [ ] Tokens arrive progressively
- [ ] Clean connection close

### FR-028 (Permission)
- [ ] ApprovalQueue updated on decision
- [ ] Approved requests proceed
- [ ] Denied requests return error

---

*Plan Version: 21*
*Generated: 2026-04-15*
