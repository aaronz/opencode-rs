## ADDED Requirements

### Requirement: SSE Endpoint Establishment

The server SHALL provide Server-Sent Events endpoint at `/sse/:session_id`.

#### Scenario: Client subscribes to session
- **WHEN** client sends GET request to `/sse/:session_id` with `Accept: text/event-stream`
- **THEN** server responds with SSE stream and keeps connection open

#### Scenario: Client subscribes to invalid session
- **WHEN** client requests `/sse/:session_id` with non-existent session
- **THEN** server responds with 404 status code

### Requirement: SSE Event Format

The server SHALL send events in standard SSE format with typed data.

#### Scenario: Token event
- **WHEN** agent produces output tokens
- **THEN** server sends `event: token\ndata: {"content": "..."}\n\n`

#### Scenario: Tool call event
- **WHEN** agent invokes a tool
- **THEN** server sends `event: tool_call\ndata: {"name": "...", "status": "..."}\n\n`

#### Scenario: Done event
- **WHEN** agent finishes processing
- **THEN** server sends `event: done\ndata: {}\n\n`

#### Scenario: Error event
- **WHEN** agent encounters an error
- **THEN** server sends `event: error\ndata: {"message": "..."}\n\n`

### Requirement: SSE Reconnection Support

The server SHALL support reconnection using `Last-Event-ID` header.

#### Scenario: Client reconnects with event ID
- **WHEN** client reconnects with `Last-Event-ID: <id>` header
- **THEN** server resumes streaming from that event onwards

#### Scenario: Event ID expired
- **WHEN** client reconnects with expired event ID
- **THEN** server starts fresh stream from current state

### Requirement: SSE Client Input

The server SHALL accept client input via POST to companion endpoint.

#### Scenario: Client sends message
- **WHEN** client POSTs `{"content": "user input"}` to `/sse/:session_id/message`
- **THEN** server routes message to the session's agent

#### Scenario: Invalid POST body
- **WHEN** client POSTs malformed JSON
- **THEN** server responds with 400 status code
