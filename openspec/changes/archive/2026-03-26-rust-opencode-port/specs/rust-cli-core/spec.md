## ADDED Requirements

### Requirement: CLI application entry point
The CLI application SHALL provide a main entry point that parses command-line arguments and launches the appropriate mode.

#### Scenario: Interactive mode
- **WHEN** user runs `opencode-rs` without subcommands
- **THEN** application launches in interactive TUI mode

#### Scenario: Version flag
- **WHEN** user runs `opencode-rs --version`
- **THEN** application prints version string and exits

#### Scenario: Help flag
- **WHEN** user runs `opencode-rs --help`
- **THEN** application prints help message with available options

### Requirement: Configuration file loading
The application SHALL load configuration from standard locations.

#### Scenario: Default config location
- **WHEN** application starts without explicit config path
- **THEN** it loads config from `~/.config/opencode-rs/config.toml`

#### Scenario: Custom config path
- **WHEN** user provides `--config <path>` flag
- **THEN** application loads config from specified path

#### Scenario: Missing config
- **WHEN** config file does not exist
- **THEN** application uses default configuration with helpful warnings

### Requirement: Environment variable support
The application SHALL respect environment variables for sensitive configuration.

#### Scenario: API key from environment
- **WHEN** API key is set via `OPENCODE_API_KEY` environment variable
- **THEN** application uses this key for authentication

#### Scenario: Provider override
- **WHEN** `OPENCODE_PROVIDER` environment variable is set
- **THEN** application uses specified provider regardless of config
