## Why

OpenCode-RS currently lacks a unified abstraction for model providers and their authentication protocols. Each provider (OpenAI, Anthropic, Gemini, OpenRouter, local endpoints) uses different auth mechanisms—Bearer tokens, custom headers, query parameters, OAuth sessions, or none. Hardcoding provider+auth couples limits flexibility, prevents enterprise gateway integration, and blocks support for new providers. A clean separation between Provider Protocol and Auth Strategy is needed to support diverse authentication requirements.

## What Changes

- **New Capability**: Provider abstraction layer with protocol-agnostic design
  - `ModelProvider` trait defining list_models, chat, embed interfaces
  - Per-provider adapters for OpenAI-compatible, Anthropic, Gemini, OpenRouter, Local endpoints

- **New Capability**: Auth Strategy abstraction layer
  - `AuthStrategy` enum: BearerApiKey, HeaderApiKey, QueryApiKey, OAuthSession, None
  - Decoupled from provider—allows any auth strategy with any provider
  - Credential management with expiration, rotation support

- **New Capability**: Provider credential configuration system
  - Structured config for API keys, endpoints, headers
  - Credential reference pattern for secure storage
  - Precedence rules for multi-source credentials (env > config > interactive)

- **Modified Capability**: None (greenfield implementation)

- **Impact**:
  - New crate: `crates/llm/src/provider.rs` (existing)
  - New crate: `crates/llm/src/auth.rs` (existing)
  - Config system additions
  - Security: credential storage encryption, runtime vs provider auth boundary

## Capabilities

### New Capabilities
- `provider-adapter`: Unified ModelProvider trait with per-provider implementations
- `auth-strategy`: AuthStrategy enum abstraction for flexible authentication
- `credential-config`: Provider credential configuration and management
- `provider-management-api`: Runtime API for provider CRUD operations

### Modified Capabilities
- (none - greenfield implementation)

## Impact

- **Affected crates**: `crates/llm`, `crates/config`
- **Dependencies**: No new external deps; leverage existing auth.rs, provider.rs in llm
- **APIs**: New provider management REST endpoints, config file schema extensions
- **Security**: Credential encryption at rest, secure memory handling for keys
