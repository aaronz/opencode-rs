## ADDED Requirements

### Requirement: ACP Real-time Event Streaming
The Agent Control Protocol (ACP) implementation SHALL support real-time streaming of agent events (status, tool calls, logs) to connected clients.

#### Scenario: Streaming tool call events
- **WHEN** an ACP-connected agent begins a tool execution
- **THEN** an event MUST be published to the control plane and streamed to the UI in real-time.

### Requirement: MCP Resource Discovery
The Model Context Protocol (MCP) implementation SHALL provide full discovery and schema-validation for resources exposed by external servers.

#### Scenario: Listing MCP resources
- **WHEN** the system initializes an MCP connection
- **THEN** it MUST retrieve and store the full list of available resources and their associated metadata.
