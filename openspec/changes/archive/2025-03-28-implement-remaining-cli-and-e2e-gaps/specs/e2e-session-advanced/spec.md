## ADDED Requirements

### Requirement: Session undo/redo E2E tests
The system SHALL provide E2E tests for session undo and redo functionality.

#### Scenario: Undo message in session
- **WHEN** a user sends a message in a session
- **AND** clicks the undo button
- **THEN** the last message is removed
- **AND** the session reverts to previous state

#### Scenario: Redo after undo
- **WHEN** a user undoes an operation
- **AND** clicks the redo button
- **THEN** the undone operation is restored

#### Scenario: Undo multiple steps
- **WHEN** a user sends 3 messages
- **AND** clicks undo 3 times
- **THEN** all 3 messages are removed in reverse order

### Requirement: Session review E2E tests
The system SHALL provide E2E tests for session review functionality.

#### Scenario: Review session changes
- **WHEN** a user opens a session with file changes
- **AND** clicks the review button
- **THEN** a diff view is displayed
- **AND** all changed files are listed

#### Scenario: Review specific file
- **WHEN** a user clicks on a file in the review view
- **THEN** the diff for that file is shown
- **AND** syntax highlighting is applied

### Requirement: Session persistence E2E tests
The system SHALL provide E2E tests for session state persistence.

#### Scenario: Session survives reload
- **WHEN** a user creates a session with messages
- **AND** reloads the page
- **THEN** the session is restored
- **AND** all messages are preserved

#### Scenario: Undo history persists
- **WHEN** a user undoes operations in a session
- **AND** reloads the page
- **THEN** undo history is preserved
- **AND** redo works as expected
