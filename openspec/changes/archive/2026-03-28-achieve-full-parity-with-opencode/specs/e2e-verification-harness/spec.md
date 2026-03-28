## ADDED Requirements

### Requirement: E2E Process Harness
The system SHALL include a test harness capable of spawning the compiled Rust CLI binary and verifying its behavior against input prompts and file system changes.

#### Scenario: Running core flow test
- **WHEN** the harness executes a test case for "session creation"
- **THEN** it MUST verify that the binary exits with code 0 and creates the expected session JSON on disk.

### Requirement: TS Test Compatibility
The e2e harness SHALL support running test cases ported from the TypeScript project, ensuring functional equivalence.

#### Scenario: Running ported TS test
- **WHEN** a ported test fixture for "tool argument validation" is run
- **THEN** the Rust binary MUST produce the exact same error messages and status codes as the TS implementation.
