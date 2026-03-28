## ADDED Requirements

### Requirement: Session undo command
The CLI SHALL provide a `session undo` command that reverts the last operation in a session.

#### Scenario: Undo last message
- **WHEN** user runs `opencode session undo --id <session-id>`
- **THEN** the last message in the session is removed
- **AND** the session state is restored to before that message

#### Scenario: Undo with no history
- **WHEN** user runs `opencode session undo` on a session with no undo history
- **THEN** an error message "Nothing to undo" is displayed
- **AND** exit code is 1

#### Scenario: Undo with steps
- **WHEN** user runs `opencode session undo --steps 3 --id <session-id>`
- **THEN** the last 3 operations are undone

### Requirement: Session redo command
The CLI SHALL provide a `session redo` command that re-applies a previously undone operation.

#### Scenario: Redo undone operation
- **WHEN** user runs `opencode session redo --id <session-id>` after an undo
- **THEN** the undone operation is re-applied
- **AND** the session state is restored

#### Scenario: Redo with no redo history
- **WHEN** user runs `opencode session redo` with no redo history
- **THEN** an error message "Nothing to redo" is displayed
- **AND** exit code is 1

### Requirement: Session history persistence
The system SHALL maintain undo/redo history for each session.

#### Scenario: History survives restart
- **WHEN** a session is closed and reopened
- **THEN** the undo/redo history is preserved
- **AND** undo/redo operations work as expected
