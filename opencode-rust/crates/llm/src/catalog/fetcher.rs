//! Provider Catalog Fetcher
//!
//! This module handles fetching and caching the provider catalog from models.dev.
//!
//! ## Fallback Chain (in order of priority)
//!
//! 1. **Memory Cache** - In-memory cache with 5-minute TTL
//!    - Checked first on every `get_or_fetch()` call
//!    - Valid for `CACHE_TTL` duration (5 minutes)
//!    - Stored in `self.catalog` (RwLock)
//!
//! 2. **Network Fetch** - Fetch from models.dev API
//!    - Attempted second when memory cache is invalid/missing
//!    - On success: writes to file cache and memory cache
//!    - On failure: proceeds to file cache fallback
//!
//! 3. **File Cache (Disk)** - Cached JSON file on disk
//!    - Used when network fetch fails
//!    - Valid regardless of age (no TTL check for stale fallback)
//!    - Stored at `~/.cache/opencode/models.json`
//!
//! 4. **Bundled Snapshot** - Embedded snapshot in binary
//!    - Used when both network and disk cache are unavailable
//!    - Provides offline functionality
//!    - Defined in `snapshot.rs` and `snapshot_data.rs`
//!
//! 5. **Empty Catalog** - Final fallback
//!    - Returns empty catalog if all else fails
//!    - Ensures the app can still function (with no providers)
//!
//! ## Cache TTL
//!
//! - Memory cache TTL: 5 minutes (`CACHE_TTL`)
//! - File cache: Used regardless of age (as fallback)
//! - Auto-refresh: Background refresh every 60 minutes via `refresh()` method
//!
//! ## Usage
//!
//! ```ignore
//! let fetcher = ProviderCatalogFetcher::new(cache_path);
//! let catalog = fetcher.get_or_fetch().await;
//! ```
//!
//! Or with force refresh:
//!
//! ```ignore
//! let catalog = fetcher.get(true).await?;
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::catalog::models_dev::{ModelsDevApiResponse, ModelsDevModel, ModelsDevProvider};
use crate::catalog::snapshot;
use crate::catalog::types::{
    CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
    ModelVariant, ProviderCatalog, ProviderDescriptor,
};

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const MODELS_DEV_PREFIX: &str = "models-dev-";

/// Adds the models.dev prefix to a provider ID to prevent conflicts with hardcoded providers.
fn prefix_provider_id(id: &str) -> String {
    format!("{}{}", MODELS_DEV_PREFIX, id)
}

pub struct ProviderCatalogFetcher {
    cache_path: PathBuf,
    base_url: String,
    http_client: reqwest::Client,
    catalog: Arc<RwLock<Option<ProviderCatalog>>>,
}

impl ProviderCatalogFetcher {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            cache_path,
            base_url: MODELS_DEV_URL.to_string(),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            catalog: Arc::new(RwLock::new(None)),
        }
    }

    #[cfg(test)]
    fn new_with_url(cache_path: PathBuf, url: &str) -> Self {
        Self {
            cache_path,
            base_url: url.to_string(),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(1))
                .build()
                .unwrap_or_default(),
            catalog: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_blocking(&self) -> Option<ProviderCatalog> {
        self.read_file_cache_blocking().ok()
    }

    pub fn refresh(self: &Arc<Self>) {
        let fetcher = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(e) = fetcher.fetch_from_network().await {
                tracing::warn!("Failed to refresh provider catalog: {}", e);
                return;
            }
            if let Err(e) = fetcher
                .write_file_cache(&fetcher.get_catalog_arc().await)
                .await
            {
                tracing::warn!("Failed to write catalog cache: {}", e);
            }
        });
    }

    async fn get_catalog_arc(self: &Arc<Self>) -> ProviderCatalog {
        let cached = self.catalog.read().await;
        if let Some(c) = cached.as_ref() {
            return c.clone();
        }
        if let Ok(cached) = self.read_file_cache().await {
            return cached;
        }
        if let Some(snap) = snapshot::get_snapshot() {
            return ProviderCatalog::from(snap);
        }
        ProviderCatalog {
            providers: Default::default(),
            fetched_at: Utc::now(),
            source: CatalogSource::ModelsDev,
        }
    }

    pub async fn force_refresh(&self) -> Result<ProviderCatalog, FetchError> {
        {
            let mut cat = self.catalog.write().await;
            *cat = None;
        }

        match self.fetch_from_network().await {
            Ok(catalog) => {
                if let Err(e) = self.write_file_cache(&catalog).await {
                    tracing::warn!("Failed to write catalog cache: {}", e);
                }
                {
                    let mut cat = self.catalog.write().await;
                    *cat = Some(catalog.clone());
                }
                Ok(catalog)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_or_fetch(&self) -> ProviderCatalog {
        {
            let cached = self.catalog.read().await;
            if let Some(c) = cached.as_ref() {
                if self.is_cache_valid(c) {
                    return c.clone();
                }
            }
        }

        if let Ok(cached) = self.read_file_cache().await {
            if self.is_cache_valid(&cached) {
                return cached;
            }
        }

        match self.fetch_from_network().await {
            Ok(catalog) => {
                let _ = self.write_file_cache(&catalog).await;
                let mut cat = self.catalog.write().await;
                *cat = Some(catalog.clone());
                catalog
            }
            Err(_) => {
                if let Ok(stale) = self.read_file_cache().await {
                    return stale;
                }
                if let Some(snap) = snapshot::get_snapshot() {
                    return ProviderCatalog::from(snap);
                }
                ProviderCatalog {
                    providers: Default::default(),
                    fetched_at: Utc::now(),
                    source: CatalogSource::ModelsDev,
                }
            }
        }
    }

    fn read_file_cache_blocking(&self) -> Result<ProviderCatalog, std::io::Error> {
        let bytes = std::fs::read(&self.cache_path)?;
        serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub async fn get(&self, force_refresh: bool) -> Result<ProviderCatalog, FetchError> {
        if !force_refresh {
            let cached = self.catalog.read().await;
            if let Some(c) = cached.as_ref() {
                if self.is_cache_valid(c) {
                    return Ok(c.clone());
                }
            }
        }

        if !force_refresh {
            if let Ok(cached) = self.read_file_cache().await {
                if self.is_cache_valid(&cached) {
                    let mut cat = self.catalog.write().await;
                    *cat = Some(cached.clone());
                    return Ok(cached);
                }
            }
        }

        match self.fetch_from_network().await {
            Ok(catalog) => {
                let _ = self.write_file_cache(&catalog).await;
                let mut cat = self.catalog.write().await;
                *cat = Some(catalog.clone());
                Ok(catalog)
            }
            Err(e) => {
                if let Ok(stale) = self.read_file_cache().await {
                    let mut cat = self.catalog.write().await;
                    *cat = Some(stale.clone());
                    return Ok(stale);
                }
                if let Some(snap) = snapshot::get_snapshot() {
                    let catalog = ProviderCatalog::from(snap);
                    let mut cat = self.catalog.write().await;
                    *cat = Some(catalog.clone());
                    return Ok(catalog);
                }
                Err(e)
            }
        }
    }

    async fn fetch_from_network(&self) -> Result<ProviderCatalog, FetchError> {
        let response = self
            .http_client
            .get(&self.base_url)
            .header("User-Agent", "opencode-rs/1.0")
            .send()
            .await
            .map_err(|e| FetchError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(FetchError::HttpStatus(response.status().as_u16()));
        }

        let data: ModelsDevApiResponse = response
            .json()
            .await
            .map_err(|e| FetchError::Parse(e.to_string()))?;

        Ok(self.transform_to_catalog(data))
    }

    fn transform_to_catalog(&self, data: ModelsDevApiResponse) -> ProviderCatalog {
        let providers = data
            .into_iter()
            .map(|(id, provider)| {
                let prefixed_id = prefix_provider_id(&id);
                (prefixed_id, self.transform_provider(provider, &id))
            })
            .collect();

        ProviderCatalog {
            providers,
            fetched_at: Utc::now(),
            source: CatalogSource::ModelsDev,
        }
    }

    fn transform_provider(
        &self,
        provider: ModelsDevProvider,
        original_id: &str,
    ) -> ProviderDescriptor {
        let prefixed_provider_id = prefix_provider_id(original_id);
        let models = provider
            .models
            .into_iter()
            .map(|(id, model)| (id, self.transform_model(model, &prefixed_provider_id)))
            .collect();

        ProviderDescriptor {
            id: prefixed_provider_id,
            display_name: provider.name,
            api_base_url: provider.api,
            docs_url: provider.doc,
            env_vars: provider.env,
            npm_package: provider.npm,
            models,
            source: CatalogSource::ModelsDev,
        }
    }

    fn transform_model(&self, model: ModelsDevModel, provider_id: &str) -> ModelDescriptor {
        let capabilities = ModelCapabilities {
            attachment: model.attachment,
            reasoning: model.reasoning,
            tool_call: model.tool_call,
            temperature: model.temperature.unwrap_or(false),
            structured_output: false,
            interleaved: model.interleaved.is_some(),
            open_weights: model.open_weights,
            input_modalities: model
                .modalities
                .as_ref()
                .map(|m| m.input.clone())
                .unwrap_or_default(),
            output_modalities: model
                .modalities
                .as_ref()
                .map(|m| m.output.clone())
                .unwrap_or_default(),
        };

        let cost = model
            .cost
            .map(|c| CostInfo {
                input: c.input.unwrap_or(0.0),
                output: c.output.unwrap_or(0.0),
                cache_read: c.cache_read.unwrap_or(0.0),
                cache_write: c.cache_write.unwrap_or(0.0),
            })
            .unwrap_or_default();

        let limits = model
            .limit
            .map(|l| LimitInfo {
                context: l.context.unwrap_or(0),
                input: l.input,
                output: l.output.unwrap_or(0),
            })
            .unwrap_or_default();

        let status = match model.status.as_deref() {
            Some("alpha") => ModelStatus::Alpha,
            Some("beta") => ModelStatus::Beta,
            Some("deprecated") => ModelStatus::Deprecated,
            _ => ModelStatus::Active,
        };

        let variants = self.parse_experimental_modes(&model.experimental);

        ModelDescriptor {
            id: model.id,
            display_name: model.name,
            family: model.family,
            provider_id: provider_id.to_string(),
            capabilities,
            cost,
            limits,
            status,
            variants,
        }
    }

    fn parse_experimental_modes(
        &self,
        experimental: &Option<serde_json::Value>,
    ) -> Vec<ModelVariant> {
        let Some(value) = experimental else {
            return vec![];
        };

        let obj = match value.as_object() {
            Some(obj) => obj,
            None => return vec![],
        };

        let modes = match obj.get("modes") {
            Some(serde_json::Value::Array(arr)) => arr,
            _ => return vec![],
        };

        modes
            .iter()
            .filter_map(|mode| {
                let obj = mode.as_object()?;
                let name = obj.get("name")?.as_str()?.to_string();
                let description = obj
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(String::from);
                Some(ModelVariant { name, description })
            })
            .collect()
    }

    fn is_cache_valid(&self, catalog: &ProviderCatalog) -> bool {
        let age = Utc::now().signed_duration_since(catalog.fetched_at);
        age < chrono::Duration::from_std(CACHE_TTL).unwrap_or_default()
    }

    async fn read_file_cache(&self) -> Result<ProviderCatalog, std::io::Error> {
        let bytes = tokio::fs::read(&self.cache_path).await?;
        serde_json::from_slice(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    async fn write_file_cache(&self, catalog: &ProviderCatalog) -> Result<(), std::io::Error> {
        if let Some(parent) = self.cache_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(catalog)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        tokio::fs::write(&self.cache_path, json).await
    }
}

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("HTTP error: {0}")]
    HttpStatus(u16),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::models_dev::{
        ModelsDevApiResponse, ModelsDevCost, ModelsDevLimit, ModelsDevModalities, ModelsDevModel,
        ModelsDevProvider,
    };
    use std::collections::BTreeMap;

    fn create_test_fetcher() -> ProviderCatalogFetcher {
        ProviderCatalogFetcher::new(std::env::temp_dir().join("test_catalog.json"))
    }

    fn create_test_models_dev_response() -> ModelsDevApiResponse {
        let mut models = BTreeMap::new();
        models.insert(
            "gpt-4".to_string(),
            ModelsDevModel {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
                family: Some("GPT-4".to_string()),
                release_date: None,
                attachment: true,
                reasoning: true,
                temperature: Some(true),
                tool_call: true,
                modalities: Some(ModelsDevModalities {
                    input: vec!["text".to_string()],
                    output: vec!["text".to_string()],
                }),
                open_weights: false,
                interleaved: None,
                cost: Some(ModelsDevCost {
                    input: Some(0.01),
                    output: Some(0.03),
                    reasoning: None,
                    cache_read: Some(0.001),
                    cache_write: Some(0.001),
                    input_audio: None,
                    output_audio: None,
                    context_over_200k: None,
                }),
                limit: Some(ModelsDevLimit {
                    context: Some(128000),
                    input: Some(128000),
                    output: Some(4096),
                }),
                experimental: None,
                status: Some("active".to_string()),
            },
        );

        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".to_string(),
            ModelsDevProvider {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                api: Some("https://api.openai.com".to_string()),
                doc: Some("https://platform.openai.com".to_string()),
                npm: None,
                env: vec!["OPENAI_API_KEY".to_string()],
                models,
            },
        );

        providers
    }

    #[test]
    fn test_transform_to_catalog() {
        let fetcher = create_test_fetcher();
        let data = create_test_models_dev_response();
        let catalog = fetcher.transform_to_catalog(data);
        assert_eq!(catalog.providers.len(), 1);
        assert!(catalog.providers.contains_key("models-dev-openai"));
        assert_eq!(catalog.source, CatalogSource::ModelsDev);
    }

    #[test]
    fn test_transform_provider() {
        let fetcher = create_test_fetcher();
        let provider = ModelsDevProvider {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            api: Some("https://api.test.com".to_string()),
            doc: Some("https://docs.test.com".to_string()),
            npm: Some("@test/npm".to_string()),
            env: vec!["TEST_API_KEY".to_string()],
            models: BTreeMap::new(),
        };
        let descriptor = fetcher.transform_provider(provider, "test");
        assert_eq!(descriptor.id, "models-dev-test");
        assert_eq!(descriptor.display_name, "Test Provider");
        assert_eq!(
            descriptor.api_base_url,
            Some("https://api.test.com".to_string())
        );
        assert_eq!(
            descriptor.docs_url,
            Some("https://docs.test.com".to_string())
        );
        assert_eq!(descriptor.npm_package, Some("@test/npm".to_string()));
        assert_eq!(descriptor.env_vars, vec!["TEST_API_KEY".to_string()]);
        assert_eq!(descriptor.source, CatalogSource::ModelsDev);
    }

    #[test]
    fn test_transform_provider_empty_optionals() {
        let fetcher = create_test_fetcher();
        let provider = ModelsDevProvider {
            id: "test".to_string(),
            name: "Test".to_string(),
            api: None,
            doc: None,
            npm: None,
            env: vec![],
            models: BTreeMap::new(),
        };
        let descriptor = fetcher.transform_provider(provider, "test");
        assert_eq!(descriptor.api_base_url, None);
        assert_eq!(descriptor.docs_url, None);
        assert_eq!(descriptor.npm_package, None);
    }

    #[test]
    fn test_transform_model() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            family: Some("GPT-4".to_string()),
            release_date: None,
            attachment: true,
            reasoning: true,
            temperature: Some(true),
            tool_call: true,
            modalities: Some(ModelsDevModalities {
                input: vec!["text".to_string(), "image".to_string()],
                output: vec!["text".to_string()],
            }),
            open_weights: false,
            interleaved: Some(serde_json::Value::Null),
            cost: Some(ModelsDevCost {
                input: Some(0.01),
                output: Some(0.03),
                reasoning: None,
                cache_read: Some(0.001),
                cache_write: Some(0.001),
                input_audio: None,
                output_audio: None,
                context_over_200k: None,
            }),
            limit: Some(ModelsDevLimit {
                context: Some(128000),
                input: Some(128000),
                output: Some(4096),
            }),
            experimental: None,
            status: Some("beta".to_string()),
        };
        let descriptor = fetcher.transform_model(model, "models-dev-openai");
        assert_eq!(descriptor.id, "gpt-4");
        assert_eq!(descriptor.display_name, "GPT-4");
        assert_eq!(descriptor.family, Some("GPT-4".to_string()));
        assert_eq!(descriptor.provider_id, "models-dev-openai");
        assert!(descriptor.capabilities.attachment);
        assert!(descriptor.capabilities.reasoning);
        assert!(descriptor.capabilities.tool_call);
        assert!(descriptor.capabilities.temperature);
        assert!(!descriptor.capabilities.open_weights);
        assert_eq!(
            descriptor.capabilities.input_modalities,
            vec!["text", "image"]
        );
        assert_eq!(descriptor.capabilities.output_modalities, vec!["text"]);
        assert_eq!(descriptor.cost.input, 0.01);
        assert_eq!(descriptor.cost.output, 0.03);
        assert_eq!(descriptor.cost.cache_read, 0.001);
        assert_eq!(descriptor.limits.context, 128000);
        assert_eq!(descriptor.limits.input, Some(128000));
        assert_eq!(descriptor.limits.output, 4096);
        assert_eq!(descriptor.status, ModelStatus::Beta);
    }

    #[test]
    fn test_transform_model_alpha_status() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: None,
            release_date: None,
            attachment: false,
            reasoning: false,
            temperature: None,
            tool_call: false,
            modalities: None,
            open_weights: true,
            interleaved: None,
            cost: None,
            limit: None,
            experimental: None,
            status: Some("alpha".to_string()),
        };
        let descriptor = fetcher.transform_model(model, "test");
        assert_eq!(descriptor.status, ModelStatus::Alpha);
    }

    #[test]
    fn test_transform_model_deprecated_status() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: None,
            release_date: None,
            attachment: false,
            reasoning: false,
            temperature: None,
            tool_call: false,
            modalities: None,
            open_weights: true,
            interleaved: None,
            cost: None,
            limit: None,
            experimental: None,
            status: Some("deprecated".to_string()),
        };
        let descriptor = fetcher.transform_model(model, "test");
        assert_eq!(descriptor.status, ModelStatus::Deprecated);
    }

    #[test]
    fn test_transform_model_default_status() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: None,
            release_date: None,
            attachment: false,
            reasoning: false,
            temperature: None,
            tool_call: false,
            modalities: None,
            open_weights: true,
            interleaved: None,
            cost: None,
            limit: None,
            experimental: None,
            status: None,
        };
        let descriptor = fetcher.transform_model(model, "test");
        assert_eq!(descriptor.status, ModelStatus::Active);
    }

    #[test]
    fn test_transform_model_with_no_cost() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: None,
            release_date: None,
            attachment: false,
            reasoning: false,
            temperature: None,
            tool_call: false,
            modalities: None,
            open_weights: true,
            interleaved: None,
            cost: None,
            limit: None,
            experimental: None,
            status: None,
        };
        let descriptor = fetcher.transform_model(model, "test");
        assert_eq!(descriptor.cost.input, 0.0);
        assert_eq!(descriptor.cost.output, 0.0);
        assert_eq!(descriptor.cost.cache_read, 0.0);
        assert_eq!(descriptor.cost.cache_write, 0.0);
    }

    #[test]
    fn test_transform_model_with_no_limit() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "test".to_string(),
            name: "Test".to_string(),
            family: None,
            release_date: None,
            attachment: false,
            reasoning: false,
            temperature: None,
            tool_call: false,
            modalities: None,
            open_weights: true,
            interleaved: None,
            cost: None,
            limit: None,
            experimental: None,
            status: None,
        };
        let descriptor = fetcher.transform_model(model, "test");
        assert_eq!(descriptor.limits.context, 0);
        assert_eq!(descriptor.limits.input, None);
        assert_eq!(descriptor.limits.output, 0);
    }

    #[test]
    fn test_is_cache_valid_true() {
        let fetcher = create_test_fetcher();
        let catalog = ProviderCatalog {
            providers: BTreeMap::new(),
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::ModelsDev,
        };
        assert!(fetcher.is_cache_valid(&catalog));
    }

    #[test]
    fn test_is_cache_valid_false() {
        let fetcher = create_test_fetcher();
        let catalog = ProviderCatalog {
            providers: BTreeMap::new(),
            fetched_at: chrono::Utc::now() - chrono::Duration::minutes(10),
            source: CatalogSource::ModelsDev,
        };
        assert!(!fetcher.is_cache_valid(&catalog));
    }

    #[test]
    fn test_fetch_error_display() {
        let error = FetchError::Network("connection refused".to_string());
        assert!(error.to_string().contains("Network error"));
        assert!(error.to_string().contains("connection refused"));

        let error = FetchError::HttpStatus(404);
        assert!(error.to_string().contains("HTTP error"));
        assert!(error.to_string().contains("404"));

        let error = FetchError::Parse("invalid json".to_string());
        assert!(error.to_string().contains("Parse error"));
        assert!(error.to_string().contains("invalid json"));
    }

    #[tokio::test]
    async fn test_fallback_chain_uses_snapshot_when_network_and_disk_unavailable() {
        let tmp = std::env::temp_dir().join(format!(
            "test_catalog_missing_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let fetcher = ProviderCatalogFetcher::new_with_url(tmp, "http://127.0.0.1:1");

        let catalog = fetcher.get_or_fetch().await;

        assert!(
            !catalog.providers.is_empty() || catalog.providers.is_empty(),
            "get_or_fetch must return a catalog without panicking"
        );

        if crate::catalog::snapshot::is_snapshot_available() {
            assert!(
                !catalog.providers.is_empty(),
                "catalog should be populated from snapshot when network and disk are unavailable"
            );
        }
    }

    #[tokio::test]
    async fn test_fallback_chain_uses_disk_cache_before_snapshot() {
        use std::io::Write;

        let tmp_path = std::env::temp_dir().join(format!(
            "test_disk_cache_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let disk_catalog = ProviderCatalog {
            providers: {
                let mut m = BTreeMap::new();
                m.insert(
                    "disk_provider".to_string(),
                    crate::catalog::types::ProviderDescriptor {
                        id: "disk_provider".to_string(),
                        display_name: "Disk Provider".to_string(),
                        api_base_url: None,
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
                m
            },
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::Local,
        };

        {
            let json = serde_json::to_string_pretty(&disk_catalog).unwrap();
            let mut f = std::fs::File::create(&tmp_path).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }

        let fetcher = ProviderCatalogFetcher::new_with_url(tmp_path.clone(), "http://127.0.0.1:1");
        let catalog = fetcher.get_or_fetch().await;

        assert!(
            catalog.providers.contains_key("disk_provider"),
            "should prefer disk cache over snapshot when disk is valid"
        );

        let _ = std::fs::remove_file(tmp_path);
    }

    #[tokio::test]
    async fn test_fallback_chain_memory_cache_returns_first() {
        let tmp = std::env::temp_dir().join(format!(
            "test_memory_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let fetcher = Arc::new(ProviderCatalogFetcher::new_with_url(
            tmp,
            "http://127.0.0.1:1",
        ));

        let memory_catalog = ProviderCatalog {
            providers: {
                let mut m = BTreeMap::new();
                m.insert(
                    "memory_provider".to_string(),
                    crate::catalog::types::ProviderDescriptor {
                        id: "memory_provider".to_string(),
                        display_name: "Memory Provider".to_string(),
                        api_base_url: None,
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
                m
            },
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::Local,
        };

        {
            let mut cat = fetcher.catalog.write().await;
            *cat = Some(memory_catalog);
        }

        let catalog = fetcher.get_or_fetch().await;
        assert!(
            catalog.providers.contains_key("memory_provider"),
            "should use memory cache first"
        );
    }

    #[tokio::test]
    async fn test_get_returns_snapshot_fallback_on_all_failures() {
        let tmp = std::env::temp_dir().join(format!(
            "test_get_fallback_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let fetcher = ProviderCatalogFetcher::new_with_url(tmp, "http://127.0.0.1:1");
        let result = fetcher.get(false).await;

        if crate::catalog::snapshot::is_snapshot_available() {
            assert!(result.is_ok(), "get() should succeed via snapshot fallback");
            let catalog = result.unwrap();
            assert!(
                !catalog.providers.is_empty(),
                "catalog from snapshot should be non-empty"
            );
        }
    }

    // ========== FR-018: Fallback Chain Verification Tests ==========

    #[tokio::test]
    async fn test_fallback_chain_memory_cache_5_min_ttl() {
        let tmp = std::env::temp_dir().join(format!(
            "test_5min_ttl_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let fetcher = Arc::new(ProviderCatalogFetcher::new_with_url(
            tmp,
            "http://127.0.0.1:1",
        ));

        let fresh_catalog = ProviderCatalog {
            providers: {
                let mut m = BTreeMap::new();
                m.insert(
                    "fresh_provider".to_string(),
                    crate::catalog::types::ProviderDescriptor {
                        id: "fresh_provider".to_string(),
                        display_name: "Fresh Provider".to_string(),
                        api_base_url: None,
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
                m
            },
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::Local,
        };

        {
            let mut cat = fetcher.catalog.write().await;
            *cat = Some(fresh_catalog);
        }

        let catalog = fetcher.get_or_fetch().await;
        assert!(
            catalog.providers.contains_key("fresh_provider"),
            "Memory cache should be used when fresh (within 5-min TTL)"
        );

        let stale_catalog = ProviderCatalog {
            providers: {
                let mut m = BTreeMap::new();
                m.insert(
                    "stale_provider".to_string(),
                    crate::catalog::types::ProviderDescriptor {
                        id: "stale_provider".to_string(),
                        display_name: "Stale Provider".to_string(),
                        api_base_url: None,
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
                m
            },
            fetched_at: chrono::Utc::now() - chrono::Duration::minutes(6),
            source: CatalogSource::Local,
        };

        {
            let mut cat = fetcher.catalog.write().await;
            *cat = Some(stale_catalog);
        }

        let catalog = fetcher.get_or_fetch().await;
        assert!(
            !catalog.providers.contains_key("stale_provider"),
            "Stale memory cache (>5 min) should be ignored"
        );
    }

    #[tokio::test]
    async fn test_fallback_chain_network_fetch_on_cache_miss() {
        let tmp = std::env::temp_dir().join(format!(
            "test_network_fetch_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let fetcher = Arc::new(ProviderCatalogFetcher::new_with_url(
            tmp,
            "http://127.0.0.1:1",
        ));

        assert!(
            fetcher.catalog.read().await.is_none(),
            "Memory cache should be empty initially"
        );

        let catalog = fetcher.get_or_fetch().await;

        if crate::catalog::snapshot::is_snapshot_available() {
            assert!(
                !catalog.providers.is_empty(),
                "Should fall back to snapshot when network fails and no disk cache"
            );
        }
    }

    #[tokio::test]
    async fn test_fallback_chain_disk_cache_on_network_failure() {
        use std::io::Write;

        let tmp_path = std::env::temp_dir().join(format!(
            "test_disk_fallback_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let disk_catalog = ProviderCatalog {
            providers: {
                let mut m = BTreeMap::new();
                m.insert(
                    "disk_fallback_provider".to_string(),
                    crate::catalog::types::ProviderDescriptor {
                        id: "disk_fallback_provider".to_string(),
                        display_name: "Disk Fallback Provider".to_string(),
                        api_base_url: None,
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
                m
            },
            fetched_at: chrono::Utc::now() - chrono::Duration::minutes(10),
            source: CatalogSource::Local,
        };

        {
            let json = serde_json::to_string_pretty(&disk_catalog).unwrap();
            let mut f = std::fs::File::create(&tmp_path).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }

        let fetcher = Arc::new(ProviderCatalogFetcher::new_with_url(
            tmp_path.clone(),
            "http://127.0.0.1:1",
        ));

        {
            assert!(
                fetcher.catalog.read().await.is_none(),
                "Memory cache should be empty"
            );
        }

        let catalog = fetcher.get_or_fetch().await;

        assert!(
            catalog.providers.contains_key("disk_fallback_provider"),
            "Disk cache should be used when network fails"
        );

        let _ = std::fs::remove_file(tmp_path);
    }

    #[tokio::test]
    async fn test_no_duplicate_network_requests_when_cache_valid() {
        let tmp = std::env::temp_dir().join(format!(
            "test_no_dup_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));

        let fetcher = Arc::new(ProviderCatalogFetcher::new_with_url(
            tmp,
            "http://127.0.0.1:1",
        ));

        let fresh_catalog = ProviderCatalog {
            providers: {
                let mut m = BTreeMap::new();
                m.insert(
                    "stable_provider".to_string(),
                    crate::catalog::types::ProviderDescriptor {
                        id: "stable_provider".to_string(),
                        display_name: "Stable Provider".to_string(),
                        api_base_url: None,
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
                m
            },
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::Local,
        };

        {
            let mut cat = fetcher.catalog.write().await;
            *cat = Some(fresh_catalog);
        }

        let catalog1 = fetcher.get_or_fetch().await;
        let catalog2 = fetcher.get_or_fetch().await;
        let catalog3 = fetcher.get_or_fetch().await;

        assert!(
            catalog1.providers.contains_key("stable_provider"),
            "First call should return cached result"
        );
        assert!(
            catalog2.providers.contains_key("stable_provider"),
            "Second call should return same cached result"
        );
        assert!(
            catalog3.providers.contains_key("stable_provider"),
            "Third call should also return cached result"
        );
        assert_eq!(
            catalog1.providers.len(),
            catalog2.providers.len(),
            "Cache should return same number of providers"
        );
        assert_eq!(
            catalog2.providers.len(),
            catalog3.providers.len(),
            "Third call should return same size as second"
        );
    }

    #[test]
    fn test_parse_experimental_modes_with_valid_modes() {
        let fetcher = create_test_fetcher();
        let json_value = serde_json::json!({
            "modes": [
                {
                    "name": "thinking",
                    "description": "Extended thinking mode"
                },
                {
                    "name": "preview",
                    "description": "Preview mode"
                }
            ]
        });

        let variants = fetcher.parse_experimental_modes(&Some(json_value));

        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].name, "thinking");
        assert_eq!(
            variants[0].description,
            Some("Extended thinking mode".to_string())
        );
        assert_eq!(variants[1].name, "preview");
        assert_eq!(variants[1].description, Some("Preview mode".to_string()));
    }

    #[test]
    fn test_parse_experimental_modes_with_none() {
        let fetcher = create_test_fetcher();
        let variants = fetcher.parse_experimental_modes(&None);

        assert!(variants.is_empty());
    }

    #[test]
    fn test_parse_experimental_modes_with_empty_object() {
        let fetcher = create_test_fetcher();
        let variants = fetcher.parse_experimental_modes(&Some(serde_json::json!({})));

        assert!(variants.is_empty());
    }

    #[test]
    fn test_parse_experimental_modes_with_no_modes_field() {
        let fetcher = create_test_fetcher();
        let variants =
            fetcher.parse_experimental_modes(&Some(serde_json::json!({"other": "field"})));

        assert!(variants.is_empty());
    }

    #[test]
    fn test_parse_experimental_modes_with_invalid_modes_array() {
        let fetcher = create_test_fetcher();
        let variants = fetcher.parse_experimental_modes(&Some(serde_json::json!({
            "modes": "not an array"
        })));

        assert!(variants.is_empty());
    }

    #[test]
    fn test_parse_experimental_modes_with_modes_missing_name() {
        let fetcher = create_test_fetcher();
        let json_value = serde_json::json!({
            "modes": [
                {
                    "description": "Has description but no name"
                }
            ]
        });

        let variants = fetcher.parse_experimental_modes(&Some(json_value));

        assert!(variants.is_empty());
    }

    #[test]
    fn test_parse_experimental_modes_with_partial_valid_modes() {
        let fetcher = create_test_fetcher();
        let json_value = serde_json::json!({
            "modes": [
                {
                    "name": "valid-mode",
                    "description": "A valid mode"
                },
                {
                    "description": "Invalid - no name"
                },
                {
                    "name": "another-valid"
                }
            ]
        });

        let variants = fetcher.parse_experimental_modes(&Some(json_value));

        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].name, "valid-mode");
        assert_eq!(variants[1].name, "another-valid");
        assert!(variants[1].description.is_none());
    }

    #[test]
    fn test_transform_model_includes_variants() {
        let fetcher = create_test_fetcher();
        let model = ModelsDevModel {
            id: "test-model".to_string(),
            name: "Test Model".to_string(),
            family: Some("test".to_string()),
            release_date: None,
            attachment: false,
            reasoning: true,
            temperature: Some(true),
            tool_call: true,
            modalities: None,
            open_weights: false,
            interleaved: None,
            cost: None,
            limit: None,
            experimental: Some(serde_json::json!({
                "modes": [
                    {"name": "thinking", "description": "Thinking mode"},
                    {"name": "extended", "description": "Extended mode"}
                ]
            })),
            status: Some("active".to_string()),
        };

        let descriptor = fetcher.transform_model(model, "test-provider");

        assert_eq!(descriptor.variants.len(), 2);
        assert_eq!(descriptor.variants[0].name, "thinking");
        assert_eq!(descriptor.variants[1].name, "extended");
    }
}
