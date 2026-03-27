## ADDED Requirements

### Requirement: TUI Session Timeline View
The TUI SHALL implement a stateful timeline view that allows users to scroll through history, view message metadata, and navigate fork points.

#### Scenario: Navigating history
- **WHEN** the user opens the timeline in a session
- **THEN** the TUI SHALL render a scrollable list of all messages and tool calls in chronological order.

### Requirement: TUI Theme Management
The TUI SHALL support dynamic loading and switching between themes defined in JSON files matching the TS implementation.

#### Scenario: Switching theme
- **WHEN** the user selects a new theme from the configuration
- **THEN** the TUI MUST immediately update its color palette and style according to the JSON definition.
