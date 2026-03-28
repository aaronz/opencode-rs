## ADDED Requirements

### Requirement: Token Usage Metadata
Every LLM provider completion SHALL return detailed token usage metadata, including input tokens, output tokens, and cache hits (if supported).

#### Scenario: Metadata in response
- **WHEN** a completion is received from OpenAI
- **THEN** the internal response object MUST populate the `usage` field with precise counts from the API response.

### Requirement: Unified Provider Options
The system SHALL support a unified set of provider options (temperature, top_p, stop_sequences) that are correctly mapped to each specific LLM backend.

#### Scenario: Passing temperature
- **WHEN** a request is made with `temperature: 0.7`
- **THEN** all enabled providers MUST include this parameter in their respective API calls.
