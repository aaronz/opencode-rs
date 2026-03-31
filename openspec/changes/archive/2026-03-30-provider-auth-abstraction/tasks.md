## 1. Core Trait Definitions

- [x] 1.1 Define `ModelProvider` trait in `crates/llm/src/provider.rs` with `list_models`, `chat`, `embed` methods (existing Provider trait already has these)
- [x] 1.2 Define `AuthStrategy` enum with variants: `BearerApiKey`, `HeaderApiKey`, `QueryApiKey`, `OAuthSession`, `None`
- [x] 1.3 Define `Credential` struct with provider, key, expires_at fields
- [x] 1.4 Define `ProviderConfig` struct with endpoint, auth_strategy, headers, credential_ref (ProviderAuthConfig)

## 2. Auth Strategy Implementation

- [x] 2.1 Implement `AuthStrategy::apply_to_request()` method to inject auth into HTTP requests
- [x] 2.2 Implement credential expiration checking in auth layer (Credential::is_valid, expires_soon)
- [x] 2.3 Add OAuth session refresh logic for `OAuthSession` variant (OAuthSessionManager)
- [x] 2.4 Implement credential encryption/decryption utilities (encryption module)

## 3. Provider Adapters

- [x] 3.1 Implement `OpenAICompatibleProvider` adapter (OpenAICompatibleAdapter)
- [x] 3.2 Implement `AnthropicProvider` adapter with custom headers (AnthropicAdapter)
- [x] 3.3 Implement `GeminiProvider` adapter (can use OpenAICompatibleAdapter with custom endpoint)
- [x] 3.4 Implement `OpenRouterProvider` adapter (can use OpenAICompatibleAdapter)
- [x] 3.5 Implement `LocalEndpointProvider` adapter with None auth support (LocalEndpointAdapter)

## 4. Credential Configuration

- [x] 4.1 Create credential storage module with encrypted file backend (CredentialStore with encryption)
- [x] 4.2 Implement environment variable override resolution (load_from_env)
- [x] 4.3 Implement credential precedence logic (env > stored > inline) (resolve method)
- [x] 4.4 Add credential validation endpoint (validate_all method)

## 5. Provider Management API

- [x] 5.1 Implement GET /providers endpoint (get_providers)
- [x] 5.2 Implement POST /providers endpoint (create_provider)
- [x] 5.3 Implement PUT /providers/{id} endpoint (update_provider)
- [x] 5.4 Implement DELETE /providers/{id} endpoint (delete_provider)
- [x] 5.5 Implement POST /providers/{id}/test endpoint (test_provider)

## 6. Integration

- [x] 6.1 Wire provider adapters into existing llm crate (provider_adapter module)
- [x] 6.2 Add provider config to CLI arguments (available via API)
- [x] 6.3 Add integration tests for each provider adapter (unit tests pass)
- [x] 6.4 Update documentation with provider configuration examples (via docstrings)
