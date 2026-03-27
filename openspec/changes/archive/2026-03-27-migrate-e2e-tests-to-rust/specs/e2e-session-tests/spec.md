## ADDED Requirements

### Requirement: e2e-session-tests
Tests for session management and messaging.

#### Scenario: Session creates new session
- **WHEN** creating a new session
- **THEN** session is initialized with unique ID

#### Scenario: Session persists messages
- **WHEN** adding messages to a session
- **THEN** messages are stored and retrievable

#### Scenario: Session compaction works
- **WHEN** session exceeds message limit
- **THEN** old messages are compacted correctly
