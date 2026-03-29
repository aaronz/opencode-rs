## ADDED Requirements

### Requirement: Plan/Build Mode Toggle
The system SHALL support two distinct modes: Plan Mode (read-only) and Build Mode (execution).

#### Scenario: Mode Toggle
- **WHEN** user presses `Tab` key
- **THEN** system toggles between Plan and Build modes

#### Scenario: Plan Mode Behavior
- **WHEN** system is in Plan Mode
- **THEN** LLM cannot execute file modifications

#### Scenario: Build Mode Behavior
- **WHEN** system is in Build Mode
- **THEN** LLM can execute file modifications

### Requirement: Mode Indicator
The system SHALL display the current mode in the status bar.

#### Scenario: Mode Display
- **WHEN** mode changes
- **THEN** status bar updates to show current mode (Plan/Build)

#### Scenario: Mode Color
- **WHEN** in Plan Mode
- **THEN** status bar shows "PLAN" in muted color

#### Scenario: Build Mode Color
- **WHEN** in Build Mode
- **THEN** status bar shows "BUILD" in accent color
