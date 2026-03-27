## ADDED Requirements

### Requirement: REST API Endpoints
The server SHALL expose REST API endpoints matching the TypeScript target's API.

#### Scenario: Get Models Endpoint
- **WHEN** GET request to /api/models
- **THEN** returns list of available models as JSON

#### Scenario: Get Providers Endpoint
- **WHEN** GET request to /api/providers
- **THEN** returns list of configured providers as JSON

#### Scenario: Get Sessions Endpoint
- **WHEN** GET request to /api/sessions
- **THEN** returns list of sessions with pagination

#### Scenario: Get Specific Session
- **WHEN** GET request to /api/sessions/{id}
- **THEN** returns session details

#### Scenario: Run Prompt Endpoint
- **WHEN** POST request to /api/run with prompt
- **THEN** starts processing and returns session ID

#### Scenario: Get Config Endpoint
- **WHEN** GET request to /api/config
- **THEN** returns current configuration

#### Scenario: Update Config Endpoint
- **WHEN** PATCH request to /api/config with changes
- **THEN** updates configuration and returns new config

### Requirement: WebSocket Support
The server SHALL support WebSocket connections for real-time updates.

#### Scenario: WebSocket Connection
- **WHEN** client connects to WebSocket endpoint
- **THEN** connection is established and client can receive events

#### Scenario: Session Updates via WS
- **WHEN** session messages are added
- **THEN** connected clients receive real-time updates via WebSocket

#### Scenario: Tool Execution Progress via WS
- **WHEN** tools are executed during agent processing
- **THEN** progress updates are sent via WebSocket

### Requirement: Server-Sent Events (SSE)
The server SHALL support SSE for streaming updates to clients.

#### Scenario: SSE Connection
- **WHEN** client connects to SSE endpoint
- **THEN** connection is established and client receives event stream

#### Scenario: Agent Stream via SSE
- **WHEN** agent processes a prompt with tool calls
- **THEN** intermediate steps are sent via SSE

#### Scenario: Session Events via SSE
- **WHEN** session is updated (messages added, etc.)
- **THEN** updates are broadcast via SSE to subscribed clients