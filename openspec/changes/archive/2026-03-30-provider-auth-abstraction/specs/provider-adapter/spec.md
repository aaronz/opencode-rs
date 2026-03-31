## ADDED Requirements

### Requirement: ModelProvider trait
The system SHALL provide a `ModelProvider` trait that defines the contract for interacting with LLM providers.

#### Scenario: List available models
- **WHEN** a client calls `list_models()` on a provider adapter
- **THEN** the adapter SHALL return a vector of `ModelInfo` containing id, name, and capabilities for each model

#### Scenario: Send chat request
- **WHEN** a client calls `chat(request)` on a provider adapter
- **THEN** the adapter SHALL send the request to the provider's endpoint and return a `ChatStream` for streaming responses

#### Scenario: Send embed request
- **WHEN** a client calls `embed(request)` on a provider adapter
- **THEN** the adapter SHALL send the request and return an `EmbedResponse` with embeddings

### Requirement: OpenAI-compatible adapter
The system SHALL provide a provider adapter that conforms to the OpenAI-compatible API specification.

#### Scenario: Configure base URL
- **WHEN** a user configures a provider with `base_url` set to an OpenAI-compatible endpoint
- **THEN** the adapter SHALL use that URL as the service endpoint

#### Scenario: Send Bearer token auth
- **WHEN** an OpenAI-compatible adapter sends a request
- **THEN** it SHALL include `Authorization: Bearer <api_key>` header

#### Scenario: Override default headers
- **WHEN** a user provides custom headers in provider configuration
- **THEN** the adapter SHALL merge them with required headers, with custom values taking precedence

### Requirement: Anthropic adapter
The system SHALL provide a dedicated adapter for Anthropic's Claude API.

#### Scenario: Send Anthropic-specific headers
- **WHEN** an Anthropic adapter sends a request
- **THEN** it SHALL include `x-api-key` and `anthropic-version` headers

#### Scenario: Handle Claude tool use
- **WHEN** a request includes tool definitions
- **THEN** the Anthropic adapter SHALL convert them to Claude's tool_use format

### Requirement: Gemini adapter
The system SHALL provide an adapter for Google's Gemini API.

#### Scenario: Configure Gemini API key
- **WHEN** a user configures a Gemini provider
- **THEN** the adapter SHALL use the configured API key for authentication

#### Scenario: Handle Gemini REST format
- **WHEN** sending a chat request to Gemini
- **THEN** the adapter SHALL convert the request to Gemini's REST JSON format

### Requirement: OpenRouter adapter
The system SHALL provide an adapter for OpenRouter aggregation service.

#### Scenario: Include OpenRouter headers
- **WHEN** an OpenRouter adapter sends a request
- **THEN** it SHALL include optional `HTTP-Referer` and `X-OpenRouter-Title` headers for site identification

#### Scenario: Route to multiple models
- **WHEN** listing models from OpenRouter
- **THEN** the adapter SHALL return the aggregated model list from all supported providers

### Requirement: Local endpoint adapter
The system SHALL provide an adapter for local or internal OpenAI-compatible endpoints.

#### Scenario: Disable authentication
- **WHEN** a user configures `auth_strategy: None`
- **THEN** the adapter SHALL NOT include any authentication headers

#### Scenario: Use custom auth headers
- **WHEN** a user provides custom authentication via configuration
- **THEN** the adapter SHALL inject those headers into every request
