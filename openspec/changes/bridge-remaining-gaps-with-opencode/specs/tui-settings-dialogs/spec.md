## ADDED Requirements

### Requirement: Settings Dialog System
The TUI SHALL provide comprehensive settings management through categorized dialogs.

#### Scenario: Opening Settings
- **WHEN** the user presses `Ctrl+,` or selects settings from command palette
- **THEN** the TUI SHALL display a settings dialog with tabs: General, Keybinds, Models, Providers

#### Scenario: Navigating Settings Tabs
- **WHEN** the user presses `Tab` or arrow keys in settings
- **THEN** the active tab SHALL change and its content SHALL be displayed

#### Scenario: Modifying Settings
- **WHEN** the user edits a setting value
- **THEN** the change SHALL be applied immediately or on save (configurable)
- **AND** invalid values SHALL show an error indicator

#### Scenario: Keybind Configuration
- **WHEN** the user is in the Keybinds tab
- **THEN** they SHALL be able to view and modify keyboard shortcuts
- **AND** conflicting keybinds SHALL be highlighted

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
