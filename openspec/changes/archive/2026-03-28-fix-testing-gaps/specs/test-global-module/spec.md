## ADDED Requirements

### Requirement: Global module tests exist
The test suite SHALL cover the Global namespace from packages/opencode/src/global/index.ts including path configuration.

#### Scenario: Global path access
- **WHEN** Global.Path.data is accessed
- **THEN** it returns a valid directory path

#### Scenario: Environment variable override
- **WHEN** OPENCODE_TEST_HOME environment variable is set
- **THEN** Global.Path.home returns the override value instead of os.homedir()

#### Scenario: Cache version migration
- **WHEN** cache version file is missing or outdated
- **THEN** the cache directory is cleared and new version is written
