## ADDED Requirements

### Requirement: Env module tests exist
The test suite SHALL cover the Env namespace from packages/opencode/src/env/index.ts including get, set, remove, and all functions.

#### Scenario: Get environment variable
- **WHEN** Env.get() is called with a key that exists
- **THEN** it returns the corresponding value

#### Scenario: Set environment variable
- **WHEN** Env.set() is called with a key and value
- **THEN** subsequent calls to Env.get() return that value

#### Scenario: Remove environment variable
- **WHEN** Env.remove() is called with a key
- **THEN** subsequent calls to Env.get() return undefined

#### Scenario: Get all environment variables
- **WHEN** Env.all() is called
- **THEN** it returns all environment variables as an object
