## ADDED Requirements

### Requirement: Status Popover System
The TUI SHALL display contextual status information in popover panels.

#### Scenario: Connection Status
- **WHEN** the user hovers over or clicks the connection indicator
- **THEN** a popover SHALL show current provider connection status
- **AND** any errors SHALL be displayed with retry options

#### Scenario: Token Usage Display
- **WHEN** the user opens the token status popover
- **THEN** current session token count SHALL be shown
- **AND** a warning SHALL appear if approaching limits

#### Scenario: Model Context Usage
- **WHEN** viewing the context usage indicator
- **THEN** a popover SHALL show context window utilization
- **AND** compacted messages count SHALL be displayed

### Requirement: Title Bar with History
The TUI SHALL have an enhanced title bar with session history navigation.

#### Scenario: Viewing Session History
- **WHEN** the user clicks the session title
- **THEN** a dropdown SHALL show recent sessions
- **AND** clicking one SHALL switch to that session

#### Scenario: Session Information
- **WHEN** hovering over the title bar
- **THEN** session metadata SHALL be displayed (created, message count, etc.)

### Requirement: Terminal Panel Integration
The TUI SHALL integrate a terminal panel for command execution.

#### Scenario: Opening Terminal Panel
- **WHEN** the user presses `Ctrl+~`
- **THEN** a terminal panel SHALL open at the bottom
- **AND** it SHALL execute commands in the current workspace

#### Scenario: Terminal Output
- **WHEN** a command executes
- **THEN** output SHALL stream to the terminal panel
- **AND** the panel SHALL be scrollable
