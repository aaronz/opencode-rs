## ADDED Requirements

### Requirement: CLI module tests exist
The test suite SHALL cover CLI command handlers from packages/opencode/src/cli/ including argument parsing, command execution, and output formatting.

#### Scenario: Command parsing
- **WHEN** CLI receives arguments
- **THEN** it correctly parses flags and positional arguments

#### Scenario: Help output
- **WHEN** --help flag is passed
- **THEN** it displays correct usage information

#### Scenario: Error handling for invalid input
- **WHEN** CLI receives invalid arguments
- **THEN** it displays an error message and exits with non-zero code
