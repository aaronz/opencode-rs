## ADDED Requirements

### Requirement: Edit files with exact string matching
The system SHALL provide a tool for editing files with exact string matching.

#### Scenario: Edit application
- **WHEN** user provides old string and new string
- **THEN** system replaces exact match in file

#### Scenario: Edit validation
- **WHEN** old string not found in file
- **THEN** system returns error with file content context

#### Scenario: Edit multiple matches
- **WHEN** old string matches multiple locations
- **THEN** system returns error with all match locations

#### Scenario: Edit with context
- **WHEN** user provides surrounding context
- **THEN** system uses context to disambiguate matches
