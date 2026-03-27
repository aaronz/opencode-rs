## ADDED Requirements

### Requirement: agent-duplicate-pr
Detect duplicate or related PRs using github-pr-search tool.

#### Scenario: Find duplicate PRs
- **WHEN** a PR is opened, the duplicate-pr agent runs
- **THEN** system uses github-pr-search tool to search for duplicate or related PRs
- **AND** uses keywords from PR title and description
- **AND** tries multiple searches with different relevant terms
- **AND** lists potential duplicates with titles, URLs, and explanation
- **AND** says "No duplicate PRs found" if none found (nothing else)

#### Scenario: Exclude current PR
- **WHEN** searching for duplicates
- **THEN** system ignores the current PR (identified by CURRENT_PR_NUMBER)
