# ACP Module Gap Analysis Report

**Date**: 2026-04-26
**PRD Source**: `acp.md` (Agent Communication Protocol - Client Side)
**Analysis Target**: `crates/acp/` (client-side ACP implementation)
**Current Implementation Location**: `crates/core/src/acp.rs` + `crates/control-plane/` (server-side)

---

## 1. Executive Summary

**Implementation Status**: ❌ **NOT IMPLEMENTED**

The PRD specifies a **client-side ACP module** at `crates/acp/` with `AcpClient` struct, state machine, and CLI commands. However:

1. **`crates/acp/` does not exist** - The directory and crate are missing entirely
2. **No AcpClient struct** exists - Only protocol types in `crates/core/src/acp.rs`
3. **No client state machine** - Only `AcpProtocol` handler exists (not an async client)
4. **No AcpError enum** with proper error variants
5. **CLI commands incomplete** - Only `status`, `connect`, `handshake` exist; missing `ack`
6. **No connection state transitions** - No `AcpConnectionState::Disconnected/Handshaking/Connected/Failed` enum

---

## 2. Gap Analysis Table

| Gap Item | Severity | Module | Description |修复建议 |
|---|---|---|---|---|
| `crates/acp/` crate does not exist | **P0** | Structure | Entire client crate is missing | Create `crates/acp/` with `Cargo.toml`, `src/lib.rs`, `src/client.rs`, `src/protocol.rs`, `src/cli.rs` |
| No `AcpClient` struct | **P0** | Core | PRD defines `AcpClient` with `http`, `state`, `bus` fields | Implement `AcpClient` in `client.rs` with reqwest::Client, Arc<Mutex<AcpState>>, Arc<BusService> |
| No `AcpConnectionState` enum | **P0** | Core | State machine requires `Disconnected`, `Handshaking`, `Connected`, `Failed(String)` | Create enum matching PRD spec |
| No `AcpState` struct | **P0** | Core | Missing `connection_state`, `client_id`, `server_id`, `session_token`, `capabilities`, `server_url` | Implement state struct with proper synchronization |
| No `AcpError` enum | **P0** | Error | PRD defines `NotConnected`, `HandshakeFailed`, `ConnectionFailed`, `ServerError`, `InvalidResponse`, `Http`, `State` | Implement error enum with thiserror |
| Missing `Ack` CLI command | **P1** | CLI | Only `status`, `connect`, `handshake`, `start` exist | Add `acp ack` command to handle `handshake_id` acknowledgement |
| Missing `send_message()` | **P0** | Core | `AcpClient::send_message()` not implemented | Implement async send_message with POST to `/api/acp/message` |
| Missing `disconnect()` | **P0** | Core | No disconnect method to transition to `Disconnected` state | Implement disconnect that publishes `acp.disconnected` bus event |
| Missing bus events | **P0** | Events | No `acp.connected` / `acp.disconnected` event publishing | Integrate with BusService for event publishing |
| `AcpStatus` struct not implemented | **P0** | Types | Missing `connected`, `client_id`, `capabilities`, `server_url` fields | Implement `AcpStatus` for `status()` method |
| `HandshakeRequest` type mismatch | **P1** | Types | Current uses `client_id`, `version`, `capabilities` but may not match server API | Verify and align with server-side expectations |
| `HandshakeResponse` missing `session_token` | **P1** | Types | PRD has `session_token: Option<String>` but current has only `session_id` | Add session_token to response handling |
| Missing wiremock tests | **P0** | Tests | No unit tests with MockServer for client behavior | Add tests in `crates/acp/tests/acp_tests.rs` |
| No connection state transitions test | **P0** | Tests | State machine transitions not tested | Add tests for `Disconnected → Handshaking → Connected` |
| No error handling tests | **P0** | Tests | `send_message()` error when not connected not tested | Add test for `Err(AcpError::NotConnected)` |
| Session sharing not implemented | **P2** | Feature | PRD mentions "Share local session with remote agent via ACP" | Implement session sharing functionality |
| Missing `ConnectRequest` type | **P2** | Types | Defined in PRD but not created | Implement `ConnectRequest` for `connect()` API |
| Missing `AckRequest` type | **P2** | Types | Defined in PRD but not created | Implement `AckRequest` for `ack()` API |
| `acp start` command unclear | **P2** | CLI | Server-side command vs client-side start | Clarify whether client can "start" local ACP service |
| Missing async state machine | **P0** | Core | Current `AcpProtocol` is synchronous handler, not async client | Implement async state machine with tokio |

---

## 3. P0/P1/P2 Classification

### P0 - Critical Blockers (Must Fix)

| Issue | Impact |
|---|---|
| `crates/acp/` crate missing | Cannot build ACP client |
| No `AcpClient` struct | Core functionality absent |
| No `AcpConnectionState` enum | State machine cannot function |
| No `AcpState` struct | Cannot track connection state |
| No `AcpError` enum | Error handling impossible |
| `send_message()` not implemented | Cannot send messages |
| `disconnect()` not implemented | Cannot gracefully disconnect |
| Bus events not published | Event system not integrated |
| `AcpStatus` not implemented | Cannot query status |
| No wiremock tests | Cannot verify client behavior |
| No state transition tests | Cannot verify state machine |

### P1 - Important Issues (Should Fix)

| Issue | Impact |
|---|---|
| `ack` CLI command missing | Cannot acknowledge handshake |
| `HandshakeRequest` type mismatch | May cause API incompatibility |
| `HandshakeResponse` missing `session_token` | Session management incomplete |
| No async state machine | Current is sync-only, incompatible with async API |

### P2 - Nice to Have (Consider for Future)

| Issue | Impact |
|---|---|
| Session sharing not implemented | Feature incomplete |
| `ConnectRequest` type missing | Incomplete API |
| `AckRequest` type missing | Incomplete API |
| `acp start` command ambiguous | Unclear behavior |
| Missing integration tests | Full cycle not tested |

---

## 4. Technical Debt

| Debt Item | Description | Remediation |
|---|---|---|
| Protocol types in core | `crates/core/src/acp.rs` contains protocol types that should be in `crates/acp/` | Move to `crates/acp/src/protocol.rs` |
| Duplicate `AcpHandshakeResponse` | CLI defines its own `AcpHandshakeResponse` in `crates/cli/src/cmd/acp.rs` | Import from shared `crates/acp` or `crates/core` |
| Mixed sync/async | `AcpProtocol` is sync but ACP API is async | Rewrite as async `AcpClient` |
| Hardcoded URLs | URLs like `/api/acp/handshake` are hardcoded in CLI | Move to configurable base URL |
| No connection timeout | `connect()` has no timeout configuration | Add configurable timeout |
| No retry logic | Failed connections not retried | Implement retry with backoff |
| Missing `version` in `AcpStatus` | Status doesn't report protocol version | Add `version` field |

---

## 5. Implementation Progress

### Current State

```
crates/
├── core/src/acp.rs        # Protocol types only (AcpMessage, AcpHandshakeRequest, etc.)
├── control-plane/         # Server-side ACP (handshake, transport, acp_stream)
├── cli/src/cmd/acp.rs     # CLI commands (incomplete, no ack)
└── [acp/]                # ❌ MISSING - Client crate not created
```

### What's Implemented

| Component | Status | Location |
|---|---|---|
| `AcpMessage` type | ✅ Done | `crates/core/src/acp.rs` |
| `AcpHandshakeRequest` type | ✅ Done | `crates/core/src/acp.rs` |
| `AcpHandshakeResponse` type | ✅ Done (partial) | `crates/core/src/acp.rs` |
| `AcpProtocol` struct | ✅ Done (sync only) | `crates/core/src/acp.rs` |
| CLI `acp status` | ✅ Done | `crates/cli/src/cmd/acp.rs` |
| CLI `acp connect` | ✅ Done | `crates/cli/src/cmd/acp.rs` |
| CLI `acp handshake` | ✅ Done | `crates/cli/src/cmd/acp.rs` |
| Session storage | ✅ Done | `crates/cli/src/cmd/acp.rs` |

### What's Missing

| Component | Status | PRD Requirement |
|---|---|---|
| `AcpClient` struct | ❌ Missing | Client with http, state, bus |
| `AcpState` struct | ❌ Missing | connection_state, client_id, server_id, session_token, capabilities, server_url |
| `AcpConnectionState` enum | ❌ Missing | Disconnected, Handshaking, Connected, Failed(String) |
| `AcpError` enum | ❌ Missing | NotConnected, HandshakeFailed, ConnectionFailed, ServerError, InvalidResponse, Http, State |
| `AcpStatus` struct | ❌ Missing | connected, client_id, capabilities, server_url |
| `send_message()` | ❌ Missing | POST to /api/acp/message |
| `disconnect()` | ❌ Missing | Transition to Disconnected, publish acp.disconnected |
| `ack()` CLI command | ❌ Missing | POST /api/acp/ack |
| Bus event publishing | ❌ Missing | acp.connected, acp.disconnected |
| State transition tests | ❌ Missing | Disconnected → Handshaking → Connected |
| Error handling tests | ❌ Missing | send_message when not connected returns NotConnected |
| Integration tests | ❌ Missing | Full connect → message → disconnect cycle |

---

## 6. Detailed Gap Findings

### 6.1 Crate Structure Missing

**PRD Specifies**:
```
crates/acp/
├── Cargo.toml
├── src/
│   ├── lib.rs       # AcpClient, AcpError, types
│   ├── client.rs    # Client implementation
│   ├── protocol.rs  # Protocol types and serialization
│   └── cli.rs       # CLI command handlers
└── tests/
    └── acp_tests.rs
```

**Current State**: `crates/acp/` does not exist at all.

### 6.2 AcpClient Missing

**PRD Defines**:
```rust
pub struct AcpClient {
    http: reqwest::Client,
    state: Arc<Mutex<AcpState>>,
    bus: Arc<BusService>,
}
```

**Current State**: No such struct exists. Only `AcpProtocol` (sync handler) in `crates/core/src/acp.rs`.

### 6.3 State Machine Not Implemented

**PRD State Machine**:
```
Disconnected
    │
    │ connect()
    ▼
Handshaking ──(handshake success)──► Connected
    │                                    │
    │                                    │ disconnect()
    │                                    ▼
    └──(handshake failure)──► Failed
```

**Current State**: No `AcpConnectionState` enum, no state transitions.

### 6.4 AcpError Missing

**PRD Defines** 7 error variants:
- `NotConnected`
- `HandshakeFailed(String)`
- `ConnectionFailed(String)`
- `ServerError(String)`
- `InvalidResponse(String)`
- `Http(reqwest::Error)`
- `State(String)`

**Current State**: No `AcpError` enum in `crates/acp/`. Only generic error handling.

### 6.5 CLI Commands Incomplete

**PRD Specifies**: `status`, `handshake`, `connect`, `ack`

**Current CLI** (`crates/cli/src/cmd/acp.rs`):
- ✅ `Start` - prints ready status
- ✅ `Connect { url }` - POST to /api/acp/connect
- ✅ `Handshake { client_id, version, capabilities }` - POST to /api/acp/handshake
- ✅ `Status` - GET /api/acp/status and show stored session
- ❌ `Ack` - **MISSING** - should POST to /api/acp/ack with `handshake_id` and `accepted`

### 6.6 Bus Integration Missing

**PRD States**:
- `connect()` should publish `acp.connected` event with `{ server_id, capabilities }`
- `disconnect()` should publish `acp.disconnected` event

**Current State**: No `bus.publish()` calls in any ACP code.

### 6.7 Types Mismatch

**PRD `HandshakeResponse`**:
```rust
struct HandshakeResponse {
    server_id: String,
    accepted_capabilities: Vec<String>,
    session_token: Option<String>,  // <-- Missing in current impl
}
```

**Current** (`crates/core/src/acp.rs`):
```rust
struct AcpHandshakeResponse {
    version: String,
    server_id: String,
    session_id: String,
    accepted: bool,
    error: Option<String>,
    // Missing: accepted_capabilities, session_token
}
```

---

## 7. Required Actions

### Immediate (Before Any ACP Work)

1. **Create `crates/acp/` crate** with proper Cargo.toml
2. **Define `AcpConnectionState` enum**
3. **Define `AcpState` struct**
4. **Define `AcpError` enum**
5. **Implement `AcpClient` struct with async methods**
6. **Add `send_message()` implementation**
7. **Add `disconnect()` implementation**
8. **Add bus event publishing**
9. **Write unit tests with wiremock**
10. **Add `ack` CLI command**

### Priority Order

1. Create crate structure and types
2. Implement AcpClient with state machine
3. Add CLI commands (status, connect, handshake, ack)
4. Write tests
5. Add session sharing feature

---

## 8. File Locations Reference

| File | Purpose |
|---|---|
| `crates/core/src/acp.rs` | Current partial ACP types (to be moved/consolidated) |
| `crates/control-plane/src/lib.rs` | Server-side ACP exports |
| `crates/control-plane/src/transport.rs` | Server-side transport layer |
| `crates/control-plane/src/handshake.rs` | Server-side handshake |
| `crates/control-plane/src/acp_stream.rs` | Event stream for ACP |
| `crates/cli/src/cmd/acp.rs` | Current CLI commands (incomplete) |
| `crates/config/src/lib.rs` | `AcpConfig` and `AcpSession` types |

---

*Report generated: 2026-04-26*
*Next Action: Implement `crates/acp/` crate from scratch per PRD specification*