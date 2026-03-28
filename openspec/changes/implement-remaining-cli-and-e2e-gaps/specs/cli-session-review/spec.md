## ADDED Requirements

### Requirement: Session review command
The CLI SHALL provide a `session review` command that displays a diff of changes in a session.

#### Scenario: Review session changes
- **WHEN** user runs `opencode session review --id <session-id>`
- **THEN** a diff view of all file changes in the session is displayed
- **AND** changes are grouped by file

#### Scenario: Review with file filter
- **WHEN** user runs `opencode session review --id <session-id> --file "*.rs"`
- **THEN** only changes to .rs files are displayed

#### Scenario: Review output format
- **WHEN** user runs `opencode session review --id <session-id> --format json`
- **THEN** changes are output as JSON for programmatic use

### Requirement: Session diff viewing
The CLI SHALL support viewing diffs for individual files in a session.

#### Scenario: View file diff
- **WHEN** user runs `opencode session diff --id <session-id> --file <path>`
- **THEN** the diff for that specific file is displayed
- **AND** line numbers are shown

#### Scenario: Diff with context
- **WHEN** user runs `opencode session diff --id <session-id> --file <path> --context 5`
- **THEN** the diff includes 5 lines of context around changes
