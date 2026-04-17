# WebSocket vs SSE Feature Comparison

**Date:** 2026-04-17
**Task:** P1-037
**Status:** Complete

## Overview

This document provides a feature comparison between WebSocket (`routes/ws/mod.rs`) and Server-Sent Events (`routes/sse.rs`) implementations in the OpenCode RS server.

## Architecture Summary

| Aspect | WebSocket | SSE |
|--------|-----------|-----|
| **Direction** | Bidirectional | Unidirectional |
| **Protocol** | `actix_ws` (full WS protocol) | HTTP + event-stream |
| **Client → Server** | Yes (text frames) | No (requires separate HTTP POST) |
| **Server → Client** | Yes (text frames) | Yes (SSE events) |
| **Reconnection** | Token + sequence replay | Last-Event-ID + token |

## Feature Comparison

### 1. Connection Establishment

**WebSocket:**
- Path: `/{session_id}` or `/?session_id=X`
- Uses `actix_ws::handle()` for WebSocket upgrade
- Returns `x-reconnect-token` header
- Connection ID format: `ws-{session_id}-{uuid}`

**SSE:**
- Path: `/{session_id}` or `/?session_id=X`
- Uses HTTP with `text/event-stream` content type
- Returns `X-Reconnect-Token` header
- Connection ID format: `sse-{session_id}-{uuid}`

### 2. Message Types

**WebSocket - Client to Server:**
```rust
enum WsClientMessage {
    Run { session_id, message, agent_type, model },
    Resume { session_id, token },
    Ping,
    Close,
}
```

**SSE - Client to Server:**
- Requires separate POST to `/{session_id}/message`
- Separate HTTP endpoint needed for sending messages

### 3. Server to Client Messages

Both use the same `StreamMessage` enum:
```rust
enum StreamMessage {
    Message { session_id, content, role },
    ToolCall { session_id, tool_name, args, call_id },
    ToolResult { session_id, call_id, output, success },
    SessionUpdate { session_id, status },
    Heartbeat { timestamp },
    Error { session_id, error, code, message },
    Connected { session_id },
}
```

### 4. Reconnection Support

**WebSocket:**
- Token-based reconnection via `reconnect_token` query param
- Client sends `Resume { session_id, token }` message
- Server replays messages after validated sequence number

**SSE:**
- `Last-Event-ID` header for automatic replay
- `reconnect_token` query parameter alternative
- Server replays messages after validated sequence number

### 5. Heartbeat

| Aspect | WebSocket | SSE |
|--------|-----------|-----|
| **Interval** | 30 seconds | Default HeartbeatManager |
| **Mechanism** | Ping/Pong frames | StreamMessage::Heartbeat |
| **Timeout** | 120 seconds | Varies |

### 6. Session Management

**WebSocket:**
- Uses `SessionHub` for multi-client session management
- Each client registers with unique connection ID
- Broadcast to all clients in a session
- Clients auto-unregister on disconnect

**SSE:**
- Connection-based tracking via `connection_monitor`
- Session replay via `reconnection_store`
- No multi-client broadcast (single connection per stream)

### 7. Event Bus Integration

Both subscribe to `InternalEvent` bus and filter by session_id:

```rust
fn event_to_stream_message(event: InternalEvent, session_id: &str) -> Option<StreamMessage> {
    let candidate = StreamMessage::from_internal_event(&event)?;
    match candidate.session_id() {
        Some(source_session) if source_session == session_id => Some(candidate),
        Some(_) => None,
        None => Some(candidate),
    }
}
```

## Use Case Recommendations

| Use Case | Recommended Protocol |
|----------|----------------------|
| Interactive agent execution | WebSocket |
| Real-time tool streaming | WebSocket |
| Simple server push | SSE |
| Browser-based clients | SSE (native EventSource) |
| Multiple concurrent clients | WebSocket (SessionHub) |
| Low-overhead streaming | WebSocket |
| Firewall-friendly (HTTP) | SSE |

## Implementation Files

| Component | File |
|-----------|------|
| WebSocket Handler | `crates/server/src/routes/ws/mod.rs` |
| WebSocket Session Hub | `crates/server/src/routes/ws/session_hub.rs` |
| SSE Handler | `crates/server/src/routes/sse.rs` |
| Shared Stream Messages | `crates/server/src/streaming/mod.rs` |
| Reconnection Store | `crates/server/src/streaming/mod.rs` |
| Connection Monitoring | `crates/server/src/streaming/conn_state.rs` |

## Verification

Both protocols are verified functional via:

1. **WebSocket Tests** (`crates/server/tests/ws_agent_streaming.rs`):
   - `test_ws_agent_streaming_connects_successfully`
   - `test_ws_agent_streaming_tool_events_broadcast`
   - `test_ws_agent_streaming_multiple_concurrent_connections`
   - `test_ws_agent_streaming_client_disconnect_handled_gracefully`

2. **SSE Tests** (`crates/server/src/routes/sse.rs`):
   - `test_sse_query_deserialization_*`
   - `test_sse_message_request_*`
   - `test_message_event_type_*`
   - `test_event_to_stream_message_*`

## Conclusion

Both WebSocket and SSE are fully functional and serve different purposes:
- **WebSocket**: Full bidirectional communication with multi-client support
- **SSE**: Simple server-to-client streaming over HTTP

The architecture correctly separates concerns, with shared `StreamMessage` types and `ReconnectionStore` for consistent message handling across both protocols.