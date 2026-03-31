## ADDED Requirements

### Requirement: File Reference Reads File Contents
The system SHALL read file contents when user enters `@filename` syntax and attach them to the conversation context.

#### Scenario: Read existing file
- **WHEN** user enters `@src/main.rs` and file exists
- **THEN** system SHALL read file contents and attach to conversation

#### Scenario: Read absolute path file
- **WHEN** user enters `@/absolute/path/to/file.txt` and file exists
- **THEN** system SHALL read file from absolute path

#### Scenario: File not found
- **WHEN** user enters `@nonexistent.txt` and file does not exist
- **THEN** system SHALL return error "File not found: nonexistent.txt"

### Requirement: File Reference Permission Check
The system SHALL check permissions before reading file contents.

#### Scenario: Read allowed file
- **WHEN** user enters `@allowed.txt` and permission is granted
- **THEN** system SHALL read and attach file contents

#### Scenario: Read denied file
- **WHEN** user enters `@denied.txt` and permission is denied
- **THEN** system SHALL return error "Permission denied: denied.txt"

#### Scenario: Read requires approval
- **WHEN** user enters `@pending.txt` and permission is "Ask"
- **THEN** system SHALL prompt user for approval before reading

### Requirement: File Reference Content Formatting
The system SHALL format file contents appropriately for LLM context.

#### Scenario: Format file with header
- **WHEN** system reads file `@src/app.ts`
- **THEN** attached content SHALL include `File: src/app.ts` header followed by contents

#### Scenario: Handle binary files
- **WHEN** user enters `@binary.bin` and file is binary
- **THEN** system SHALL return error "Cannot read binary file"

#### Scenario: Handle large files
- **WHEN** user enters `@large.txt` and file exceeds size limit (default 1MB)
- **THEN** system SHALL return error "File too large: exceeds 1MB limit"

### Requirement: Multiple File References
The system SHALL support multiple file references in a single input.

#### Scenario: Multiple file references
- **WHEN** user enters `@file1.txt @file2.txt`
- **THEN** system SHALL read both files and attach both contents
