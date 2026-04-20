# Implementation Plan v35 - models.dev Integration

**Project:** opencode-rs
**Iteration:** 35
**Date:** 2026-04-20
**Status:** Draft
**Priority:** P0 Critical Blockers First

---

## 1. Executive Summary

This plan implements the models.dev integration for opencode-rs, focusing on P0 critical blockers that prevent the app from functioning when offline. The plan is structured with P0 items first, followed by P1 and P2 items.

### Key Constraints
- No subagents/task tools allowed
- All analysis must be done directly in current session
- Use only Read, Write, Edit, Grep, LSP tools

---

## 2. Priority Matrix

### P0 - Critical Blockers (Must Fix)

| FR | Issue | Module | Files | Estimated Effort |
|----|-------|--------|-------|-----------------|
| FR-019 | Bundled snapshot fallback | `catalog/snapshot.rs` | 3-4 days |
| FR-020 | Snapshot generation script | `scripts/` | 1-2 days |
| FR-018 | Complete models.dev integration | `catalog/` | 1 day (snapshot integration) |

### P1 - High Priority (Should Fix)

| FR | Issue | Module | Files | Estimated Effort |
|----|-------|--------|-------|-----------------|
| FR-021 | Dynamic provider loading | `llm/` | 3-4 days |
| FR-022 | Amazon Bedrock region prefixing | `bedrock.rs` | 2-3 days |
| FR-023 | GitLab Duo discovery | `llm/` | 3-4 days |
| FR-024 | Cloudflare AI Gateway | `llm/` | 2-3 days |
| FR-025 | Experimental modes mapping | `models.rs` | 1-2 days |

### P2 - Medium Priority (Nice to Have)

| FR | Issue | Module | Files | Estimated Effort |
|----|-------|--------|-------|-----------------|
| FR-026 | CLI `--refresh` flag | `models.rs`, `cli/` | 0.5 day |
| FR-027 | Alpha model filtering | `models.rs` | 0.5 day |
| FR-028 | Per-provider headers | `opencode_config/` | 1 day |
| FR-029 | Model variants support | `models.rs` | 1-2 days |

---

## 3. Implementation Phases

### Phase 1: P0 Critical Blockers

#### 1.1 FR-019: Bundled Snapshot Fallback

**Objective:** Create offline fallback so app functions without network

**Files to Create:**
- `crates/llm/src/catalog/snapshot.rs` - New module
- `crates/llm/src/catalog/snapshot_data.rs` - Embedded data (auto-generated)

**Files to Modify:**
- `crates/llm/src/catalog/mod.rs` - Add snapshot module
- `crates/llm/src/catalog/fetcher.rs` - Use snapshot on network failure

**Implementation Steps:**

1. Create `snapshot.rs` module with:
   ```rust
   pub struct SnapshotCatalog { ... }
   pub fn get_snapshot() -> Option<SnapshotCatalog> { ... }
   pub fn is_snapshot_available() -> bool { ... }
   ```

2. Create data structure for bundled snapshot:
   ```rust
   const BUNDLED_SNAPSHOT: &[u8] = include_bytes!("snapshot_data.json");
   ```

3. Modify `fetcher.rs` fallback logic:
   ```rust
   Err(_) => {
       // Try file cache first
       self.read_file_cache().await?
           // Then try bundled snapshot
           .or_else(|| get_snapshot())
           // Finally return empty catalog
           .unwrap_or_else(|| ProviderCatalog { ... })
   }
   ```

4. Add unit tests for snapshot loading

**Acceptance Criteria:**
- [ ] `snapshot.rs` module created with `SnapshotCatalog`, `get_snapshot()`, `is_snapshot_available()`
- [ ] Bundled snapshot data embedded in binary
- [ ] Fetcher falls back to snapshot when offline (no network + no cache)
- [ ] App functions with snapshot when completely offline
- [ ] Unit tests pass

---

#### 1.2 FR-020: Snapshot Generation Script

**Objective:** Create build-time script to generate embedded snapshot data

**Files to Create:**
- `scripts/generate-snapshot.sh` - Generation script (Bash)
- `scripts/models-snapshot-template.json` - Template for snapshot data

**Files to Modify:**
- `Cargo.toml` (root) - Add build script integration
- `build.sh` - Add snapshot generation step

**Implementation Steps:**

1. Create `scripts/generate-snapshot.sh`:
   ```bash
   #!/bin/bash
   # Fetch from models.dev API
   # Transform to Rust format
   # Write to crates/llm/src/catalog/snapshot_data.rs
   ```

2. The script should:
   - Fetch `https://models.dev/api.json`
   - Transform JSON to Rust struct
   - Output as `include_bytes!()` compatible file
   - Include timestamp metadata

3. Add to `build.sh` as optional pre-build step:
   ```bash
   if [ "$GENERATE_SNAPSHOT" = "true" ]; then
       ./scripts/generate-snapshot.sh
   fi
   ```

4. Document in `CONTRIBUTING.md`

**Acceptance Criteria:**
- [ ] `scripts/generate-snapshot.sh` created and executable
- [ ] Fetches latest models.dev data
- [ ] Generates embeddable Rust code in `snapshot_data.rs`
- [ ] Can be run during build/CI
- [ ] Includes timestamp and version metadata

---

#### 1.3 FR-018: Complete models.dev Integration

**Objective:** Finish partial implementation by integrating snapshot fallback

**Current Status:** Core fetching, caching, catalog types done

**Remaining Work:**
- Integrate FR-019 snapshot into fetcher
- Ensure clean fallback chain: memory cache → disk cache → bundled snapshot → empty catalog

**Files to Modify:**
- `crates/llm/src/catalog/fetcher.rs` - Update fallback chain

**Implementation Steps:**

1. Update fallback chain in `fetcher.rs`:
   ```rust
   async fn fetch_or_get_cached(&self) -> Result<ProviderCatalog> {
       // 1. Check memory cache (5-min TTL)
       if let Some(cached) = self.memory_cache.get() {
           if !cached.is_expired() {
               return Ok(cached.catalog.clone());
           }
       }

       // 2. Try network fetch
       match self.fetch_from_models_dev().await {
           Ok(catalog) => {
               self.memory_cache.put(catalog.clone());
               self.write_file_cache(&catalog).await.ok();
               return Ok(catalog);
           }
           Err(_) => {
               // 3. Try file cache
               if let Ok(cached) = self.read_file_cache().await {
                   return Ok(cached);
               }
               // 4. Try bundled snapshot
               if let Some(snapshot) = get_snapshot() {
                   return Ok(snapshot.into());
               }
               // 5. Return empty (should not reach here with FR-019)
               return Ok(ProviderCatalog::empty());
           }
       }
   }
   ```

**Acceptance Criteria:**
- [ ] Clean fallback chain implemented
- [ ] Memory cache → disk cache → bundled snapshot → empty catalog
- [ ] All existing tests pass

---

### Phase 2: P1 High Priority

#### 2.1 FR-021: Dynamic Provider Loading

**Objective:** Enable dynamic loading of @ai-sdk/* provider packages

**Files to Create:**
- `crates/llm/src/bundled_providers.rs` - New module with provider registry

**Files to Modify:**
- `crates/llm/src/lib.rs` - Add module export
- `crates/llm/src/provider.rs` - Integrate dynamic loading

**Implementation Steps:**

1. Create `bundled_providers.rs`:
   ```rust
   use std::future::Future;
   use std::pin::Pin;

   type ProviderFactory = Pin<Box<dyn Future<Output = Result<Box<dyn Provider>>>;

   pub struct BundledProvider {
       pub name: &'static str,
       pub package: &'static str,
       pub factory: fn() -> ProviderFactory,
   }

   pub const BUNDLED_PROVIDERS: &[BundledProvider] = &[
       BundledProvider {
           name: "anthropic",
           package: "@ai-sdk/anthropic",
           factory: || Box::pin(async { Ok(Box::new(AnthropicProvider::new()) as Box<dyn Provider>) }),
       },
       // ... 17 more providers
   ];
   ```

2. Note: In Rust, we can't dynamically import npm packages. Instead, we implement a registry pattern that mimics dynamic loading with statically compiled providers.

3. Implement lazy loading:
   ```rust
   pub async fn get_provider(name: &str) -> Result<Arc<dyn Provider>> {
       static PROVIDERS: once_cell::sync::Lazy<HashMap<&str, Arc<dyn Provider>>> =
           once_cell::sync::Lazy::new(|| {
               BUNDLED_PROVIDERS
                   .iter()
                   .map(|p| (p.name, p.factory()))
                   .collect()
           });

       PROVIDERS.get(name).cloned().ok_or_else(|| ...)
   }
   ```

**Acceptance Criteria:**
- [ ] `BUNDLED_PROVIDERS` constant defined with 20+ providers
- [ ] Provider factory functions implemented
- [ ] Lazy loading on first use
- [ ] Registry accessible for provider selection

---

#### 2.2 FR-022: Amazon Bedrock Region Prefix Handling

**Objective:** Support cross-region inference for Bedrock models

**Files to Modify:**
- `crates/llm/src/bedrock.rs` - Add region handling

**Implementation Steps:**

1. Add region detection function:
   ```rust
   fn get_region_prefix(model_id: &str) -> Option<&str> {
       let parts: Vec<&str> = model_id.split('.').collect();
       match parts.first()? {
           "us" | "us-east-1" | "us-west-2" => Some("us"),
           "eu" | "eu-west-1" => Some("eu"),
           "jp" | "apac" | "au" => Some("apac"),
           _ => None,
       }
   }
   ```

2. Add region endpoint mapping:
   ```rust
   fn get_bedrock_endpoint(region: &str) -> String {
       match region {
           "us" => "https://bedrock.us-east-1.amazonaws.com".to_string(),
           "eu" => "https://bedrock.eu-west-1.amazonaws.com".to_string(),
           "apac" => "https://bedrock.ap-northeast-1.amazonaws.com".to_string(),
           _ => "https://bedrock.us-east-1.amazonaws.com".to_string(), // default
       }
   }
   ```

3. Update model routing in `BedrockProvider`:
   ```rust
   async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
       let model_id = &request.model;
       let region = get_region_prefix(model_id).unwrap_or("us");
       let endpoint = get_bedrock_endpoint(region);
       // ... route to endpoint
   }
   ```

**Acceptance Criteria:**
- [ ] Region prefix detection function implemented
- [ ] Endpoint routing based on region
- [ ] US/EU/APAC region support
- [ ] Fallback to default region if no prefix

---

#### 2.3 FR-023: GitLab Duo Workflow Discovery

**Objective:** Enable discovery of GitLab AI models via AI Gateway

**Files to Create:**
- `crates/llm/src/gitlab.rs` - New module for GitLab discovery

**Files to Modify:**
- `crates/llm/src/lib.rs` - Add module export
- `crates/llm/src/catalog/merge.rs` - Add GitLab provider to merger

**Implementation Steps:**

1. Create `gitlab.rs`:
   ```rust
   pub struct GitLabProvider {
       instance_url: String,
       client: reqwest::Client,
   }

   impl GitLabProvider {
       pub async fn discover_models(&self) -> Result<Vec<Model>> {
           // Fetch from /api/v1/ai/models via AI Gateway
           let response = self.client
               .get(format!("{}/api/v1/ai/models", self.instance_url))
               .header("Authorization", format!("Bearer {}", self.token))
               .send()
               .await?;

           // Parse and transform to OpenCode Model type
           // ...
       }
   }
   ```

2. Implement feature flag handling:
   ```rust
   fn should_enable_gitlab_duo(feature_flags: &[String]) -> bool {
       feature_flags.iter().any(|f| f.contains("gitlab_duo_workflow"))
   }
   ```

**Acceptance Criteria:**
- [ ] GitLab AI model discovery implemented
- [ ] AI Gateway header support
- [ ] Feature flag handling
- [ ] Duo Workflow model support

---

#### 2.4 FR-024: Cloudflare AI Gateway Provider

**Objective:** Add Cloudflare AI Gateway routing support

**Files to Create:**
- `crates/llm/src/ai_gateway.rs` - New module

**Files to Modify:**
- `crates/llm/src/lib.rs` - Add module export
- `crates/llm/src/provider.rs` - Register AI Gateway provider

**Implementation Steps:**

1. Create `ai_gateway.rs`:
   ```rust
   pub struct AiGatewayProvider {
       account_id: String,
       client: reqwest::Client,
   }

   impl AiGatewayProvider {
       pub fn new(account_id: String) -> Self {
           Self {
               account_id,
               client: reqwest::Client::new(),
           }
       }

       pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
           // Route through AI Gateway
           let url = format!(
               "https://gateway.ai.cloudflare.com/v1/{}/openai",
               self.account_id
           );
           // ... forward request to AI Gateway
       }
   }
   ```

**Acceptance Criteria:**
- [ ] AI Gateway provider registered
- [ ] Provider/model URL mapping
- [ ] Unified endpoint routing

---

#### 2.5 FR-025: Experimental Modes Mapping

**Objective:** Map `experimental.modes` from models.dev to `model.variants`

**Files to Modify:**
- `crates/llm/src/models.rs` - Add variants field and parsing
- `crates/llm/src/catalog/models_dev.rs` - Add experimental.modes parsing

**Implementation Steps:**

1. Add `variants` field to Model struct:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ModelVariant {
       pub name: String,
       pub description: Option<String>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct Model {
       // ... existing fields
       pub variants: Vec<ModelVariant>,
   }
   ```

2. Parse experimental.modes in catalog merger:
   ```rust
   fn parse_experimental_modes(modes: Option<Vec<String>>) -> Vec<ModelVariant> {
       modes.map(|m| m.into_iter().map(|name| ModelVariant {
           name,
           description: None,
       }).collect()).unwrap_or_default()
   }
   ```

3. Update model picker in TUI to show variants

**Acceptance Criteria:**
- [ ] `variants: Vec<ModelVariant>` field in Model struct
- [ ] Experimental modes parsed from models.dev
- [ ] Mode variant selection in model picker

---

### Phase 3: P2 Medium Priority

#### 3.1 FR-026: CLI `--refresh` Flag

**Files to Modify:**
- `crates/cli/src/cmd/models.rs` - Add refresh flag
- `crates/llm/src/catalog/fetcher.rs` - Add force_refresh parameter

**Implementation Steps:**

1. Add `--refresh` flag to CLI:
   ```rust
   #[derive(Parser)]
   pub enum ModelsSubcommand {
       /// List available models
       List(ListOptions),
       /// Refresh model cache
       Refresh,
   }

   pub async fn run_refresh() -> Result<()> {
       let fetcher = CatalogFetcher::new();
       fetcher.force_refresh().await?;
       println!("Model cache refreshed successfully");
       Ok(())
   }
   ```

2. Add `force_refresh()` method to fetcher

**Acceptance Criteria:**
- [ ] `--refresh` flag parsed
- [ ] Force fetch from models.dev API
- [ ] Update disk cache
- [ ] Display refresh status

---

#### 3.2 FR-027: Alpha Model Filtering

**Files to Modify:**
- `crates/llm/src/models.rs` - Add filtering logic
- `crates/llm/src/catalog/fetcher.rs` - Add filtering on load

**Implementation Steps:**

1. Add `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` check:
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

2. Apply filter when loading models

**Acceptance Criteria:**
- [ ] Alpha model filtering logic implemented
- [ ] `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` flag checked
- [ ] Alpha models hidden by default
- [ ] Alpha models visible when flag is true

---

#### 3.3 FR-028: Per-Provider Headers Configuration

**Files to Modify:**
- `crates/opencode_config/` - Add headers field to ProviderOptions

**Implementation Steps:**

1. Add `headers` field:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ProviderOptions {
       pub base_url: Option<String>,
       pub headers: Option<HashMap<String, String>>,
   }
   ```

2. Pass headers in HTTP requests

**Acceptance Criteria:**
- [ ] `headers` field added to ProviderOptions
- [ ] Headers passed in HTTP requests
- [ ] Config file parsing support

---

#### 3.4 FR-029: Model Variants Support

**Files to Modify:**
- `crates/llm/src/models.rs` - Already covered by FR-025

**Note:** FR-029 is largely covered by FR-025. This FR focuses on the UI/picker side of variant selection.

---

## 4. Technical Debt Items

| TD | Issue | Priority | Fix |
|----|-------|----------|-----|
| TD-1 | Dual model sources | High | Consolidate to single catalog source (FR-018) |
| TD-2 | `transform.rs` misnamed | Low | Consider renaming to `message_transform.rs` |
| TD-3 | No `models-dev` prefix | Medium | Add prefix in `fetcher.rs` for disambiguation |

---

## 5. File Inventory

### Files to CREATE (New)

| File | Purpose | FR |
|------|---------|-----|
| `crates/llm/src/catalog/snapshot.rs` | Bundled snapshot module | FR-019 |
| `crates/llm/src/catalog/snapshot_data.rs` | Embedded snapshot data | FR-019 |
| `scripts/generate-snapshot.sh` | Snapshot generation script | FR-020 |
| `scripts/models-snapshot-template.json` | Template for snapshot | FR-020 |
| `crates/llm/src/bundled_providers.rs` | Dynamic provider registry | FR-021 |
| `crates/llm/src/gitlab.rs` | GitLab Duo discovery | FR-023 |
| `crates/llm/src/ai_gateway.rs` | Cloudflare AI Gateway | FR-024 |

### Files to MODIFY (Existing)

| File | Changes | FR |
|------|---------|-----|
| `crates/llm/src/catalog/mod.rs` | Add snapshot module | FR-019 |
| `crates/llm/src/catalog/fetcher.rs` | Snapshot fallback + force_refresh | FR-018, FR-019, FR-026 |
| `Cargo.toml` | Build script integration | FR-020 |
| `build.sh` | Add snapshot generation step | FR-020 |
| `crates/llm/src/lib.rs` | Add new modules | FR-021, FR-023, FR-024 |
| `crates/llm/src/provider.rs` | Integrate dynamic loading | FR-021 |
| `crates/llm/src/bedrock.rs` | Region prefix handling | FR-022 |
| `crates/llm/src/models.rs` | variants, alpha filter, CLI refresh | FR-025, FR-027, FR-026 |
| `crates/llm/src/catalog/merge.rs` | GitLab provider | FR-023 |
| `crates/cli/src/cmd/models.rs` | Add --refresh flag | FR-026 |
| `crates/opencode_config/` | Add headers to ProviderOptions | FR-028 |

---

## 6. Dependencies

### Internal Dependencies
- FR-019 requires FR-020 (snapshot data file)
- FR-018 integration depends on FR-019

### External Dependencies
- `reqwest` - HTTP client (already in Cargo.toml)
- `serde_json` - JSON parsing (already in Cargo.toml)
- `chrono` - Timestamps (already in Cargo.toml)

---

## 7. Testing Strategy

### Unit Tests
- `snapshot.rs`: Test snapshot loading, fallback chain
- `fetcher.rs`: Test cache expiry, refresh, fallback
- `bedrock.rs`: Test region prefix detection
- `models.rs`: Test alpha filtering, variant parsing

### Integration Tests
- Full offline scenario: no network, no cache → should use bundled snapshot
- Cache refresh scenario: `--refresh` flag forces fetch

---

## 8. Timeline Estimate

| Phase | Tasks | Total Effort |
|-------|-------|--------------|
| Phase 1 (P0) | FR-019, FR-020, FR-018 | 5-7 days |
| Phase 2 (P1) | FR-021, FR-022, FR-023, FR-024, FR-025 | 11-16 days |
| Phase 3 (P2) | FR-026, FR-027, FR-028, FR-029 | 3-5 days |
| **Total** | | **19-28 days** |

---

## 9. Verification

Before marking each FR complete:
1. All unit tests pass (`cargo test`)
2. Code compiles without warnings (`cargo clippy -- -D warnings`)
3. Acceptance criteria checked off
4. Documentation updated if needed

---

*Plan Version: 35*
*Created: 2026-04-20*
*Based on: spec_v35.md + gap-analysis.md*
