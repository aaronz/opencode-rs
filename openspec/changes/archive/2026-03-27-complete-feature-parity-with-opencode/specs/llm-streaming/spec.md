## ADDED Requirements

### Requirement: Token Usage Tracking
All LLM providers SHALL accurately track and return token usage metadata (input, output, cache hits).

#### Scenario: Metadata in completion response
- **WHEN** a completion request is made to any provider
- **THEN** the returned object MUST include a `usage` field with detailed token counts.

### Requirement: Provider-Specific Options
The system SHALL support passing provider-specific options (like temperature, top_p, or frequency_penalty) consistently across all implementations.

#### Scenario: Setting temperature for OpenAI
- **WHEN** a request is made with `temperature: 0.5`
- **THEN** the OpenAI provider MUST include this parameter in the API call to OpenAI.
