# PRD: HTTP Server API

## Scope

This document is the sole HTTP API authority for the PRD set.

It defines:

- Canonical API ownership by resource group
- Route-shape expectations for the Rust port
- Authentication and transport expectations
- Compatibility guidance for legacy route shapes

Other PRD documents may reference the API, but they must not duplicate route tables or redefine route ownership.

---

## Source of Truth

The Rust port should derive exact route shapes and request/response schemas from the current upstream OpenCode API surface at implementation time.

For planning purposes, the PRD set treats the following as authoritative sources in order:

1. Current upstream OpenAPI / current upstream server routes
2. Current upstream subsystem specs
3. Historical specs only as legacy reference

Historical nested route documents such as older project-scoped session route specs may be useful for compatibility analysis, but they are **not** the default canonical shape for new design work unless explicitly adopted.

---

## API Design Principles

### Single Authority Rule

- This document owns HTTP route structure
- [01-core-architecture.md](./01-core-architecture.md) owns entities and invariants, not routes
- [06-configuration-system.md](./06-configuration-system.md) owns config schema, not transport routes

### Top-Level Resource Grouping

The preferred API topology is top-level grouping by resource type rather than deeply nesting all operations under project/session path chains.

Canonical resource families include:

- Global/runtime
- Project
- Session
- Message
- Permission / approval
- File and search
- Config / provider / model
- MCP / external integration
- Streaming / event transport

### Compatibility Guidance

- Legacy nested project/session routes may be documented for migration or compatibility only
- New route design for the Rust port should avoid duplicating legacy and current topologies simultaneously as canonical

---

## Canonical Resource Groups

### Global / Runtime

Global endpoints cover process/runtime concerns such as:

- Health/status
- Event streams
- Documentation / OpenAPI exposure
- Runtime-level logging or diagnostics

### Project

Project endpoints cover project discovery and project-scoped metadata, including:

- Listing projects
- Determining current/active project context
- Project initialization flows where applicable

### Session

Session endpoints cover the lifecycle of conversational execution contexts, including:

- Create/list/get/update/delete session
- Abort/fork/share/summarize/revert workflows
- Session diff/checkpoint style operations where supported

### Message

Message endpoints cover ordered session history, including:

- List messages in a session
- Append/send a message or prompt into a session
- Retrieve specific message records

### Permission / Approval

Permission endpoints cover human-in-the-loop approval state, including:

- Listing pending permission requests
- Responding to an approval request associated with a session/run

### Files and Search

File/search endpoints cover read-only and session-adjacent workspace inspection, including:

- File lookup
- Content retrieval
- Symbol or grep-style search
- File status/diff views where supported

### Config / Providers / Models

These endpoints expose runtime configuration and provider/model metadata needed by clients, including:

- Get/update effective configuration
- Enumerate configured providers or provider auth state
- List/select model metadata

### MCP / External Integration

MCP endpoints cover runtime MCP server visibility and integration management.

### Streaming Transport

Streaming/event endpoints provide incremental updates to clients, typically via SSE and/or websocket-style transport depending on the host interface.

---

## Route Shape Guidance

For the Rust port, exact paths should be generated from the selected upstream API baseline, but the following conventions should be preserved:

- Resource-oriented plural/singular naming must be chosen consistently within one topology
- A session identifier is the primary locator for message, permission-response, diff, summarize, share, and revert actions
- File/search endpoints may be global or session-adjacent depending on final upstream parity, but ownership must remain documented here only

---

## Authentication

The server API must support authenticated local and remote client access appropriate to the hosting mode.

Common patterns include:

- Local password/basic-auth style protection for desktop/web server modes
- Provider-specific auth flows exposed through provider/config subsystems
- Session sharing controls for public or semi-public access paths

Exact auth payloads and handshake details should follow the chosen upstream API baseline.

---

## Transport and Streaming

The API should support both request/response and event-driven interaction patterns.

Expected transport modes:

- Standard HTTP JSON request/response endpoints
- Server-sent events for live updates where appropriate
- Optional websocket or equivalent bidirectional streaming where the selected upstream baseline uses it

The PRD does not require one specific streaming protocol unless upstream parity requires it.

---

## Versioning and Compatibility Notes

- The Rust port should define one primary API versioning strategy and apply it consistently
- Historical route shapes may be retained in a compatibility layer, but they must be clearly labeled legacy
- The PRD set should not treat both legacy nested project/session paths and newer top-level resource groups as simultaneously canonical

---

## Legacy / Historical Route Shapes

Historical upstream specs include nested project/session route families such as:

```text
/project/:projectID/session/:sessionID/...
```

These are useful as migration/reference material only unless the Rust port explicitly commits to preserving them.

If legacy compatibility is implemented, it should be documented as an adapter layer rather than the primary route topology.

---

## API Schemas

### Session Schemas

#### Create Session

**Request:**
```json
POST /api/sessions
Content-Type: application/json

{
  "project_id": "string (optional)",
  "agent_type": "build | plan | general | explore | debug | refactor | review",
  "parent_session_id": "string (optional, for fork)"
}
```

**Response (201):**
```json
{
  "id": "sess_abc123",
  "project_id": "proj_xyz",
  "agent_type": "build",
  "state": "Idle",
  "parent_session_id": null,
  "created_at": "2026-04-26T10:00:00Z",
  "updated_at": "2026-04-26T10:00:00Z"
}
```

#### Get Session

**Request:**
```json
GET /api/sessions/{session_id}
```

**Response (200):**
```json
{
  "id": "sess_abc123",
  "project_id": "proj_xyz",
  "state": "Idle",
  "agent_type": "build",
  "message_count": 42,
  "preview": "Last message content...",
  "created_at": "2026-04-26T10:00:00Z",
  "updated_at": "2026-04-26T10:30:00Z"
}
```

#### List Sessions

**Request:**
```json
GET /api/sessions
```

**Response (200):**
```json
{
  "sessions": [
    {
      "id": "sess_abc123",
      "project_id": "proj_xyz",
      "state": "Idle",
      "agent_type": "build",
      "message_count": 42,
      "preview": "Last message...",
      "created_at": "2026-04-26T10:00:00Z",
      "updated_at": "2026-04-26T10:30:00Z"
    }
  ],
  "total": 1
}
```

#### Fork Session

**Request:**
```json
POST /api/sessions/{session_id}/fork
Content-Type: application/json

{
  "fork_at_message": 10  // optional, forks at specific message index
}
```

**Response (201):**
```json
{
  "id": "sess_child456",
  "parent_session_id": "sess_abc123",
  "lineage_path": "sess_abc123",
  "state": "Idle",
  "created_at": "2026-04-26T11:00:00Z"
}
```

#### Share Session

**Request:**
```json
POST /api/sessions/{session_id}/share
Content-Type: application/json

{
  "mode": "manual | auto",
  "expires_at": "2026-04-27T10:00:00Z (optional)"
}
```

**Response (200):**
```json
{
  "share_url": "https://opencode.example.com/share/sess_abc123",
  "share_id": "sess_abc123",
  "mode": "manual",
  "expires_at": "2026-04-27T10:00:00Z"
}
```

---

### Message Schemas

#### Send Message

**Request:**
```json
POST /api/sessions/{session_id}/messages
Content-Type: application/json

{
  "content": "string",
  "role": "user | system | assistant"
}
```

**Response (200):**
```json
{
  "id": "msg_789",
  "session_id": "sess_abc123",
  "role": "user",
  "content": "Hello!",
  "created_at": "2026-04-26T10:05:00Z"
}
```

#### List Messages

**Request:**
```json
GET /api/sessions/{session_id}/messages?limit=50&offset=0
```

**Response (200):**
```json
{
  "messages": [
    {
      "id": "msg_001",
      "session_id": "sess_abc123",
      "role": "user",
      "content": "Hello!",
      "created_at": "2026-04-26T10:00:00Z"
    },
    {
      "id": "msg_002",
      "session_id": "sess_abc123",
      "role": "assistant",
      "content": "Hi!",
      "created_at": "2026-04-26T10:00:01Z"
    }
  ],
  "total": 42,
  "has_more": true
}
```

---

### Project Schemas

#### Create Project

**Request:**
```json
POST /api/projects
Content-Type: application/json

{
  "name": "my-project",
  "root_path": "/path/to/project",
  "worktree_root": "/path/to/worktree (optional)"
}
```

**Response (201):**
```json
{
  "id": "proj_xyz",
  "name": "my-project",
  "root_path": "/path/to/project",
  "worktree_root": null,
  "created_at": "2026-04-26T10:00:00Z"
}
```

#### Get Project

**Request:**
```json
GET /api/projects/{project_id}
```

**Response (200):**
```json
{
  "id": "proj_xyz",
  "name": "my-project",
  "root_path": "/path/to/project",
  "worktree_root": null,
  "session_count": 5,
  "created_at": "2026-04-26T10:00:00Z"
}
```

#### List Projects

**Request:**
```json
GET /api/projects
```

**Response (200):**
```json
{
  "projects": [
    {
      "id": "proj_xyz",
      "name": "my-project",
      "root_path": "/path/to/project",
      "session_count": 5
    }
  ],
  "total": 1
}
```

---

### Provider Schemas

#### List Providers

**Request:**
```json
GET /api/providers
```

**Response (200):**
```json
{
  "providers": [
    {
      "name": "openai",
      "display_name": "OpenAI",
      "models": ["gpt-4o", "gpt-4", "gpt-3.5-turbo"],
      "auth_status": "configured"
    },
    {
      "name": "anthropic",
      "display_name": "Anthropic",
      "models": ["claude-3-5-sonnet-20240620", "claude-3-opus"],
      "auth_status": "not_configured"
    }
  ]
}
```

#### Get Provider

**Request:**
```json
GET /api/providers/{provider_name}
```

**Response (200):**
```json
{
  "name": "openai",
  "display_name": "OpenAI",
  "default_model": "gpt-4o",
  "models": [
    {
      "id": "gpt-4o",
      "context_window": 128000,
      "supports_streaming": true,
      "pricing": {
        "input": 0.005,
        "output": 0.015
      }
    }
  ],
  "auth_status": "configured"
}
```

---

### Config Schemas

#### Get Config

**Request:**
```json
GET /api/config
```

**Response (200):**
```json
{
  "server": {
    "port": 8080,
    "host": "127.0.0.1"
  },
  "agent": {
    "default": "build",
    "build": {
      "model": "gpt-4o",
      "temperature": 0.7
    }
  },
  "providers": {
    "openai": {
      "enabled": true
    }
  }
}
```

#### Update Config

**Request:**
```json
PATCH /api/config
Content-Type: application/json

{
  "server": {
    "port": 9000
  }
}
```

**Response (200):**
```json
{
  "server": {
    "port": 9000,
    "host": "127.0.0.1"
  },
  "updated": true
}
```

---

### Error Response Schema

All endpoints may return errors in the following format:

```json
HTTP 4xx/5xx

{
  "error": {
    "code": "SESSION_NOT_FOUND",
    "message": "Session 'sess_nonexistent' not found",
    "details": {}
  }
}
```

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | `INVALID_REQUEST` | Malformed request |
| 401 | `UNAUTHORIZED` | Authentication required |
| 403 | `FORBIDDEN` | Permission denied |
| 404 | `SESSION_NOT_FOUND` | Session does not exist |
| 404 | `PROJECT_NOT_FOUND` | Project does not exist |
| 409 | `CONFLICT` | State conflict (e.g., session already running) |
| 422 | `VALIDATION_ERROR` | Request validation failed |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Server error |
| 503 | `SERVICE_UNAVAILABLE` | Service unavailable |

---

### Streaming / SSE

For streaming responses (e.g., agent execution):

**Endpoint:**
```json
GET /api/sessions/{session_id}/stream
```

**Response (200):**
```
Content-Type: text/event-stream

event: message
data: {"type": "chunk", "content": "Hello"}

event: message
data: {"type": "chunk", "content": " world"}

event: done
data: {"type": "done", "final_content": "Hello world"}
```

**Event Types:**
| Event | Description |
|-------|-------------|
| `chunk` | Partial content response |
| `tool_call` | Tool invocation started |
| `tool_result` | Tool execution completed |
| `error` | Error occurred |
| `done` | Response complete |

---

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md) — canonical entities and invariants
- [06-configuration-system.md](./06-configuration-system.md) — config ownership and precedence
- [02-agent-system.md](./02-agent-system.md) — session/agent execution model
- [04-mcp-system.md](./04-mcp-system.md) — MCP integration expectations
- [10-provider-model-system.md](./10-provider-model-system.md) — provider/model behavior
- [ERROR_CODE_CATALOG.md](../ERROR_CODE_CATALOG.md) — error codes

---

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md) — canonical entities and invariants
- [06-configuration-system.md](./06-configuration-system.md) — config ownership and precedence
- [02-agent-system.md](./02-agent-system.md) — session/agent execution model
- [04-mcp-system.md](./04-mcp-system.md) — MCP integration expectations
- [10-provider-model-system.md](./10-provider-model-system.md) — provider/model behavior
