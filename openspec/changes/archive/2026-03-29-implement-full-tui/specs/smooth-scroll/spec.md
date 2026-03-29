## ADDED Requirements

### Requirement: Smooth Scrolling
The system SHALL implement smooth scrolling with acceleration.

#### Scenario: Scroll Acceleration
- **WHEN** user holds PageDown key
- **THEN** scroll speed increases gradually

#### Scenario: Maximum Velocity
- **WHEN** scroll velocity reaches maximum
- **THEN** system caps velocity to prevent overshooting

#### Scenario: Deceleration
- **WHEN** user releases scroll key
- **THEN** system gradually slows down before stopping

### Requirement: Scroll Configuration
The system SHALL allow scroll behavior configuration.

#### Scenario: Scroll Speed Setting
- **WHEN** user sets `scroll_speed` in config
- **THEN** system uses that as base scroll speed

#### Scenario: Acceleration Toggle
- **WHEN** user sets `scroll_acceleration.enabled` to false
- **THEN** system uses fixed-step scrolling

### Requirement: Scroll Position Tracking
The system SHALL track scroll position accurately.

#### Scenario: Scroll Offset
- **WHEN** user scrolls
- **THEN** system maintains correct offset in message history

#### Scenario: Scroll Reset
- **WHEN** new message arrives
- **THEN** system can auto-scroll to bottom (if configured)
