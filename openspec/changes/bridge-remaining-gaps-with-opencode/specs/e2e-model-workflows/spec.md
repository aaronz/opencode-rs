## ADDED Requirements

### Requirement: Model Selection Dialog Tests
The E2E harness SHALL verify model selection UI functionality.

#### Scenario: Opening Model Dialog
- **WHEN** the model selection keybind is triggered
- **THEN** the model dialog SHALL appear
- **AND** available models SHALL be listed

#### Scenario: Filtering Models
- **WHEN** the user types in the model filter
- **THEN** the list SHALL filter to matching models
- **AND** provider grouping SHALL be maintained

#### Scenario: Selecting Paid Models
- **WHEN** a paid model is selected
- **THEN** appropriate billing indicators SHALL appear
- **AND** the selection SHALL require confirmation

### Requirement: Provider Configuration Tests
The E2E harness SHALL verify provider management.

#### Scenario: Adding New Provider
- **WHEN** provider configuration is submitted
- **THEN** the provider SHALL be saved
- **AND** connection SHALL be tested

#### Scenario: Provider Connection Test
- **WHEN** a provider is configured with invalid credentials
- **THEN** an error SHALL be displayed
- **AND** the provider SHALL be marked as disconnected
