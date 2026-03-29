## ADDED Requirements

### Requirement: Theme Presets
The system SHALL include multiple built-in theme presets.

#### Scenario: Theme Selection
- **WHEN** user selects a theme from settings
- **THEN** system applies the theme immediately

#### Scenario: Built-in Themes
- **WHEN** system initializes
- **THEN** catppuccin, tokyonight, nord, gruvbox themes are available

### Requirement: Theme Switching
The system SHALL allow runtime theme switching.

#### Scenario: Theme Switch Command
- **WHEN** user types `/themes`
- **THEN** system opens theme selection overlay

#### Scenario: Theme Persistence
- **WHEN** user selects a theme
- **THEN** system saves the selection to config file

### Requirement: Truecolor Support
The system SHALL support 24-bit truecolor.

#### Scenario: Color Rendering
- **WHEN** theme uses RGB colors
- **THEN** terminal renders colors accurately if supported

#### Scenario: Color Fallback
- **WHEN** terminal doesn't support truecolor
- **THEN** system falls back to 256-color palette
