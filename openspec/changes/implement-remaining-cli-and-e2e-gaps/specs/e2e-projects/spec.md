## ADDED Requirements

### Requirement: Project management E2E tests
The system SHALL provide E2E tests for project management.

#### Scenario: Create new project
- **WHEN** a user clicks "New Project"
- **AND** enters a project name
- **THEN** a new project is created
- **AND** it appears in the projects list

#### Scenario: Switch projects
- **WHEN** a user selects a different project from the dropdown
- **THEN** the project context switches
- **AND** project-specific sessions are displayed

#### Scenario: Close project
- **WHEN** a user closes a project tab
- **THEN** the project is removed from the active list
- **AND** sessions are preserved

### Requirement: Workspace E2E tests
The system SHALL provide E2E tests for workspace functionality.

#### Scenario: New session in workspace
- **WHEN** a user creates a new session in a workspace
- **THEN** the session is associated with that workspace
- **AND** workspace context is available

#### Scenario: Workspace persistence
- **WHEN** a user reloads the page
- **THEN** the current workspace is restored
- **AND** all workspace sessions are accessible

### Requirement: Project edit E2E tests
The system SHALL provide E2E tests for project editing.

#### Scenario: Rename project
- **WHEN** a user renames a project
- **THEN** the new name is saved
- **AND** all references are updated

#### Scenario: Delete project
- **WHEN** a user deletes a project
- **THEN** a confirmation dialog appears
- **AND** upon confirmation, the project is removed
