## 1. Core Trait Definitions

- [ ] 1.1 Define `ModelProvider` trait in `crates/llm/src/provider.rs` with `list_models`, `chat`, `embed` methods
- [ ] 1.2 Define `AuthStrategy` enum with variants: `BearerApiKey`, `HeaderApiKey`, `QueryApiKey`, `OAuthSession`, `None`
- [ ] 1.3 Define `Credential` struct with provider, key, expires_at fields
- [ ] 1.4 Define `ProviderConfig` struct with endpoint, auth_strategy, headers, credential_ref

## 2. Auth Strategy Implementation

- [ ] 2.1 Implement `AuthStrategy::apply_to_request()` method to inject auth into HTTP requests
- [ ] 2.2 Implement credential expiration checking in auth layer
- [ ] 2.3 Add OAuth session refresh logic for `OAuthSession` variant
- [ ] 2.4 Implement credential encryption/decryption utilities

## 3. Provider Adapters

- [ ] 3.1 Implement `OpenAICompatibleProvider` adapter
- [ ] 3.2 Implement `AnthropicProvider` adapter with custom headers
- [ ] 3.3 Implement `GeminiProvider` adapter
- [ ] 3.4 Implement `OpenRouterProvider` adapter
- [ ] 3.5 Implement `LocalEndpointProvider` adapter with None auth support

## 4. Credential Configuration

- [ ] 4.1 Create credential storage module with encrypted file backend
- [ ] 4.2 Implement environment variable override resolution
- [ ] 4.3 Implement credential precedence logic (env > stored > inline)
- [ ] 4.4 Add credential validation endpoint

## 5. Provider Management API

- [ ] 5.1 Implement GET /providers endpoint
- [ ] 5.2 Implement POST /providers endpoint
- [ ] 5.3 Implement PUT /providers/{id} endpoint
- [ ] 5.4 Implement DELETE /providers/{id} endpoint
- [ ] 5.5 Implement POST /providers/{id}/test endpoint

## 6. Integration

- [ ] 6.1 Wire provider adapters into existing llm crate
- [ ] 6.2 Add provider config to CLI arguments
- [ ] 6.3 Add integration tests for each provider adapter
- [ ] 6.4 Update documentation with provider configuration examples
