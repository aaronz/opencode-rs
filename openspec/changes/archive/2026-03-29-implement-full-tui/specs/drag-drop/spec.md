## ADDED Requirements

### Requirement: File Drop Support
The system SHALL support dropping files into the terminal.

#### Scenario: File Drop Detection
- **WHEN** user drops a file onto the terminal
- **THEN** system detects the file path

#### Scenario: File Path Insertion
- **WHEN** file is dropped
- **THEN** system inserts file path as a chip in the input

### Requirement: Image Drop Support
The system SHALL support dropping images for multimodal models.

#### Scenario: Image Drop
- **WHEN** user drops an image file
- **WHEN** model supports images
- **THEN** system includes image in context

### Requirement: Drop Validation
The system SHALL validate dropped files.

#### Scenario: File Existence
- **WHEN** dropped file doesn't exist
- **THEN** system shows error message

#### Scenario: File Size Limit
- **WHEN** dropped file exceeds size limit
- **THEN** system warns user and offers to truncate or skip
