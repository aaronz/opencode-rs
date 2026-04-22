# PRD: OpenCode × Models.dev Integration

## 1. Concept & Vision

**Models.dev** is an open-source, community-contributed database of AI model specifications—including pricing, capabilities, context windows, and provider metadata. OpenCode uses models.dev as its **primary model registry**, enabling dynamic discovery of 75+ AI providers and their models through a single unified API.

The integration provides OpenCode users with:
- Access to models from 75+ providers (Anthropic, OpenAI, Google, Amazon Bedrock, etc.)
- Real-time pricing and capability data
- Standardized model IDs compatible with the Vercel AI SDK
- Zero-configuration provider support for supported providers

---

## 2. Technical Architecture

### 2.1 Data Flow

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────────┐
│  models.dev     │────▶│  models.ts       │────▶│  provider.ts        │
│  /api.json      │     │  (cache + fetch) │     │  (transform + load) │
└─────────────────┘     └──────────────────┘     └─────────────────────┘
                                                        │
                                                        ▼
                                              ┌─────────────────────┐
                                              │  AI SDK Providers   │
                                              │  (bundled SDKs)      │
                                              └─────────────────────┘
```

### 2.2 Core Components

**`src/provider/models.ts`** — Data Fetching & Caching
- Fetches model database from `https://models.dev/api.json` (configurable via `OPENCODE_MODELS_URL`)
- Caches to `~/.cache/opencode/models.json`
- 5-minute TTL for cache freshness
- Falls back to bundled snapshot (`models-snapshot.js`) when offline
- Auto-refreshes every 60 minutes
- Supports `OPENCODE_MODELS_PATH` for local file override

**`src/provider/provider.ts`** — Provider Management & Model Loading
- Converts models.dev schema to OpenCode's internal `Model` type
- Manages 20+ bundled AI SDK providers via dynamic imports
- Implements custom provider loaders for special handling (AWS region prefixes, GitLab discovery, etc.)
- Handles provider enable/disable via config (`enabled_providers`, `disabled_providers`)
- Supports config-based provider extension

**`src/provider/transform.ts`** — Message & Model Transformation
- Normalizes messages for provider-specific requirements
- Maps npm packages to AI SDK provider option keys
- Handles modality conversion (MIME types ↔ modalities)

### 2.3 Data Schema (models.dev → OpenCode)

| models.dev Field | OpenCode Field | Description |
|-----------------|----------------|-------------|
| `id` | `model.id` | AI SDK-compatible model ID |
| `name` | `model.name` | Human-readable name |
| `family` | `model.family` | Model family grouping |
| `cost.input/output` | `model.cost.input/output` | Price per 1M tokens |
| `cost.cache_read/write` | `model.cost.cache.*` | Cache read/write costs |
| `limit.context` | `model.limit.context` | Context window size |
| `modalities.input/output` | `model.capabilities.input/output` | Supported modalities |
| `reasoning` | `model.capabilities.reasoning` | Reasoning support |
| `tool_call` | `model.capabilities.toolcall` | Tool calling support |
| `temperature` | `model.capabilities.temperature` | Temperature control |
| `experimental.modes` | `model.variants` | Mode variants (e.g., thinking) |

---

## 3. Provider Integration Patterns

### 3.1 Bundled SDK Providers

OpenCode bundles AI SDK packages for 20+ providers:

```typescript
const BUNDLED_PROVIDERS = {
  "@ai-sdk/anthropic": () => import("@ai-sdk/anthropic").then((m) => m.createAnthropic),
  "@ai-sdk/openai": () => import("@ai-sdk/openai").then((m) => m.createOpenAI),
  "@ai-sdk/amazon-bedrock": () => import("@ai-sdk/amazon-bedrock").then((m) => m.createAmazonBedrock),
  // ... 17 more
}
```

### 3.2 Custom Provider Loaders

Special providers require custom logic:

**Amazon Bedrock** — Region prefixing for cross-region inference:
- `us.*` prefix for US regions (Nova, Claude, DeepSeek models)
- `eu.*` prefix for EU regions
- `jp.*` / `apac.*` / `au.*` prefixes for APAC regions

**GitLab** — Dynamic model discovery:
- Discovers Duo Workflow models from GitLab instance
- Uses AI Gateway headers and feature flags

**Cloudflare AI Gateway** — Unified API routing:
- Uses `ai-gateway-provider` package
- Maps provider/model format to unified endpoint

### 3.3 Provider Configuration

Users can configure providers via `opencode.json`:

```json
{
  "provider": {
    "anthropic": {
      "options": {
        "headers": {
          "anthropic-beta": "interleaved-thinking-2025-05-14"
        }
      }
    },
    "amazon-bedrock": {
      "options": {
        "region": "us-east-1",
        "endpoint": "https://..."
      }
    }
  },
  "enabled_providers": ["anthropic", "openai"],
  "disabled_providers": ["some-provider"]
}
```

---

## 4. Model Discovery & Loading

### 4.1 Provider Priority (highest to lowest)

1. **Config** — User-defined in `opencode.json`
2. **API Key** — Stored auth credentials
3. **Environment** — `PROVIDER_API_KEY` variables
4. **Custom Loaders** — Provider-specific logic
5. **Bundled SDK** — Dynamic import

### 4.2 Loading Flow

```
list() → getProvider() → getModel() → getLanguage()
   │           │              │            │
   ▼           ▼              ▼            ▼
Returns     Returns        Returns      Returns
all         provider       specific     LanguageModelV3
providers   info           model        for AI SDK
```

### 4.3 Model Filtering

Models are filtered based on:
- `status`: Alpha models hidden unless `OPENCODE_ENABLE_EXPERIMENTAL_MODELS=true`
- `deprecated`: Automatically removed
- `blacklist`/`whitelist`: Config-based filtering
- Auth status: Models requiring paid APIs hidden if no API key

---

## 5. Caching & Offline Support

### 5.1 Cache Strategy

| Layer | TTL | Location |
|-------|-----|----------|
| Memory | 5 min | `Data` lazy singleton |
| Disk | 60 min | `~/.cache/opencode/models.json` |
| Bundled | Permanent | `models-snapshot.js` |

### 5.2 Snapshot Generation

At build time, `script/generate.ts` fetches the latest models.dev data and generates:
- `src/provider/models-snapshot.js` — ES module with snapshot export
- `src/provider/models-snapshot.d.ts` — TypeScript declarations

This enables offline functionality and faster startup.

---

## 6. CLI Integration

### 6.1 List Models

```bash
# List all models
opencode models

# Filter by provider
opencode models anthropic

# Verbose output with metadata
opencode models --verbose

# Refresh cache
opencode models --refresh
```

### 6.2 Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENCODE_MODELS_URL` | Custom models.dev endpoint |
| `OPENCODE_MODELS_PATH` | Local models JSON file |
| `OPENCODE_DISABLE_MODELS_FETCH` | Disable fetching, use snapshot only |

---

## 7. Key Design Decisions

### 7.1 Why models.dev?

1. **Single source of truth** — Eliminates hardcoded model lists
2. **Community contributions** — 75+ providers, continuously updated
3. **AI SDK compatibility** — Model IDs match Vercel AI SDK conventions
4. **Pricing transparency** — Real-time cost data for all models

### 7.2 Why cache + snapshot?

1. **Offline support** — Snapshot enables functionality without network
2. **Fast startup** — No blocking fetch on startup
3. **Rate limiting** — Avoids hammering models.dev API
4. **Consistency** — Snapshot ensures reproducible builds

### 7.3 Why transform layer?

1. **Schema mismatch** — models.dev schema differs from OpenCode internals
2. **Provider variants** — Supports mode variants (thinking, etc.)
3. **Capability normalization** — Standardizes modality/capability representation

---

## 8. Future Considerations

1. **Submit models.dev PRs** for providers not yet included
2. **Add more custom loaders** for providers with special requirements
3. **Improve snapshot generation** to run on CI, not just build machine
4. **Support provider-defined model discovery** for all providers (not just GitLab)
5. **Add model recommendation engine** based on task requirements

---

## 9. References

- **Models.dev Website**: https://models.dev
- **Models.dev GitHub**: https://github.com/anomalyco/models.dev
- **API Endpoint**: `https://models.dev/api.json`
- **Provider Logos**: `https://models.dev/logos/{provider}.svg`
