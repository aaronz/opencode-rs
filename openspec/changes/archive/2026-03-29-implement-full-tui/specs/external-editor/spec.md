## ADDED Requirements

### Requirement: External Editor Integration
The system SHALL support opening an external editor for prompt composition.

#### Scenario: Editor Trigger
- **WHEN** user presses `ctrl+x e`
- **THEN** system opens `$EDITOR` with a temporary file

#### Scenario: Editor Content Sync
- **WHEN** user saves and closes the editor
- **THEN** system reads the file content and inserts it into the input field

#### Scenario: Editor Cancellation
- **WHEN** user closes editor without saving
- **THEN** system discards changes and returns to input

### Requirement: Editor Configuration
The system SHALL respect the `$EDITOR` environment variable.

#### Scenario: Editor Selection
- **WHEN** `$EDITOR` is set to `vim`
- **THEN** system opens vim for editing

#### Scenario: Default Editor
- **WHEN** `$EDITOR` is not set
- **THEN** system falls back to `nano` or shows an error

### Requirement: Editor File Management
The system SHALL manage temporary files for editor integration.

#### Scenario: Temp File Creation
- **WHEN** editor is triggered
- **THEN** system creates temporary file in system temp directory

#### Scenario: Temp File Cleanup
- **WHEN** editor session ends
- **THEN** system removes the temporary file
