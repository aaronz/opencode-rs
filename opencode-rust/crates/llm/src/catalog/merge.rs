use std::collections::BTreeMap;

use opencode_config::ProviderConfig;

use crate::catalog::types::{
    CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
    ProviderCatalog, ProviderDescriptor,
};
use crate::gitlab::GitLabProvider;

pub struct CatalogMerger {
    catalog: ProviderCatalog,
}

impl CatalogMerger {
    pub fn new(catalog: ProviderCatalog) -> Self {
        Self { catalog }
    }

    pub fn with_local_providers(mut self) -> Self {
        self.add_local_providers();
        self
    }

    pub fn with_config_overrides(
        mut self,
        config_providers: &BTreeMap<String, ProviderConfig>,
    ) -> Self {
        for (provider_id, config) in config_providers {
            let provider_id = provider_id.clone();
            if let Some(existing) = self.catalog.providers.get_mut(&provider_id) {
                Self::apply_overrides_to_provider(existing, config);
            } else if let Some(synthetic) = Self::create_synthetic_provider(&provider_id, config) {
                self.catalog.providers.insert(provider_id, synthetic);
            }
        }
        self
    }

    fn apply_overrides_to_provider(provider: &mut ProviderDescriptor, config: &ProviderConfig) {
        if let Some(name) = &config.name {
            provider.display_name = name.clone();
        }
        if let Some(base_url) = config.options.as_ref().and_then(|o| o.base_url.as_ref()) {
            provider.api_base_url = Some(base_url.clone());
        }
        if let Some(models_override) = &config.models {
            let models_to_remove: Vec<String> = models_override
                .iter()
                .filter(|(_, model_config)| model_config.visible == Some(false))
                .map(|(model_id, _)| model_id.clone())
                .collect();
            for model_id in models_to_remove {
                provider.models.remove(&model_id);
            }
            for (model_id, model_config) in models_override {
                if let Some(existing_model) = provider.models.get_mut(model_id) {
                    if let Some(name) = &model_config.name {
                        existing_model.display_name = name.clone();
                    }
                }
            }
        }
    }

    fn create_synthetic_provider(
        provider_id: &str,
        config: &ProviderConfig,
    ) -> Option<ProviderDescriptor> {
        let name = config
            .name
            .clone()
            .unwrap_or_else(|| provider_id.to_string());
        let base_url = config
            .options
            .as_ref()
            .and_then(|o| o.base_url.clone())
            .or_else(|| {
                config
                    .id
                    .as_ref()
                    .map(|id| format!("https://api.example.com/v1/{}", id))
            });

        let mut models = BTreeMap::new();
        if let Some(models_config) = &config.models {
            for (model_id, model_config) in models_config {
                let visible = model_config.visible.unwrap_or(true);
                if !visible {
                    continue;
                }
                models.insert(
                    model_id.clone(),
                    ModelDescriptor {
                        id: model_id.clone(),
                        display_name: model_config
                            .name
                            .clone()
                            .unwrap_or_else(|| model_id.clone()),
                        family: None,
                        provider_id: provider_id.to_string(),
                        capabilities: ModelCapabilities::default(),
                        cost: CostInfo::default(),
                        limits: LimitInfo::default(),
                        status: ModelStatus::Active,
                        variants: vec![],
                    },
                );
            }
        }

        Some(ProviderDescriptor {
            id: provider_id.to_string(),
            display_name: name,
            api_base_url: base_url,
            docs_url: None,
            env_vars: vec![],
            npm_package: None,
            models,
            source: CatalogSource::Config,
        })
    }

    fn add_local_providers(&mut self) {
        let local_providers = vec![
            ("ollama", "Ollama", "http://localhost:11434"),
            ("lmstudio", "LM Studio", "http://localhost:1234"),
            ("local", "Local Inference", "http://localhost:8080"),
        ];

        for (id, name, base_url) in local_providers {
            if !self.catalog.providers.contains_key(id) {
                self.catalog.providers.insert(
                    id.to_string(),
                    ProviderDescriptor {
                        id: id.to_string(),
                        display_name: name.to_string(),
                        api_base_url: Some(base_url.to_string()),
                        docs_url: None,
                        env_vars: vec![],
                        npm_package: None,
                        models: BTreeMap::new(),
                        source: CatalogSource::Local,
                    },
                );
            }
        }
    }

    pub fn with_enabled_filter(mut self, enabled: Option<&[String]>, disabled: &[String]) -> Self {
        if let Some(enabled_list) = enabled {
            let enabled_set: std::collections::HashSet<_> = enabled_list.iter().collect();
            self.catalog
                .providers
                .retain(|id, _| enabled_set.contains(id));
        }
        for disabled_id in disabled {
            self.catalog.providers.remove(disabled_id);
        }
        self
    }

    pub async fn with_gitlab_providers(mut self, gitlab_providers: Vec<GitLabProvider>) -> Self {
        for provider in gitlab_providers {
            let models = match provider.discover_models().await {
                Ok(m) => m,
                Err(_) => continue,
            };

            let model_descriptors: BTreeMap<String, ModelDescriptor> = models
                .into_iter()
                .map(|m| {
                    (
                        m.id.clone(),
                        ModelDescriptor {
                            id: m.id.clone(),
                            display_name: m.name.clone(),
                            family: None,
                            provider_id: "gitlab".to_string(),
                            capabilities: ModelCapabilities::default(),
                            cost: CostInfo::default(),
                            limits: LimitInfo::default(),
                            status: ModelStatus::Active,
                            variants: vec![],
                        },
                    )
                })
                .collect();

            let provider_id = "gitlab".to_string();
            self.catalog.providers.insert(
                provider_id.clone(),
                ProviderDescriptor {
                    id: provider_id,
                    display_name: "GitLab".to_string(),
                    api_base_url: Some(provider.instance_url),
                    docs_url: None,
                    env_vars: vec![],
                    npm_package: None,
                    models: model_descriptors,
                    source: CatalogSource::Local,
                },
            );
        }
        self
    }

    pub fn build(self) -> ProviderCatalog {
        self.catalog
    }
}

pub fn merge_catalogs(
    models_dev: ProviderCatalog,
    config_providers: &BTreeMap<String, ProviderConfig>,
    enabled: Option<&[String]>,
    disabled: &[String],
) -> ProviderCatalog {
    CatalogMerger::new(models_dev)
        .with_local_providers()
        .with_config_overrides(config_providers)
        .with_enabled_filter(enabled, disabled)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::{
        CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
        ProviderCatalog, ProviderDescriptor,
    };
    use opencode_config::ProviderConfig;
    use std::collections::BTreeMap;

    fn create_test_catalog() -> ProviderCatalog {
        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderDescriptor {
                id: "openai".to_string(),
                display_name: "OpenAI".to_string(),
                api_base_url: Some("https://api.openai.com".to_string()),
                docs_url: None,
                env_vars: vec!["OPENAI_API_KEY".to_string()],
                npm_package: None,
                models: BTreeMap::new(),
                source: CatalogSource::ModelsDev,
            },
        );
        providers.insert(
            "anthropic".to_string(),
            ProviderDescriptor {
                id: "anthropic".to_string(),
                display_name: "Anthropic".to_string(),
                api_base_url: Some("https://api.anthropic.com".to_string()),
                docs_url: None,
                env_vars: vec!["ANTHROPIC_API_KEY".to_string()],
                npm_package: None,
                models: BTreeMap::new(),
                source: CatalogSource::ModelsDev,
            },
        );
        ProviderCatalog {
            providers,
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::ModelsDev,
        }
    }

    fn create_test_model(id: &str, display_name: &str) -> ModelDescriptor {
        ModelDescriptor {
            id: id.to_string(),
            display_name: display_name.to_string(),
            family: None,
            provider_id: "test".to_string(),
            capabilities: ModelCapabilities::default(),
            cost: CostInfo::default(),
            limits: LimitInfo::default(),
            status: ModelStatus::Active,
            variants: vec![],
        }
    }

    #[test]
    fn test_catalog_merger_new() {
        let catalog = create_test_catalog();
        let merger = CatalogMerger::new(catalog.clone());
        let result = merger.build();
        assert_eq!(result.providers.len(), catalog.providers.len());
    }

    #[test]
    fn test_catalog_merger_with_local_providers() {
        let catalog = create_test_catalog();
        let result = CatalogMerger::new(catalog).with_local_providers().build();
        assert!(result.providers.contains_key("ollama"));
        assert!(result.providers.contains_key("lmstudio"));
        assert!(result.providers.contains_key("local"));
    }

    #[test]
    fn test_catalog_merger_does_not_override_existing_providers() {
        let catalog = create_test_catalog();
        let result = CatalogMerger::new(catalog).with_local_providers().build();
        assert_eq!(result.providers.get("openai").unwrap().id, "openai");
    }

    #[test]
    fn test_catalog_merger_with_empty_config_overrides() {
        let catalog = create_test_catalog();
        let config_providers = BTreeMap::new();
        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        assert_eq!(result.providers.len(), 2);
    }

    #[test]
    fn test_catalog_merger_with_config_override_display_name() {
        let mut catalog = create_test_catalog();
        let mut models = BTreeMap::new();
        models.insert("gpt-4".to_string(), create_test_model("gpt-4", "GPT-4"));
        catalog.providers.get_mut("openai").unwrap().models = models;

        let mut config = ProviderConfig::default();
        config.name = Some("OpenAI Updated".to_string());

        let mut config_providers = BTreeMap::new();
        config_providers.insert("openai".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        assert_eq!(
            result.providers.get("openai").unwrap().display_name,
            "OpenAI Updated"
        );
    }

    #[test]
    fn test_catalog_merger_with_config_override_base_url() {
        let catalog = create_test_catalog();

        let mut config = ProviderConfig::default();
        let mut options = opencode_config::ProviderOptions::default();
        options.base_url = Some("https://custom.openai.com".to_string());
        config.options = Some(options);

        let mut config_providers = BTreeMap::new();
        config_providers.insert("openai".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        assert_eq!(
            result.providers.get("openai").unwrap().api_base_url,
            Some("https://custom.openai.com".to_string())
        );
    }

    #[test]
    fn test_catalog_merger_with_config_hides_models() {
        let mut catalog = create_test_catalog();
        let mut models = BTreeMap::new();
        models.insert("gpt-4".to_string(), create_test_model("gpt-4", "GPT-4"));
        models.insert(
            "gpt-3.5".to_string(),
            create_test_model("gpt-3.5", "GPT-3.5"),
        );
        catalog.providers.get_mut("openai").unwrap().models = models;

        let mut model_config = std::collections::HashMap::new();
        model_config.insert(
            "gpt-3.5".to_string(),
            opencode_config::ModelConfig {
                name: None,
                visible: Some(false),
                id: None,
                variants: None,
                extra: None,
            },
        );

        let mut config = ProviderConfig::default();
        config.models = Some(model_config);

        let mut config_providers = BTreeMap::new();
        config_providers.insert("openai".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        let openai_models = &result.providers.get("openai").unwrap().models;
        assert!(openai_models.contains_key("gpt-4"));
        assert!(!openai_models.contains_key("gpt-3.5"));
    }

    #[test]
    fn test_catalog_merger_creates_synthetic_provider() {
        let catalog = create_test_catalog();
        let mut config = ProviderConfig::default();
        config.name = Some("Custom Provider".to_string());
        let mut options = opencode_config::ProviderOptions::default();
        options.base_url = Some("https://custom.example.com".to_string());
        config.options = Some(options);

        let mut config_providers = BTreeMap::new();
        config_providers.insert("custom".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        assert!(result.providers.contains_key("custom"));
        assert_eq!(
            result.providers.get("custom").unwrap().display_name,
            "Custom Provider"
        );
    }

    #[test]
    fn test_catalog_merger_synthetic_provider_with_models() {
        let catalog = create_test_catalog();
        let mut model_config = std::collections::HashMap::new();
        model_config.insert(
            "custom-model".to_string(),
            opencode_config::ModelConfig {
                name: Some("Custom Model".to_string()),
                visible: None,
                id: None,
                variants: None,
                extra: None,
            },
        );

        let mut config = ProviderConfig::default();
        config.models = Some(model_config);

        let mut config_providers = BTreeMap::new();
        config_providers.insert("custom".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        let custom_models = &result.providers.get("custom").unwrap().models;
        assert!(custom_models.contains_key("custom-model"));
        assert_eq!(
            custom_models.get("custom-model").unwrap().display_name,
            "Custom Model"
        );
    }

    #[test]
    fn test_catalog_merger_synthetic_provider_hidden_models() {
        let catalog = create_test_catalog();
        let mut model_config = std::collections::HashMap::new();
        model_config.insert(
            "visible-model".to_string(),
            opencode_config::ModelConfig {
                name: None,
                visible: Some(true),
                id: None,
                variants: None,
                extra: None,
            },
        );
        model_config.insert(
            "hidden-model".to_string(),
            opencode_config::ModelConfig {
                name: None,
                visible: Some(false),
                id: None,
                variants: None,
                extra: None,
            },
        );

        let mut config = ProviderConfig::default();
        config.models = Some(model_config);

        let mut config_providers = BTreeMap::new();
        config_providers.insert("custom".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        let custom_models = result
            .providers
            .get("custom")
            .unwrap()
            .models
            .keys()
            .collect::<Vec<_>>();
        assert!(custom_models.contains(&&"visible-model".to_string()));
        assert!(!custom_models.contains(&&"hidden-model".to_string()));
    }

    #[test]
    fn test_catalog_merger_enabled_filter() {
        let catalog = create_test_catalog();
        let result = CatalogMerger::new(catalog)
            .with_enabled_filter(Some(&["openai".to_string()]), &[])
            .build();
        assert!(result.providers.contains_key("openai"));
        assert!(!result.providers.contains_key("anthropic"));
    }

    #[test]
    fn test_catalog_merger_disabled_filter() {
        let catalog = create_test_catalog();
        let result = CatalogMerger::new(catalog)
            .with_enabled_filter(None, &["anthropic".to_string()])
            .build();
        assert!(result.providers.contains_key("openai"));
        assert!(!result.providers.contains_key("anthropic"));
    }

    #[test]
    fn test_catalog_merger_enabled_and_disabled_together() {
        let mut catalog = create_test_catalog();
        catalog.providers.insert(
            "custom".to_string(),
            ProviderDescriptor {
                id: "custom".to_string(),
                display_name: "Custom".to_string(),
                api_base_url: None,
                docs_url: None,
                env_vars: vec![],
                npm_package: None,
                models: BTreeMap::new(),
                source: CatalogSource::Config,
            },
        );

        let result = CatalogMerger::new(catalog)
            .with_enabled_filter(
                Some(&["openai".to_string(), "custom".to_string()]),
                &["anthropic".to_string()],
            )
            .build();
        assert!(result.providers.contains_key("openai"));
        assert!(result.providers.contains_key("custom"));
        assert!(!result.providers.contains_key("anthropic"));
    }

    #[test]
    fn test_catalog_merger_empty_enabled_allows_non_disabled() {
        let catalog = create_test_catalog();
        let result = CatalogMerger::new(catalog)
            .with_enabled_filter(Some(&[]), &["anthropic".to_string()])
            .build();
        assert!(!result.providers.contains_key("anthropic"));
    }

    #[test]
    fn test_merge_catalogs_function() {
        let catalog = create_test_catalog();
        let config_providers = BTreeMap::new();
        let result = merge_catalogs(catalog, &config_providers, None, &["anthropic".to_string()]);
        assert!(result.providers.contains_key("openai"));
        assert!(!result.providers.contains_key("anthropic"));
        assert!(result.providers.contains_key("ollama"));
    }

    #[test]
    fn test_catalog_source_for_local_providers() {
        let catalog = create_test_catalog();
        let result = CatalogMerger::new(catalog).with_local_providers().build();
        assert_eq!(
            result.providers.get("ollama").unwrap().source,
            CatalogSource::Local
        );
    }

    #[test]
    fn test_catalog_source_for_synthetic_provider() {
        let catalog = create_test_catalog();
        let config = ProviderConfig::default();

        let mut config_providers = BTreeMap::new();
        config_providers.insert("newprovider".to_string(), config);

        let result = CatalogMerger::new(catalog)
            .with_config_overrides(&config_providers)
            .build();
        assert_eq!(
            result.providers.get("newprovider").unwrap().source,
            CatalogSource::Config
        );
    }
}
