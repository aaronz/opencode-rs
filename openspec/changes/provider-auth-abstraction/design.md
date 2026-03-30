## Context

OpenCode-RS is being built as a Rust-native AI coding agent platform. The existing implementation has `crates/llm/src/auth.rs`, `provider.rs`, and `openai_browser_auth.rs` demonstrating early auth patterns. However, these are tightly coupled to specific providers. A unified abstraction is needed to:

1. Support multiple LLM providers (OpenAI, Anthropic, Gemini, OpenRouter, local endpoints)
2. Decouple authentication strategies from provider implementations
3. Enable enterprise gateway integration with custom auth headers
4. Support credential configuration with proper security handling

## Goals / Non-Goals

**Goals:**
- Define `ModelProvider` trait for protocol-agnostic LLM interactions
- Define `AuthStrategy` enum for flexible authentication
- Create provider adapters for v1 target providers
- Implement credential configuration with secure storage
- Support runtime provider management (add/list/remove)

**Non-Goals:**
- OAuth/browser-based login (defer to v1.5+)
- Provider-specific advanced features beyond basic chat/embed
- Multi-provider failover/routing logic (future consideration)
- Telemetry/metrics for provider usage

## Decisions

### 1. AuthStrategy Enum vs Trait
**Decision**: Use enum with provider-specific data variants
**Rationale**: Simpler than trait objects for v1, exhaustiveness ensures compile-time coverage of all auth types
**Alternatives considered**: Trait-based auth (over-engineered for fixed set of auth strategies)

### 2. Credential Storage
**Decision**: Encrypted at-rest storage with environment variable override
**Rationale**: Matches enterprise requirements; env vars for CI/CD; encrypted file for interactive use
**Alternatives considered**: Database-backed (adds deps), pure env vars (poor UX for interactive)

### 3. Provider Protocol vs Auth Separation
**Decision**: Explicit separation—Provider knows which AuthStrategy to use by default, but user can override
**Rationale**: Most providers have canonical auth, but enterprise gateways need flexibility
**Alternatives considered**: Single enum combining provider+auth (too rigid)

### 4. Auth Error Handling
**Decision**: Domain-specific error types with provider context
**Rationale**: User needs to know if auth failed due to missing creds vs invalid creds vs provider outage

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Provider API changes break adapter | Version adapter interfaces; provider impls are isolated |
| Credential exposure in logs | Sanitize all logging of auth data; use secret masking |
| Memory exposure of API keys | Zeroize key memory after use; avoid String for sensitive data |
| Blocking auth on network calls | Async auth refresh with timeout; fail-fast on invalid creds |

## Migration Plan

1. **Phase 1**: Define core traits (`ModelProvider`, `AuthStrategy`, `CredentialConfig`)
2. **Phase 2**: Implement v1 providers (OpenAI-compatible, Anthropic, Gemini, OpenRouter, Local)
3. **Phase 3**: Add credential storage with encryption
4. **Phase 4**: Wire into existing llm crate integration
5. **Phase 5**: Add provider management API endpoints

No rollback needed—greenfield implementation adds new code without modifying existing paths until integration.

## Open Questions

1. **Q**: Should provider adapters be plugins or compile-time?
   - **A**: Compile-time for v1; plugin system deferred to future

2. **Q**: How to handle provider rate limits?
   - **A**: Not in scope for auth abstraction; future middleware layer

3. **Q**: Support for vision/multimodal models?
   - **A**: Handled per-provider in adapter implementation, not at abstraction layer
