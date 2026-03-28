## ADDED Requirements

### Requirement: File tree E2E tests
The system SHALL provide E2E tests for file tree navigation.

#### Scenario: Expand directory
- **WHEN** a user clicks on a collapsed directory
- **THEN** the directory expands
- **AND** its contents are displayed

#### Scenario: Collapse directory
- **WHEN** a user clicks on an expanded directory
- **THEN** the directory collapses
- **AND** its contents are hidden

#### Scenario: File tree scroll
- **WHEN** a file tree has many items
- **THEN** scrolling works smoothly
- **AND** lazy loading fetches items as needed

#### Scenario: Select file in tree
- **WHEN** a user clicks on a file
- **THEN** the file is selected
- **AND** its contents are displayed in the viewer

### Requirement: File viewer E2E tests
The system SHALL provide E2E tests for file viewer functionality.

#### Scenario: Open file from tree
- **WHEN** a user clicks on a file in the tree
- **THEN** the file opens in the viewer
- **AND** syntax highlighting is applied

#### Scenario: File viewer scroll
- **WHEN** a user scrolls in the file viewer
- **THEN** the content scrolls smoothly
- **AND** line numbers remain visible

#### Scenario: Switch between files
- **WHEN** a user clicks on a different file
- **THEN** the new file opens
- **AND** the previous file tab remains open

### Requirement: File open E2E tests
The system SHALL provide E2E tests for file opening workflows.

#### Scenario: Open file via command palette
- **WHEN** a user opens the command palette
- **AND** types a filename
- **THEN** matching files are shown
- **AND** selecting one opens it

#### Scenario: Open file via drag and drop
- **WHEN** a user drags a file into the window
- **THEN** the file is opened
- **AND** its contents are displayed
