## ADDED Requirements

### Requirement: Shell Command Execution
The system SHALL execute shell commands when user enters `!command` syntax.

#### Scenario: Execute simple command
- **WHEN** user enters "!ls -la"
- **THEN** system SHALL execute "ls -la" and return output

#### Scenario: Execute command with pipes
- **WHEN** user enters "!cat file.txt | head -n 10"
- **THEN** system SHALL execute the piped command and return output

#### Scenario: Command execution timeout
- **WHEN** user enters "!sleep 300" (long-running command)
- **THEN** system SHALL timeout after default (30 seconds) and return error

### Requirement: Shell Permission Check
The system SHALL check permissions before executing shell commands.

#### Scenario: Execute with full permission
- **WHEN** user enters "!rm file.txt" and scope is "Full"
- **THEN** system SHALL execute command after approval (if required)

#### Scenario: Execute with restricted permission
- **WHEN** user enters "!rm file.txt" and scope is "Restricted"
- **THEN** system SHALL return error "Shell commands not allowed in restricted mode"

#### Scenario: Execute requires approval
- **WHEN** user enters "!dangerous.sh" and permission is "Ask"
- **THEN** system SHALL prompt user for approval before executing

### Requirement: Shell Output Handling
The system SHALL handle shell command output appropriately.

#### Scenario: Capture stdout
- **WHEN** user enters "!echo hello"
- **THEN** system SHALL return stdout "hello"

#### Scenario: Capture stderr
- **WHEN** user enters "!ls /nonexistent 2>&1"
- **THEN** system SHALL return stderr output

#### Scenario: Capture exit code
- **WHEN** user enters "!false"
- **THEN** system SHALL return exit code 1

#### Scenario: Very large output
- **WHEN** user enters "!find /" (produces huge output)
- **THEN** system SHALL truncate output at 100KB and warn user

### Requirement: Shell Command Context
Shell commands SHALL execute in the context of the current working directory.

#### Scenario: Execute in project directory
- **WHEN** user enters "!git status"
- **THEN** command SHALL execute in current working directory

#### Scenario: Execute with environment variables
- **WHEN** user enters "!echo $PATH"
- **THEN** system SHALL include current environment variables
