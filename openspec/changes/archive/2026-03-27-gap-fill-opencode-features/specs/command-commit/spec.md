## ADDED Requirements

### Requirement: command-commit
Git commit and push with prefix conventions.

#### Scenario: Commit with prefix
- **WHEN** user invokes the commit command
- **THEN** system shows git diff and git diff --cached
- **AND** system shows git status --short
- **AND** creates a commit with a proper prefix (docs:, tui:, core:, ci:, ignore:, wip:)
- **AND** pushes the commit

#### Scenario: Handle conflicts
- **WHEN** there are merge conflicts
- **THEN** system does NOT fix them
- **AND** notifies the user about the conflicts
