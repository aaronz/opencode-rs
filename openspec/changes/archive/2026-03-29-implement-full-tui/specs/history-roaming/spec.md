## ADDED Requirements

### Requirement: Prompt History Navigation
The system SHALL allow navigation through prompt history.

#### Scenario: History Recall
- **WHEN** input field is empty and user presses Up arrow
- **THEN** system shows previous prompt from history

#### Scenario: History Forward
- **WHEN** user presses Down arrow while viewing history
- **THEN** system shows next prompt or clears input

### Requirement: History Context Awareness
The system SHALL adapt arrow key behavior based on context.

#### Scenario: History Mode
- **WHEN** input field is empty
- **WHEN** user presses Up arrow
- **THEN** system navigates history

#### Scenario: Cursor Mode
- **WHEN** input field has content
- **WHEN** user presses Up arrow
- **THEN** system moves cursor up within multiline input

### Requirement: History Persistence
The system SHALL persist prompt history across sessions.

#### Scenario: History Save
- **WHEN** user submits a prompt
- **THEN** system adds it to history

#### Scenario: History Load
- **WHEN** system starts
- **THEN** system loads previous session history

### Requirement: History Size Limit
The system SHALL limit history size.

#### Scenario: History Cap
- **WHEN** history exceeds 100 entries
- **THEN** system removes oldest entries
