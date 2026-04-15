use std::collections::BTreeMap;

use opencode_config::ProviderConfig;

use crate::catalog::types::{
    CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
    ProviderCatalog, ProviderDescriptor,
};

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
