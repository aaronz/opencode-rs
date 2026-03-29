## MODIFIED Requirements

### Requirement: Command Palette Replacement
The command palette SHALL be replaced with slash command autocomplete.

#### Scenario: Autocomplete Trigger
- **WHEN** user types `/` in input
- **THEN** system shows floating autocomplete instead of palette

#### Scenario: Command Filtering
- **WHEN** user types `/com`
- **THEN** autocomplete filters to matching commands

#### Scenario: Command Execution
- **WHEN** user selects command from autocomplete
- **THEN** system executes command and closes menu

### Requirement: Command Palette Backward Compatibility
The old command palette format SHALL be deprecated.

#### Scenario: Ctrl+P Migration
- **WHEN** user presses `Ctrl+P`
- **THEN** system shows slash command autocomplete (not old palette)

#### Scenario: Palette Deprecation
- **WHEN** old command palette code is removed
- **THEN** system uses new autocomplete exclusively

### Requirement: Command Registry Integration
The command palette SHALL integrate with the command registry.

#### Scenario: Registry Query
- **WHEN** autocomplete is triggered
- **THEN** system queries command registry for available commands

#### Scenario: Dynamic Commands
- **WHEN** new commands are registered
- **THEN** autocomplete includes them immediately
