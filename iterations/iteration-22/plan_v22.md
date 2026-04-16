# Implementation Plan - Iteration 22

**Project:** OpenCode RS
**Iteration:** 22
**Phase:** HTTP API Execution Gap Resolution
**Last Updated:** 2026-04-16
**Status:** Planning

---

## 1. Priority Overview

| Priority | Items | Focus |
|----------|-------|-------|
| **P0** | FR-031, FR-032 | Session Execute Endpoint + Run Endpoint Tool Integration |
| **P1** | FR-033, FR-034, FR-035, FR-036 | Status, WebSocket, Permission, Streaming |
| **P2** | FR-037, FR-038 | SDK Documentation, LSP Integration Docs |

---

## 2. P0 Implementation Plan (Current Sprint)

### P0-1: FR-031 - Session Execute Endpoint

**File to create:** `crates/server/src/routes/execute.rs`

**Dependencies:**
- `crates/agent/src/lib.rs` - AgentExecutor
- `crates/tools/src/lib.rs` - ToolRegistry
- `crates/server/src/routes/execute/types.rs` - Existing types (ExecuteRequest, ExecuteMode, ExecuteEvent)

**Implementation Steps:**

1. Create `server/src/routes/execute.rs`
2. Implement `execute_session` handler with signature:
   ```rust
   async fn execute_session(
       req: HttpRequest,
       path: Path<Uuid>,
       body: Json<ExecuteRequest>,
       session_repo: Data<SessionRepository>,
       agent_executor: Data<AgentExecutor>,
       tool_registry: Data<ToolRegistry>,
   ) -> impl IntoResponse
   ```
3. Add route registration in `session.rs::init()`:
   ```rust
   cfg.route("/{id}/execute", web::post().to(execute_session));
   ```
4. Implement SSE streaming for ExecuteEvent types:
   - `tool_call` event
   - `tool_result` event
   - `message` event
   - `complete` event
   - `error` event
5. Add authentication middleware to endpoint

**Route:** `POST /api/sessions/{id}/execute`

---

### P0-2: FR-032 - Run Endpoint Tool Integration

**File to modify:** `crates/server/src/routes/run.rs`

**Dependencies:**
- `crates/agent/src/lib.rs` - AgentExecutor
- `crates/tools/src/lib.rs` - ToolRegistry

**Implementation Steps:**

1. Refactor `run_prompt` function to use `AgentExecutor` instead of basic LLM chat
2. Integrate `ToolRegistry` for tool discovery and execution
3. Enable SSE streaming for progressive token response
4. Propagate tool errors properly to response

**Route:** `POST /api/run`

---

## 3. P1 Implementation Plan (Next Sprint)

### P1-1: FR-033 - Server Status Endpoint

**File to create:** `crates/server/src/routes/status.rs`

**Route:** `GET /api/status`

**Response fields:**
- `version`: String
- `status`: String (running/degraded/error)
- `uptime_seconds`: u64
- `active_sessions`: u32
- `total_sessions`: u32
- `providers`: Vec<ProviderStatus>
- `plugins`: Vec<PluginStatus>

---

### P1-2: FR-034 - WebSocket Agent Streaming

**Files to modify:** `ws.rs`, `acpws.rs`

**Implementation Steps:**

1. Create event emitter in agent runtime
2. Broadcast AgentEvent to connected WebSocket clients
3. Support multiple concurrent connections per session
4. Handle client disconnection gracefully

---

### P1-3: FR-035 - Permission Reply Integration

**File to modify:** `crates/server/src/routes/permission.rs`

**Implementation Steps:**

1. Connect `permission_reply` handler to `PermissionManager`
2. Update `ApprovalQueue` with user decisions
3. Trigger re-evaluation of pending requests after approval
4. Log decisions to audit trail

---

### P1-4: FR-036 - Streaming Response Support

**Files to modify:** `run.rs`, `execute.rs`

**Implementation Steps:**

1. Support `Accept: text/event-stream` header detection
2. Stream LLM tokens as they arrive using SSE format
3. Handle connection interruption gracefully

---

## 4. P2 Implementation Plan

### P2-1: FR-037 - SDK Documentation

**Tasks:**
1. Add comprehensive `cargo doc` comments to all public items in `crates/sdk/src/`
2. Create working examples for all public APIs
3. Prepare README with installation instructions for crates.io

### P2-2: FR-038 - LSP Integration Documentation

**Tasks:**
1. Document VSCode LSP client setup
2. Document Neovim LSP setup
3. List supported LSP features

---

## 5. Implementation Order

```
Phase 1 (Current Sprint):
├── FR-031: Session Execute Endpoint (P0)
└── FR-032: Run Endpoint Tool Integration (P0)

Phase 2 (Next Sprint):
├── FR-033: Server Status Endpoint (P1)
├── FR-034: WebSocket Agent Streaming (P1)
├── FR-035: Permission Reply Integration (P1)
└── FR-036: Streaming Response Support (P1)

Phase 3 (Future):
├── FR-037: SDK Documentation (P2)
└── FR-038: LSP Integration Documentation (P2)
```

---

## 6. Acceptance Criteria Summary

### P0 Must Pass Before Release
- [ ] `POST /api/sessions/{id}/execute` returns 200 for valid session
- [ ] Tools from ToolRegistry are discovered and executed
- [ ] Tool execution results appear in response
- [ ] `POST /api/run` executes tools via AgentExecutor
- [ ] Streaming works for LLM tokens in both endpoints

### P1 (Next Sprint)
- [ ] `GET /api/status` returns 200 with required fields
- [ ] WebSocket streams agent events in real-time
- [ ] Permission approvals trigger tool execution
- [ ] SSE streaming works with `Accept: text/event-stream`

### P2 (Future)
- [ ] `cargo doc` generates complete documentation
- [ ] IDE LSP setup documented

---

*Plan generated from spec_v22.md (Iteration 22)*
