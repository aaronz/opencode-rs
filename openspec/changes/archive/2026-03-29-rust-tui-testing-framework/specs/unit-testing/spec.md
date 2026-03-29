## ADDED Requirements

### Requirement: Pure State Machine Testing
The testing framework SHALL provide utilities for testing TUI state machines in isolation without any rendering or terminal dependencies.

#### Scenario: Basic State Transition
- **WHEN** a state machine receives a key event via the testing utility
- **THEN** the state MUST transition according to the defined reducer function

#### Scenario: State Equality Verification
- **WHEN** comparing two state instances for equality
- **THEN** the framework SHALL support custom equality predicates for complex state types

### Requirement: Event-Driven Update Testing
The framework SHALL support testing the update (reducer) function independently from the view layer.

#### Scenario: Single Event Processing
- **WHEN** update(state, event) is called with a valid event
- **THEN** the function SHALL return a new state reflecting the event's effect

#### Scenario: Multiple Event Sequence
- **WHEN** update is called with a sequence of events
- **THEN** each event SHALL be processed in order, producing cumulative state changes

#### Scenario: No-Op Event Handling
- **WHEN** update receives an event that should not change state
- **THEN** the original state SHALL be returned unchanged

### Requirement: State Property Validation
The framework SHALL provide assertion helpers for validating state properties after transitions.

#### Scenario: Selected Item Validation
- **WHEN** testing navigation in a list
- **THEN** the selected index MUST be within bounds after key events

#### Scenario: Mode State Validation
- **WHEN** testing mode switching (e.g., normal to insert mode)
- **THEN** the current mode MUST be correctly reflected in state

### Requirement: Test Isolation
Each state machine test SHALL execute in isolation without side effects between tests.

#### Scenario: Independent Test Execution
- **WHEN** multiple tests run sequentially
- **THEN** each test MUST start with a clean state instance
