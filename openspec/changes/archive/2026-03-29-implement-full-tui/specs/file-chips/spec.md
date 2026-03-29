## ADDED Requirements

### Requirement: File Reference Chips
The system SHALL render file references as colored "chips" in the input field.

#### Scenario: Chip Creation
- **WHEN** user selects a file from the file picker (via `@`)
- **THEN** system inserts a colored chip representing the file path

#### Scenario: Chip Display
- **WHEN** a chip is present in the input
- **THEN** system renders it with distinct background color and rounded appearance

#### Scenario: Atomic Deletion
- **WHEN** user presses Backspace on a chip
- **THEN** system deletes the entire chip, not individual characters

#### Scenario: Chip Navigation
- **WHEN** user uses arrow keys to navigate input
- **THEN** cursor moves over chips as single units

### Requirement: File Picker Integration
The system SHALL integrate file picking with the chip system.

#### Scenario: File Picker Trigger
- **WHEN** user types `@` in the input field
- **THEN** system opens file picker overlay

#### Scenario: File Selection
- **WHEN** user selects a file from the picker
- **THEN** system inserts chip with file name and full path

### Requirement: Chip Context Attachment
The system SHALL attach file content as context when chips are present.

#### Scenario: Context Inclusion
- **WHEN** user submits input containing file chips
- **THEN** system includes file contents in the LLM context
