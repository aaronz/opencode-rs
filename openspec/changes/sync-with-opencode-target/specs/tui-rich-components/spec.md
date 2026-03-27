## ADDED Requirements

### Requirement: TUI Timeline View
The TUI SHALL implement a session timeline view that allows users to browse and navigate through the history of messages and tool calls.

#### Scenario: Navigating timeline
- **WHEN** the user opens the timeline in a session
- **THEN** they MUST be able to scroll through historical messages and see associated metadata.

### Requirement: TUI Fork Dialog
The TUI SHALL provide a dialog for forking a session from a specific point in the timeline.

#### Scenario: Forking from message
- **WHEN** the user selects a message in the timeline and chooses "Fork"
- **THEN** a new session MUST be created starting from that message.

### Requirement: TUI Theme Management
The TUI SHALL support dynamic theme loading and switching from a set of predefined JSON theme files.

#### Scenario: Switching theme
- **WHEN** the user selects a different theme from the TUI settings
- **THEN** the TUI MUST immediately update its visual appearance matching the JSON theme definition.
