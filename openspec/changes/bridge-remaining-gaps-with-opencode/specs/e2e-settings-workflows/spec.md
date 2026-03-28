## ADDED Requirements

### Requirement: Settings Configuration Tests
The E2E harness SHALL verify all settings categories.

#### Scenario: General Settings
- **WHEN** general settings are modified (theme, font, etc.)
- **THEN** changes SHALL apply immediately
- **AND** persist across restarts

#### Scenario: Keybind Settings
- **WHEN** keybinds are customized
- **THEN** new shortcuts SHALL work immediately
- **AND** conflicts SHALL be detected and reported

#### Scenario: Model Settings
- **WHEN** default model is changed
- **THEN** new conversations SHALL use the selected model
- **AND** existing conversations SHALL be unaffected

#### Scenario: Provider Settings
- **WHEN** provider API keys are updated
- **THEN** connections SHALL be re-tested
- **AND** errors SHALL be surfaced clearly
