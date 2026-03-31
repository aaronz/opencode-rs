## ADDED Requirements

### Requirement: List providers
The system SHALL provide an API to list all configured providers.

#### Scenario: GET /providers
- **WHEN** a client sends GET request to /providers
- **THEN** the system SHALL return a list of providers with id, type, endpoint, and auth_strategy (without secrets)

#### Scenario: Empty provider list
- **WHEN** no providers are configured
- **THEN** the system SHALL return an empty array

### Requirement: Add provider
The system SHALL provide an API to add a new provider configuration.

#### Scenario: POST /providers
- **WHEN** a client sends POST to /providers with valid provider config
- **THEN** the system SHALL create the provider and return its ID

#### Scenario: Duplicate provider
- **WHEN** adding a provider with same ID as existing
- **THEN** the system SHALL return error `ProviderError::AlreadyExists`

### Requirement: Update provider
The system SHALL provide an API to update an existing provider.

#### Scenario: PUT /providers/{id}
- **WHEN** a client sends PUT to update an existing provider
- **THEN** the system SHALL apply the changes and return updated provider info

#### Scenario: Update non-existent provider
- **WHEN** updating a provider that doesn't exist
- **THEN** the system SHALL return error `ProviderError::NotFound`

### Requirement: Remove provider
The system SHALL provide an API to remove a provider.

#### Scenario: DELETE /providers/{id}
- **WHEN** a client sends DELETE to remove a provider
- **THEN** the system SHALL remove the provider and return success

#### Scenario: Delete provider with active sessions
- **WHEN** deleting a provider that has active chat sessions
- **THEN** the system SHALL return warning but allow deletion

### Requirement: Test provider connection
The system SHALL provide an endpoint to test provider connectivity.

#### Scenario: POST /providers/{id}/test
- **WHEN** a client sends POST to test a provider
- **THEN** the system SHALL attempt a minimal request and return success/failure with details
