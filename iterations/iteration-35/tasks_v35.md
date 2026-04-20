# Task List v35 - models.dev Integration

**Project:** opencode-rs
**Iteration:** 35
**Date:** 2026-04-20
**Status:** Draft
**Priority:** P0 Critical Blockers First

---

## Phase 1: P0 Critical Blockers (Must Fix)

### FR-019: ✅ Done

**Status:** ✅ Done
**Priority:** P0
**Module:** `catalog/snapshot.rs`
**Effort:** 3-4 days

**Tasks:**

- [x] **T-019-1:** Create `crates/llm/src/catalog/snapshot.rs` module
  - [x] Define `SnapshotCatalog` struct matching `ProviderCatalog` structure
  - [x] Implement `get_snapshot() -> Option<SnapshotCatalog>` function
  - [x] Implement `is_snapshot_available() -> bool` function
  - [x] Add `From<SnapshotCatalog> for ProviderCatalog` impl

- [x] **T-019-2:** Create `crates/llm/src/catalog/snapshot_data.rs`
  - [x] Create minimal snapshot data structure with major providers/models
  - [x] Include at least: OpenAI, Anthropic, Google, GitHub Copilot models
  - [x] Format as `include_bytes!()` compatible static byte array

- [x] **T-019-3:** Modify `crates/llm/src/catalog/mod.rs`
  - [x] Add `pub mod snapshot;` declaration
  - [x] Export snapshot module in lib.rs

- [x] **T-019-4:** Modify `crates/llm/src/catalog/fetcher.rs`
  - [x] Update fallback chain: memory cache → disk cache → bundled snapshot → empty
  - [x] Add snapshot loading in network error path
  - [x] Add tests for snapshot fallback

- [x] **T-019-5:** Add unit tests for snapshot module
  - [x] Test `is_snapshot_available()` returns true when snapshot exists
  - [x] Test `get_snapshot()` returns Some when available
  - [x] Test fallback chain in fetcher

**Acceptance Criteria:**
- [x] `snapshot.rs` module created with required functions
- [x] Bundled snapshot data embedded in binary
- [x] Fetcher falls back to snapshot when offline
- [x] App functions with snapshot when no network
- [x] Unit tests pass

---

### FR-020: ✅ Done

**Status:** ✅ Done
**Priority:** P0
**Module:** `scripts/`
**Effort:** 1-2 days

**Tasks:**

- [x] **T-020-1:** Create `scripts/generate-snapshot.sh`
  - [x] Make script executable (`chmod +x`)
  - [x] Add shebang `#!/bin/bash`
  - [x] Fetch from `https://models.dev/api.json` via curl
  - [x] Transform JSON to Rust struct format
  - [x] Write output to `crates/llm/src/catalog/snapshot_catalog.json`
  - [x] Include timestamp and version metadata in output

- [x] **T-020-2:** Add snapshot generation to `build.sh`
  - [x] Add `GENERATE_SNAPSHOT` environment variable check
  - [x] Run script before cargo build if enabled

**Acceptance Criteria:**
- [ ] `scripts/generate-snapshot.sh` created and executable
- [ ] Fetches latest models.dev data
- [ ] Generates embeddable Rust code
- [ ] Can be run during build/CI
- [ ] Includes timestamp and version metadata

---

### FR-018: ✅ Done

**Status:** ⚠️ Partial (snapshot integration remaining)
**Priority:** P0
**Module:** `catalog/`
**Effort:** 1 day

**Tasks:**

- [ ] **T-018-1:** Verify fallback chain in `fetcher.rs`
  - [ ] Memory cache (5-min TTL) checked first
  - [ ] Network fetch attempted second
  - [ ] File cache (disk) used on network failure
  - [ ] Bundled snapshot used if file cache fails/missing
  - [ ] Empty catalog as last resort

- [ ] **T-018-2:** Update `fetch_or_get_cached()` method
  - [ ] Ensure clean fallback chain implemented
  - [ ] Add comments documenting fallback order
  - [ ] Verify all existing tests pass

**Acceptance Criteria:**
- [ ] Clean fallback chain implemented: memory → network → disk → snapshot → empty
- [ ] All existing tests pass
- [ ] No duplicate network requests on cache hit

---

## Phase 2: P1 High Priority

### FR-021: ✅ Done

**Status:** ❌ Not Implemented
**Priority:** P1
**Module:** `llm/`
**Effort:** 3-4 days

**Tasks:**

- [ ] **T-021-1:** Create `crates/llm/src/bundled_providers.rs`
  - [ ] Define `BundledProvider` struct with name, package, factory
  - [ ] Define `BUNDLED_PROVIDERS` constant with 20+ providers:
    - [ ] `@ai-sdk/anthropic`
    - [ ] `@ai-sdk/openai`
    - [ ] `@ai-sdk/amazon-bedrock`
    - [ ] `@ai-sdk/google-generativeai`
    - [ ] `@ai-sdk/vertex`
    - [ ] `@ai-sdk/azure`
    - [ ] `@ai-sdk/cohere`
    - [ ] `@ai-sdk/mistral`
    - [ ] `@ai-sdk/bedrock`
    - [ ] `@ai-sdk/ollama`
    - [ ] `@ai-sdk/perplexity`
    - [ ] `@ai-sdk/amazon`
    - [ ] `@ai-sdk/fireworks`
    - [ ] `@ai-sdk/groq`
    - [ ] `@ai-sdk/metas`
    - [ ] `@ai-sdk/nvidia`
    - [ ] `@ai-sdk/oidc`
    - [ ] `@ai-sdk/cloudflare`
    - [ ] `@ai-sdk/xai`
    - [ ] `@ai-sdk/together`
  - [ ] Implement `ProviderFactory` type alias
  - [ ] Implement lazy loading via `once_cell::sync::Lazy`

- [ ] **T-021-2:** Add `bundled_providers` module to `lib.rs`
  - [ ] Export `BUNDLED_PROVIDERS` constant

- [ ] **T-021-3:** Integrate with provider registry
  - [ ] Update `provider.rs` to use dynamic loading
  - [ ] Implement `get_provider(name: &str)` function

**Acceptance Criteria:**
- [ ] `BUNDLED_PROVIDERS` constant defined with 20+ entries
- [ ] Dynamic import implementation (registry pattern)
- [ ] Provider factory functions
- [ ] Lazy loading on first use
- [ ] Registry accessible for provider selection

---

### FR-022: ✅ Done

**Status:** ✅ Done
**Priority:** P1
**Module:** `bedrock.rs`
**Effort:** 2-3 days

**Tasks:**

- [x] **T-022-1:** Implement region prefix detection function
  - [x] `get_region_prefix(model_id: &str) -> Option<&str>`
  - [x] Support prefixes: `us.*`, `eu.*`, `jp.*`, `apac.*`, `au.*`
  - [x] Map to region codes: `us`, `eu`, `apac`

- [x] **T-022-2:** Implement region endpoint mapping
  - [x] `get_bedrock_endpoint(region: &str) -> String`
  - [x] US endpoint: `https://bedrock.us-east-1.amazonaws.com`
  - [x] EU endpoint: `https://bedrock.eu-west-1.amazonaws.com`
  - [x] APAC endpoint: `https://bedrock.ap-northeast-1.amazonaws.com`
  - [x] Default endpoint: `https://bedrock.us-east-1.amazonaws.com`

- [x] **T-022-3:** Update BedrockProvider to use region routing
  - [x] Extract region from model ID prefix
  - [x] Route to appropriate endpoint
  - [x] Fallback to default region if no prefix

- [x] **T-022-4:** Add unit tests for region handling
  - [x] Test `get_region_prefix()` with various model IDs
  - [x] Test `get_bedrock_endpoint()` returns correct URLs
  - [x] Test default fallback

**Acceptance Criteria:**
- [x] Region prefix detection function implemented
- [x] Endpoint routing based on region
- [x] US/EU/APAC region support
- [x] Fallback to default region if no prefix
- [x] Unit tests pass

---

### FR-023: ✅ Done

**Status:** ✅ Done
**Priority:** P1
**Module:** `gitlab.rs` (new)
**Effort:** 3-4 days

**Tasks:**

- [x] **T-023-1:** Create `crates/llm/src/gitlab.rs` module
  - [x] Define `GitLabProvider` struct with instance_url, token, client
  - [x] Implement `new()` constructor
  - [x] Implement `discover_models() -> Result<Vec<Model>>`

- [x] **T-023-2:** Implement AI Gateway discovery
  - [x] Fetch from `/api/v1/ai/models` endpoint
  - [x] Add `Authorization: Bearer {token}` header
  - [x] Parse response to OpenCode `Model` type

- [x] **T-023-3:** Implement feature flag handling
  - [x] `should_enable_gitlab_duo(feature_flags: &[String]) -> bool`
  - [x] Check for `gitlab_duo_workflow` flag
  - [x] Filter models based on feature flags

- [x] **T-023-4:** Add GitLab to catalog merger
  - [x] Update `crates/llm/src/catalog/merge.rs`
  - [x] Add GitLab provider to provider list
  - [x] Enable via config (`gitlab.instance_url`)

- [x] **T-023-5:** Export module in `lib.rs`

**Acceptance Criteria:**
- [x] GitLab AI model discovery implemented
- [x] AI Gateway header support
- [x] Feature flag handling
- [x] Duo Workflow model support
- [x] Integration with catalog merger

---

### FR-024: ✅ Done

**Status:** ✅ Done
**Priority:** P1
**Module:** `ai_gateway.rs` (new)
**Effort:** 2-3 days

**Tasks:**

- [x] **T-024-1:** Create `crates/llm/src/ai_gateway.rs` module
  - [x] Define `AiGatewayProvider` struct with account_id, client
  - [x] Implement `new(account_id: String) -> Self`
  - [x] Implement `complete()` method for inference

- [x] **T-024-2:** Implement AI Gateway routing
  - [x] Base URL: `https://gateway.ai.cloudflare.com/v1/{account_id}/openai`
  - [x] Forward requests to AI Gateway
  - [x] Handle response mapping

- [x] **T-024-3:** Add provider to registry
  - [x] Update `provider.rs`
  - [x] Register `ai_gateway` provider
  - [x] Export in `lib.rs`

**Acceptance Criteria:**
- [x] AI Gateway provider registered
- [x] Provider/model URL mapping
- [x] Unified endpoint routing
- [x] Integration with provider selection

---

### FR-025: ✅ Done

**Status:** ✅ Done
**Priority:** P1
**Module:** `models.rs`
**Effort:** 1-2 days

**Tasks:**

- [x] **T-025-1:** Add `ModelVariant` struct
  - [x] Fields: `name: String`, `description: Option<String>`
  - [x] Add Serialize, Deserialize, Debug, Clone derives

- [x] **T-025-2:** Add `variants` field to `Model` struct
  - [x] `variants: Vec<ModelVariant>`
  - [x] Update JSON serialization/deserialization
  - [x] Update all `Model::new()` callers

- [x] **T-025-3:** Parse `experimental.modes` in catalog merger
  - [x] Add `parse_experimental_modes()` function
  - [x] Map from `models_dev::Model` to `Model` with variants
  - [x] Handle None/empty variants

- [ ] **T-025-4:** Update TUI model picker (if needed)
  - [ ] Show variant selection UI
  - [ ] Pass selected variant to inference

**Acceptance Criteria:**
- [x] `variants: Vec<ModelVariant>` field in Model struct
- [x] Experimental modes parsed from models.dev
- [ ] Mode variant selection in model picker
- [x] Unit tests for variant parsing

---

## Phase 3: P2 Medium Priority

### FR-026: CLI `--refresh` Flag

**Status:** ✅ Done
**Priority:** P2
**Module:** `cli/src/cmd/models.rs`
**Effort:** 0.5 day

**Tasks:**

- [ ] **T-026-1:** Add `Refresh` subcommand to CLI
  - [ ] `#[derive(Parser)]` enum with `Refresh` variant
  - [ ] Add help text: "Force refresh model cache from models.dev"

- [ ] **T-026-2:** Implement `run_refresh()` function
  - [ ] Create new `CatalogFetcher`
  - [ ] Call `force_refresh().await`
  - [ ] Print success/error message

- [ ] **T-026-3:** Add `force_refresh()` to `CatalogFetcher`
  - [ ] Add `force_refresh(&self) -> Result<()>`
  - [ ] Bypass memory cache TTL check
  - [ ] Force network fetch
  - [ ] Update disk cache

- [ ] **T-026-4:** Wire up refresh command in `run()` method

**Acceptance Criteria:**
- [ ] `--refresh` flag parsed
- [ ] Force fetch from models.dev API
- [ ] Update disk cache
- [ ] Display refresh status
- [ ] Existing CLI tests still pass

---

### FR-027: Alpha Model Filtering

**Status:** ❌ Not Implemented
**Priority:** P2
**Module:** `models.rs`
**Effort:** 0.5 day

**Tasks:**

- [ ] **T-027-1:** Implement `is_model_visible()` function
  - [ ] Check `model.status == ModelStatus::Alpha`
  - [ ] If Alpha, check `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` env var
  - [ ] Return `true` only if env var equals "true"

- [ ] **T-027-2:** Apply filter in model loading
  - [ ] Filter models in `ModelRegistry::list_models()`
  - [ ] Filter models in `CatalogFetcher::load_models()`
  - [ ] Add integration with `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag

- [ ] **T-027-3:** Add unit tests
  - [ ] Test alpha model hidden by default
  - [ ] Test alpha model visible when flag is "true"
  - [ ] Test non-alpha models always visible

**Acceptance Criteria:**
- [ ] Alpha model filtering logic implemented
- [ ] `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag checked
- [ ] Alpha models hidden by default
- [ ] Alpha models visible when flag is true
- [ ] Unit tests pass

---

### FR-028: Per-Provider Headers Configuration

**Status:** ❌ Not Implemented
**Priority:** P2
**Module:** `opencode_config/`
**Effort:** 1 day

**Tasks:**

- [ ] **T-028-1:** Add `headers` field to `ProviderOptions`
  - [ ] Type: `Option<HashMap<String, String>>`
  - [ ] Update `Serialize`/`Deserialize` impls
  - [ ] Update JSON schema documentation

- [ ] **T-028-2:** Modify HTTP request construction
  - [ ] Add headers to provider HTTP clients
  - [ ] Merge config headers with default headers
  - [ ] Apply headers in `BedrockProvider`, `OpenAIProvider`, etc.

- [ ] **T-028-3:** Update config examples
  - [ ] Add example in `opencode.json.example`
  - [ ] Document header format

**Acceptance Criteria:**
- [ ] `headers` field added to ProviderOptions
- [ ] Headers passed in HTTP requests
- [ ] Config file parsing support
- [ ] Example configuration documented

---

### FR-029: Model Variants Support

**Status:** ❌ Not Implemented (superseded by FR-025)
**Priority:** P2
**Module:** `models.rs`
**Effort:** 1-2 days

**Tasks:**

- [ ] **T-029-1:** UI variant selection (if not covered by FR-025)
  - [ ] Update model picker dialog
  - [ ] Show variant dropdown/selection
  - [ ] Store selected variant in session

- [ ] **T-029-2:** Pass variant to provider at inference time
  - [ ] Update `CompletionRequest` to include variant
  - [ ] Update provider implementations to use variant
  - [ ] Test variant propagation

**Note:** Much of this FR is covered by FR-025. Verify before implementing.

**Acceptance Criteria:**
- [ ] Variants field added to Model struct
- [ ] Variant selection in model picker
- [ ] Variant passed to provider at inference time

---

## Technical Debt Tasks

### TD-1: Consolidate Dual Model Sources

**Status:** ⚠️ High - Dual sources (hardcoded + catalog)
**Priority:** High
**Module:** `models.rs`

**Tasks:**

- [ ] **T-TD1-1:** Audit hardcoded models vs catalog models
  - [ ] List all hardcoded models in `models.rs`
  - [ ] List all models from catalog fetcher
  - [ ] Identify duplicates/conflicts

- [ ] **T-TD1-2:** Plan consolidation approach
  - [ ] Option A: Migrate all to catalog (preferred)
  - [ ] Option B: Keep hardcoded as fallback
  - [ ] Document decision

- [ ] **T-TD1-3:** Implement consolidation
  - [ ] Remove duplicate model definitions
  - [ ] Ensure catalog is single source of truth
  - [ ] Update tests

---

### TD-2: Rename transform.rs

**Status:** ❌ Low - Module does message transformation
**Priority:** Low
**Module:** `llm/src/transform.rs`

**Tasks:**

- [ ] **T-TD2-1:** Rename module to `message_transform.rs`
  - [ ] Update `mod.rs` import
  - [ ] Update all `use` statements across codebase
  - [ ] Verify all tests pass

---

### TD-3: Add models-dev Prefix to Provider IDs

**Status:** ⚠️ Medium - Provider ID conflicts possible
**Priority:** Medium
**Module:** `fetcher.rs`

**Tasks:**

- [ ] **T-TD3-1:** Add prefix to provider IDs from models.dev
  - [ ] Change `provider.id` to `format!("models-dev-{}", provider.id)`
  - [ ] Update `ProviderCatalog` transformation
  - [ ] Update tests

---

## Verification Checklist

For each completed task:

- [ ] Code compiles without errors (`cargo build`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test`)
- [ ] Acceptance criteria checked
- [ ] File path matches specification
- [ ] No hardcoded secrets or test data in production code

---

## File Summary

### Files to CREATE (7 new files)

| File | FR | Type |
|------|----|------|
| `crates/llm/src/catalog/snapshot.rs` | FR-019 | New module |
| `crates/llm/src/catalog/snapshot_data.rs` | FR-019 | Data file |
| `scripts/generate-snapshot.sh` | FR-020 | Script |
| `scripts/models-snapshot-template.json` | FR-020 | Template |
| `crates/llm/src/bundled_providers.rs` | FR-021 | New module |
| `crates/llm/src/gitlab.rs` | FR-023 | New module |
| `crates/llm/src/ai_gateway.rs` | FR-024 | New module |

### Files to MODIFY (13 files)

| File | FR | Changes |
|------|----|---------|
| `crates/llm/src/catalog/mod.rs` | FR-019 | Add snapshot module |
| `crates/llm/src/catalog/fetcher.rs` | FR-018, FR-019, FR-026 | Snapshot fallback, force_refresh |
| `crates/llm/src/lib.rs` | FR-021, FR-023, FR-024 | Export new modules |
| `crates/llm/src/provider.rs` | FR-021 | Dynamic loading |
| `crates/llm/src/bedrock.rs` | FR-022 | Region prefix handling |
| `crates/llm/src/models.rs` | FR-025, FR-027, FR-029 | variants, alpha filter |
| `crates/llm/src/catalog/merge.rs` | FR-023 | GitLab provider |
| `crates/llm/src/catalog/models_dev.rs` | FR-025 | experimental.modes parsing |
| `crates/cli/src/cmd/models.rs` | FR-026 | --refresh flag |
| `crates/opencode_config/` | FR-028 | headers field |
| `Cargo.toml` | FR-020 | Build script |
| `build.sh` | FR-020 | Snapshot generation step |
| `CONTRIBUTING.md` | FR-020 | Documentation |

---

## Task Statistics

| Category | Count | Estimated Days |
|----------|-------|----------------|
| P0 Tasks | 3 FRs, 12 subtasks | 5-7 days |
| P1 Tasks | 5 FRs, 25 subtasks | 11-16 days |
| P2 Tasks | 4 FRs, 14 subtasks | 3-5 days |
| TD Tasks | 3 items | TBD |
| **Total** | **14 FRs, 51+ tasks** | **19-28 days** |

---

*Task List Version: 35*
*Created: 2026-04-20*
*Based on: spec_v35.md + gap-analysis.md*
