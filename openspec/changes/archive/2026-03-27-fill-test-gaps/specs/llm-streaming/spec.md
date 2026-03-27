## ADDED Requirements

### Requirement: Async LLM Streaming
The LLM provider system SHALL support full async streaming of responses with proper error handling and cancellation.

#### Scenario: OpenAI Streaming
- **WHEN** user calls `run --prompt "hello"` with OpenAI provider
- **THEN** response streams token-by-token to stdout in real-time

#### Scenario: Anthropic Streaming
- **WHEN** user calls `run --prompt "hello"` with Anthropic provider
- **THEN** response streams with proper Claude message format

#### Scenario: Ollama Local Streaming
- **WHEN** user calls `run --prompt "hello"` with Ollama provider
- **THEN** response streams from local Ollama server

#### Scenario: Provider Error Handling
- **WHEN** LLM API returns an error status code
- **THEN** error is returned with proper error message and code

#### Scenario: Request Cancellation
- **WHEN** user presses Ctrl+C during streaming response
- **THEN** request is cancelled and partial response is discarded

### Requirement: Model Listing
The system SHALL provide accurate model lists per provider with proper filtering.

#### Scenario: List All Models
- **WHEN** user runs `opencode-rs models`
- **THEN** all available models from all providers are displayed

#### Scenario: Filter by Provider
- **WHEN** user runs `opencode-rs models --provider openai`
- **THEN** only OpenAI models are displayed

#### Scenario: JSON Output
- **WHEN** user runs `opencode-rs models --json`
- **THEN** models are returned as JSON array

### Requirement: Provider Configuration
The system SHALL support multiple LLM providers with proper API key handling.

#### Scenario: OpenAI Provider
- **WHEN** OPENAI_API_KEY is set and provider is "openai"
- **THEN** requests are sent to OpenAI API with proper authentication

#### Scenario: Anthropic Provider
- **WHEN** ANTHROPIC_API_KEY is set and provider is "anthropic"
- **THEN** requests are sent to Anthropic API with proper headers

#### Scenario: Ollama Provider
- **WHEN** provider is "ollama" and Ollama server is running locally
- **THEN** requests are sent to localhost:11434

#### Scenario: Missing API Key
- **WHEN** provider is specified but API key is not set
- **THEN** clear error message is displayed explaining how to set API key