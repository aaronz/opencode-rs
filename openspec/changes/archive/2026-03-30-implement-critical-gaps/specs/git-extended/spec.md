## ADDED Requirements

### Requirement: git_log tool displays commit history
The system SHALL provide a git_log tool that displays commit history.

#### Scenario: View recent commits
- **WHEN** user calls git_log tool without options
- **THEN** tool SHALL return:
  - Commit hash
  - Author
  - Date
  - Commit message
  - For most recent commits (default: 10)

#### Scenario: View commits for specific file
- **WHEN** user calls git_log tool with file path
- **THEN** tool SHALL return commit history affecting that file only

#### Scenario: View commits with limit
- **WHEN** user calls git_log tool with limit parameter
- **THEN** tool SHALL return exactly that number of commits

#### Scenario: View commits with path filter
- **WHEN** user calls git_log tool with path filter
- **THEN** tool SHALL return only commits that modified files matching the path

### Requirement: git_show tool displays commit details
The system SHALL provide a git_show tool that displays detailed commit information.

#### Scenario: Show commit details
- **WHEN** user calls git_show tool with commit hash
- **THEN** tool SHALL return:
  - Full commit message
  - Author and committer
  - Commit date
  - Diff of changes

#### Scenario: Show specific file at commit
- **WHEN** user calls git_show tool with commit and file path
- **THEN** tool SHALL return the file content at that commit

#### Scenario: Show tag information
- **WHEN** user calls git_show tool with tag name
- **THEN** tool SHALL return tag details and associated commit

### Requirement: Git tools respect repository context
Git tools SHALL operate in the correct repository context.

#### Scenario: Non-git directory
- **WHEN** user calls git tool outside a git repository
- **THEN** tool SHALL return error indicating not a git repository
