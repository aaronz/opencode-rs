## ADDED Requirements

### Requirement: command-issues
Find issues on GitHub using gh cli.

#### Scenario: Search GitHub issues
- **WHEN** user invokes the issues command with a query
- **THEN** system searches existing issues on anomalyco/opencode
- **AND** considers similar titles, descriptions, error messages, related functionality
- **AND** returns matching issues with number, title, explanation, and link
- **AND** says "no matches found" if no clear matches exist
