## ADDED Requirements

### Requirement: Command palette E2E tests
The system SHALL provide E2E tests for command palette functionality.

#### Scenario: Open command palette
- **WHEN** a user presses Ctrl+Shift+P
- **THEN** the command palette appears
- **AND** it shows available commands

#### Scenario: Search commands
- **WHEN** a user types in the command palette
- **THEN** commands are filtered by the search term
- **AND** matching commands are highlighted

#### Scenario: Execute command
- **WHEN** a user selects a command from the palette
- **THEN** the command executes
- **AND** the palette closes

#### Scenario: Recent commands
- **WHEN** a user opens the command palette
- **THEN** recently used commands appear at the top
- **AND** they are ordered by recency

### Requirement: Quick actions E2E tests
The system SHALL provide E2E tests for quick action workflows.

#### Scenario: Quick new session
- **WHEN** a user opens the command palette
- **AND** types "new session"
- **THEN** the new session command appears
- **AND** executing it creates a session

#### Scenario: Quick model switch
- **WHEN** a user opens the command palette
- **AND** types "switch model"
- **THEN** model switching options appear
- **AND** selecting one changes the active model

### Requirement: Keyboard shortcuts E2E tests
The system SHALL provide E2E tests for keyboard shortcuts.

#### Scenario: Shortcut displays in palette
- **WHEN** a user views a command in the palette
- **THEN** its keyboard shortcut is displayed
- **AND** the shortcut works when pressed

#### Scenario: Custom shortcuts
- **WHEN** a user configures a custom shortcut
- **THEN** it appears in the palette
- **AND** pressing the shortcut executes the command
