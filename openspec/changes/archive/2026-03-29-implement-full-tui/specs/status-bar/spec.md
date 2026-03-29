## MODIFIED Requirements

### Requirement: Status Bar Mode Indicator
The status bar SHALL display the current mode (Plan/Build).

#### Scenario: Mode Display
- **WHEN** mode changes
- **THEN** status bar shows "PLAN" or "BUILD" with appropriate color

#### Scenario: Mode Color
- **WHEN** in Plan Mode
- **THEN** mode indicator uses muted color

#### Scenario: Build Mode Color
- **WHEN** in Build Mode
- **THEN** mode indicator uses accent color

### Requirement: Status Bar Model Display
The status bar SHALL display current model and provider.

#### Scenario: Model Display
- **WHEN** system is running
- **THEN** status bar shows provider and model name

#### Scenario: Model Update
- **WHEN** user switches models
- **THEN** status bar updates immediately

### Requirement: Status Bar Leader Key Hints
The status bar SHALL show leader key hints.

#### Scenario: Leader Key Hint
- **WHEN** system is idle
- **THEN** status bar shows "ctrl+x: commands" hint

#### Scenario: Active Leader State
- **WHEN** leader key is pressed
- **THEN** status bar shows available actions

### Requirement: Status Bar Connection Status
The status bar SHALL display connection status.

#### Scenario: Connection Indicator
- **WHEN** connected to LLM provider
- **THEN** status bar shows green indicator

#### Scenario: Disconnection
- **WHEN** connection is lost
- **THEN** status bar shows red indicator
