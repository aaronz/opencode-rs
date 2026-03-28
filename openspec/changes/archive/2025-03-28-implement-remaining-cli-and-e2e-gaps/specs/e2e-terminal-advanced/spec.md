## ADDED Requirements

### Requirement: Terminal tabs E2E tests
The system SHALL provide E2E tests for terminal tab functionality.

#### Scenario: Create new terminal tab
- **WHEN** a user clicks the "+" button in the terminal panel
- **THEN** a new terminal tab is created
- **AND** it becomes the active tab

#### Scenario: Switch between terminal tabs
- **WHEN** a user has multiple terminal tabs open
- **AND** clicks on a different tab
- **THEN** the selected tab becomes active
- **AND** its content is displayed

#### Scenario: Close terminal tab
- **WHEN** a user clicks the X on a terminal tab
- **THEN** the tab is closed
- **AND** focus moves to another tab

### Requirement: Terminal reconnect E2E tests
The system SHALL provide E2E tests for terminal reconnection.

#### Scenario: Reconnect after disconnect
- **WHEN** a terminal session loses connection
- **AND** the connection is restored
- **THEN** the terminal reconnects automatically
- **AND** previous output is preserved

#### Scenario: Reconnect with running process
- **WHEN** a terminal has a running process
- **AND** the connection drops and reconnects
- **THEN** the process state is maintained
- **OR** the user is notified of process termination

### Requirement: Terminal initialization E2E tests
The system SHALL provide E2E tests for terminal initialization.

#### Scenario: Terminal starts with shell
- **WHEN** a user opens the terminal panel
- **THEN** a shell session starts automatically
- **AND** the prompt is displayed

#### Scenario: Terminal respects working directory
- **WHEN** a user opens the terminal panel
- **THEN** the shell starts in the project root directory
