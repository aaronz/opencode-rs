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

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md) — canonical entities and invariants
- [06-configuration-system.md](./06-configuration-system.md) — config ownership and precedence
- [02-agent-system.md](./02-agent-system.md) — session/agent execution model
- [04-mcp-system.md](./04-mcp-system.md) — MCP integration expectations
- [10-provider-model-system.md](./10-provider-model-system.md) — provider/model behavior
