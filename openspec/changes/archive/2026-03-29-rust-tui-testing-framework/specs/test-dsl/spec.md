## ADDED Requirements

### Requirement: Fluent Test Builder
The framework SHALL provide a builder pattern for constructing TUI tests with a fluent API.

#### Scenario: Builder Initialization
- **WHEN** starting to build a test
- **THEN** the builder SHALL provide a chainable API starting with tui()

#### Scenario: Type Text Action
- **WHEN** specifying type_text("hello")
- **THEN** the builder SHALL queue keystrokes to be sent to the TUI

#### Scenario: Key Press Action
- **WHEN** specifying press_key(Key::Enter)
- **THEN** the builder SHALL queue the specific key to be sent

#### Scenario: Expect Screen Action
- **WHEN** specifying expect_screen("expected content")
- **THEN** the builder SHALL queue an assertion for screen content

### Requirement: Action Chaining
The DSL SHALL support chaining multiple actions in sequence.

#### Scenario: Sequential Actions
- **WHEN** multiple actions are chained
- **THEN** they SHALL execute in order with proper timing

#### Scenario: Mixed Input and Assertions
- **WHEN** input actions and assertions are interleaved
- **THEN** each SHALL execute at the appropriate point in the sequence

### Requirement: Wait Conditions
The DSL SHALL support waiting for conditions before proceeding.

#### Scenario: Wait for Text
- **WHEN** wait_for("Loading...") is specified
- **THEN** the test SHALL wait until the text appears or timeout

#### Scenario: Wait for Timeout
- **WHEN** wait(Duration::from_secs(1)) is specified
- **THEN** the test SHALL pause for the specified duration

### Requirement: Assertion Macros
The framework SHALL provide declarative assertion macros for common scenarios.

#### Scenario: Screen Contains Assertion
- **WHEN** using assert_screen().contains("text")
- **THEN** the test SHALL fail if the screen doesn't contain the text

#### Scenario: Screen Matches Snapshot
- **WHEN** using assert_screen().matches_snapshot()
- **THEN** the test SHALL use insta for snapshot comparison

#### Scenario: State Property Assertion
- **WHEN** using assert_state().selected(5)
- **THEN** the test SHALL verify the internal state has correct values
