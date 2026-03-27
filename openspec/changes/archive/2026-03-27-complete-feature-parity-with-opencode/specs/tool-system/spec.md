## ADDED Requirements

### Requirement: Tool Argument Mapping
The system SHALL ensure that all tool arguments in Rust match the TypeScript definitions exactly to ensure test compatibility.

#### Scenario: Validating grep arguments
- **WHEN** the grep tool is called with `include` and `exclude` patterns
- **THEN** it MUST correctly apply these filters as defined in the target project.

### Requirement: Multi-Edit Tool
The system SHALL implement the `multi-edit` tool for applying multiple changes across different files in a single operation.

#### Scenario: Applying multiple edits
- **WHEN** the agent provides a list of edits for different files
- **THEN** the `multi-edit` tool MUST apply all changes or fail atomically if configured.
