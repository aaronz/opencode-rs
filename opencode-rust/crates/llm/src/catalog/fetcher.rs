use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::catalog::models_dev::{ModelsDevApiResponse, ModelsDevModel, ModelsDevProvider};
use crate::catalog::types::{
    CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
    ProviderCatalog, ProviderDescriptor,
};

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

pub struct ProviderCatalogFetcher {
    cache_path: PathBuf,
    http_client: reqwest::Client,
    catalog: Arc<RwLock<Option<ProviderCatalog>>>,
}

impl ProviderCatalogFetcher {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            cache_path,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
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
        ProviderCatalog {
            providers: Default::default(),
            fetched_at: Utc::now(),
            source: CatalogSource::ModelsDev,
        }
    }

    pub async fn get_or_fetch(&self) -> ProviderCatalog {
        if let Ok(cached) = self.read_file_cache().await {
            if self.is_cache_valid(&cached) {
                return cached;
            }
        }
        match self.fetch_from_network().await {
            Ok(catalog) => {
                let _ = self.write_file_cache(&catalog).await;
                catalog
            }
            Err(_) => self
                .read_file_cache()
                .await
                .unwrap_or_else(|_| ProviderCatalog {
                    providers: Default::default(),
                    fetched_at: Utc::now(),
                    source: CatalogSource::ModelsDev,
                }),
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
                Err(e)
            }
        }
    }

    async fn fetch_from_network(&self) -> Result<ProviderCatalog, FetchError> {
        let response = self
            .http_client
            .get(MODELS_DEV_URL)
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
            .map(|(id, provider)| (id, self.transform_provider(provider)))
            .collect();

        ProviderCatalog {
            providers,
            fetched_at: Utc::now(),
            source: CatalogSource::ModelsDev,
        }
    }

    fn transform_provider(&self, provider: ModelsDevProvider) -> ProviderDescriptor {
        let models = provider
            .models
            .into_iter()
            .map(|(id, model)| (id, self.transform_model(model, &provider.id)))
            .collect();

        ProviderDescriptor {
            id: provider.id,
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

        ModelDescriptor {
            id: model.id,
            display_name: model.name,
            family: model.family,
            provider_id: provider_id.to_string(),
            capabilities,
            cost,
            limits,
            status,
        }
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
        assert!(catalog.providers.contains_key("openai"));
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
        let descriptor = fetcher.transform_provider(provider);
        assert_eq!(descriptor.id, "test");
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
        let descriptor = fetcher.transform_provider(provider);
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
        let descriptor = fetcher.transform_model(model, "openai");
        assert_eq!(descriptor.id, "gpt-4");
        assert_eq!(descriptor.display_name, "GPT-4");
        assert_eq!(descriptor.family, Some("GPT-4".to_string()));
        assert_eq!(descriptor.provider_id, "openai");
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
}
