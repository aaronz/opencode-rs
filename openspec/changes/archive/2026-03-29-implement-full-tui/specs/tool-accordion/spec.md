## ADDED Requirements

### Requirement: Tool Execution Accordion
The system SHALL display tool executions in a collapsible accordion format.

#### Scenario: Tool Call Start
- **WHEN** AI starts executing a tool
- **THEN** system shows a single-line status with loading animation

#### Scenario: Tool Call Success
- **WHEN** tool execution completes successfully
- **THEN** system shows green checkmark with tool name

#### Scenario: Tool Call Failure
- **WHEN** tool execution fails
- **THEN** system shows red X with error code

### Requirement: Accordion Expansion
The system SHALL allow users to expand tool output on demand.

#### Scenario: Toggle Details
- **WHEN** user presses `ctrl+x d` or types `/details`
- **THEN** system expands/collapses all tool outputs

#### Scenario: Individual Expansion
- **WHEN** user clicks/activates a specific tool call
- **THEN** system expands only that tool's output

### Requirement: Tool Output Formatting
The system SHALL format tool output appropriately.

#### Scenario: Command Output
- **WHEN** tool is a shell command
- **THEN** system preserves ANSI colors in output

#### Scenario: JSON Output
- **WHEN** tool returns JSON
- **THEN** system pretty-prints the JSON with syntax highlighting
