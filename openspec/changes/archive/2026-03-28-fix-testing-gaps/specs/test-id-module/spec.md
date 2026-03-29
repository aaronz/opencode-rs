## ADDED Requirements

### Requirement: Identifier generation tests exist
The test suite SHALL cover all exported functions from packages/opencode/src/id/id.ts including schema generation, ID creation, and timestamp extraction.

#### Scenario: Schema validation
- **WHEN** Identifier.schema() is called with a valid prefix
- **THEN** it returns a Zod schema that validates IDs with that prefix

#### Scenario: ID creation with prefix
- **WHEN** Identifier.create() is called with a prefix
- **THEN** the resulting ID starts with that prefix followed by underscore

#### Scenario: Ascending vs descending order
- **WHEN** two IDs are created with same prefix but different timestamps in ascending mode
- **THEN** the earlier timestamp ID is lexicographically less than the later

#### Scenario: Timestamp extraction
- **WHEN** Identifier.timestamp() is called with a valid ID
- **THEN** it returns a number representing the creation timestamp
