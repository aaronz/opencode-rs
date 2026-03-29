## MODIFIED Requirements

### Requirement: Input Widget Chip Support
The input widget SHALL support rendering file reference chips with atomic deletion.

#### Scenario: Chip Rendering
- **WHEN** input contains chip elements
- **THEN** widget renders chips with distinct background color and rounded corners

#### Scenario: Chip Atomic Deletion
- **WHEN** user presses Backspace on a chip
- **THEN** widget deletes entire chip, not individual characters

#### Scenario: Chip Navigation
- **WHEN** user uses arrow keys
- **THEN** cursor moves over chips as single units

### Requirement: Input Widget Leader Key Awareness
The input widget SHALL integrate with the leader key state machine.

#### Scenario: Leader Key Indicator
- **WHEN** leader key is active
- **THEN** widget shows visual indicator (e.g., "LEADER" prefix)

#### Scenario: Leader Key Timeout Display
- **WHEN** waiting for leader key action
- **THEN** widget shows countdown or timeout indicator

### Requirement: Input Widget Shell Integration
The input widget SHALL support shell command prefix highlighting.

#### Scenario: Shell Prefix Highlighting
- **WHEN** input starts with `!`
- **THEN** widget renders prefix in distinct color

### Requirement: Input Widget Multiline Support
The input widget SHALL maintain multiline editing capabilities.

#### Scenario: Multiline Input
- **WHEN** user presses Shift+Enter
- **THEN** widget creates new line without submitting

#### Scenario: Line Navigation
- **WHEN** user uses Up/Down arrows in multiline input
- **THEN** cursor moves between lines
