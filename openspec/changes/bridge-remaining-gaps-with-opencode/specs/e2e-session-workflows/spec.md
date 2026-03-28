## ADDED Requirements

### Requirement: E2E Session Management Tests
The test harness SHALL verify session creation, persistence, and switching.

#### Scenario: Creating a New Session
- **WHEN** the test runs `opencode-rs session --new`
- **THEN** a new session SHALL be created and saved to disk
- **AND** the session JSON SHALL contain expected fields

#### Scenario: Listing Sessions
- **WHEN** the test runs `opencode-rs list --json`
- **THEN** the output SHALL contain an array of sessions
- **AND** each session SHALL have id, created_at, message_count

#### Scenario: Resuming a Session
- **WHEN** the test runs `opencode-rs session --id <id>`
- **THEN** the session SHALL load with all previous messages

#### Scenario: Session Persistence
- **WHEN** a session is modified
- **AND** the application restarts
- **THEN** the session SHALL retain all changes

### Requirement: E2E Model Connection Tests
The test harness SHALL verify model provider connections and switching.

#### Scenario: Listing Providers
- **WHEN** the test runs `opencode-rs providers --json`
- **THEN** all configured providers SHALL be listed
- **AND** each SHALL show connection status

#### Scenario: Switching Models
- **WHEN** the test changes the active model
- **THEN** subsequent prompts SHALL use the new model
- **AND** the config SHALL persist the selection

#### Scenario: Provider Error Handling
- **WHEN** a provider returns an error
- **THEN** the CLI SHALL display a user-friendly error message
- **AND** suggest troubleshooting steps
