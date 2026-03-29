## ADDED Requirements

### Requirement: CLI Binary Testing
The framework SHALL provide utilities for testing CLI applications using assert_cmd.

#### Scenario: Binary Discovery
- **WHEN** testing a cargo binary
- **THEN** the framework SHALL support automatic binary path resolution via Command::cargo_bin

#### Scenario: Argument Passing
- **WHEN** executing the CLI with arguments
- **THEN** the arguments SHALL be passed to the subprocess correctly

#### Scenario: Exit Code Verification
- **WHEN** the CLI exits
- **THEN** the test SHALL be able to assert success, failure, or specific exit codes

### Requirement: Standard Input/Output Testing
The framework SHALL support testing CLI applications with stdin input and stdout/stderr output verification.

#### Scenario: Stdin Input Simulation
- **WHEN** providing input via stdin
- **THEN** the CLI SHALL receive the input as if from a user

#### Scenario: Stdout Assertion
- **WHEN** asserting stdout content
- **THEN** the framework SHALL support contains, starts_with, ends_with, and exact matching

#### Scenario: Stderr Capture
- **WHEN** the CLI writes to stderr
- **THEN** the test SHALL be able to capture and assert on stderr content

### Requirement: Predicate-Based Assertions
The framework SHALL leverage predicates for flexible, composable assertions.

#### Scenario: String Containment
- **WHEN** asserting output contains "expected text"
- **THEN** predicates::str::contains SHALL be used for matching

#### Scenario: Pattern Matching
- **WHEN** asserting output matches a pattern
- **THEN** predicates::str::is_match SHALL support regex patterns

#### Scenario: Combined Predicates
- **WHEN** multiple conditions must be met
- **THEN** predicates SHALL be combinable with and/or logic

### Requirement: Interactive CLI Testing
The framework SHALL support testing interactive CLI sessions with multiple input/output exchanges.

#### Scenario: Multi-Turn Interaction
- **WHEN** the CLI requires multiple inputs
- **THEN** the framework SHALL support writing stdin in sequence

#### Scenario: Prompt Response
- **WHEN** the CLI displays a prompt and waits for input
- **THEN** the test SHALL be able to detect the prompt and respond
