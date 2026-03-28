## ADDED Requirements

### Requirement: Full Functional Subcommands
All CLI subcommands, including `upgrade`, `uninstall`, and `acp`, SHALL be fully implemented with logic matching the TypeScript version.

#### Scenario: Executing uninstall
- **WHEN** the user runs `opencode uninstall`
- **THEN** the system MUST correctly identify and remove all local data, config, and installation files.

### Requirement: JSON Output Uniformity
All CLI commands SHALL support a `--json` flag that returns structured machine-readable data matching the TS project's output schemas.

#### Scenario: Getting stats in JSON
- **WHEN** `opencode stats --json` is executed
- **THEN** the stdout MUST contain a valid JSON object with the expected keys (e.g., total_tokens, sessions_count).
