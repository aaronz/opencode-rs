## ADDED Requirements

### Requirement: Util package tests exist
The test suite SHALL cover utility functions from packages/util/src/ including encoding, retry, and helper functions.

#### Scenario: Base64 encoding/decoding
- **WHEN** encode64() is called with a string
- **THEN** it returns a base64 encoded string
- **WHEN** decode64() is called with a base64 string
- **THEN** it returns the original string

#### Scenario: Retry logic
- **WHEN** a function is wrapped with retry logic and fails
- **THEN** it retries the specified number of times before throwing

#### Scenario: Lazy evaluation
- **WHEN** a lazy value is created
- **THEN** it is only evaluated when accessed
