## ADDED Requirements

### Requirement: Tool Registry
The system SHALL maintain a registry of all available tools with proper discovery and listing.

#### Scenario: List Available Tools
- **WHEN** user runs `opencode-rs debug --skill <name>`
- **THEN** all tools for that skill are listed with descriptions

#### Scenario: Tool Discovery
- **WHEN** skill is loaded
- **THEN** all tools defined in skill are automatically registered

### Requirement: Grep Tool
The grep tool SHALL search files with proper pattern matching and output formatting.

#### Scenario: Basic Grep
- **WHEN** grep tool is called with pattern "error" and path "/src"
- **THEN** all lines containing "error" are returned with file:line format

#### Scenario: Grep with Context
- **WHEN** grep tool is called with context=3
- **THEN** 3 lines before and after match are included in output

#### Scenario: Grep No Matches
- **WHEN** grep tool searches for pattern that doesn't exist
- **THEN** empty result is returned with exit code 0

#### Scenario: Grep File Pattern
- **WHEN** grep tool is called with glob "*.rs"
- **THEN** only Rust files are searched

### Requirement: Read Tool
The read tool SHALL read files with proper encoding handling and error reporting.

#### Scenario: Read Existing File
- **WHEN** read tool is called with path to existing file
- **THEN** file content is returned as string

#### Scenario: Read Non-existent File
- **WHEN** read tool is called with path to non-existent file
- **THEN** error is returned with file not found message

#### Scenario: Read Binary File
- **WHEN** read tool is called with path to binary file
- **THEN** error is returned indicating file is binary

### Requirement: Write Tool
The write tool SHALL write content to files with proper directory creation.

#### Scenario: Write New File
- **WHEN** write tool is called with path and content
- **THEN** file is created with specified content

#### Scenario: Write to Existing File
- **WHEN** write tool overwrites an existing file
- **THEN** file contains new content (no confirmation needed for agent)

#### Scenario: Write Creates Directories
- **WHEN** write tool writes to path with non-existent parent directories
- **THEN** parent directories are created automatically

### Requirement: Bash Tool
The bash tool SHALL execute shell commands with proper output capture and error handling.

#### Scenario: Execute Command
- **WHEN** bash tool executes "ls -la"
- **THEN** command output is returned with exit code

#### Scenario: Command Failure
- **WHEN** bash tool executes command that returns non-zero exit
- **THEN** error is returned with exit code and stderr

#### Scenario: Command Timeout
- **WHEN** bash tool executes command that hangs beyond timeout
- **THEN** command is killed and timeout error is returned