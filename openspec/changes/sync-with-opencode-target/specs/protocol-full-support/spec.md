## ADDED Requirements

### Requirement: Full ACP Implementation
The system SHALL support the full Agent Control Protocol (ACP) for managing and observing remote agents.

#### Scenario: Subscribing to agent events
- **WHEN** a client subscribes to an agent's event stream via ACP
- **THEN** it MUST receive real-time updates for all tool calls and message transitions.

### Requirement: Full MCP Implementation
The system SHALL support the full Model Context Protocol (MCP) for discovering and executing tools and resources from external servers.

#### Scenario: Calling MCP tool
- **WHEN** the agent invokes a tool provided by an MCP server
- **THEN** the system MUST correctly route the call and return the result to the agent.
