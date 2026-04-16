# Specification Document - Iteration 22

**Project:** OpenCode RS - Rust Implementation of OpenCode AI Coding Agent
**Iteration:** 22
**Date:** 2026-04-15
**Phase:** HTTP API Execution Gap Resolution
**PRD Reference:** PRD.md (v1.0, 2026-04-11)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Gap Analysis Summary](#2-gap-analysis-summary)
3. [Feature Requirements (FR)](#3-feature-requirements-fr)
4. [P0 - Critical Blocking Issues](#4-p0---critical-blocking-issues)
5. [P1 - High Priority Issues](#5-p1---high-priority-issues)
6. [P2 - Medium Priority Issues](#6-p2---medium-priority-issues)
7. [Technical Debt](#7-technical-debt)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [Acceptance Criteria](#9-acceptance-criteria)
10. [API Endpoint Specification](#10-api-endpoint-specification)
11. [Cross-References](#11-cross-references)

---

## 1. Executive Summary

### Overall Implementation Status

| Category | Status | Gap |
|----------|--------|-----|
| Crate Structure | ✅ 100% | All 17 PRD crates + extras |
| LLM Providers | ✅ 100% | OpenAI, Anthropic, Ollama + extras |
| Tool System | ✅ 100% | Full registry with all tools |
| Agent Modes | ✅ 100% | Build, Plan, General + extras |
| TUI | ✅ 100% | Comprehensive ratatui implementation |
| HTTP API CRUD | ✅ 100% | Sessions, messages, etc. |
| **HTTP API Execution** | **❌ 0%** | **P0 BLOCKING** |
| ACP Routes | ✅ 100% | All 4 routes implemented |
| Storage | ✅ 100% | SQLite complete |
| Auth/Security | ✅ 100% | JWT, OAuth, encryption |
| Permission System | ⚠️ 90% | Core OK, HTTP integration partial |
| Plugin System | ✅ 100% | WASM runtime |
| MCP | ✅ 100% | Protocol, client, server |
| SDK | ✅ 100% | Full API |
| Config | ✅ 100% | Comprehensive |

**Overall Implementation:** ~93% (dropped from 95% due to P0 execution gap unchanged)

### Critical Findings from Iteration-22 Analysis

| Priority | Count | Critical Items |
|----------|-------|----------------|
| **P0** | 2 | `POST /api/sessions/{id}/execute` missing, Run endpoint tool integration |
| **P1** | 4 | `/api/status` missing, WebSocket not integrated, Permission reply not connected, Streaming missing |
| **P2** | 2 | SDK documentation, LSP integration documentation |

### Key Observation

The P0 gap (`POST /api/sessions/{id}/execute`) identified in iteration-21 **remains unfixed**. The types were added in `execute/types.rs` (`ExecuteRequest`, `ExecuteMode`, `ExecuteEvent`) but no handler implementation exists and the route is not registered.

---

## 2. Gap Analysis Summary

### Gap Severity Overview

| Priority | Count | FR Reference |
|----------|-------|--------------|
| **P0** | 2 | FR-031, FR-032 |
| **P1** | 4 | FR-033, FR-034, FR-035, FR-036 |
| **P2** | 2 | FR-037, FR-038 |

### Compliance Scorecard

| Category | Status | Gap Severity | FR Reference |
|----------|--------|--------------|--------------|
| Crate Structure | ✅ Compliant | - | - |
| LLM Providers | ✅ Compliant | - | - |
| Tool System | ✅ Compliant | - | - |
| Agent Modes | ✅ Compliant | - | - |
| TUI | ✅ Compliant | - | - |
| HTTP API CRUD | ✅ Compliant | - | - |
| **HTTP API Execution** | ❌ **Critical Gap** | **P0** | FR-031, FR-032 |
| ACP Routes | ✅ Compliant | - | - |
| Storage | ✅ Compliant | - | - |
| Auth/Security | ✅ Compliant | - | - |
| Permission HTTP Integration | ⚠️ Partial | P1 | FR-035 |
| WebSocket Streaming | ⚠️ Not Integrated | P1 | FR-034 |
| Server Status Endpoint | ❌ Missing | P1 | FR-033 |
| Streaming Response | ❌ Missing | P1 | FR-036 |
| SDK Documentation | ⚠️ Incomplete | P2 | FR-037 |
| LSP Integration Docs | ⚠️ Incomplete | P2 | FR-038 |

---

## 3. Feature Requirements (FR)

| FR-ID | Feature | Priority | Status | PRD Reference |
|-------|---------|----------|--------|---------------|
| FR-031 | Session Execute Endpoint | **P0** | Not Implemented | PRD §6.1 |
| FR-032 | Run Endpoint Tool Integration | **P0** | Not Implemented | PRD §6.1 |
| FR-033 | Server Status Endpoint | P1 | Not Implemented | PRD §6.1 |
| FR-034 | WebSocket Agent Streaming | P1 | Not Integrated | PRD §6.1 |
| FR-035 | Permission Reply Integration | P1 | Partial | PRD §3.4, §6.2 |
| FR-036 | Streaming Response Support | P1 | Not Implemented | PRD §6.1 |
| FR-037 | SDK Documentation | P2 | Incomplete | PRD §4.2 |
| FR-038 | LSP Integration Documentation | P2 | Incomplete | PRD §4.2 |

---

### FR-031: Session Execute Endpoint

**Priority:** P0 (Critical - Blocking)
**Status:** Not Implemented
**PRD Reference:** PRD.md §6.1 - `POST /api/session/{id}/execute`

#### Requirement

Implement `POST /api/sessions/{id}/execute` endpoint that provides full agent execution with tool access via HTTP API.

#### Current State

- Types exist in `server/src/routes/execute/types.rs`:
  - `ExecuteRequest`
  - `ExecuteMode`
  - `ExecuteEvent`
- **No handler implementation exists**
- **Route is not registered in `session.rs::init()`**

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-031.1 | Endpoint Handler | Create `server/src/routes/execute.rs` with handler |
| FR-031.2 | Route Registration | Register `POST /{id}/execute` in `session.rs::init()` |
| FR-031.3 | ToolRegistry Integration | Integrate with tool registry for tool discovery |
| FR-031.4 | AgentExecutor Integration | Use AgentExecutor for agent lifecycle |
| FR-031.5 | Session State Update | Update session state after execution |
| FR-031.6 | Streaming Response | Support SSE streaming of execution events |

#### API Contract

```rust
// POST /api/sessions/{id}/execute
Request {
    "prompt": String,
    "mode": Option<ExecuteMode>,  // build, plan, general
    "stream": Option<bool>        // default: true
}

// Response (streaming): text/event-stream
Event types:
- tool_call:    {"tool": "read", "params": {...}}
- tool_result: {"tool": "read", "result": {...}}
- message:      {"role": "assistant", "content": "..."}
- complete:     {"session_state": {...}}
- error:        {"error": "..."}
```

#### Implementation Path

1. Create `server/src/routes/execute.rs`
2. Implement `execute_session` handler using `ExecuteRequest`
3. Integrate `AgentExecutor` with `ToolRegistry`
4. Register route: `cfg.route("/{id}/execute", web::post().to(execute_session));`
5. Add SSE streaming for `ExecuteEvent` types
6. Add authentication middleware

#### Acceptance Criteria

- [ ] `POST /api/sessions/{id}/execute` responds with 200 for valid session
- [ ] Tools from ToolRegistry are discovered and executed
- [ ] Tool execution results appear in response
- [ ] Invalid session ID returns 404
- [ ] Unauthenticated request returns 401
- [ ] Streaming SSE works with `Accept: text/event-stream`

---

### FR-032: Run Endpoint Tool Integration

**Priority:** P0 (Critical - Blocking)
**Status:** Not Implemented
**PRD Reference:** PRD.md §6.1

#### Requirement

Refactor `run_prompt` in `server/src/routes/run.rs` to integrate with AgentExecutor and ToolRegistry instead of basic LLM chat.

#### Current State

- `run_prompt` only performs basic LLM chat
- Does not integrate with `ToolRegistry`
- Does not use `AgentExecutor`
- Completes entire request before responding

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-032.1 | AgentExecutor Integration | Refactor to use AgentExecutor for execution |
| FR-032.2 | ToolRegistry Integration | Enable tool discovery and execution |
| FR-032.3 | Progressive Response | Stream tokens as they arrive |
| FR-032.4 | Error Propagation | Return proper errors for tool failures |

#### Acceptance Criteria

- [ ] `POST /api/run` executes tools via AgentExecutor
- [ ] Tool results are included in response
- [ ] Streaming works for LLM tokens
- [ ] Tool errors are properly propagated

---

### FR-033: Server Status Endpoint

**Priority:** P1 (High)
**Status:** Not Implemented
**PRD Reference:** PRD.md §6.1 - `GET /api/status`

#### Requirement

Implement `GET /api/status` endpoint for server health and status information.

#### Current State

- Only `/health` exists
- No `/api/status` endpoint

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-033.1 | Version Info | Return server version |
| FR-033.2 | Status | Return server status (running/degraded/error) |
| FR-033.3 | Uptime | Return uptime in seconds |
| FR-033.4 | Session Stats | Return active and total session counts |
| FR-033.5 | Provider Status | Return status for each configured LLM provider |
| FR-033.6 | Plugin List | Return loaded plugin list |

#### API Contract

```rust
// GET /api/status
Response {
    "version": "1.0.0",
    "status": "running",
    "uptime_seconds": 3600,
    "active_sessions": 5,
    "total_sessions": 142,
    "providers": [
        {"name": "openai", "status": "ready", "model": "gpt-4"},
        {"name": "anthropic", "status": "ready", "model": "claude-3-opus"}
    ],
    "plugins": [
        {"name": "example-plugin", "version": "1.0.0", "status": "loaded"}
    ]
}
```

#### Acceptance Criteria

- [ ] `GET /api/status` returns 200
- [ ] Response contains all specified fields
- [ ] Endpoint does not require authentication
- [ ] Response time < 100ms

---

### FR-034: WebSocket Agent Streaming

**Priority:** P1 (High)
**Status:** Not Integrated
**PRD Reference:** PRD.md §6.1

#### Requirement

Connect WebSocket endpoints to agent runtime for real-time streaming of agent execution output.

#### Current State

- WebSocket endpoints exist (`/api/ws`, `/api/acpws`)
- Not integrated with agent execution
- No connection to agent runtime events

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-034.1 | Event Emitter | Create event emitter in agent runtime |
| FR-034.2 | WebSocket Broadcast | Broadcast events to connected clients |
| FR-034.3 | Multi-Connection | Support multiple concurrent connections per session |
| FR-034.4 | Disconnection Handling | Handle client disconnection gracefully |

#### Event Types

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

#### Acceptance Criteria

- [ ] WebSocket receives tool call events in real-time
- [ ] WebSocket receives token-by-token LLM output
- [ ] Multiple clients can connect to same session
- [ ] Client disconnection doesn't crash server

---

### FR-035: Permission Reply Integration

**Priority:** P1 (High)
**Status:** Partial
**PRD Reference:** PRD.md §3.4, §6.2

#### Requirement

Connect `permission_reply` handler to actual permission system to process user approval/denial decisions.

#### Current State

- `permission_reply` handler exists in `server/src/routes/permission.rs`
- Logs decision but doesn't propagate to permission system
- Doesn't update `ApprovalQueue` state

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-035.1 | ApprovalQueue Update | permission_reply updates ApprovalQueue |
| FR-035.2 | Execution Trigger | Approved requests proceed to execution |
| FR-035.3 | Denial Handling | Denied requests return error to agent |
| FR-035.4 | Audit Trail | Decision is logged to audit trail |

#### Acceptance Criteria

- [ ] User approval triggers tool execution
- [ ] User denial returns PermissionDenied error
- [ ] Audit log records decision

---

### FR-036: Streaming Response Support

**Priority:** P1 (High)
**Status:** Not Implemented
**PRD Reference:** PRD.md §6.1

#### Requirement

Implement chunked/streaming response for `POST /api/run` and execute endpoints.

#### Current State

- `run_prompt` completes entire LLM request before responding
- No progressive updates

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-036.1 | SSE Support | Support `Accept: text/event-stream` header |
| FR-036.2 | Token Streaming | Stream LLM tokens as they arrive |
| FR-036.3 | SSE Formatting | Include proper SSE formatting |
| FR-036.4 | Interruption Handling | Handle connection interruption gracefully |

#### Acceptance Criteria

- [ ] `curl -H "Accept: text/event-stream" -d '{"prompt":"hi"}' /api/run` streams
- [ ] Each token arrives as separate SSE event
- [ ] Connection close terminates cleanly

---

### FR-037: SDK Documentation

**Priority:** P2 (Medium)
**Status:** Incomplete
**PRD Reference:** PRD.md §4.2

#### Requirement

Complete SDK documentation for publishing to crates.io.

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-037.1 | API Docs | Comprehensive `cargo doc` comments |
| FR-037.2 | Examples | Working examples for all public APIs |
| FR-037.3 | Crate.io Entry | README with installation instructions |

#### Acceptance Criteria

- [ ] `cargo doc` generates complete documentation
- [ ] All public items have doc comments
- [ ] Examples compile and run

---

### FR-038: LSP Integration Documentation

**Priority:** P2 (Medium)
**Status:** Incomplete
**PRD Reference:** PRD.md §4.2

#### Requirement

Document IDE extension support for LSP integration.

#### Functional Requirements

| ID | Requirement | Description |
|----|-------------|-------------|
| FR-038.1 | IDE Setup | Document VSCode/Neovim setup |
| FR-038.2 | Extension Config | LSP extension configuration guide |
| FR-038.3 | Protocol Support | Document supported LSP features |

#### Acceptance Criteria

- [ ] Documentation for VSCode LSP client setup
- [ ] Documentation for Neovim LSP setup
- [ ] List of supported LSP features

---

## 4. P0 - Critical Blocking Issues

### P0-1: Session Execute Endpoint Missing

**Issue:** `POST /api/sessions/{id}/execute` endpoint not implemented

**Location:** `server/src/routes/execute.rs` (does not exist)

**Impact:** Cannot execute agent tasks with tools via HTTP API

**Root Cause:** Types defined in `execute/types.rs` but no handler implementation, route not registered

**Fix Required:**
1. Create `server/src/routes/execute.rs` with `execute_session` handler
2. Integrate `AgentExecutor` with `ToolRegistry`
3. Register route: `cfg.route("/{id}/execute", web::post().to(execute_session));`
4. Add SSE streaming for events

---

### P0-2: Run Endpoint Tool Integration Missing

**Issue:** `run_prompt` in `run.rs` only performs basic LLM chat

**Location:** `server/src/routes/run.rs`

**Impact:** HTTP API cannot use file operations, grep, git, etc.

**Fix Required:**
1. Refactor `run_prompt` to use `AgentExecutor`
2. Integrate with `ToolRegistry` for tool discovery
3. Enable streaming response

---

## 5. P1 - High Priority Issues

### P1-1: Server Status Endpoint Missing

**Issue:** `GET /api/status` not implemented

**Location:** `server/src/routes/`

**Impact:** API inconsistency with PRD specification

**Fix Required:** Add `/api/status` endpoint with server status info

---

### P1-2: WebSocket Not Integrated with Agent

**Issue:** WebSocket endpoints don't stream agent execution

**Location:** `server/src/routes/ws.rs`, `server/src/routes/acpws.rs`

**Impact:** No real-time agent output via WebSocket

**Fix Required:**
1. Create event emitter in agent runtime
2. Broadcast events to WebSocket connections
3. Handle multiple connections per session

---

### P1-3: Permission Reply Not Connected

**Issue:** `permission_reply` handler logs but doesn't update permission state

**Location:** `server/src/routes/permission.rs`

**Impact:** Permission approvals don't affect actual tool execution

**Fix Required:**
1. Connect to `PermissionManager`
2. Update `ApprovalQueue` with decisions
3. Trigger re-evaluation of pending requests

---

### P1-4: Streaming Response Not Implemented

**Issue:** `run_prompt` completes entire request before responding

**Location:** `server/src/routes/run.rs`

**Impact:** No progressive response updates

**Fix Required:**
1. Use `actix-web` streaming responses
2. Stream LLM tokens as they arrive
3. Support `Accept: text/event-stream` header

---

## 6. P2 - Medium Priority Issues

| Issue | Module | Impact | Fix Required |
|-------|--------|--------|--------------|
| SDK documentation incomplete | sdk | Limited usability | Add cargo doc comments, examples |
| LSP integration documentation | lsp | IDE integration unclear | Document extension support |

---

## 7. Technical Debt

### Active Technical Debt

| ID | Item | Severity | Module | Status |
|----|------|----------|--------|--------|
| TD-020 | Execute endpoint types unused | Medium | server | Not fixed |
| TD-021 | Run endpoint tool integration | High | server | Not fixed |
| TD-022 | Permission reply not connected | Medium | permission | Not fixed |
| TD-023 | WebSocket agent integration | High | server | Not fixed |
| TD-024 | Streaming response | Medium | server | Not fixed |

---

## 8. Implementation Roadmap

### Phase 1: P0 Resolution (Current Sprint)

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| Implement execute endpoint handler | `POST /api/sessions/{id}/execute` working | FR-031 |
| Refactor run_prompt to use AgentExecutor | Tools accessible via /api/run | FR-032 |

### Phase 2: P1 Resolution (Next Sprint)

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| Add GET /api/status | Server status endpoint | FR-033 |
| Integrate WebSocket with agent runtime | Real-time streaming | FR-034 |
| Connect permission reply handler | ApprovalQueue integration | FR-035 |
| Implement SSE streaming | Progressive responses | FR-036 |

### Phase 3: P2 Resolution

| Task | Deliverable | FR Reference |
|------|-------------|--------------|
| Complete SDK documentation | crates.io ready | FR-037 |
| Document LSP integration | IDE setup guide | FR-038 |

---

## 9. Acceptance Criteria

### P0 Acceptance Criteria (Must Pass Before Release)

#### FR-031: Session Execute Endpoint
- [ ] `POST /api/sessions/{session_id}/execute` endpoint exists
- [ ] Request with valid session ID and prompt returns 200
- [ ] Tools from ToolRegistry are available during execution
- [ ] Tool execution results are included in response
- [ ] Invalid session ID returns 404
- [ ] Unauthenticated request returns 401

#### FR-032: Run Endpoint Tool Integration
- [ ] `POST /api/run` executes tools via AgentExecutor
- [ ] Tool results appear in response
- [ ] Streaming works for LLM tokens

### P1 Acceptance Criteria

#### FR-033: Server Status Endpoint
- [ ] `GET /api/status` returns 200
- [ ] Response contains version, status, uptime_seconds
- [ ] Response contains providers array
- [ ] Endpoint is accessible without authentication

#### FR-034: WebSocket Agent Streaming
- [ ] WebSocket connects to `/api/ws`
- [ ] Tool call events appear during execution
- [ ] LLM tokens stream in real-time

#### FR-035: Permission Reply Integration
- [ ] `POST /api/permission/reply` updates ApprovalQueue
- [ ] Approved permissions allow tool execution
- [ ] Denied permissions return error

#### FR-036: Streaming Response Support
- [ ] `POST /api/run` with `Accept: text/event-stream` streams
- [ ] Each LLM token appears as separate SSE event

### P2 Acceptance Criteria

#### FR-037: SDK Documentation
- [ ] `cargo doc` generates complete documentation
- [ ] All public items have doc comments

#### FR-038: LSP Integration Documentation
- [ ] VSCode LSP setup documented
- [ ] Neovim LSP setup documented

---

## 10. API Endpoint Specification

### Current Implementation

```
GET  /health                           - Health check ✅
GET  /api/status                        - ❌ MISSING (P1) FR-033
GET  /api/config                        - Config routes ✅
GET  /api/providers                     - Provider routes ✅
GET  /api/models                        - Model routes ✅
GET  /api/sessions                      - List sessions ✅
POST /api/sessions                      - Create session ✅
GET  /api/sessions/{id}                 - Get session ✅
DELETE /api/sessions/{id}               - Delete session ✅
POST /api/sessions/{id}/execute         - ❌ MISSING (P0) FR-031
POST /api/sessions/{id}/fork            - Fork session ✅
POST /api/sessions/{id}/prompt          - Prompt session ✅
POST /api/sessions/{id}/command         - Run command ✅
POST /api/sessions/{id}/abort           - Abort session ✅
GET  /api/sessions/{id}/messages        - List messages ✅
POST /api/sessions/{id}/messages        - Add message ✅
GET  /api/sessions/{id}/diff            - Get diff ✅
GET  /api/sessions/{id}/snapshots       - List snapshots ✅
POST /api/sessions/{id}/revert          - Revert to snapshot ✅
POST /api/sessions/{id}/share           - Share session ✅
DELETE /api/sessions/{id}/share         - Remove share ✅
POST /api/sessions/{id}/summarize       - Summarize session ✅
GET  /api/share/{id}                   - Get shared session ✅
POST /api/run                           - Run prompt (basic LLM only ⚠️) FR-032
WS   /api/ws                            - WebSocket ✅ (not integrated) FR-034
SSE  /api/sse                           - SSE ✅ (not integrated)
GET  /api/acp/status                    - ACP status ✅
POST /api/acp/handshake                 - ACP handshake ✅
POST /api/acp/connect                   - ACP connect ✅
POST /api/acp/ack                        - ACP acknowledge ✅
GET  /api/acp/events                    - ACP events stream ✅
WS   /api/acpws                         - ACP WebSocket ✅ (not integrated) FR-034
```

### Required Additions (Iteration-22)

```
POST /api/sessions/{id}/execute        - Full agent execution ⬅️ FR-031 (P0)
POST /api/run                           - Run with tool integration ⬅️ FR-032 (P0)
GET  /api/status                        - Server status ⬅️ FR-033 (P1)
```

---

## 11. Cross-References

### PRD Cross-References

| Section | Topic | Status |
|---------|-------|--------|
| §2 | Architecture Overview | ✅ Complete |
| §3.1 | LLM Provider Support | ✅ Complete |
| §3.2 | Tool System | ✅ Complete |
| §3.3 | Agent Modes | ✅ Complete |
| §3.4 | User Interfaces | ⚠️ HTTP partial |
| §3.5 | Session Management | ✅ Complete |
| §3.6 | MCP Support | ✅ Complete |
| §4 | Feature Requirements | ⚠️ P0/P1 gaps |
| §5 | Non-Functional Requirements | ✅ Complete |
| §6 | API Specification | ⚠️ Execution missing |
| §7 | Data Models | ✅ Complete |
| §9 | Configuration | ✅ Complete |

### Gap-to-FR Mapping

| Gap ID | Description | FR-ID | Priority |
|--------|-------------|-------|----------|
| P0-1 | Execute endpoint missing | FR-031 | P0 |
| P0-2 | Run endpoint no tool integration | FR-032 | P0 |
| P1-1 | Missing /api/status | FR-033 | P1 |
| P1-2 | WebSocket not integrated | FR-034 | P1 |
| P1-3 | Permission reply not connected | FR-035 | P1 |
| P1-4 | Streaming not implemented | FR-036 | P1 |
| P2-1 | SDK documentation | FR-037 | P2 |
| P2-2 | LSP documentation | FR-038 | P2 |

---

*Document generated from PRD.md v1.0 and Gap Analysis Report*
*Iteration 22 - HTTP API Execution Gap Resolution*
*Analysis Date: 2026-04-15*