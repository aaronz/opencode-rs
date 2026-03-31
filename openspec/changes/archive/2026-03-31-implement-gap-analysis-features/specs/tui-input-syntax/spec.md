## ADDED Requirements

### Requirement: File Inclusion Syntax

The TUI SHALL support `@filename` syntax to include file contents in messages.

#### Scenario: Include single file
- **WHEN** user types `@src/main.rs explain this code`
- **THEN** TUI reads `src/main.rs` and appends contents to the message

#### Scenario: File not found
- **WHEN** user types `@nonexistent.txt`
- **THEN** TUI shows error "File not found: nonexistent.txt"

#### Scenario: Include multiple files
- **WHEN** user types `@file1.rs @file2.rs compare these files`
- **THEN** TUI reads both files and includes both contents

#### Scenario: Escape @ character
- **WHEN** user types `\@not_a_file`
- **THEN** TUI treats `@not_a_file` as literal text

### Requirement: Shell Execution Syntax

The TUI SHALL support `!command` syntax to execute shell commands.

#### Scenario: Execute shell command
- **WHEN** user types `!ls -la`
- **THEN** TUI executes `ls -la` and includes output in the message

#### Scenario: Command fails
- **WHEN** user types `!invalid_command`
- **THEN** TUI includes error output in the message

#### Scenario: Long-running command
- **WHEN** user types command that takes > 5 seconds
- **THEN** TUI shows progress indicator while waiting

#### Scenario: Command output too large
- **WHEN** command output exceeds 10KB
- **THEN** TUI truncates output and shows "Output truncated (X KB)"

### Requirement: TUI Command Syntax

The TUI SHALL support `/command` syntax for internal commands.

#### Scenario: Help command
- **WHEN** user types `/help`
- **THEN** TUI displays available commands and descriptions

#### Scenario: Clear command
- **WHEN** user types `/clear`
- **THEN** TUI clears the conversation history

#### Scenario: Config command
- **WHEN** user types `/config`
- **THEN** TUI shows current configuration

#### Scenario: Unknown command
- **WHEN** user types `/unknown`
- **THEN** TUI shows "Unknown command: unknown. Type /help for available commands."

### Requirement: Syntax Detection and Parsing

The TUI SHALL correctly detect and parse input syntax at the start of input.

#### Scenario: Mixed syntax
- **WHEN** user types `@file.rs !grep "pattern" analyze the results`
- **THEN** TUI processes @ and ! in order, then includes remaining text

#### Scenario: Syntax in middle of input
- **WHEN** user types `check @file.rs for issues`
- **THEN** TUI treats `@file.rs` as literal text (not file inclusion)

#### Scenario: Empty @ or !
- **WHEN** user types `@` or `!` alone
- **THEN** TUI treats them as literal text

### Requirement: Syntax Autocompletion

The TUI SHALL provide autocompletion for syntax prefixes.

#### Scenario: @ file completion
- **WHEN** user types `@src/` and presses Tab
- **THEN** TUI shows matching files in src/ directory

#### Scenario: / command completion
- **WHEN** user types `/he` and presses Tab
- **THEN** TUI completes to `/help`

#### Scenario: ! command completion
- **WHEN** user types `!git ` and presses Tab
- **THEN** TUI shows git subcommands
