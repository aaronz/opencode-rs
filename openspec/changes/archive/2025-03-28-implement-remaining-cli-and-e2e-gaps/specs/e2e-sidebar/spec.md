## ADDED Requirements

### Requirement: Sidebar navigation E2E tests
The system SHALL provide E2E tests for sidebar navigation.

#### Scenario: Toggle sidebar
- **WHEN** a user presses Ctrl+B
- **THEN** the sidebar is toggled visible/hidden

#### Scenario: Navigate session list
- **WHEN** a user clicks on a session in the sidebar
- **THEN** that session is opened
- **AND** the main view updates

#### Scenario: Sidebar sections
- **WHEN** a user views the sidebar
- **THEN** sections for Sessions, Files, and Tools are visible
- **AND** each section can be expanded/collapsed

### Requirement: Sidebar session links E2E tests
The system SHALL provide E2E tests for sidebar session links.

#### Scenario: Recent sessions displayed
- **WHEN** a user views the sidebar
- **THEN** recent sessions are listed
- **AND** they are ordered by last access time

#### Scenario: Session link click
- **WHEN** a user clicks on a session link
- **THEN** the session opens in the main view
- **AND** the URL updates

#### Scenario: Session link context menu
- **WHEN** a user right-clicks a session link
- **THEN** a context menu appears
- **AND** options include Delete, Rename, and Fork

### Requirement: Sidebar popover actions E2E tests
The system SHALL provide E2E tests for sidebar popover actions.

#### Scenario: New session popover
- **WHEN** a user clicks the "+" button in sidebar
- **THEN** a popover appears with session creation options

#### Scenario: Settings popover
- **WHEN** a user clicks the settings icon
- **THEN** a settings popover appears
- **AND** quick settings are accessible
