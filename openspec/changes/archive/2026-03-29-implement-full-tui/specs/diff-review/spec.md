## ADDED Requirements

### Requirement: Diff Review Overlay
The system SHALL display code changes as a diff view before applying them.

#### Scenario: Diff Display
- **WHEN** AI proposes file changes
- **THEN** system shows diff view with deletions in red and additions in green

#### Scenario: Side-by-Side Mode
- **WHEN** terminal width is sufficient (>80 columns)
- **THEN** system shows diff in side-by-side layout

#### Scenario: Stacked Mode
- **WHEN** terminal width is narrow (<80 columns)
- **THEN** system shows diff in stacked layout

### Requirement: Diff Confirmation Workflow
The system SHALL require user confirmation before applying changes.

#### Scenario: Accept Change
- **WHEN** user presses `Y` during diff review
- **THEN** system applies the change to the file

#### Scenario: Reject Change
- **WHEN** user presses `N` during diff review
- **THEN** system discards the change

#### Scenario: Edit Change
- **WHEN** user presses `E` during diff review
- **THEN** system opens external editor to modify the change

### Requirement: Diff Syntax Highlighting
The system SHALL apply syntax highlighting to diff content.

#### Scenario: Language Detection
- **WHEN** diff is displayed
- **THEN** system detects file type and applies appropriate syntax highlighting
