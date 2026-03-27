## ADDED Requirements

### Requirement: Multiedit Tool Implementation
The system SHALL implement the `multiedit` tool to allow applying multiple edits across multiple files in a single atomic operation.

#### Scenario: Applying multiple file edits
- **WHEN** the agent provides a list of edits for different files to the `multiedit` tool
- **THEN** all edits MUST be applied correctly or non should be applied if any fails.

### Requirement: Codesearch Tool Implementation
The system SHALL implement the `codesearch` tool for efficient searching through the codebase using AST-aware patterns.

#### Scenario: Searching for AST pattern
- **WHEN** the agent calls `codesearch` with a specific code pattern (e.g., function definition)
- **THEN** the system MUST return all matching file locations and code snippets.

### Requirement: Truncation-dir Tool Implementation
The system SHALL implement the `truncation-dir` tool to manage large directory listings by truncating and providing summaries.

#### Scenario: Listing large directory
- **WHEN** `truncation-dir` is called on a directory with thousands of files
- **THEN** it MUST return a manageable summary and list of important files instead of the full list.
