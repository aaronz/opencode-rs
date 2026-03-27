## ADDED Requirements

### Requirement: Full Functional Parity for Subcommands
The system SHALL implement the functional logic for all CLI subcommands, including `upgrade`, `uninstall`, and `acp`, matching the TS implementation.

#### Scenario: Running uninstall command
- **WHEN** the user executes `opencode uninstall`
- **THEN** the system MUST correctly perform the removal of all associated configuration and data as per the target project.

### Requirement: CLI Argument Compatibility
The Rust CLI MUST accept the exact same flags and positional arguments as the TypeScript version for all shared commands.

#### Scenario: Running run with flags
- **WHEN** the user runs `opencode run --agent build --model gpt-4o`
- **THEN** the Rust version MUST correctly parse these flags and execute the agent with the specified model.
