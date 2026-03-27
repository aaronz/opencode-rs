## ADDED Requirements

### Requirement: Config File Loading
The system SHALL load configuration from file with proper path resolution.

#### Scenario: Default Config Location
- **WHEN** no config file is specified
- **THEN** system looks for config in ~/.config/opencode-rs/config.toml

#### Scenario: Custom Config Path
- **WHEN** user runs with --config /path/to/config.toml
- **THEN** config is loaded from specified path

#### Scenario: Missing Config File
- **WHEN** config file doesn't exist
- **THEN** default configuration is used

### Requirement: Environment Variable Override
The system SHALL allow environment variables to override config file settings.

#### Scenario: Provider Override
- **WHEN** OPENCODE_PROVIDER environment variable is set
- **THEN** it overrides provider setting from config file

#### Scenario: Model Override
- **WHEN** OPENCODE_MODEL environment variable is set
- **THEN** it overrides model setting from config file

#### Scenario: API Key Override
- **WHEN** OPENCODE_API_KEY environment variable is set
- **THEN** it overrides api_key setting from config file

### Requirement: CLI Flag Override
The system SHALL allow CLI flags to override all other configuration sources.

#### Scenario: Model CLI Override
- **WHEN** user runs with --model gpt-4o flag
- **THEN** it overrides all other configuration sources

#### Scenario: Provider CLI Override
- **WHEN** user runs with --agent plan flag
- **THEN** agent mode is set to plan regardless of config

### Requirement: Config Validation
The system SHALL validate configuration and report errors.

#### Scenario: Invalid Provider
- **WHEN** config specifies unknown provider
- **THEN** error is displayed listing valid providers

#### Scenario: Missing Required Field
- **WHEN** config is missing provider
- **THEN** error is displayed indicating missing required field