# PRD: auth Module

## Module Overview

**Module Name:** `auth`
**Type:** Integration
**Source:** `/packages/opencode/src/auth/`

## Purpose

Authentication and credential management for API providers. Manages API keys, OAuth tokens, and secure credential storage.

## Functionality

### Core Features

1. **Auth Methods**

   | Method | Description |
   |--------|-------------|
   | `api` | API key authentication |
   | `oauth` | OAuth 2.0 flow |

2. **Auth Storage**
   - Encrypted credential storage
   - Provider-specific auth
   - Token refresh handling

3. **Auth Interface**

   ```typescript
   interface Auth {
     provider: string
     type: 'api' | 'oauth'
     key?: string
     access?: string
     refresh?: string
     metadata?: Record<string, any>
     expiresAt?: number
   }

   interface AuthService {
     get(provider: string): Promise<Auth | undefined>
     set(provider: string, auth: Auth): Promise<void>
     delete(provider: string): Promise<void>
     all(): Promise<Record<string, Auth>>
   }
   ```

### CLI Commands

```bash
# Authenticate with a provider
opencode auth openai
opencode auth anthropic
opencode auth github
```

### Supported Providers

- OpenAI
- Anthropic
- GitHub Copilot
- GitLab
- And more...

## Dependencies

- Storage module for persistence
- Crypto for encryption

## Acceptance Criteria

1. Auth credentials are stored securely
2. OAuth flow works correctly
3. API keys are properly retrieved
4. Token refresh is handled
5. Auth can be managed via CLI

## Rust Implementation Guidance

The Rust equivalent should:
- Use `rusqlite` for storage
- Use `keyring` or `secret_service` for secure storage
- Implement OAuth 2.0 flow
- Use proper error handling
