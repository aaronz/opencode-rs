## ADDED Requirements

### Requirement: e2e-tool-tests
End-to-end tests for tool implementations.

#### Scenario: Tool grep executes correctly
- **WHEN** running grep tool with a valid pattern
- **THEN** returns matching lines from target files

#### Scenario: Tool read reads file content
- **WHEN** reading a file using the read tool
- **THEN** returns file content with proper formatting

#### Scenario: Tool write creates/updates files
- **WHEN** writing content to a new file
- **THEN** file is created with correct content

#### Scenario: Tool edit modifies files
- **WHEN** editing a file with a replacement
- **THEN** file content is updated correctly

#### Scenario: Tool bash executes commands
- **WHEN** running a shell command
- **THEN** command executes and returns output

#### Scenario: Tool skill loads and executes skills
- **WHEN** invoking a skill by name
- **THEN** skill content is loaded and executed
