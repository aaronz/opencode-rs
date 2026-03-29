## ADDED Requirements

### Requirement: Slash Command Autocomplete
The system SHALL provide a floating autocomplete menu when the user types `/` in the input field.

#### Scenario: Slash Command Menu Appears
- **WHEN** user types `/` in the input field
- **THEN** system displays a floating autocomplete menu above the input

#### Scenario: Command Filtering
- **WHEN** user types `/com` in the input field
- **THEN** system filters commands to show only those starting with "com" (compact, etc.)

#### Scenario: Command Selection
- **WHEN** user selects a command from the autocomplete menu
- **THEN** system executes the selected command and closes the menu

#### Scenario: Menu Dismissal
- **WHEN** user presses `Esc` while autocomplete menu is open
- **THEN** system closes the menu without executing any command

### Requirement: Slash Command Registry
The system SHALL maintain a registry of available slash commands with metadata.

#### Scenario: Command Registration
- **WHEN** system initializes
- **THEN** all built-in commands are registered with name, aliases, and description

#### Scenario: Command Aliases
- **WHEN** user types `/summarize` or `/compact`
- **THEN** both trigger the same compaction functionality
