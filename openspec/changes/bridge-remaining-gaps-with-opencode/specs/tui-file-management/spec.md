## ADDED Requirements

### Requirement: File Tree Component
The TUI SHALL display a navigable file tree with selection capabilities.

#### Scenario: Displaying File Tree
- **WHEN** the user opens the file tree panel (`Ctrl+Shift+F`)
- **THEN** a tree view SHALL show the current workspace directory structure
- **AND** directories SHALL be collapsible/expandable

#### Scenario: Navigating File Tree
- **WHEN** the user uses arrow keys
- **THEN** the selection SHALL move up/down through files and directories
- **AND** `Enter` SHALL expand/collapse directories or select files

#### Scenario: File Selection
- **WHEN** the user presses `Space` on a file
- **THEN** the file SHALL be marked as selected
- **AND** multiple files MAY be selected for batch operations

#### Scenario: Large Directory Handling
- **WHEN** a directory contains >1000 files
- **THEN** the tree SHALL use virtual scrolling
- **AND** only visible items SHALL be rendered

### Requirement: File/Directory Selection Dialogs
The TUI SHALL provide modal dialogs for selecting files or directories.

#### Scenario: Opening File Dialog
- **WHEN** a tool requires a file path
- **THEN** a file selection dialog SHALL appear
- **AND** the user SHALL navigate and select a file

#### Scenario: Opening Directory Dialog
- **WHEN** a tool requires a directory path
- **THEN** a directory selection dialog SHALL appear
- **AND** only directories SHALL be selectable

#### Scenario: Filtering Files
- **WHEN** in a file dialog
- **THEN** the user SHALL be able to filter by file extension
- **AND** hidden files SHALL be toggleable
