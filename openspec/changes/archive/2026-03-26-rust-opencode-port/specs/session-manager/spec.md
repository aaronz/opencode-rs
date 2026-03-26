## ADDED Requirements

### Requirement: Session creation
The system SHALL create a new session when user starts the application.

#### Scenario: New session
- **WHEN** user starts application
- **THEN** a new session is created with unique ID

#### Scenario: Session ID generation
- **WHEN** new session is created
- **THEN** session ID is generated using UUID v4

### Requirement: Message history
The system SHALL maintain conversation history within a session.

#### Scenario: Add user message
- **WHEN** user sends a message
- **THEN** message is added to session history with timestamp

#### Scenario: Add assistant message
- **WHEN** assistant generates a response
- **THEN** response is added to session history with timestamp

#### Scenario: Message context
- **WHEN** user sends subsequent message
- **THEN** previous messages are included in context for LLM

### Requirement: Session persistence
The system SHALL persist session data to disk.

#### Scenario: Auto-save
- **WHEN** session has new messages
- **THEN** session is automatically saved to disk

#### Scenario: Load existing session
- **WHEN** user provides session ID
- **THEN** existing session is loaded from disk

#### Scenario: Session storage location
- **WHEN** session is saved
- **THEN** it is stored in `~/.local/share/opencode-rs/sessions/`

### Requirement: Session list
The system SHALL provide ability to list all sessions.

#### Scenario: List sessions
- **WHEN** user requests session list
- **THEN** system returns all sessions with metadata (ID, date, preview)

#### Scenario: Session sorting
- **WHEN** sessions are listed
- **THEN** they are sorted by last updated time (newest first)

### Requirement: Session deletion
The system SHALL allow deletion of sessions.

#### Scenario: Delete session
- **WHEN** user requests to delete a session
- **THEN** session is permanently removed from storage

### Requirement: Context window management
The system SHALL manage context window size to fit LLM limits.

#### Scenario: Context truncation
- **WHEN** conversation exceeds context limit
- **THEN** oldest messages are truncated to fit limit

#### Scenario: Token estimation
- **WHEN** messages are prepared for API
- **THEN** system estimates token count and adjusts accordingly
