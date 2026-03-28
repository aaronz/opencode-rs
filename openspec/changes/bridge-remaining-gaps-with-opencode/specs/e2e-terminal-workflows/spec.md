## ADDED Requirements

### Requirement: E2E Settings Workflow Tests
The test harness SHALL verify settings modification and persistence.

#### Scenario: Reading Settings
- **WHEN** the test reads config via `--json` output
- **THEN** all settings SHALL be returned as structured data

#### Scenario: Modifying Settings
- **WHEN** the test changes a setting
- **THEN** the change SHALL be reflected in subsequent reads
- **AND** the config file SHALL be updated

#### Scenario: Invalid Settings Handling
- **WHEN** an invalid setting value is provided
- **THEN** an appropriate error SHALL be returned
- **AND** the invalid change SHALL not be persisted

### Requirement: E2E Terminal Workflow Tests
The test harness SHALL verify terminal command execution.

#### Scenario: Executing Commands
- **WHEN** a bash tool is invoked
- **THEN** the command SHALL execute in the workspace directory
- **AND** output SHALL be captured and returned

#### Scenario: Long-running Commands
- **WHEN** a command runs for >30 seconds
- **THEN** the system SHALL handle timeout appropriately
- **AND** partial output SHALL be preserved

#### Scenario: Interactive Commands
- **WHEN** an interactive command is attempted
- **THEN** the system SHALL detect interactivity
- **AND** either handle it or return an informative error
