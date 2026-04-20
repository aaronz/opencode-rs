# PRD: provider Module

## Module Overview

**Module Name:** `provider`
**Type:** Core
**Source:** `/packages/opencode/src/provider/`

## Purpose

AI model provider abstraction layer, integrating with models.dev for dynamic model discovery. Provides unified interface to 75+ AI providers with pricing, capabilities, and API configuration.

## Functionality

### Core Features

1. **Dynamic Model Discovery**
   - Fetches model database from `https://models.dev/api.json`
   - Caches to `~/.cache/opencode/models.json`
   - Falls back to bundled snapshot when offline
   - Auto-refresh every 60 minutes

2. **Provider Management**
   - Manages 20+ bundled AI SDK providers
   - Dynamic provider loading via dynamic imports
   - Provider enable/disable via config

3. **Custom Provider Loaders**

   **Amazon Bedrock** - Region prefixing:
   - `us.*` for US regions
   - `eu.*` for EU regions
   - `jp.*`, `apac.*`, `au.*` for APAC regions

   **GitLab** - Dynamic model discovery:
   - Discovers Duo Workflow models
   - Uses AI Gateway headers

   **Cloudflare AI Gateway** - Unified API routing

4. **Model Transformation**
   - Converts models.dev schema to OpenCode internal types
   - Handles variant modes (thinking, etc.)
   - Normalizes capabilities and modalities

### Supported Providers (75+)

| Provider | SDK Package | Features |
|----------|-------------|----------|
| Anthropic | `@ai-sdk/anthropic` | Claude models |
| OpenAI | `@ai-sdk/openai` | GPT models |
| Google | `@ai-sdk/google` | Gemini models |
| Amazon Bedrock | `@ai-sdk/amazon-bedrock` | AWS models |
| Azure | `@ai-sdk/azure` | Azure OpenAI |
| GitHub Copilot | `@ai-sdk/github-copilot` | Copilot models |
| Groq | `@ai-sdk/groq` | Fast inference |
| DeepInfra | `@ai-sdk/deepinfra` | Various models |
| Cerebras | `@ai-sdk/cerebras` | Fast inference |
| Cohere | `@ai-sdk/cohere` | Command models |
| TogetherAI | `@ai-sdk/togetherai` | Various models |
| Perplexity | `@ai-sdk/perplexity` | Online models |
| XAI | `@ai-sdk/xai` | Grok models |
| Mistral | `@ai-sdk/mistral` | Mistral models |
| Cloudflare Workers AI | Custom | CF models |
| Cloudflare AI Gateway | `ai-gateway-provider` | Unified routing |
| GitLab | `gitlab-ai-provider` | Duo models |
| OpenRouter | `@openrouter/ai-sdk-provider` | Router |
| Vercel | `@ai-sdk/vercel` | Vercel AI |
| NVIDIA | `@ai-sdk/nvidia` | NIM models |
| And many more... | | |

### API Surface

```typescript
interface ProviderService {
  list(): Promise<Record<ProviderID, ProviderInfo>>
  getProvider(id: ProviderID): Promise<ProviderInfo>
  getModel(provider: ProviderID, model: ModelID): Promise<Model>
  getLanguage(model: Model): Promise<LanguageModelV3>
  closest(provider: ProviderID, query: string[]): Promise<{provider, model} | undefined>
  getSmallModel(provider: ProviderID): Promise<Model | undefined>
  defaultModel(): Promise<{provider: ProviderID, model: ModelID}>
}

interface ProviderInfo {
  id: ProviderID
  name: string
  source: 'env' | 'config' | 'custom' | 'api'
  env: string[]  // Required env vars
  options: Record<string, any>
  models: Record<ModelID, Model>
}

interface Model {
  id: ModelID
  providerID: ProviderID
  name: string
  family?: string
  api: { id: string, url: string, npm: string }
  status: 'alpha' | 'beta' | 'deprecated' | 'active'
  cost: { input: number, output: number, cache: { read: number, write: number } }
  limit: { context: number, input?: number, output: number }
  capabilities: {
    temperature: boolean
    reasoning: boolean
    attachment: boolean
    toolcall: boolean
    input: { text: boolean, audio: boolean, image: boolean, video: boolean, pdf: boolean }
    output: { text: boolean, audio: boolean, image: boolean, video: boolean, pdf: boolean }
    interleaved: boolean | { field: 'reasoning_content' | 'reasoning_details' }
  }
  release_date: string
  variants: Record<string, any>
}
```

### Key Files

| File | Purpose |
|------|---------|
| `provider.ts` | Main provider service (64KB) |
| `models.ts` | models.dev API client and caching |
| `transform.ts` | Message/model transformation |
| `schema.ts` | Provider and model schemas |
| `auth.ts` | Provider authentication |
| `error.ts` | Provider errors |

### Data Schema (models.dev → OpenCode)

| models.dev Field | OpenCode Field |
|-----------------|----------------|
| `id` | `model.id` |
| `name` | `model.name` |
| `cost.input/output` | `model.cost.input/output` |
| `cost.cache_read/write` | `model.cost.cache.*` |
| `limit.context` | `model.limit.context` |
| `modalities` | `model.capabilities.*` |
| `reasoning` | `model.capabilities.reasoning` |
| `tool_call` | `model.capabilities.toolcall` |
| `experimental.modes` | `model.variants` |

### Cache Strategy

| Layer | TTL | Location |
|-------|-----|----------|
| Memory | 5 min | Lazy singleton |
| Disk | 60 min | `~/.cache/opencode/models.json` |
| Bundled | Permanent | `models-snapshot.js` |

### Configuration

```json
{
  "provider": {
    "anthropic": {
      "options": {
        "headers": { "anthropic-beta": "..." }
      }
    },
    "amazon-bedrock": {
      "options": {
        "region": "us-east-1"
      }
    }
  },
  "enabled_providers": ["anthropic", "openai"],
  "disabled_providers": []
}
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENCODE_MODELS_URL` | Custom models.dev endpoint |
| `OPENCODE_MODELS_PATH` | Local models JSON file |
| `OPENCODE_DISABLE_MODELS_FETCH` | Use snapshot only |

## Dependencies

- `ai` - Vercel AI SDK
- `@ai-sdk/*` - Provider SDKs

## Acceptance Criteria

1. All 75+ providers available
2. Model data correctly fetched and cached
3. Provider selection works with priority (config > API > env > custom)
4. Region prefixing for Bedrock works correctly
5. Offline mode works with bundled snapshot

## Rust Implementation Guidance

The Rust equivalent should:
- Use `reqwest` for HTTP requests
- Use `rusqlite` for caching
- Use `serde` for JSON parsing
- Use `tokio` for async operations
- Consider using `async-trait` for trait methods
- Implement proper error types
