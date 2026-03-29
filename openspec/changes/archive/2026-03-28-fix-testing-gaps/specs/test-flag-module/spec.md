## ADDED Requirements

### Requirement: Flag module tests exist
The test suite SHALL cover all exported flags from packages/opencode/src/flag/flag.ts including boolean flags, string flags, and dynamic getters.

#### Scenario: Boolean flag defaults
- **WHEN** boolean flags are evaluated without environment variables set
- **THEN** they return expected default values (false for most, true for OPENCODE_EXPERIMENTAL_MARKDOWN)

#### Scenario: String flag defaults
- **WHEN** string flags are evaluated without environment variables set
- **THEN** they return undefined

#### Scenario: Dynamic getter evaluation
- **WHEN** dynamic getters are accessed
- **THEN** they read from process.env at access time

#### Scenario: Truthy value detection
- **WHEN** environment variable is set to "true" or "1"
- **THEN** the corresponding flag evaluates to true

#### Scenario: Falsy value detection
- **WHEN** environment variable is set to "false" or "0"
- **THEN** the corresponding flag evaluates to false
