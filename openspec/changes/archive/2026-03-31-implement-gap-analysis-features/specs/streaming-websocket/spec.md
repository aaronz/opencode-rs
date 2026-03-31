## ADDED Requirements

### Requirement: WebSocket Connection Establishment

The server SHALL accept WebSocket connections at `/ws` endpoint with optional session ID parameter.

#### Scenario: Client connects without session
- **WHEN** client connects to `/ws`
- **THEN** server creates a new session and accepts the WebSocket connection

#### Scenario: Client connects with existing session
- **WHEN** client connects to `/ws/:session_id`
- **THEN** server validates session exists and accepts the WebSocket connection

#### Scenario: Client connects to invalid session
- **WHEN** client connects to `/ws/:session_id` with non-existent session
- **THEN** server rejects connection with 404 status code

### Requirement: WebSocket Message Protocol

The server SHALL use JSON messages over WebSocket with type-based routing.

#### Scenario: Client sends agent message
- **WHEN** client sends `{"type": "message", "content": "user input"}`
- **THEN** server routes message to the session's agent for processing

#### Scenario: Server streams agent response
- **WHEN** agent produces output tokens
- **THEN** server sends `{"type": "token", "content": "..."}` messages to client

#### Scenario: Server sends tool call notification
- **WHEN** agent invokes a tool
- **THEN** server sends `{"type": "tool_call", "name": "...", "status": "..."}` message

#### Scenario: Server sends completion notification
- **WHEN** agent finishes processing
- **THEN** server sends `{"type": "done"}` message

### Requirement: WebSocket Connection Lifecycle

The server SHALL manage WebSocket connection lifecycle with proper cleanup.

#### Scenario: Client disconnects
- **WHEN** client closes WebSocket connection
- **THEN** server cleans up connection resources

#### Scenario: Connection timeout
- **WHEN** no messages received for 5 minutes
- **THEN** server closes connection with timeout status

#### Scenario: Server error during streaming
- **WHEN** agent encounters an error
- **THEN** server sends `{"type": "error", "message": "..."}` and closes connection

### Requirement: WebSocket Authentication

The server SHALL authenticate WebSocket connections using existing session auth.

#### Scenario: Authenticated connection
- **WHEN** client connects with valid auth token in headers
- **THEN** server accepts connection and associates with authenticated session

#### Scenario: Unauthenticated connection
- **WHEN** client connects without auth token
- **THEN** server rejects connection with 401 status code
