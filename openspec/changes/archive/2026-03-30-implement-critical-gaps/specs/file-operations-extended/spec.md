## ADDED Requirements

### Requirement: stat tool provides file metadata
The system SHALL provide a stat tool that returns file system metadata.

#### Scenario: Get file metadata
- **WHEN** user calls stat tool on a file path
- **THEN** tool SHALL return:
  - File size
  - Creation/modification timestamps
  - File type (regular file, directory, symlink)
  - Permissions
  - Owner information (if available)

#### Scenario: stat on non-existent file
- **WHEN** user calls stat tool on non-existent path
- **THEN** tool SHALL return appropriate error indicating file not found

### Requirement: move tool relocates files
The system SHALL provide a move tool that relocates files and directories.

#### Scenario: Move file to new location
- **WHEN** user calls move tool with source and destination
- **THEN** tool SHALL:
  - Move file from source to destination
  - Create parent directories if needed
  - Return success confirmation

#### Scenario: Move directory
- **WHEN** user calls move tool on a directory
- **THEN** tool SHALL move directory with all contents recursively

#### Scenario: Move to existing location
- **WHEN** user calls move tool but destination already exists
- **THEN** tool SHALL return error indicating conflict

### Requirement: delete tool removes files
The system SHALL provide a delete tool for file removal.

#### Scenario: Delete single file
- **WHEN** user calls delete tool on a file
- **THEN** tool SHALL remove the file and return success

#### Scenario: Delete directory recursively
- **WHEN** user calls delete tool on a directory
- **THEN** tool SHALL remove directory and all contents recursively

#### Scenario: Delete non-existent file
- **WHEN** user calls delete tool on non-existent path
- **THEN** tool SHALL return error indicating file not found

### Requirement: File operations respect permissions
All file operations SHALL respect the permission system.

#### Scenario: Permission denied
- **WHEN** user attempts operation without proper permissions
- **THEN** tool SHALL return permission denied error
