## ADDED Requirements

### Requirement: Storage module tests exist
The test suite SHALL cover database operations from packages/opencode/src/storage/ including CRUD operations and transactions.

#### Scenario: Session creation
- **WHEN** a new session is created
- **THEN** it can be retrieved by ID

#### Scenario: Session update
- **WHEN** session data is modified
- **THEN** the changes are persisted

#### Scenario: Session deletion
- **WHEN** a session is deleted
- **THEN** it can no longer be retrieved

#### Scenario: Query with filters
- **WHEN** sessions are queried with filters
- **THEN** only matching sessions are returned
