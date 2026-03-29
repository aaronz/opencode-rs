## ADDED Requirements

### Requirement: Provider module tests exist
The test suite SHALL cover AI provider implementations from packages/opencode/src/provider/ including model listing, API calls, and error handling.

#### Scenario: Provider initialization
- **WHEN** a provider is instantiated
- **THEN** it has correct name and default settings

#### Scenario: Model listing
- **WHEN** available models are requested
- **THEN** the provider returns a list of supported models

#### Scenario: API request construction
- **WHEN** a chat request is built
- **THEN** it includes required fields (model, messages, temperature)

#### Scenario: Error handling
- **WHEN** API returns an error
- **THEN** the provider throws an appropriate error with message
