## ADDED Requirements

### Requirement: Leader Key Activation
The system SHALL support a leader key mechanism similar to Tmux. When the user presses the configured leader key (default: `ctrl+x`), the system SHALL enter a "waiting for action" state.

#### Scenario: Leader Key Pressed
- **WHEN** user presses `ctrl+x`
- **THEN** system enters `LeaderKeyState::WaitingForAction` and displays visual indicator

#### Scenario: Leader Key Timeout
- **WHEN** 2000ms elapses after leader key press without another key
- **THEN** system returns to `LeaderKeyState::Idle`

### Requirement: Leader Key Actions
The system SHALL support single-key actions after leader key activation.

#### Scenario: Compact Session
- **WHEN** user presses `ctrl+x` then `c`
- **THEN** system triggers session compaction

#### Scenario: Quit Application
- **WHEN** user presses `ctrl+x` then `q`
- **THEN** system exits the application

#### Scenario: Open External Editor
- **WHEN** user presses `ctrl+x` then `e`
- **THEN** system opens `$EDITOR` for prompt composition

#### Scenario: List Sessions
- **WHEN** user presses `ctrl+x` then `l`
- **THEN** system shows session list overlay

#### Scenario: Toggle Details
- **WHEN** user presses `ctrl+x` then `d`
- **THEN** system toggles tool execution details visibility

#### Scenario: Switch Model
- **WHEN** user presses `ctrl+x` then `m`
- **THEN** system opens model selection dialog

### Requirement: Leader Key Configuration
The system SHALL allow users to configure the leader key binding.

#### Scenario: Custom Leader Key
- **WHEN** user sets `leader_key` in config to `"ctrl+a"`
- **THEN** system uses `ctrl+a` as the leader key instead of `ctrl+x`
