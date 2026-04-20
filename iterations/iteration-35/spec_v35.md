# opencode-rs Specification v35

**Project:** opencode-rs (Rust Implementation)
**Version:** 35
**Date:** 2026-04-20
**Based on:** PRD #35 (models.dev Integration) + Iteration 35 Gap Analysis

---

## 1. Overview

opencode-rs is a Rust reimplementation of the original opencode (TypeScript/Bun v1.4.5) AI coding agent. This document defines the specification for opencode-rs, tracking feature requirements, implementation status, and gap analysis against the models.dev integration PRD.

---

## 2. Command Specification

### 2.1 Core Commands

| Command | Status | Implementation | Notes |
|---------|--------|----------------|-------|
| `tui` | ✅ Implemented | Default TUI interface | `[project]` alias |
| `attach` | ✅ Implemented | Server attachment | URL as arg or interactive |
| `run` | ✅ Implemented | Agent execution | |
| `debug` | ✅ Implemented | Debug mode | |
| `agent` | ✅ Implemented | Agent selection | |
| `upgrade` | ✅ Implemented | Auto-upgrade | |
| `uninstall` | ✅ Implemented | Removal | |
| `serve` | ✅ Implemented | HTTP server | |
| `web` | ✅ Implemented | Web interface | |
| `stats` | ✅ Implemented | Statistics | |
| `session` | ✅ Implemented | Session management | |
| `db` | ✅ Implemented | Database CLI | |
| `export` | ✅ Implemented | Session portability | |
| `import` | ✅ Implemented | Session portability | |

### 2.2 Provider/Model Commands

| Command | Status | Implementation | Notes |
|---------|--------|----------------|-------|
| `providers` | ✅ Implemented | `crates/cli/src/cmd/providers.rs` | **FR-001**: Dynamic provider registry |
| `models` | ✅ Implemented | `crates/llm/src/models.rs` + catalog | **FR-018**: models.dev integration |
| `models --refresh` | ❌ Not done | `crates/cli/src/cmd/models.rs` | **FR-026**: CLI refresh flag |
| `acp` | ✅ Implemented | ACP protocol | |
| `mcp` | ✅ Implemented | MCP client/server | |

### 2.3 Missing Commands

| Command | Original | Status | Priority | Implementation |
|---------|----------|--------|----------|----------------|
| `completion` | ✅ | ❌ Missing | P1 | **FR-003**: Shell completion generation |
| `plugin` | ✅ | ❌ Missing CLI | P1 | **FR-004**: Plugin CLI commands |

### 2.4 Incomplete Commands

| Command | Status | Issue | Priority | Implementation |
|---------|--------|-------|----------|----------------|
| `pr` | ✅ Implemented | PR fetch/checkout/list | P1 | **FR-005**: Complete |
| `github` | ⚠️ Renamed | `git-hub` exists, incomplete | P2 | **FR-006**: GitHub integration |

### 2.5 Rust-Exclusive Commands

| Command | Status | Notes |
|---------|--------|-------|
| `account` | ✅ Implemented | Account management |
| `config` | ✅ Implemented | Config management |
| `bash` | ✅ Implemented | Shell integration |
| `terminal` | ✅ Implemented | Terminal integration |
| `git-hub` | ✅ Implemented | GitHub workflows |
| `git-lab` | ✅ Implemented | GitLab integration |
| `generate` | ✅ Implemented | Code generation |
| `thread` | ✅ Implemented | Thread management |
| `workspace-serve` | ✅ Implemented | Workspace server |
| `palette` | ✅ Implemented | Command palette |
| `shortcuts` | ✅ Implemented | Keyboard shortcuts |
| `workspace` | ✅ Implemented | Workspace management |
| `ui` | ✅ Implemented | UI controls |
| `project` | ✅ Implemented | Project management |
| `files` | ✅ Implemented | File operations |
| `prompt` | ✅ Implemented | Prompt management |
| `quick` | ✅ Implemented | Quick actions |
| `desktop` | ✅ Implemented | Desktop mode |

---

## 3. models.dev Integration Overview

### 3.1 Technical Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────────┐
│  models.dev     │────▶│  catalog/       │────▶│  provider.rs        │
│  /api.json      │     │  (cache + fetch)│     │  (transform + load) │
└─────────────────┘     └──────────────────┘     └─────────────────────┘
                                                        │
                                                        ▼
                                              ┌─────────────────────┐
                                              │  AI SDK Providers   │
                                              │  (bundled SDKs)      │
                                              └─────────────────────┘
```

### 3.2 Core Components

**`crates/llm/src/catalog/fetcher.rs`** — Data Fetching & Caching
- Fetches model database from `https://models.dev/api.json` (configurable via `OPENCODE_MODELS_URL`)
- Caches to `~/.cache/opencode/models.json`
- 5-minute TTL for cache freshness
- Falls back to bundled snapshot (`models-snapshot.rs`) when offline
- Auto-refreshes every 60 minutes
- Supports `OPENCODE_MODELS_PATH` for local file override

**`crates/llm/src/catalog/`** — Provider Management & Model Loading
- Converts models.dev schema to OpenCode's internal `Model` type
- Manages 20+ bundled AI SDK providers via dynamic imports
- Implements custom provider loaders for special handling (AWS region prefixes, GitLab discovery, etc.)
- Handles provider enable/disable via config (`enabled_providers`, `disabled_providers`)
- Supports config-based provider extension

**`crates/llm/src/transform.rs`** — Message & Model Transformation
- Normalizes messages for provider-specific requirements
- Maps npm packages to AI SDK provider option keys
- Handles modality conversion (MIME types ↔ modalities)

---

## 4. Feature Requirements

### FR-001: Dynamic Provider Registry

**Priority:** P0 (Critical)
**Module:** `crates/cli/src/cmd/providers.rs:94`
**Status:** ✅ Completed

**Specification:**
- Remove hardcoded provider list
- Read providers dynamically from provider registry
- Display all 18 providers available in TUI dialog

**Acceptance Criteria:**
- [x] `opencode-rs providers` lists all registered providers
- [x] Provider list matches ConnectProviderDialog options
- [x] No duplicate or missing providers

---

### FR-002: Expanded Model Catalog

**Priority:** P1 (High)
**Module:** `crates/llm/src/models.rs`
**Issue:** Only ~17 models vs 89 in original (81% gap)
**Status:** ⚠️ Superseded by FR-018 (models.dev integration)

**Specification:**
- Expand model registry to 50+ models minimum via models.dev catalog
- Add missing provider models from models.dev database

**Acceptance Criteria:**
- [x] Model catalog expanded via models.dev integration
- [x] All original provider models available

---

### FR-018: models.dev Integration

**Priority:** P0 (Critical)
**Module:** `crates/llm/src/catalog/`
**Status:** ⚠️ Partial Implementation

**Specification:**
- Fetch model database from `https://models.dev/api.json`
- Cache with 5-minute memory TTL, 60-minute disk cache
- Support `OPENCODE_MODELS_URL` and `OPENCODE_MODELS_PATH` overrides
- Convert models.dev schema to OpenCode `Model` type

**Acceptance Criteria:**
- [x] Fetcher fetches from models.dev API
- [x] Memory cache with 5-min TTL
- [x] Disk cache with 60-min TTL
- [x] Flag support for URL/PATH overrides
- [x] Catalog types (ProviderCatalog, ModelDescriptor)
- [x] Config overrides integration

---

### FR-019: Bundled Snapshot Fallback

**Priority:** P0 (Critical)
**Module:** `crates/llm/src/catalog/snapshot.rs`
**Status:** ❌ Not Implemented

**Specification:**
- Create embedded snapshot of models.dev data for offline fallback
- When network unavailable and disk cache stale/missing, use bundled snapshot
- Snapshot contains minimal catalog data to enable basic functionality

**Implementation:**
```
crates/llm/src/catalog/snapshot.rs
├── BUNDLED_SNAPSHOT constant
├── SnapshotCatalog struct
├── get_snapshot() -> SnapshotCatalog
└── is_snapshot_available() -> bool
```

**Acceptance Criteria:**
- [ ] `snapshot.rs` module created
- [ ] Bundled snapshot data embedded in binary
- [ ] Fetcher falls back to snapshot when offline
- [ ] App functions with snapshot when no network

---

### FR-020: Snapshot Generation Script

**Priority:** P0 (Critical)
**Module:** `scripts/generate-snapshot.sh`
**Status:** ❌ Not Implemented

**Specification:**
- Create build-time script to fetch and convert models.dev data
- Generates Rust code for embedding in binary
- Run as part of build process or CI

**Implementation:**
```bash
scripts/generate-snapshot.sh
├── Fetch from https://models.dev/api.json
├── Transform to Rust struct format
├── Write to crates/llm/src/catalog/snapshot_data.rs
└── Update timestamp
```

**Acceptance Criteria:**
- [ ] `scripts/generate-snapshot.sh` created
- [ ] Fetches latest models.dev data
- [ ] Generates embeddable Rust code
- [ ] Can be run during build/CI

---

### FR-021: Dynamic Provider Loading

**Priority:** P1 (High)
**Module:** `crates/llm/src/`
**Status:** ❌ Not Implemented

**Specification:**
- Implement `BUNDLED_PROVIDERS` registry with dynamic imports
- Support 20+ AI SDK provider packages:
  - `@ai-sdk/anthropic`
  - `@ai-sdk/openai`
  - `@ai-sdk/amazon-bedrock`
  - And 17+ more

**Implementation:**
```rust
const BUNDLED_PROVIDERS: &[(&str, fn() -> ProviderFactory)] = &[
    ("anthropic", || async { import("@ai-sdk/anthropic") }),
    ("openai", || async { import("@ai-sdk/openai") }),
    // ... 17 more
];
```

**Acceptance Criteria:**
- [ ] `BUNDLED_PROVIDERS` constant defined
- [ ] Dynamic import implementation for each provider
- [ ] Provider factory functions
- [ ] Lazy loading on first use

---

### FR-022: Amazon Bedrock Region Prefix Handling

**Priority:** P1 (High)
**Module:** `crates/llm/src/bedrock.rs`
**Status:** ❌ Not Implemented

**Specification:**
- Implement region prefix detection for Bedrock models
- Prefix mappings:
  - `us.*` → US regions (Nova, Claude, DeepSeek)
  - `eu.*` → EU regions
  - `jp.*`, `apac.*`, `au.*` → APAC regions

**Implementation:**
```rust
fn get_region_prefix(model_id: &str) -> Option<&str> {
    // e.g., "us.amazon.nova-pro" -> Some("us")
    // e.g., "eu.claude-3-sonnet" -> Some("eu")
}
```

**Acceptance Criteria:**
- [ ] Region prefix detection function
- [ ] Endpoint routing based on region
- [ ] US/EU/APAC region support
- [ ] Fallback to default region if no prefix

---

### FR-023: GitLab Duo Workflow Discovery

**Priority:** P1 (High)
**Module:** `crates/llm/src/`
**Status:** ❌ Not Implemented

**Specification:**
- Implement GitLab model discovery via AI Gateway
- Use AI Gateway headers and feature flags
- Discover Duo Workflow models from GitLab instance

**Implementation:**
```rust
async fn discover_gitlab_models(instance_url: &str) -> Result<Vec<Model>> {
    // Fetch from /api/v1/ai/models via AI Gateway
    // Apply feature flags
    // Return model list
}
```

**Acceptance Criteria:**
- [ ] GitLab AI model discovery implemented
- [ ] AI Gateway header support
- [ ] Feature flag handling
- [ ] Duo Workflow model support

---

### FR-024: Cloudflare AI Gateway Provider

**Priority:** P1 (High)
**Module:** `crates/llm/src/`
**Status:** ❌ Not Implemented

**Specification:**
- Add Cloudflare AI Gateway provider
- Use `ai-gateway-provider` package pattern
- Map provider/model format to unified endpoint

**Acceptance Criteria:**
- [ ] AI Gateway provider registered
- [ ] Provider/model URL mapping
- [ ] Unified endpoint routing

---

### FR-025: Experimental Modes (Model Variants)

**Priority:** P1 (High)
**Module:** `crates/llm/src/models.rs`
**Status:** ❌ Not Implemented

**Specification:**
- Map `experimental.modes` from models.dev to `model.variants`
- Support mode variants (e.g., thinking mode, extended thinking)

**PRD Schema Mapping:**
| models.dev Field | OpenCode Field | Status |
|-----------------|----------------|--------|
| `experimental.modes` | `model.variants` | ❌ Not done |

**Acceptance Criteria:**
- [ ] `variants` field added to Model struct
- [ ] Experimental modes parsed from models.dev
- [ ] Mode variant selection in model picker

---

### FR-026: CLI `--refresh` Flag

**Priority:** P2 (Medium)
**Module:** `crates/cli/src/cmd/models.rs`
**Status:** ❌ Not Implemented

**Specification:**
- Add `--refresh` flag to `opencode models` command
- Force cache refresh, bypass TTL checks

**Usage:**
```bash
opencode models --refresh
```

**Acceptance Criteria:**
- [ ] `--refresh` flag parsed
- [ ] Force fetch from models.dev API
- [ ] Update disk cache
- [ ] Display refresh status

---

### FR-027: Alpha Model Filtering

**Priority:** P2 (Medium)
**Module:** `crates/llm/src/models.rs`
**Status:** ❌ Not Implemented

**Specification:**
- Filter alpha/experimental models based on `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag
- Alpha models hidden by default unless flag is true

**Implementation:**
```rust
fn is_model_visible(model: &Model) -> bool {
    if model.status == ModelStatus::Alpha {
        return std::env::var("OPENCODE_ENABLE_EXPERIMENTAL_MODELS")
            .map(|v| v == "true")
            .unwrap_or(false);
    }
    true
}
```

**Acceptance Criteria:**
- [ ] Alpha model filtering logic implemented
- [ ] `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag checked
- [ ] Alpha models hidden by default
- [ ] Alpha models visible when flag is true

---

### FR-028: Per-Provider Headers Configuration

**Priority:** P2 (Medium)
**Module:** `opencode_config/`
**Status:** ❌ Not Implemented

**Specification:**
- Add `headers` field to `ProviderOptions`
- Support custom headers per provider in config

**Config Schema:**
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

**Acceptance Criteria:**
- [ ] `headers` field added to ProviderOptions
- [ ] Headers passed in HTTP requests
- [ ] Config file parsing support

---

### FR-029: Model Variants Support

**Priority:** P2 (Medium)
**Module:** `crates/llm/src/models.rs`
**Status:** ❌ Not Implemented

**Specification:**
- Add `variants` field to Model struct for mode variants
- Support extended thinking, preview modes, etc.

**Acceptance Criteria:**
- [ ] `variants: Vec<ModelVariant>` field in Model
- [ ] Variant selection in model picker
- [ ] Variant passed to provider at inference time

---

## 5. Authentication Specification

### 5.1 Auth Method Matrix

| Provider | opencode Auth | opencode-rs Auth | Status |
|----------|--------------|-----------------|--------|
| Google | OAuth | ❌ Not wired | **FR-014** needed |
| OpenAI | OAuth + API Key | ✅ OAuth + API Key | Complete |
| GitHub Copilot | OAuth | ❌ Not wired | **FR-015** needed |
| Kimi | API Key | ❌ None | **FR-012, FR-013** needed |
| Z.AI | API Key | ❌ None | **FR-012, FR-013** needed |
| MiniMax | API Key | ❌ None | **FR-012, FR-013** needed |
| Anthropic | API Key | ⚠️ API Key (no validation) | **FR-012** needed |
| Ollama | Local | ✅ Local | Complete |

---

## 6. models.dev Data Schema Mapping

### 6.1 PRD Schema Mapping (models.dev → OpenCode)

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
| `experimental.modes` | `model.variants` | ❌ Not done (**FR-025**) |

---

## 7. Technical Debt

| ID | Issue | Module | Severity | Fix |
|----|-------|--------|----------|-----|
| TD-1 | Dual model sources | `models.rs` | High | Consolidate to single catalog source (FR-018) |
| TD-2 | `transform.rs` misnamed | `llm/src/transform.rs` | Low | Module does message transformation |
| TD-3 | No `models-dev` prefix in provider IDs | `fetcher.rs` | Medium | Avoid provider ID conflicts |
| TD-4 | Alpha model filtering missing | `models.rs` | Medium | **FR-027** |

---

## 8. Priority Matrix

### P0 - Critical Blockers (Iteration 35)

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-019 | No bundled snapshot fallback | `catalog/snapshot.rs` | App fails silently when offline |
| FR-020 | No snapshot generation script | `scripts/` | Cannot generate offline snapshot |
| FR-018 | models.dev integration partial | `catalog/` | Core data fetching done, snapshot not |

### P1 - High Priority

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-021 | No dynamic provider loading | `llm/` | Cannot dynamically load @ai-sdk providers |
| FR-022 | No Bedrock region prefixing | `bedrock.rs` | Cross-region inference not supported |
| FR-023 | No GitLab Duo discovery | `llm/` | Cannot discover GitLab AI models |
| FR-024 | No AI Gateway support | `llm/` | Cannot route via AI Gateway |
| FR-025 | No experimental modes mapping | `models.rs` | `experimental.modes` not mapped to `variants` |

### P2 - Medium Priority

| FR | Issue | Module | Gap |
|----|-------|--------|-----|
| FR-026 | No CLI `--refresh` | `models.rs:11-24` | Cannot refresh model cache from CLI |
| FR-027 | No alpha model filtering | `models.rs` | Alpha models always visible |
| FR-028 | No per-provider headers | `opencode_config/` | Cannot set custom provider headers |
| FR-029 | Model variants support | `models.rs` | Variants field not implemented |

---

## 9. Iteration 35 Deliverables

### Must Fix (P0)

1. **FR-019**: Create bundled snapshot module (`snapshot.rs`)
2. **FR-020**: Create snapshot generation script (`scripts/generate-snapshot.sh`)
3. **FR-018**: Complete models.dev integration (snapshot fallback)

### Should Fix (P1)

4. **FR-021**: Implement dynamic provider loading (BUNDLED_PROVIDERS)
5. **FR-022**: Add Amazon Bedrock region prefix handling
6. **FR-023**: Implement GitLab Duo Workflow discovery
7. **FR-024**: Add Cloudflare AI Gateway provider
8. **FR-025**: Map experimental.modes to model.variants

### Nice to Have (P2)

9. **FR-026**: Add CLI `--refresh` flag
10. **FR-027**: Implement alpha model filtering
11. **FR-028**: Add per-provider headers support
12. **FR-029**: Add model variants to Model struct

---

## 10. Acceptance Criteria Summary

### P0 Criteria (Required Before Release)

- [ ] **FR-019**: Bundled snapshot fallback works when offline
- [ ] **FR-020**: Snapshot generation script functional
- [ ] **FR-018**: models.dev integration complete with snapshot

### P1 Criteria (Next Sprint)

- [ ] **FR-021**: Dynamic provider loading via BUNDLED_PROVIDERS
- [ ] **FR-022**: Amazon Bedrock region prefix handling
- [ ] **FR-023**: GitLab Duo Workflow discovery
- [ ] **FR-024**: Cloudflare AI Gateway provider
- [ ] **FR-025**: Experimental modes mapped to variants

### P2 Criteria (Medium Term)

- [ ] **FR-026**: CLI `--refresh` flag works
- [ ] **FR-027**: Alpha models filtered by flag
- [ ] **FR-028**: Per-provider headers configurable
- [ ] **FR-029**: Model variants selectable

---

## 11. Code References

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
| Bedrock | `crates/llm/src/bedrock.rs` | 1-200 |

---

## 12. Cross-References

| File | Topic | Related FR |
|------|-------|------------|
| `crates/llm/src/catalog/fetcher.rs` | Catalog fetching | FR-018, FR-019, FR-020 |
| `crates/llm/src/catalog/types.rs` | Catalog types | FR-018 |
| `crates/llm/src/catalog/models_dev.rs` | models.dev types | FR-018 |
| `crates/llm/src/catalog/merge.rs` | Config merging | FR-018 |
| `crates/llm/src/catalog/snapshot.rs` | Snapshot fallback | FR-019 (NEW) |
| `crates/llm/src/models.rs` | Model registry | FR-002, FR-018, FR-025, FR-027, FR-029 |
| `crates/llm/src/bedrock.rs` | Bedrock provider | FR-022 |
| `crates/cli/src/cmd/models.rs` | CLI models | FR-026 |
| `crates/core/src/flag.rs` | Flags | FR-018, FR-027 |

---

## 13. Gap Analysis Summary

| Gap Item | Severity | Module | File:Line | Status |
|----------|----------|--------|-----------|--------|
| No bundled snapshot fallback | **P0** | LLM Catalog | `fetcher.rs:82-89` | **FR-019** |
| No snapshot generation script | **P0** | Scripts | N/A | **FR-020** |
| No dynamic provider loading | P1 | LLM | N/A | **FR-021** |
| No Bedrock region prefixing | P1 | LLM/Bedrock | `bedrock.rs` | **FR-022** |
| No GitLab discovery | P1 | LLM | N/A | **FR-023** |
| No AI Gateway support | P1 | LLM | N/A | **FR-024** |
| No experimental.modes mapping | P1 | LLM | `models.rs` | **FR-025** |
| No CLI `--refresh` | P2 | CLI | `models.rs:11-24` | **FR-026** |
| No alpha model filtering | P2 | LLM | `models.rs` | **FR-027** |
| No per-provider headers | P2 | Config | `opencode_config/` | **FR-028** |
| Model variants not implemented | P2 | LLM | `models.rs` | **FR-029** |
| Dual model sources | TD | LLM | `models.rs` | **FR-018** |

---

*Specification Version: 35*
*Generated: 2026-04-20*
*Based on: PRD #35 (models.dev Integration) Gap Analysis*