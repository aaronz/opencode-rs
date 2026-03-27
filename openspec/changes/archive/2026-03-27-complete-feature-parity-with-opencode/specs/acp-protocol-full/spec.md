## ADDED Requirements

### Requirement: ACP Agent Lifecycle
The system SHALL implement the Agent Control Protocol for managing the full lifecycle of remote agents.

#### Scenario: Spawning remote agent
- **WHEN** a request is made to spawn an agent via ACP
- **THEN** the system SHALL initialize the remote agent and return its connection metadata.

### Requirement: ACP Event Stream
The system SHALL support streaming events (status, tool calls, logs) from ACP agents to the client.

#### Scenario: Receiving tool events
- **WHEN** a remote agent performs a tool call
- **THEN** the event MUST be streamed back to the control plane and recorded in the session.
