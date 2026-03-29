## ADDED Requirements

### Requirement: Buffer Snapshot Capture
The framework SHALL capture the complete terminal buffer state for snapshot comparison using ratatui's TestBackend.

#### Scenario: Full Buffer Capture
- **WHEN** a snapshot test is executed
- **THEN** the entire buffer content SHALL be captured as the baseline

#### Scenario: Snapshot Directory Organization
- **WHEN** snapshots are created
- **THEN** they SHALL be stored in a `snapshots/` directory following insta conventions

### Requirement: Snapshot Assertion
The framework SHALL integrate with insta for declarative snapshot testing.

#### Scenario: Snapshot Match
- **WHEN** rendered output matches the stored snapshot
- **THEN** the test SHALL pass without any output

#### Scenario: Snapshot Mismatch
- **WHEN** rendered output differs from the stored snapshot
- **THEN** insta SHALL provide a diff and offer to update the snapshot

#### Scenario: New Snapshot Creation
- **WHEN** running tests for the first time
- **THEN** the framework SHALL create initial snapshot files automatically

### Requirement: Partial Buffer Comparison
The framework SHALL support comparing specific regions of the buffer rather than the entire screen.

#### Scenario: Region-Based Snapshot
- **WHEN** a test specifies a buffer region (x, y, width, height)
- **THEN** only that region SHALL be compared in snapshots

#### Scenario: Cell-Level Assertions
- **WHEN** testing specific cell content
- **THEN** the framework SHALL provide helpers to assert individual cell attributes (character, style, foreground, background)

### Requirement: Style and Color Validation
The framework SHALL capture and compare terminal styling (colors, attributes) accurately.

#### Scenario: Color Fidelity
- **WHEN** rendering colored content
- **THEN** the color values SHALL be captured and compared exactly

#### Scenario: Text Attribute Preservation
- **WHEN** rendering bold, italic, underline, or other attributes
- **THEN** these attributes SHALL be preserved in snapshots
