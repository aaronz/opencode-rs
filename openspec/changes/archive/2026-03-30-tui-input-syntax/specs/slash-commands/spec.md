## ADDED Requirements

### Requirement: Slash Command Invocation
The system SHALL recognize commands starting with `/` and invoke the appropriate handler.

#### Scenario: Invoke help command
- **WHEN** user enters "/help"
- **THEN** system SHALL display help information

#### Scenario: Invoke clear command
- **WHEN** user enters "/clear"
- **THEN** system SHALL clear conversation history

#### Scenario: Invoke retry command
- **WHEN** user enters "/retry"
- **THEN** system SHALL retry last failed request

#### Scenario: Invoke model command
- **WHEN** user enters "/model gpt-4"
- **THEN** system SHALL switch to specified model

#### Scenario: Unknown command
- **WHEN** user enters "/unknowncommand"
- **THEN** system SHALL return error "Unknown command: unknowncommand"

### Requirement: Command Arguments
The system SHALL parse and pass arguments to commands.

#### Scenario: Command with single argument
- **WHEN** user enters "/model claude-3"
- **THEN** system SHALL pass "claude-3" as argument to model command

#### Scenario: Command with multiple arguments
- **WHEN** user enters "/context 5"
- **THEN** system SHALL pass "5" as argument to context command

#### Scenario: Command with quoted argument
- **WHEN** user enters '/command "arg with spaces"'
- **THEN** system SHALL pass "arg with spaces" as single argument

#### Scenario: Case insensitive commands
- **WHEN** user enters "/HELP"
- **THEN** system SHALL recognize as "/help"

### Requirement: Built-in Commands
The system SHALL implement these built-in commands:

#### Command: /help
- **Description**: Show available commands and usage
- **Arguments**: None or command name to get specific help
- **Usage**: `/help` or `/help model`

#### Command: /clear
- **Description**: Clear conversation history
- **Arguments**: None
- **Usage**: `/clear`

#### Command: /retry
- **Description**: Retry the last failed request
- **Arguments**: None
- **Usage**: `/retry`

#### Command: /model
- **Description**: Switch or show current model
- **Arguments**: Optional model name
- **Usage**: `/model` (show current) or `/model gpt-4` (switch)

#### Command: /context
- **Description**: Adjust context window size
- **Arguments**: Number of messages to include
- **Usage**: `/context 10`

#### Command: /quit or /exit
- **Description**: Exit the application
- **Arguments**: None
- **Usage**: `/quit`

### Requirement: Command Registration
The system SHALL allow dynamic command registration.

#### Scenario: Register new command
- **WHEN** command registry receives registration for "/custom"
- **THEN** command SHALL be available for invocation

#### Scenario: Override built-in command
- **WHEN** user registers custom "/help" handler
- **THEN** custom handler SHALL be used instead of built-in
