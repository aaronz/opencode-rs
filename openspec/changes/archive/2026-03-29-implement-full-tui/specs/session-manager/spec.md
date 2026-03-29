## ADDED Requirements

### Requirement: Session List
The system SHALL display a list of previous sessions.

#### Scenario: Session List Display
- **WHEN** user types `/sessions` or presses `ctrl+x l`
- **THEN** system shows overlay with all sessions

#### Scenario: Session Metadata
- **WHEN** session list is displayed
- **THEN** each session shows: ID, last message preview, timestamp

### Requirement: Session Switching
The system SHALL allow switching between sessions.

#### Scenario: Session Selection
- **WHEN** user selects a session from the list
- **THEN** system loads that session's messages and context

#### Scenario: Session Creation
- **WHEN** user types `/new` or presses `ctrl+x n`
- **THEN** system creates a new session

### Requirement: Session Persistence
The system SHALL persist sessions to disk.

#### Scenario: Session Save
- **WHEN** user exits or switches sessions
- **THEN** system saves current session state

#### Scenario: Session Restore
- **WHEN** system starts
- **THEN** system loads the last active session

### Requirement: Session Search
The system SHALL support searching through sessions.

#### Scenario: Session Search
- **WHEN** user types in session list search
- **THEN** system filters sessions by content or timestamp
