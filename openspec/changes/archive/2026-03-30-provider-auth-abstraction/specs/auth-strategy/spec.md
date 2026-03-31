## ADDED Requirements

### Requirement: AuthStrategy enum
The system SHALL define an `AuthStrategy` enum to represent different authentication methods.

#### Scenario: BearerApiKey strategy
- **WHEN** `AuthStrategy::BearerApiKey` is selected
- **THEN** requests SHALL include `Authorization: Bearer <key>` header

#### Scenario: HeaderApiKey strategy
- **WHEN** `AuthStrategy::HeaderApiKey` is selected with header name "x-api-key"
- **THEN** requests SHALL include `x-api-key: <key>` header

#### Scenario: QueryApiKey strategy
- **WHEN** `AuthStrategy::QueryApiKey` is selected with parameter name "key"
- **THEN** requests SHALL include `?key=<key>` query parameter

#### Scenario: OAuthSession strategy
- **WHEN** `AuthStrategy::OAuthSession` is selected with access token
- **THEN** requests SHALL include the access token with Bearer scheme, and SHALL attempt refresh when expired

#### Scenario: None strategy
- **WHEN** `AuthStrategy::None` is selected
- **THEN** requests SHALL NOT include any authentication headers

### Requirement: Credential expiration handling
The system SHALL track credential expiration and support automatic refresh.

#### Scenario: Expired credential detected
- **WHEN** a request returns 401 and credential has `expires_at` set
- **THEN** the system SHALL attempt to refresh the credential if a refresh mechanism is available

#### Scenario: No refresh available
- **WHEN** credential is expired and no refresh mechanism exists
- **THEN** the system SHALL return an `AuthError::CredentialExpired` error

### Requirement: Credential data structure
The system SHALL store credentials with provider reference, key value, and optional expiration.

#### Scenario: Store API key credential
- **WHEN** saving a credential with provider "openai", key "sk-...", and no expiration
- **THEN** the system SHALL persist the credential with provider identifier, encrypted key, and null expiration

#### Scenario: Store OAuth session
- **WHEN** saving an OAuth session with access token, refresh token, and expires_in
- **THEN** the system SHALL compute `expires_at` from current time + expires_in seconds
