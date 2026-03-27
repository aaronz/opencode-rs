## ADDED Requirements

### Requirement: Atomic Multiedit Tool
The `multiedit` tool SHALL support applying multiple edits across different files, ensuring that all edits are applied correctly or none are applied if any individual edit fails.

#### Scenario: Successful atomic edit
- **WHEN** the agent provides a valid list of edits for multiple files
- **THEN** all files MUST be updated with the new content.

#### Scenario: Rollback on single failure
- **WHEN** one edit in a list of `multiedit` operations is invalid (e.g., file not found or old string mismatch)
- **THEN** no changes SHALL be written to any file in the request.

### Requirement: Directory Truncation Summary
The `truncation_dir` tool SHALL provide a manageable summary of large directory listings, including file counts and a subset of representative entries.

#### Scenario: Summarizing large directory
- **WHEN** `truncation_dir` is called on a directory with thousands of files
- **THEN** it SHALL return a summary string indicating total files and listing only the first 50 entries.
