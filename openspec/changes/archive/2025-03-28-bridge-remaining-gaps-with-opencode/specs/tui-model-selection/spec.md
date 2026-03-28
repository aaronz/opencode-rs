## ADDED Requirements

### Requirement: Model Selection Dialog
The TUI SHALL provide dialogs for selecting AI models with filtering and search.

#### Scenario: Selecting a Model
- **WHEN** the user opens model selection (`Ctrl+M`)
- **THEN** a dialog SHALL display available models grouped by provider
- **AND** the user SHALL be able to filter by provider or search by name

#### Scenario: Paid vs Unpaid Model Display
- **WHEN** displaying models
- **THEN** paid models SHALL be marked with a $ indicator
- **AND** unavailable models SHALL be grayed out with reason

### Requirement: Provider Management Dialog
The TUI SHALL allow managing LLM provider configurations.

#### Scenario: Adding a Provider
- **WHEN** the user selects "Add Provider"
- **THEN** a form SHALL appear for API key, endpoint, and model selection
- **AND** the connection SHALL be tested before saving

#### Scenario: Provider Status
- **WHEN** viewing providers
- **THEN** each provider SHALL show connection status (connected/error)
