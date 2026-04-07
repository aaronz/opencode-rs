# Constitution C-057: Server API Endpoints

**Version**: 1.0  
**Date**: 2026-04-07  
**Iteration**: v16  
**Status**: Adopted

---

## Preamble

This Constitution documents the required HTTP API endpoints for the OpenCode-RS server, including health checks, session management, and permission handling.

## Article 1: Endpoint Inventory

### Section 1.1: Required Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|--------------|
| GET | `/health` | Health check | ❌ |
| POST | `/api/sessions/{id}/abort` | Abort running session | ✅ |
| POST | `/api/sessions/{id}/permissions/{req_id}/reply` | Permission decision | ✅ |

### Section 1.2: Endpoint Definitions

#### GET /health

**Purpose**: Kubernetes-compatible health check for container orchestration.

**Response**:
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

**Status Codes**:
- `200 OK`: Server is healthy
- `503 Service Unavailable`: Server is shutting down (future)

---

#### POST /api/sessions/{id}/abort

**Purpose**: Abort a running session, stopping any in-progress agent execution.

**Path Parameters**:
- `id` (string): Session UUID

**Request Body**: None

**Response**:
```json
{
  "session_id": "uuid-string",
  "status": "aborted",
  "message_count": 42
}
```

**Status Codes**:
- `200 OK`: Session aborted successfully
- `404 Not Found`: Session does not exist
- `500 Internal Server Error`: Storage error

**Side Effects**:
- Sets `session.state = SessionState::Aborted`
- Persists updated session to storage

---

#### POST /api/sessions/{id}/permissions/{req_id}/reply

**Purpose**: Allow/deny a pending permission request.

**Path Parameters**:
- `id` (string): Session UUID
- `req_id` (string): Permission request ID

**Request Body**:
```json
{
  "decision": "allow"  // or "deny"
}
```

**Response**:
```json
{
  "status": "ok",
  "session_id": "uuid-string",
  "request_id": "req-uuid",
  "decision": "allow"
}
```

**Status Codes**:
- `200 OK`: Decision recorded
- `400 Bad Request`: Invalid decision value (must be "allow" or "deny")
- `500 Internal Server Error`: Storage error

## Article 2: Implementation Requirements

### Section 2.1: Framework

Server MUST use **actix-web** framework for HTTP handling.

### Section 2.2: State Management

All endpoints receive `web::Data<ServerState>` containing:
- `storage: Arc<StorageService>`
- `models: Arc<ModelRegistry>`
- `config: Arc<RwLock<Config>>`
- `event_bus: SharedEventBus`
- `reconnection_store: ReconnectionStore`

### Section 2.3: Error Handling

All errors MUST return JSON in format:
```json
{
  "error": "error_code",
  "message": "Human-readable description"
}
```

Use `json_error()` helper from `routes::error` module.

### Section 2.4: Authentication

All `/api/*` endpoints require `x-api-key` header. The `/health` endpoint is public.

## Article 3: Route Registration

### Section 3.1: Health Route

Registered in `crates/server/src/lib.rs`:
```rust
.route("/health", web::get().to(health_check))
```

### Section 3.2: Session Routes

Registered in `crates/server/src/routes/session.rs::init()`:
```rust
cfg.route("/{id}/abort", web::post().to(abort_session));
cfg.route("/{id}/permissions/{req_id}/reply", web::post().to(permission_reply));
```

## Article 4: Testing Requirements

### Section 4.1: Required Tests

| Endpoint | Test Name | Validates |
|---------|-----------|-----------|
| GET /health | `test_health_check` | Returns 200 with status "ok" |
| POST permissions | `test_permission_reply_allows_allow_decision` | "allow" returns 200 |
| POST permissions | `test_permission_reply_allows_deny_decision` | "deny" returns 200 |
| POST permissions | `test_permission_reply_rejects_invalid_decision` | Invalid returns 400 |

### Section 4.2: Test Pattern

Use actix-web's `#[actix_web::test]` attribute with `TestRequest`:

```rust
#[actix_web::test]
async fn test_health_check() {
    let req = TestRequest::default().to_http_request();
    let resp = health_check().await.respond_to(&req);
    assert_eq!(resp.status(), StatusCode::OK);
}
```

## Article 5: OpenAPI Documentation

### Section 5.1: Utoipa Integration

Server crate MUST include `utoipa` and `utoipa-actix-web` dependencies for OpenAPI 3.1 documentation.

### Section 5.2: Annotations

All public endpoints SHOULD have `#[utoipa::path]` attributes for automatic documentation generation.

## Article 6: Adoption

This Constitution is effective immediately upon merge to main branch.

---

**Ratified**: 2026-04-07  
**Expires**: Never  
**Amendments**: Requires RFC process
