## ADDED Requirements

### Requirement: LLM Provider trait
The system SHALL define a Provider trait for pluggable LLM backends.

#### Scenario: Provider trait definition
- **WHEN** implementing a new LLM provider
- **THEN** the provider MUST implement chat and stream_chat methods

#### Scenario: Async chat completion
- **WHEN** provider receives a list of messages
- **THEN** provider returns a complete response asynchronously

#### Scenario: Streaming chat completion
- **WHEN** provider receives a streaming request
- **THEN** provider returns a stream of response chunks

### Requirement: OpenAI provider
The system SHALL provide an OpenAI-compatible provider.

#### Scenario: OpenAI API call
- **WHEN** user configures OpenAI as provider
- **THEN** system sends requests to OpenAI API endpoint

#### Scenario: Model selection
- **WHEN** user specifies model in config (e.g., gpt-4o, gpt-4o-mini)
- **THEN** provider uses specified model for API calls

#### Scenario: API key authentication
- **WHEN** OpenAI provider makes API request
- **THEN** request includes Bearer token authentication

### Requirement: Anthropic provider
The system SHALL provide an Anthropic Claude-compatible provider.

#### Scenario: Anthropic API call
- **WHEN** user configures Anthropic as provider
- **THEN** system sends requests to Anthropic API endpoint

#### Scenario: Claude model selection
- **WHEN** user specifies Claude model (e.g., claude-sonnet-4-20250514)
- **THEN** provider uses specified model for API calls

### Requirement: Ollama provider
The system SHALL provide an Ollama provider for local models.

#### Scenario: Ollama local call
- **WHEN** user configures Ollama as provider
- **THEN** system sends requests to local Ollama server

#### Scenario: Default Ollama endpoint
- **WHEN** Ollama provider is used without custom endpoint
- **THEN** provider connects to http://localhost:11434

### Requirement: Message format
The system SHALL support standard message format across providers.

#### Scenario: System message
- **WHEN** conversation includes system message
- **THEN** message is included in provider request appropriately

#### Scenario: User message
- **WHEN** user sends a message
- **THEN** message is formatted as user role for provider

#### Scenario: Assistant message
- **WHEN** assistant responds
- **THEN** message is stored with assistant role for context

### Requirement: Provider fallback
The system SHALL handle provider errors gracefully.

#### Scenario: API rate limit
- **WHEN** provider returns rate limit error
- **THEN** system waits and retries with exponential backoff

#### Scenario: API error
- **WHEN** provider returns an error
- **THEN** error is transformed to application error with helpful message
