## ADDED Requirements

### Requirement: command-changelog
Generate an UPCOMING_CHANGELOG.md file from merged PRs since the last tag.

#### Scenario: Generate changelog
- **WHEN** user invokes the changelog command
- **THEN** system creates UPCOMING_CHANGELOG.md with sections for TUI, Desktop, Core, and Misc
- **AND** iterates through each PR merged since the last tag
- **AND** spawns subagents to summarize user-facing changes
- **AND** appends summaries to appropriate sections
