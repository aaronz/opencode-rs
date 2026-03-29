## ADDED Requirements

### Requirement: Test dependencies are fixed
The test infrastructure SHALL allow all tests to execute without module resolution errors.

#### Scenario: Bun install completes successfully
- **WHEN** bun install is run from the project root
- **THEN** all dependencies resolve without errors

#### Scenario: Tests can import source modules
- **WHEN** a test file imports from src/
- **THEN** the import resolves correctly without ENOENT errors

#### Scenario: Tests run without unhandled errors
- **WHEN** bun test is executed
- **THEN** tests run to completion without unhandled rejection errors

#### Scenario: Shared workspace dependencies work
- **WHEN** packages depend on workspace:*
- **THEN** the symlinks resolve correctly
