# Specification Document - Iteration 21

**Project:** OpenCode Rust Port
**Iteration:** 21
**Date:** 2026-04-15
**Phase:** Implementation & Gap Resolution

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Implementation Status](#2-implementation-status)
3. [PRD Coverage Analysis](#3-prd-coverage-analysis)
4. [Gap Analysis - P0/P1/P2 Issues](#4-gap-analysis---p0p1p2-issues)
5. [Feature Requirements](#5-feature-requirements)
6. [Test Roadmap](#6-test-roadmap)
7. [Technical Debt](#7-technical-debt)
8. [Acceptance Criteria](#8-acceptance-criteria)
9. [Recommendations](#9-recommendations)

---

## 1. Executive Summary

**Overall Implementation Status:** ~95% complete

### Critical Findings from Iteration-21 Analysis

| Category | Status | Notes |
|----------|--------|-------|
| Crate Structure | 100% | All 17 PRD crates + 1 extra (control-plane) |
| LLM Providers | 100%+ | All required + 30+ implementations |
| Tool System | 100% | All required tools + rich registry |
| Agent Modes | 100% | All 3 PRD modes + 4 extra modes |
| TUI | 100% | Comprehensive ratatui implementation |
| HTTP API | 85% | Core CRUD complete, **execution missing** |
| ACP Routes | 100% | All 4 routes implemented |
| Testing | 100% | Unit, integration, TUI, benchmarks |

### Key Gaps Identified

| Priority | Count | Critical Items |
|----------|-------|----------------|
| **P0** | 2 | `/api/sessions/{id}/execute` endpoint, WebSocket streaming integration |
| **P1** | 3 | `/api/status` endpoint, streaming response, permission reply handler |
| **P2** | 4 | SDK documentation, LSP integration, Web UI, experimental features |

### Progress Since Iteration-18

| Category | Change |
|----------|--------|
| **Resolved** | Duplicate directory_scanner.rs removed |
| **Resolved** | Two ToolRegistry implementations documented |
| **Resolved** | ACP E2E tests complete (1083 lines) |
| **Still Pending** | P0 gaps in HTTP API execution |
| **Still Pending** | ratatui-testing framework completion |

---

## 2. Implementation Status

### 2.1 Crate-Level Status

| Crate | Lines | Status | Notes |
|-------|-------|--------|-------|
| `crates/core/` | ~large | ✅ Done | 57 modules, session, tool, checkpoint, mcp, permission |
| `crates/storage/` | ~large | ✅ Done | SQLite with compaction, migration, crash recovery |
| `crates/agent/` | ~large | ✅ Done | 13 modules, build/plan/explore/review/refactor/debug agents |
| `crates/tools/` | ~large | ✅ Done | 36 modules, read/write/edit/grep/glob/git/bash/websearch |
| `crates/plugin/` | 3673 | ✅ Done | WASM runtime, hooks, tool registration |
| `crates/tui/` | ~large | ✅ Done | 42 modules, 6000+ lines, ratatui components |
| `crates/server/` | 2221 | ⚠️ Partial | Core CRUD done, execution missing |
| `crates/mcp/` | ~large | ✅ Done | Protocol, client, server, connection pool |
| `crates/lsp/` | ~large | ✅ Done | LSP client, diagnostics |
| `crates/llm/` | ~large | ✅ Done | 33+ providers |
| `crates/git/` | ~large | ✅ Done | GitHub, GitLab, workflow triggers |
| `crates/config/` | 1600+ | ✅ Done | Real config logic |
| `crates/cli/` | ~large | ✅ Done | 44 files, desktop/web/serve commands |
| `crates/control-plane/` | 2351 | ✅ Done | ACP transport (extra, not in PRD) |
| `crates/auth/` | ~large | ✅ Done | JWT, OAuth, argon2/bcrypt, AES-GCM |
| `crates/sdk/` | ~small | ✅ Done | Client library |
| `crates/permission/` | ~medium | ✅ Done | PermissionManager, ApprovalQueue |
| `crates/ratatui-testing/` | ~medium | ⚠️ Partial | PtySimulator partial, 4 stubs remaining |

### 2.2 Phase Status

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| Phase 0 | Project Foundation | ✅ Done | 100% |
| Phase 1 | Authority (Core/Config/Storage/Server) | ✅ Done | ~98% |
| Phase 2 | Runtime Core (Agent/Tools/Plugin/TUI Plugin) | ✅ Done | ~95% |
| Phase 3 | Infrastructure (MCP/LSP/Providers/Formatters/Skills/TUI) | ✅ Done | ~92% |
| Phase 4 | Interface (Desktop/Web/GitHub-GitLab) | ✅ Done | ~95% |
| Phase 5 | Hardening (Compatibility/Convention) | ✅ Done | ~90% |
| Phase 6 | Release Qualification | ❌ Not Started | ~0% |

---

## 3. PRD Coverage Analysis

### 3.1 Required vs Implemented

| PRD Requirement | Status | Implementation | Gap |
|-----------------|--------|----------------|-----|
| **Crates** | | | |
| core | ✅ Required | ✅ Full | None |
| cli | ✅ Required | ✅ Full | None |
| llm | ✅ Required | ✅ Full | None |
| tools | ✅ Required | ✅ Full | None |
| agent | ✅ Required | ✅ Full | None |
| tui | ✅ Required | ✅ Full | None |
| lsp | ✅ Required | ✅ Full | None |
| storage | ✅ Required | ✅ Full | None |
| server | ✅ Required | ⚠️ Partial | Execution missing |
| auth | ✅ Required | ✅ Full | None |
| permission | ✅ Required | ✅ Full | None |
| plugin | ✅ Required | ✅ Full | None |
| git | ✅ Required | ✅ Full | None |
| mcp | ✅ Required | ✅ Full | None |
| sdk | ✅ Required | ✅ Full | None |
| config | ✅ Required | ✅ Full | None |
| control-plane | ❌ Not in PRD | ✅ Extra | Extra |
| **Providers** | | | |
| OpenAI | ✅ Required | ✅ Full | None |
| Anthropic Claude | ✅ Required | ✅ Full | None |
| Ollama (local) | ✅ Required | ✅ Full | None |
| Azure | Not in PRD | ✅ Implemented | Extra |
| AWS Bedrock | Not in PRD | ✅ Implemented | Extra |
| **Tools** | | | |
| Read | P0 | ✅ Full | None |
| Write | P0 | ✅ Full | None |
| Edit | P0 | ✅ Full | None |
| Grep | P0 | ✅ Full | None |
| Glob | P1 | ✅ Full | None |
| Git | P1 | ✅ Full | None |
| Bash | P1 | ✅ Full | None |
| WebSearch | P2 | ✅ Full | None |
| **Agent Modes** | | | |
| Build | Required | ✅ Full | None |
| Plan | Required | ✅ Full | None |
| General | Required | ✅ Full | None |

### 3.2 API Endpoint Comparison

| Endpoint | PRD Spec | Implementation | Status |
|----------|----------|---------------|--------|
| GET /api/status | ✅ | ❌ Missing | **P1 Gap** |
| POST /api/session | ✅ | ✅ | Complete |
| GET /api/session/{id} | ✅ | ✅ | Complete |
| POST /api/session/{id}/execute | ✅ | ❌ Missing | **P0 Gap** |
| GET /api/session/{id}/history | ✅ | ✅ (messages) | Complete |
| WS /api/ws | ⚠️ Implied | ✅ | Complete |
| ACP routes | ✅ | ✅ | 100% |

### 3.3 WebSocket/Streaming Status

| Feature | Status | Notes |
|---------|--------|-------|
| `/api/ws` WebSocket endpoint | ✅ Exists | Not integrated with agent execution |
| `/api/acpws` ACP WebSocket | ✅ Exists | Not integrated with agent execution |
| `/api/sse` endpoint | ✅ Exists | Not integrated with agent execution |
| Real-time streaming in `run_prompt` | ❌ Missing | Completes entire request before responding |
| Agent execution streaming | ❌ Missing | P0 Gap |

---

## 4. Gap Analysis - P0/P1/P2 Issues

### 4.1 P0 - Critical (Blocking)

| ID | Issue | Module | Impact | Fix Required |
|----|-------|--------|--------|--------------|
| **P0-21-1** | `/api/sessions/{id}/execute` endpoint missing | server | Cannot execute agent tasks via HTTP API | Implement agent execution endpoint integrating ToolRegistry with LLM |
| **P0-21-2** | WebSocket streaming not integrated with agent execution | server | No real-time agent output via WebSocket | Connect agent runtime events to WebSocket stream |

#### Gap Detail: P0-21-1 - Missing Execute Endpoint

**Current State:**
- `POST /api/run` performs basic LLM chat only
- Does not integrate with ToolRegistry for actual tool execution
- Uses `run_prompt` function which completes entire request before responding

**Required Implementation:**
```
POST /api/sessions/{id}/execute
- Accept: { "prompt": string, "mode"?: "build"|"plan"|"general" }
- Integrate ToolRegistry for tool discovery and execution
- Use AgentExecutor for agent lifecycle
- Return: Server-Sent Events or WebSocket stream of execution updates
```

**Implementation Path:**
1. Create `server/src/routes/execute.rs`
2. Integrate `crates/agent/` with tool registry
3. Add streaming response support
4. Add authentication middleware

#### Gap Detail: P0-21-2 - WebSocket Streaming Not Integrated

**Current State:**
- WebSocket endpoints exist (`/api/ws`, `/api/acpws`)
- Do not stream agent execution output
- No connection to agent runtime events

**Required Implementation:**
1. Create event emitter in agent runtime
2. Broadcast tool execution events to WebSocket
3. Stream LLM output tokens as they arrive
4. Handle client disconnection gracefully

---

### 4.2 P1 - High Priority

| ID | Issue | Module | Impact | Fix Required |
|----|-------|--------|--------|--------------|
| **P1-21-1** | Missing `GET /api/status` endpoint | server | API inconsistency with PRD | Add `/api/status` with server status info |
| **P1-21-2** | Streaming response in run endpoint | server | No progressive response | Implement chunked/streaming response |
| **P1-21-3** | Permission reply not connected | server | Logs decision but doesn't update state | Connect `permission_reply` handler to permission system |

#### Gap Detail: P1-21-1 - Missing Status Endpoint

**PRD Specifies:**
```
GET /api/status - Server status
```

**Current Implementation:**
- Only `/health` exists
- No `/api/status` endpoint

**Required Implementation:**
```rust
// GET /api/status
Response: {
    "version": "string",
    "status": "running"|"degraded"|"error",
    "uptime_seconds": u64,
    "active_sessions": u32,
    "total_sessions": u32,
    "providers": Vec<ProviderStatus>,
    "plugins": Vec<PluginStatus>
}
```

#### Gap Detail: P1-21-2 - Streaming Response

**Current State:**
- `run_prompt` in `server/src/routes/run.rs`
- Completes entire LLM request before responding
- No progressive updates

**Required Implementation:**
- Use `actix-web` streaming responses
- Stream LLM tokens as they arrive
- Support `Accept: text/event-stream` header

#### Gap Detail: P1-21-3 - Permission Reply Handler

**Current State:**
- `permission_reply` handler exists in `server/src/routes/permission.rs`
- Logs decision but doesn't propagate to permission system
- Doesn't update `ApprovalQueue` state

**Required Implementation:**
- Connect to `PermissionManager`
- Update `ApprovalQueue` with user decisions
- Trigger re-evaluation of pending requests

---

### 4.3 P2 - Medium Priority

| ID | Issue | Module | Impact | Fix Required |
|----|-------|--------|--------|--------------|
| **P2-21-1** | SDK documentation for crates.io | sdk | Limited SDK usability | Add cargo doc comments and examples |
| **P2-21-2** | LSP full IDE integration | lsp | IDE integration incomplete | Document IDE extension support |
| **P2-21-3** | Web UI | out of scope | No browser-based UI | Future roadmap item |
| **P2-21-4** | Experimental features stabilization | various | API may break | Stabilize or remove experimental features |

---

## 5. Feature Requirements

### 5.1 New Feature IDs (Iteration-21)

| ID | Feature | Component | Priority | Status |
|----|---------|-----------|----------|--------|
| FR-024 | Session Execute API | server | P0 | Not Implemented |
| FR-025 | WebSocket Agent Streaming | server | P0 | Not Implemented |
| FR-026 | Server Status Endpoint | server | P1 | Not Implemented |
| FR-027 | Streaming Response Support | server | P1 | Not Implemented |
| FR-028 | Permission Reply Integration | server | P1 | Not Implemented |
| FR-029 | Hook Determinism Test | plugin | P2 | Not Implemented |
| FR-030 | Security Test Suite | server | P2 | Not Implemented |

### 5.2 Feature Requirements Detail

#### FR-024: Session Execute API

**Feature ID:** FR-024
**Priority:** P0
**PRD Reference:** PRD.md - 07-server-api.md

**Description:**
Implement `POST /api/sessions/{id}/execute` endpoint that provides full agent execution with tool access via HTTP API.

**Functional Requirements:**
- FR-024.1: Accept session ID and prompt in request body
- FR-024.2: Integrate with ToolRegistry for tool discovery
- FR-024.3: Use AgentExecutor for agent lifecycle management
- FR-024.4: Support streaming response via SSE or WebSocket
- FR-024.5: Apply session's permission context to tool execution
- FR-024.6: Return execution results (messages, tool invocations, state changes)

**API Contract:**
```rust
// POST /api/sessions/{id}/execute
Request {
    "prompt": String,
    "mode": Option<SessionMode>,  // build, plan, general
    "stream": Option<bool>        // default: true
}

Response (streaming): text/event-stream
event: tool_call
data: {"tool": "read", "params": {...}}

event: tool_result
data: {"tool": "read", "result": {...}}

event: message
data: {"role": "assistant", "content": "..."}

event: complete
data: {"session_state": {...}}
```

**Acceptance Criteria:**
- [ ] Endpoint responds to POST /api/sessions/{id}/execute
- [ ] Tools are discovered and made available during execution
- [ ] Tool execution results are returned to the caller
- [ ] Streaming works correctly with SSE
- [ ] Authentication is required
- [ ] Session state is updated after execution

---

#### FR-025: WebSocket Agent Streaming

**Feature ID:** FR-025
**Priority:** P0
**PRD Reference:** PRD.md - 07-server-api.md

**Description:**
Connect WebSocket endpoints to agent runtime for real-time streaming of agent execution output.

**Functional Requirements:**
- FR-025.1: Agent runtime emits events during execution
- FR-025.2: WebSocket handler broadcasts events to connected clients
- FR-025.3: Support multiple concurrent WebSocket connections per session
- FR-025.4: Handle client disconnection gracefully
- FR-025.5: Support both `/api/ws` and `/api/acpws` endpoints

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
- [ ] Reconnection recovers session state

---

#### FR-026: Server Status Endpoint

**Feature ID:** FR-026
**Priority:** P1
**PRD Reference:** PRD.md - 07-server-api.md

**Description:**
Implement `GET /api/status` endpoint for server health and status information.

**Functional Requirements:**
- FR-026.1: Return server version
- FR-026.2: Return server status (running/degraded/error)
- FR-026.3: Return uptime in seconds
- FR-026.4: Return active and total session counts
- FR-026.5: Return provider status for each configured LLM provider
- FR-026.6: Return loaded plugin list

**API Contract:**
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

**Acceptance Criteria:**
- [ ] Endpoint responds to GET /api/status
- [ ] Returns valid JSON with all specified fields
- [ ] Does not require authentication (public endpoint)
- [ ] Response time < 100ms

---

#### FR-027: Streaming Response Support

**Feature ID:** FR-027
**Priority:** P1
**PRD Reference:** PRD.md - 07-server-api.md

**Description:**
Implement chunked/streaming response for `POST /api/run` and execute endpoints.

**Functional Requirements:**
- FR-027.1: Support `Accept: text/event-stream` header
- FR-027.2: Stream LLM tokens as they arrive
- FR-027.3: Include proper SSE formatting
- FR-027.4: Handle connection interruption gracefully

**Acceptance Criteria:**
- [ ] `curl -H "Accept: text/event-stream" -d '{"prompt":"hi"}' /api/run` streams
- [ ] Each token arrives as separate SSE event
- [ ] Connection close terminates cleanly

---

#### FR-028: Permission Reply Integration

**Feature ID:** FR-028
**Priority:** P1
**PRD Reference:** PRD.md - 07-server-api.md, 02-agent-system.md

**Description:**
Connect `permission_reply` handler to actual permission system to process user approval/denial decisions.

**Functional Requirements:**
- FR-028.1: permission_reply handler updates ApprovalQueue
- FR-028.2: Approved requests proceed to execution
- FR-028.3: Denied requests return error to agent
- FR-028.4: Decision is logged to audit trail

**Acceptance Criteria:**
- [ ] User approval triggers tool execution
- [ ] User denial returns PermissionDenied error
- [ ] Audit log records decision

---

#### FR-029: Hook Determinism Test

**Feature ID:** FR-029
**Priority:** P2
**PRD Reference:** PRD.md - 08-plugin-system.md

**Description:**
Add explicit test verifying deterministic hook execution ordering.

**Functional Requirements:**
- FR-029.1: Test verifies consistent ordering across 100 iterations
- FR-029.2: Register multiple plugins with different priorities
- FR-029.3: Verify `sorted_plugin_names()` returns same order each time

**Acceptance Criteria:**
- [ ] Test exists in `plugin/src/lib.rs`
- [ ] Test runs 100 iterations without failure
- [ ] Test documents expected ordering behavior

---

#### FR-030: Security Test Suite

**Feature ID:** FR-030
**Priority:** P2
**PRD Reference:** PRD.md - Security Requirements

**Description:**
Add security-focused tests for SQL injection, path traversal, and request validation.

**Functional Requirements:**
- FR-030.1: SQL injection tests for session/message operations
- FR-030.2: Path traversal tests for file operations
- FR-030.3: Request smuggling tests
- FR-030.4: Auth bypass tests

**Acceptance Criteria:**
- [ ] All security tests pass
- [ ] No false positives for legitimate requests
- [ ] Tests document expected behavior

---

### 5.3 Existing Feature Requirements (Carried Forward)

| ID | Feature | Component | Status |
|----|---------|-----------|--------|
| FR-023.1 | PtySimulator | ratatui-testing | ⚠️ Partial |
| FR-023.2 | BufferDiff | ratatui-testing | ❌ Not Implemented |
| FR-023.3 | StateTester | ratatui-testing | ❌ Not Implemented |
| FR-023.4 | TestDsl | ratatui-testing | ❌ Not Implemented |
| FR-023.5 | CliTester | ratatui-testing | ❌ Not Implemented |

---

## 6. Test Roadmap

### 6.1 Test Ownership by Workspace Area

| Area | Primary Location | Responsibility |
|------|------------------|----------------|
| Core entities, IDs, persistence | `crates/core/` | Project, Session, Message, Part, error types |
| Config normalization | `crates/core/` + `crates/config/` | parsing, merging, env expansion |
| Agent execution model | `crates/agent/` | primary/subagent, permission boundaries |
| Tool registry | `crates/tools/` | registry, validation, execution, caching |
| LSP integration | `crates/lsp/` | server registration, diagnostics |
| Storage | `crates/storage/` | durability, snapshots, checkpoint/revert |
| HTTP API | `crates/server/` | routes, auth, streaming |
| MCP integration | `crates/mcp/` | server lifecycle, qualification |
| TUI | `crates/tui/` | layout, commands, navigation |

### 6.2 Phase Status

| Phase | Description | Tests Required | Status |
|-------|-------------|----------------|--------|
| Phase 0 | Test Harness | Fixture helpers, mock providers | ✅ Partially done |
| Phase 1 | Authority | Core/config/API invariants | ⚠️ In progress |
| Phase 2 | Runtime | Agent/tools/plugin/TUI | ⚠️ In progress |
| Phase 3 | Subsystem | MCP/LSP/providers/formatters | ⚠️ Partial |
| Phase 4 | Interface | Desktop/web/GitHub-GitLab | ❌ Not started |
| Phase 5 | Hardening | Compatibility/convention | ❌ Not started |
| Phase 6 | Release | Performance/security/recovery | ❌ Not started |

### 6.3 Required Test Coverage

| Test Category | Current | Required | Gap |
|---------------|---------|----------|-----|
| Unit tests (inline) | ✅ Extensive | ✅ Complete | None |
| Integration tests | ✅ 14 files | ✅ Complete | None |
| TUI tests | ⚠️ Partial | ratatui-testing | ratatui-testing incomplete |
| Route-group tests | ❌ Missing | MCP/config/provider routes | P2-1 |
| API negative tests | ⚠️ Partial | malformed requests | P2-2 |
| Hook determinism | ❌ Missing | 100-iteration test | P2-3 |
| Security tests | ❌ Missing | injection/traversal | P2-4 |

---

## 7. Technical Debt

### 7.1 Active Technical Debt

| TD | Item | Location | Severity | Action | Status |
|----|------|----------|----------|--------|--------|
| TD-001 | Empty `crates/config/` crate | config | **RESOLVED** | N/A | Fixed |
| TD-002 | DirectoryScanner discovery mismatch | tools | **RESOLVED** | N/A | Fixed |
| TD-003 | Custom tools not registered | tools | **RESOLVED** | N/A | Fixed |
| TD-004 | Non-deterministic hook execution | plugin | **RESOLVED** | N/A | Fixed |
| TD-005 | Plugin register_tool() missing | plugin | **RESOLVED** | N/A | Fixed |
| TD-006 | ACP transport layer E2E | control-plane | **RESOLVED** | N/A | Fixed |
| TD-007 | Deprecated `mode` field | config | DEFERRED | Remove in v4.0 | Deferred |
| TD-008 | Deprecated `tools` field | config | DEFERRED | Remove after migration | Deferred |
| TD-009 | Deprecated `theme` field | config | **RESOLVED** | Moved to tui.json | Fixed |
| TD-010 | Deprecated `keybinds` field | config | **RESOLVED** | Moved to tui.json | Fixed |
| TD-011 | Duplicate `directory_scanner.rs` | config/core | **RESOLVED** | Removed duplicate | Fixed |
| TD-012 | Two ToolRegistry impls | core/tools | DOCUMENTED | Intentional separation | Fixed |
| TD-013 | ratatui-testing BufferDiff | testing | MEDIUM | Implement cell-by-cell diff | NOT FIXED |
| TD-014 | ratatui-testing StateTester | testing | MEDIUM | Implement state capture | NOT FIXED |
| TD-015 | ratatui-testing TestDsl | testing | MEDIUM | Implement fluent DSL | NOT FIXED |
| TD-016 | ratatui-testing CliTester | testing | MEDIUM | Implement CLI testing | NOT FIXED |
| TD-017 | Run endpoint basic LLM only | server | HIGH | Refactor to use AgentExecutor | NOT FIXED |
| TD-018 | Permission reply not connected | server | HIGH | Connect to PermissionManager | NOT FIXED |
| TD-019 | Experimental GitLab Duo | git | MEDIUM | Stabilize or remove | NOT FIXED |

---

## 8. Acceptance Criteria

### 8.1 P0 Acceptance Criteria (Must Pass Before Release)

#### FR-024: Session Execute API
- [ ] `POST /api/sessions/{session_id}/execute` endpoint exists
- [ ] Request with valid session ID and prompt returns 200
- [ ] Tools from ToolRegistry are available during execution
- [ ] Tool execution results are included in response
- [ ] Invalid session ID returns 404
- [ ] Unauthenticated request returns 401

#### FR-025: WebSocket Agent Streaming
- [ ] WebSocket connects to `/api/ws`
- [ ] Tool call events appear in WebSocket during execution
- [ ] LLM tokens stream in real-time
- [ ] Client disconnection doesn't cause server error
- [ ] Multiple clients can connect simultaneously

### 8.2 P1 Acceptance Criteria

#### FR-026: Server Status Endpoint
- [ ] `GET /api/status` returns 200
- [ ] Response contains version, status, uptime_seconds
- [ ] Response contains active_sessions, total_sessions
- [ ] Response contains providers array
- [ ] Response contains plugins array
- [ ] Endpoint is accessible without authentication

#### FR-027: Streaming Response Support
- [ ] `POST /api/run` with `Accept: text/event-stream` streams
- [ ] Each LLM token appears as separate SSE event
- [ ] Connection close terminates cleanly

#### FR-028: Permission Reply Integration
- [ ] `POST /api/permission/reply` updates ApprovalQueue
- [ ] Approved permissions allow tool execution
- [ ] Denied permissions return error
- [ ] Decision is logged

### 8.3 P2 Acceptance Criteria

#### FR-023 (ratatui-testing Framework)
- [ ] PtySimulator injects KeyEvent and MouseEvent
- [ ] BufferDiff compares buffers cell-by-cell
- [ ] StateTester captures and compares state
- [ ] TestDsl composes all components
- [ ] CliTester spawns processes and captures output

#### FR-029: Hook Determinism Test
- [ ] Test verifies 100-iteration consistency
- [ ] Test documents ordering behavior

#### FR-030: Security Test Suite
- [ ] SQL injection tests pass (no injection possible)
- [ ] Path traversal tests pass (no traversal possible)
- [ ] Request validation tests pass

---

## 9. Recommendations

### 9.1 Immediate Actions (P0)

1. **Implement `/api/sessions/{id}/execute` endpoint (FR-024)**
   - Create `server/src/routes/execute.rs`
   - Integrate `crates/agent/src/executor.rs` with tool registry
   - Add streaming response support via SSE
   - Add integration tests

2. **Integrate WebSocket with agent runtime (FR-025)**
   - Create event emitter trait in agent runtime
   - Implement WebSocket broadcast for agent events
   - Add connection management for multiple clients
   - Add integration tests

### 9.2 Short-term Actions (P1)

3. **Add `GET /api/status` endpoint (FR-026)**
   - Create status route handler
   - Collect provider and plugin status
   - Add monitoring endpoint

4. **Implement streaming response (FR-027)**
   - Modify `run_prompt` to stream
   - Add SSE formatting
   - Test with curl

5. **Connect permission reply handler (FR-028)**
   - Integrate with PermissionManager
   - Update ApprovalQueue on decision
   - Add audit logging

### 9.3 Medium-term Actions (P2)

6. **Complete ratatui-testing framework (FR-023)**
   - Implement BufferDiff cell-by-cell comparison
   - Implement StateTester state capture
   - Implement TestDsl fluent API
   - Implement CliTester process management

7. **Add hook determinism test (FR-029)**
   - Add 100-iteration test in plugin tests
   - Document expected ordering

8. **Add security test suite (FR-030)**
   - Add SQL injection tests
   - Add path traversal tests
   - Add request validation tests

### 9.4 Phase 6: Release Qualification

When P0/P1/P2 items are complete:

1. Run full test suite: `cargo test --all-features`
2. Run clippy: `cargo clippy --all-targets --all-features -- -D warnings`
3. Run formatting: `cargo fmt --all -- --check`
4. Run doc tests: `cargo test --doc`
5. Run benchmarks: `cargo bench --all`
6. Memory profiling
7. Security audit
8. Documentation completeness check

---

## Appendix A: File Structure

```
opencode-rust/
├── iterations/
│   └── iteration-21/
│       ├── spec_v21.md              # This document
│       ├── PRD.md                   # Product requirements
│       ├── gap_analysis.md          # Gap analysis report
│       └── 16-test-plan.md         # Test roadmap
├── crates/
│   ├── core/                        # 57 modules
│   ├── storage/                     # SQLite persistence
│   ├── agent/                       # 13 modules
│   ├── tools/                       # 36 modules
│   ├── plugin/                      # WASM runtime
│   ├── tui/                         # 42 modules
│   ├── server/                      # 18 route modules (execute missing)
│   ├── mcp/                         # Protocol, client, server
│   ├── lsp/                         # LSP client
│   ├── llm/                         # 33+ providers
│   ├── git/                         # GitHub, GitLab
│   ├── config/                      # Config system
│   ├── cli/                         # 44 command files
│   ├── control-plane/               # ACP transport
│   ├── auth/                        # JWT, OAuth
│   ├── sdk/                         # Client library
│   ├── permission/                  # Permission system
│   └── ratatui-testing/             # TUI testing framework
└── tests/
    └── src/
        └── common/                  # Test fixtures
```

---

## Appendix B: Feature Requirements Index

| ID | Feature | Component | Priority | Status |
|----|---------|-----------|----------|--------|
| FR-023.1 | PtySimulator | ratatui-testing | P2 | ⚠️ Partial |
| FR-023.2 | BufferDiff | ratatui-testing | P2 | ❌ Not Implemented |
| FR-023.3 | StateTester | ratatui-testing | P2 | ❌ Not Implemented |
| FR-023.4 | TestDsl | ratatui-testing | P2 | ❌ Not Implemented |
| FR-023.5 | CliTester | ratatui-testing | P2 | ❌ Not Implemented |
| FR-024 | Session Execute API | server | **P0** | ❌ Not Implemented |
| FR-025 | WebSocket Agent Streaming | server | **P0** | ❌ Not Implemented |
| FR-026 | Server Status Endpoint | server | P1 | ❌ Not Implemented |
| FR-027 | Streaming Response Support | server | P1 | ❌ Not Implemented |
| FR-028 | Permission Reply Integration | server | P1 | ❌ Not Implemented |
| FR-029 | Hook Determinism Test | plugin | P2 | ❌ Not Implemented |
| FR-030 | Security Test Suite | server | P2 | ❌ Not Implemented |

---

## Appendix C: Iteration History

| Iteration | Date | Key Changes |
|-----------|------|-------------|
| 15 | 2026-04-13 | Initial PRD analysis, 3 P0 issues identified |
| 16 | 2026-04-14 | ACP E2E tests (1083 lines), Phase 6 tests |
| 17 | 2026-04-14 | P1 items progress, comprehensive spec |
| 18 | 2026-04-14 | Duplicate directory_scanner removed, ratatui-testing spec |
| 21 | 2026-04-15 | Gap analysis focused on HTTP API execution (P0), new FR-024 to FR-030 |

---

## Appendix D: API Endpoint Specification

### D.1 Current Implementation (Partial)

```
GET  /health                  - Health check ✅
GET  /api/config              - Config routes ✅
GET  /api/providers           - Provider routes ✅
GET  /api/models              - Model routes ✅
GET  /api/sessions            - List sessions ✅
POST /api/sessions            - Create session ✅
GET  /api/sessions/{id}       - Get session ✅
DELETE /api/sessions/{id}      - Delete session ✅
POST /api/sessions/{id}/fork  - Fork session ✅
POST /api/sessions/{id}/prompt - Prompt session ✅
POST /api/sessions/{id}/command - Run command ✅
POST /api/sessions/{id}/abort - Abort session ✅
GET  /api/sessions/{id}/messages - List messages ✅
POST /api/sessions/{id}/messages - Add message ✅
GET  /api/sessions/{id}/diff  - Get diff ✅
GET  /api/sessions/{id}/snapshots - List snapshots ✅
POST /api/sessions/{id}/revert - Revert to snapshot ✅
POST /api/sessions/{id}/share - Share session ✅
DELETE /api/sessions/{id}/share - Remove share ✅
POST /api/sessions/{id}/summarize - Summarize session ✅
GET  /api/share/{id}          - Get shared session ✅
GET  /api/run                 - Run prompt (basic LLM only) ⚠️
WS   /api/ws                  - WebSocket ✅ (not integrated)
SSE  /api/sse                 - SSE ✅ (not integrated)
GET  /api/acp/status         - ACP status ✅
POST /api/acp/handshake       - ACP handshake ✅
POST /api/acp/connect         - ACP connect ✅
POST /api/acp/ack             - ACP acknowledge ✅
GET  /api/acp/events          - ACP events stream ✅
WS   /api/acpws               - ACP WebSocket ✅ (not integrated)
```

### D.2 Required Additions (Iteration-21)

```
POST /api/sessions/{id}/execute - Full agent execution ⬅️ **NEW FR-024**
GET  /api/status               - Server status ⬅️ **NEW FR-026**
```

---

*Document generated: 2026-04-15*
*Iteration: 21*
*Phase: Gap Resolution & Implementation*