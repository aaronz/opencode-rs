## ADDED Requirements

### Requirement: Missing CLI Subcommands
The system SHALL implement all subcommands present in the TypeScript version, including `web`, `uninstall`, `upgrade`, and `import`.

#### Scenario: Running upgrade command
- **WHEN** the user executes `opencode upgrade`
- **THEN** the system SHALL check for updates and perform the upgrade process.

### Requirement: JSON Output Flags
All relevant CLI subcommands SHALL support a `--json` flag for machine-readable output.

#### Scenario: JSON output for stats
- **WHEN** the user executes `opencode stats --json`
- **THEN** the system SHALL return the statistics in a valid JSON format.
