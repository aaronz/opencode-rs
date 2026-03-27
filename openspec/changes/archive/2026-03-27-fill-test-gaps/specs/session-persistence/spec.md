## ADDED Requirements

### Requirement: Session Creation
The system SHALL create new sessions with unique IDs and proper initialization.

#### Scenario: Create New Session
- **WHEN** user runs `opencode-rs run` without session ID
- **THEN** new session is created with unique UUID

#### Scenario: Session Has Initial State
- **WHEN** new session is created
- **THEN** session has empty message list and creation timestamp

### Requirement: Session Message Storage
The system SHALL store all messages in session with proper role tracking.

#### Scenario: Add User Message
- **WHEN** user sends a message
- **THEN** message is stored with role "user" and timestamp

#### Scenario: Add Assistant Message
- **WHEN** LLM generates a response
- **THEN** message is stored with role "assistant" and timestamp

#### Scenario: Message History Retrieval
- **WHEN** user resumes a session
- **THEN** all previous messages are loaded and displayed

### Requirement: Session Persistence
The system SHALL save sessions to disk and load them back.

#### Scenario: Save Session
- **WHEN** session ends or user exits
- **THEN** session is saved to disk as JSON

#### Scenario: Load Session
- **WHEN** user runs `opencode-rs session show <id>`
- **THEN** session is loaded from disk and displayed

#### Scenario: Session List
- **WHEN** user runs `opencode-rs list`
- **THEN** all saved sessions are listed with metadata

### Requirement: Session Deletion
The system SHALL support deleting sessions from disk.

#### Scenario: Delete Session
- **WHEN** user runs `opencode-rs session delete <id>`
- **THEN** session file is removed from disk

#### Scenario: Delete Non-existent Session
- **WHEN** user deletes session that doesn't exist
- **THEN** error message is displayed