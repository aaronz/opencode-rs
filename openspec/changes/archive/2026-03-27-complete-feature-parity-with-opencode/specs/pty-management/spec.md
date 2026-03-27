## ADDED Requirements

### Requirement: PTY Session Support
The system SHALL support creating and managing Pseudo-Terminal (PTY) sessions for executing interactive shell commands.

#### Scenario: Successful PTY spawn
- **WHEN** the agent requests a PTY session for a command
- **THEN** the system SHALL spawn the process in a PTY and return the session handle.

### Requirement: PTY Signal Handling
The system SHALL correctly propagate signals (like SIGINT, SIGTERM) to the processes running within the PTY.

#### Scenario: Propagating Ctrl+C
- **WHEN** the user sends a SIGINT to the CLI
- **THEN** the signal MUST be propagated to the active process in the PTY.
