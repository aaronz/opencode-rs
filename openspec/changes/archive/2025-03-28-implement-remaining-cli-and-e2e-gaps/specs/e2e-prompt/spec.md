## ADDED Requirements

### Requirement: Prompt async E2E tests
The system SHALL provide E2E tests for async prompt handling.

#### Scenario: Submit prompt while processing
- **WHEN** a user submits a prompt
- **AND** the system is still processing a previous prompt
- **THEN** the new prompt is queued
- **AND** processed after the current one completes

#### Scenario: Cancel async operation
- **WHEN** a user clicks cancel during processing
- **THEN** the current operation is cancelled
- **AND** a partial response may be shown

### Requirement: Prompt shell E2E tests
The system SHALL provide E2E tests for shell command prompts.

#### Scenario: Execute shell command
- **WHEN** a user types a shell command in the prompt
- **AND** presses Enter with shell modifier
- **THEN** the command executes in the integrated terminal

#### Scenario: Shell command output
- **WHEN** a shell command executes
- **THEN** the output appears in the terminal panel
- **AND** the prompt remains available

### Requirement: Prompt multiline E2E tests
The system SHALL provide E2E tests for multiline prompt input.

#### Scenario: Enter multiline text
- **WHEN** a user presses Shift+Enter in the prompt
- **THEN** a new line is added to the input
- **AND** the message is not submitted

#### Scenario: Submit multiline message
- **WHEN** a user presses Enter in multiline mode
- **THEN** the entire multiline message is submitted

### Requirement: Prompt history E2E tests
The system SHALL provide E2E tests for prompt history.

#### Scenario: Navigate history up
- **WHEN** a user presses Up arrow in an empty prompt
- **THEN** the previous message appears in the prompt

#### Scenario: Navigate history down
- **WHEN** a user presses Down arrow after navigating up
- **THEN** the next message appears (or prompt clears)

#### Scenario: History persistence
- **WHEN** a user reloads the page
- **THEN** prompt history is preserved
- **AND** up/down navigation still works

### Requirement: Prompt mentions E2E tests
The system SHALL provide E2E tests for @ mentions in prompts.

#### Scenario: Type @ to show mentions
- **WHEN** a user types "@" in the prompt
- **THEN** a dropdown appears with mentionable items
- **AND** options include agents, files, and sessions

#### Scenario: Select mention
- **WHEN** a user selects an item from the @ dropdown
- **THEN** it is inserted as a formatted mention
- **AND** the context is included in the message
