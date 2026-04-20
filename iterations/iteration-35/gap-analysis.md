# Gap Analysis: OpenCode × Models.dev Integration (PRD #35)

**Project:** opencode-rs (Rust Implementation)
**Analysis Date:** 2026-04-20
**PRD Reference:** models.dev integration PRD (inline in task)
**Iteration:** 35
**Analysis Mode:** Current Implementation vs. PRD Requirements

---

## 1. Executive Summary

The models.dev integration is **partially implemented**. Core fetching, caching, and catalog types exist, but several key PRD requirements are missing.

| Category | PRD Requirement | Current Status | Gap |
|----------|-----------------|---------------|-----|
| Data Fetching | Fetch from models.dev/api.json | ✅ Done | None |
| Caching | 5-min memory TTL, 60-min disk cache | ✅ Done | None |
| Catalog Types | ProviderCatalog, ModelDescriptor | ✅ Done | None |
| Config Overrides | Per-provider config via opencode.json | ✅ Done | None |
| ModelRegistry | Hardcoded + catalog population | ⚠️ Partial | Dual model sources |
| Bundled Snapshot | Offline fallback via models-snapshot.js | ❌ Not done | **P0 Gap** |
| Snapshot Generation | Build-time script/generate.ts | ❌ Not done | **P0 Gap** |
| CLI Refresh | `opencode models --refresh` | ❌ Not done | **P2 Gap** |
| Bundled SDK Providers | Dynamic imports for @ai-sdk/* packages | ❌ Not done | **P1 Gap** |
| Custom Provider Loaders | Bedrock regions, GitLab discovery | ❌ Not done | **P1 Gap** |
| Alpha Model Filter | OPENCODE_ENABLE_EXPERIMENTAL_MODELS | ❌ Not done | **P2 Gap** |

---

## 2. Gap Analysis by Feature

### 2.1 Bundled Snapshot Fallback (P0 - OPEN)

**PRD Requirement:**
> "Falls back to bundled snapshot (`models-snapshot.js`) when offline"

**Current Implementation:**
- `fetcher.rs` fetches from `https://models.dev/api.json`
- Falls back to empty `ProviderCatalog` on network failure:
```rust
Err(_) => self
    .read_file_cache()
    .await
    .unwrap_or_else(|_| ProviderCatalog {
        providers: Default::default(),
        fetched_at: Utc::now(),
        source: CatalogSource::ModelsDev,
    }),
```

**Gap:** No bundled snapshot for offline fallback.

---

### 2.2 Snapshot Generation Script (P0 - OPEN)

**PRD Requirement:**
> "At build time, `script/generate.ts` fetches the latest models.dev data and generates: `src/provider/models-snapshot.js`"

**Current State:**
- No `script/generate.ts` exists
- No `models-snapshot.js` exists
- No TypeScript declarations (`models-snapshot.d.ts`)

**Gap:** Build-time snapshot generation not implemented.

---

### 2.3 Bundled SDK Provider Dynamic Imports (P1 - OPEN)

**PRD Requirement:**
```typescript
const BUNDLED_PROVIDERS = {
  "@ai-sdk/anthropic": () => import("@ai-sdk/anthropic").then((m) => m.createAnthropic),
  "@ai-sdk/openai": () => import("@ai-sdk/openai").then((m) => m.createOpenAI),
  "@ai-sdk/amazon-bedrock": () => import("@ai-sdk/amazon-bedrock").then((m) => m.createAmazonBedrock),
  // ... 17 more
}
```

**Current Implementation:**
- Individual provider modules in `crates/llm/src/` (openai.rs, anthropic.rs, etc.)
- No dynamic imports of @ai-sdk/* packages
- Static linking instead of dynamic provider loading

**Gap:** No dynamic provider loading via AI SDK packages.

---

### 2.4 Custom Provider Loaders (P1 - OPEN)

**PRD Requirement:**
> "Special providers require custom logic: Amazon Bedrock region prefixing, GitLab Duo Workflow discovery, Cloudflare AI Gateway routing"

**Amazon Bedrock:**
- `us.*` prefix for US regions (Nova, Claude, DeepSeek models)
- `eu.*` prefix for EU regions
- `jp.*` / `apac.*` / `au.*` prefixes for APAC regions

**GitLab:**
- Discovers Duo Workflow models from GitLab instance
- Uses AI Gateway headers and feature flags

**Cloudflare AI Gateway:**
- Uses `ai-gateway-provider` package
- Maps provider/model format to unified endpoint

**Current State:**
- Generic Bedrock implementation in `crates/llm/src/bedrock.rs`
- No region prefix handling
- No GitLab Duo Workflow discovery
- No Cloudflare AI Gateway support

**Gap:** Provider-specific custom loaders not implemented.

---

### 2.5 CLI Refresh Option (P2 - OPEN)

**PRD Requirement:**
> `opencode models --refresh`

**Current Implementation (`crates/cli/src/cmd/models.rs`):**
- `--provider`, `--json`, `--visibility`, `--switch` supported
- `--refresh` NOT implemented

**Gap:** No cache refresh mechanism via CLI.

---

### 2.6 Alpha Model Filtering (P2 - OPEN)

**PRD Requirement:**
> "status: Alpha models hidden unless `OPENCODE_ENABLE_EXPERIMENTAL_MODELS=true`"

**Current Implementation:**
- `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag exists in `flag.rs`
- But no filtering logic in `ModelRegistry` or catalog fetcher

**Gap:** Alpha model filtering not implemented.

---

### 2.7 Provider Headers Configuration (P2 - OPEN)

**PRD Requirement:**
```json
{
  "provider": {
    "anthropic": {
      "options": {
        "headers": {
          "anthropic-beta": "interleaved-thinking-2025-05-14"
        }
      }
    }
  }
}
```

**Current Implementation:**
- `ProviderOptions` in `opencode_config` has `base_url`
- No `headers` field in `ProviderOptions`

**Gap:** Per-provider custom headers not supported.

---

## 3. P0/P1/P2 Issue Classification

### P0 - Critical Blockers

| # | Issue | Module | Impact | Fix Suggestion |
|---|-------|--------|--------|----------------|
| P0-1 | **No bundled snapshot fallback** | `llm/src/catalog/` | App fails silently when offline with no models | Create `models-snapshot.rs` with embedded catalog data |
| P0-2 | **No snapshot generation script** | `script/` | Cannot generate offline snapshot at build time | Create `script/generate.sh` to fetch and convert models.dev data |

### P1 - High Priority

| # | Issue | Module | Impact | Fix Suggestion |
|---|-------|--------|--------|----------------|
| P1-1 | **No dynamic provider loading** | `llm/src/` | Cannot dynamically load @ai-sdk providers | Implement `BUNDLED_PROVIDERS` registry with dynamic imports |
| P1-2 | **No Amazon Bedrock region prefixing** | `llm/src/bedrock.rs` | Cross-region inference not supported | Add region prefix handling in bedrock provider |
| P1-3 | **No GitLab Duo Workflow discovery** | `llm/src/` | Cannot discover GitLab AI models | Implement GitLab model discovery |
| P1-4 | **No Cloudflare AI Gateway support** | `llm/src/` | Cannot route via AI Gateway | Add AI Gateway provider |

### P2 - Medium Priority

| # | Issue | Module | Impact | Fix Suggestion |
|---|-------|--------|--------|----------------|
| P2-1 | **No CLI `--refresh`** | `cli/src/cmd/models.rs` | Cannot refresh model cache from CLI | Add `--refresh` flag to force fetch |
| P2-2 | **No alpha model filtering** | `llm/src/models.rs` | Alpha models always visible | Filter alpha models when `OPENCODE_ENABLE_EXPERIMENTAL_MODELS=false` |
| P2-3 | **No per-provider headers** | `opencode_config/` | Cannot set custom provider headers | Add `headers` field to `ProviderOptions` |

---

## 4. Technical Debt

| TD # | Issue | File | Severity | Description |
|------|-------|------|----------|-------------|
| TD-1 | Dual model sources | `models.rs` | High | Hardcoded models + catalog population creates potential inconsistency |
| TD-2 | `transform.rs` misnamed | `llm/src/transform.rs` | Low | Module does message transformation, not model/provider transformation |
| TD-3 | No `models-dev` prefix in provider IDs | `fetcher.rs` | Medium | Provider IDs from models.dev may conflict with existing provider IDs |

---

## 5. Implementation Progress Summary

### Completed

| Feature | Status | Notes |
|---------|--------|-------|
| models.dev API types | ✅ Done | `models_dev.rs` |
| Catalog fetcher | ✅ Done | `fetcher.rs` with 5-min TTL |
| Disk caching | ✅ Done | `~/.cache/opencode/models.json` |
| Catalog types | ✅ Done | `ProviderCatalog`, `ModelDescriptor`, etc. |
| Catalog merger | ✅ Done | Config overrides, local providers, enabled/disabled filters |
| Provider config integration | ✅ Done | `CatalogMerger::with_config_overrides()` |
| Flag support | ✅ Done | `OPENCODE_MODELS_URL`, `OPENCODE_MODELS_PATH`, `OPENCODE_DISABLE_MODELS_FETCH` |
| CLI models command | ✅ Done | List, switch, visibility commands |

### Open Issues

| Issue | Priority | Status |
|-------|----------|--------|
| Bundled snapshot fallback | P0 | Not implemented |
| Snapshot generation script | P0 | Not implemented |
| Dynamic provider loading | P1 | Not implemented |
| Amazon Bedrock region prefixing | P1 | Not implemented |
| GitLab Duo discovery | P1 | Not implemented |
| Cloudflare AI Gateway | P1 | Not implemented |
| CLI `--refresh` | P2 | Not implemented |
| Alpha model filtering | P2 | Not implemented |
| Per-provider headers | P2 | Not implemented |

---

## 6. Gap Summary Table

| Gap Item | Severity | Module | File:Line | 修复建议 |
|----------|----------|--------|-----------|---------|
| No bundled snapshot fallback | **P0** | LLM Catalog | `fetcher.rs:82-89` | Create `models-snapshot.rs` with embedded data |
| No snapshot generation script | **P0** | Scripts | N/A | Create `script/generate.sh` |
| No dynamic provider loading | P1 | LLM | N/A | Implement `BUNDLED_PROVIDERS` registry |
| No Bedrock region prefixing | P1 | LLM/Bedrock | `bedrock.rs` | Add region prefix handling |
| No GitLab discovery | P1 | LLM | N/A | Implement GitLab AI model discovery |
| No AI Gateway support | P1 | LLM | N/A | Add Cloudflare AI Gateway provider |
| No CLI `--refresh` | P2 | CLI | `models.rs:11-24` | Add refresh flag |
| No alpha model filtering | P2 | LLM | `models.rs` | Filter by `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` |
| No per-provider headers | P2 | Config | `opencode_config/` | Add headers field to ProviderOptions |
| Dual model sources | TD | LLM | `models.rs` | Consolidate to single catalog source |

---

## 7. Recommended Priority for Iteration 35

### Immediate (P0 - Required)

1. **P0-1: Create Bundled Snapshot**
   - Create `crates/llm/src/catalog/snapshot.rs`
   - Embed minimal catalog data for offline fallback
   - Modify `fetcher.rs` to use snapshot when network unavailable

2. **P0-2: Create Snapshot Generation Script**
   - Create `scripts/generate-snapshot.sh`
   - Fetch from `https://models.dev/api.json`
   - Generate Rust code for embedding

### Short-term (P1)

3. **P1-1: Implement Dynamic Provider Loading**
   - Define `BUNDLED_PROVIDERS` map with dynamic imports
   - Create provider factory functions

4. **P1-2: Add Amazon Bedrock Region Prefix Handling**
   - Detect region from model ID prefix
   - Route to appropriate Bedrock endpoint

### Medium-term (P2)

5. **P2-1: Add CLI `--refresh` Flag**
   - Implement cache refresh in `models.rs`
   - Add `force_refresh: bool` to fetcher

6. **P2-2: Implement Alpha Model Filtering**
   - Check `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag
   - Filter alpha models when flag is false

7. **P2-3: Add Per-Provider Headers**
   - Extend `ProviderOptions` with `headers: HashMap<String, String>`
   - Pass headers in HTTP requests

---

## 8. Code References

| Component | File | Key Lines |
|-----------|------|-----------|
| Catalog Fetcher | `crates/llm/src/catalog/fetcher.rs` | 1-268 |
| Catalog Types | `crates/llm/src/catalog/types.rs` | 1-76 |
| models.dev Types | `crates/llm/src/catalog/models_dev.rs` | 1-80 |
| Catalog Merger | `crates/llm/src/catalog/merge.rs` | 1-177 |
| ModelRegistry | `crates/llm/src/models.rs` | 1-1233 |
| CLI Models | `crates/cli/src/cmd/models.rs` | 1-375 |
| Flags | `crates/core/src/flag.rs` | 146-155, 346-347 |
| Transform | `crates/llm/src/transform.rs` | 1-270 |

---

## 9. Test Coverage Status

| Component | Unit Tests | Integration Tests |
|-----------|------------|-------------------|
| Catalog Fetcher | 11 ✅ | 0 ❌ |
| Catalog Types | 0 ✅ | 0 ❌ |
| models_dev Types | 0 ✅ | 0 ❌ |
| Catalog Merger | 13 ✅ | 0 ❌ |
| ModelRegistry | 9 ✅ | 0 ❌ |
| CLI Models | 13 ✅ | 0 ❌ |

---

## 10. PRD Schema Mapping (models.dev → OpenCode)

| models.dev Field | OpenCode Field | Status |
|-----------------|----------------|--------|
| `id` | `model.id` | ✅ Done |
| `name` | `model.display_name` | ✅ Done |
| `family` | `model.family` | ✅ Done |
| `cost.input/output` | `model.cost.input/output` | ✅ Done |
| `cost.cache_read/write` | `model.cost.cache.*` | ✅ Done |
| `limit.context` | `model.limits.context` | ✅ Done |
| `modalities.input/output` | `model.capabilities.input/output_modalities` | ✅ Done |
| `reasoning` | `model.capabilities.reasoning` | ✅ Done |
| `tool_call` | `model.capabilities.tool_call` | ✅ Done |
| `temperature` | `model.capabilities.temperature` | ✅ Done |
| `experimental.modes` | `model.variants` | ❌ Not done |

---

*Report Generated: 2026-04-20*
*Analysis Method: Direct codebase inspection*
*PRD Reference: models.dev integration PRD*
*Previous Analysis: iteration-34/gap-analysis.md (different PRD)*