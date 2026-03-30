## ADDED Requirements

### Requirement: Provider credential configuration
The system SHALL support configuring provider credentials through a structured configuration system.

#### Scenario: Define provider with inline credentials
- **WHEN** a user provides `api_key` directly in provider config
- **THEN** the system SHALL use that key for authentication

#### Scenario: Define provider with credential reference
- **WHEN** a user provides `credentialRef` pointing to a stored credential
- **THEN** the system SHALL retrieve the credential from secure storage and use it

#### Scenario: Environment variable override
- **WHEN** environment variable matching provider's `env_override` pattern is set
- **THEN** it SHALL take precedence over file-based configuration

### Requirement: Credential precedence
The system SHALL define clear precedence for credential sources.

#### Scenario: Multiple credential sources present
- **WHEN** a provider has credentials from env, config file, and stored credential
- **THEN** the precedence SHALL be: environment variable > stored credential > inline config

#### Scenario: Missing required credential
- **WHEN** a provider is used but no valid credential exists
- **THEN** the system SHALL return `AuthError::MissingCredential` with provider name

### Requirement: Secure credential storage
The system SHALL encrypt credentials at rest.

#### Scenario: Encrypt stored credentials
- **WHEN** saving credentials to disk
- **THEN** the system SHALL use AES-256-GCM encryption with a derived key

#### Scenario: Master password for encryption
- **WHEN** initializing encrypted credential storage
- **THEN** the system SHALL require a master password or derive one from system keyring

### Requirement: Credential validation
The system SHALL validate credentials before use.

#### Scenario: Validate API key format
- **WHEN** a credential is configured
- **THEN** the system MAY perform basic format validation (e.g., non-empty, expected prefix)

#### Scenario: Test credential connectivity
- **WHEN** user requests validation of a provider credential
- **THEN** the system SHALL attempt a minimal API call and report success/failure
