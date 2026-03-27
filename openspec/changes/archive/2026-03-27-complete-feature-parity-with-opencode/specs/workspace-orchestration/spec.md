## ADDED Requirements

### Requirement: Workspace Service Management
The system SHALL allow starting and stopping local workspace services for remote orchestration.

#### Scenario: Starting workspace serve
- **WHEN** the user executes `opencode workspace-serve`
- **THEN** the system SHALL start an HTTP server exposing workspace management endpoints.

### Requirement: Remote Workspace Synchronization
The system SHALL support synchronizing file state between a local workspace and a remote agent.

#### Scenario: Syncing changed files
- **WHEN** a file is modified locally
- **THEN** the system SHALL detect the change and push it to the connected remote workspace if configured.
