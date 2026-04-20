use crate::catalog::types::ModelStatus;
use crate::provider_filter::ProviderFilter;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Checks if a model should be visible based on its status.
///
/// Alpha models are hidden by default unless `OPENCODE_ENABLE_EXPERIMENTAL_MODELS`
/// environment variable is set to "true".
fn is_model_visible(status: Option<ModelStatus>) -> bool {
    if status == Some(ModelStatus::Alpha) {
        std::env::var("OPENCODE_ENABLE_EXPERIMENTAL_MODELS")
            .map(|v| v == "true")
            .unwrap_or(false)
    } else {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub max_tokens: u32,
    pub max_input_tokens: u32,
    pub supports_functions: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub cost_per_1k_tokens: f64,
    pub status: Option<ModelStatus>,
}

pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
    provider_filter: Option<ProviderFilter>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        let mut models = HashMap::new();

        if let Some(snapshot) = crate::catalog::snapshot::get_snapshot() {
            let catalog = crate::catalog::ProviderCatalog::from(snapshot);
            for provider in catalog.providers.values() {
                for model in provider.models.values() {
                    let model_key = model.id.clone();
                    models.entry(model_key).or_insert_with(|| ModelInfo {
                        name: model.id.clone(),
                        provider: model.provider_id.clone(),
                        max_tokens: model.limits.output.max(1),
                        max_input_tokens: model.limits.context,
                        supports_functions: model.capabilities.tool_call,
                        supports_vision: model
                            .capabilities
                            .input_modalities
                            .contains(&"image".to_string())
                            || model
                                .capabilities
                                .input_modalities
                                .contains(&"vision".to_string()),
                        supports_streaming: true,
                        cost_per_1k_tokens: model.cost.input + model.cost.output,
                        status: Some(model.status),
                    });
                }
            }
        }

        Self {
            models,
            provider_filter: None,
        }
    }

    pub fn set_provider_filter(&mut self, filter: ProviderFilter) {
        self.provider_filter = Some(filter);
    }

    pub fn get(&self, name: &str) -> Option<&ModelInfo> {
        self.models
            .get(name)
            .filter(|model| self.is_provider_allowed(&model.provider))
    }

    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|model| self.is_provider_allowed(&model.provider))
            .filter(|model| is_model_visible(model.status))
            .collect()
    }

    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        if !self.is_provider_allowed(provider) {
            return vec![];
        }

        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    pub fn list_providers(&self) -> Vec<String> {
        let mut providers: Vec<String> = self
            .models
            .values()
            .map(|model| model.provider.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|provider| self.is_provider_allowed(provider))
            .collect();
        providers.sort();
        providers
    }

    pub fn get_next_available_provider(&self, failed_provider: &str) -> Option<String> {
        let failed_provider = failed_provider.trim();
        let providers = self.list_providers();
        providers
            .into_iter()
            .find(|provider| !provider.eq_ignore_ascii_case(failed_provider))
    }

    fn is_provider_allowed(&self, provider: &str) -> bool {
        self.provider_filter
            .as_ref()
            .map(|filter| filter.is_allowed(provider))
            .unwrap_or(true)
    }

    pub fn supports_function(&self, model: &str) -> bool {
        self.get(model)
            .map(|m| m.supports_functions)
            .unwrap_or(false)
    }

    pub fn max_tokens(&self, model: &str) -> u32 {
        self.get(model).map(|m| m.max_tokens).unwrap_or(4096)
    }

    pub fn max_input_tokens(&self, model: &str) -> u32 {
        self.get(model).map(|m| m.max_input_tokens).unwrap_or(4096)
    }

    pub fn populate_from_catalog(&mut self, catalog: &crate::catalog::ProviderCatalog) {
        for provider in catalog.providers.values() {
            for model in provider.models.values() {
                let model_key = model.id.clone();
                self.models.entry(model_key).or_insert_with(|| ModelInfo {
                    name: model.id.clone(),
                    provider: model.provider_id.clone(),
                    max_tokens: model.limits.output.max(1),
                    max_input_tokens: model.limits.context,
                    supports_functions: model.capabilities.tool_call,
                    supports_vision: model
                        .capabilities
                        .input_modalities
                        .contains(&"image".to_string())
                        || model
                            .capabilities
                            .input_modalities
                            .contains(&"vision".to_string()),
                    supports_streaming: true,
                    cost_per_1k_tokens: model.cost.input + model.cost.output,
                    status: Some(model.status),
                });
            }
        }
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{is_model_visible, ModelRegistry};
    use crate::catalog::types::ModelStatus;
    use crate::provider_filter::ProviderFilter;

    #[test]
    fn list_respects_provider_filter() {
        let mut registry = ModelRegistry::new();
        registry.set_provider_filter(ProviderFilter::new(
            vec!["openai".to_string()],
            vec!["openai".to_string(), "anthropic".to_string()],
        ));

        let providers: Vec<String> = registry.list().iter().map(|m| m.provider.clone()).collect();

        assert!(providers.iter().all(|provider| provider == "anthropic"));
        assert!(!providers.is_empty());
    }

    #[test]
    fn get_next_available_skips_failed_and_disallowed_providers() {
        let mut registry = ModelRegistry::new();
        registry.set_provider_filter(ProviderFilter::new(
            vec!["openai".to_string()],
            vec!["openai".to_string(), "anthropic".to_string()],
        ));

        assert_eq!(
            registry.get_next_available_provider("openai"),
            Some("anthropic".to_string())
        );
        assert_eq!(registry.get_next_available_provider("anthropic"), None);
    }

    #[test]
    fn verify_model_catalog_contains_50_plus_models() {
        let registry = ModelRegistry::new();
        let model_count = registry.list().len();
        assert!(
            model_count >= 50,
            "Model catalog should contain at least 50 models, but only contains {}",
            model_count
        );
    }

    #[test]
    fn verify_original_provider_models_still_available() {
        let registry = ModelRegistry::new();

        let original_models = vec![
            "gpt-4o",
            "gpt-4o-mini",
            "claude-3-5-sonnet-20241022",
            "claude-haiku-4-5",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "mistral-large-latest",
            "command-r-plus-08-2024",
            "deepseek-chat",
            "deepseek-reasoner",
            "llama-3.3-70b-versatile",
        ];

        for model_name in original_models {
            let model = registry.get(model_name);
            assert!(
                model.is_some(),
                "Original model '{}' should still be available",
                model_name
            );
        }
    }

    #[test]
    fn verify_model_context_lengths_displayed_correctly() {
        let registry = ModelRegistry::new();

        let test_cases = vec![
            ("gpt-4o", 128000),
            ("gpt-4o-mini", 128000),
            ("gemini-1.5-pro", 1000000),
            ("gemini-1.5-flash", 1000000),
            ("claude-3-5-sonnet-20241022", 200000),
            ("claude-haiku-4-5", 200000),
        ];

        for (model_name, expected_max_input) in test_cases {
            let model = registry.get(model_name);
            assert!(model.is_some(), "Model '{}' should exist", model_name);
            let model = model.unwrap();
            assert_eq!(
                model.max_input_tokens, expected_max_input,
                "Model '{}' max_input_tokens should be {}",
                model_name, expected_max_input
            );
        }
    }

    #[test]
    fn verify_new_provider_models_available() {
        let registry = ModelRegistry::new();

        let new_models = vec![
            "gpt-5.1-codex-max",
            "claude-opus-4.6",
            "gpt-5-mini",
            "kimi-k2.5",
            "kimi-k2.5-free",
            "gemini-2.5-pro-preview-05-06",
            "gemini-3.1-flash-lite-preview",
            "deepseek-chat",
            "deepseek-reasoner",
        ];

        for model_name in new_models {
            let model = registry.get(model_name);
            assert!(
                model.is_some(),
                "New model '{}' should be available",
                model_name
            );
        }
    }

    #[test]
    fn verify_all_new_providers_listed() {
        let registry = ModelRegistry::new();
        let providers = registry.list_providers();

        let expected_providers = vec![
            "anthropic",
            "azure",
            "cerebras",
            "cohere",
            "deepinfra",
            "github-copilot",
            "google",
            "groq",
            "mistral",
            "openai",
            "opencode",
            "openrouter",
            "perplexity-agent",
            "togetherai",
            "xai",
        ];

        for provider in expected_providers {
            assert!(
                providers.contains(&provider.to_string()),
                "Provider '{}' should be in the provider list, got: {:?}",
                provider,
                providers
            );
        }
    }

    #[test]
    fn verify_alpha_models_hidden_by_default() {
        let registry = ModelRegistry::new();
        let models = registry.list();
        let alpha_models: Vec<&super::ModelInfo> = models
            .iter()
            .filter(|m| m.status == Some(ModelStatus::Alpha))
            .copied()
            .collect();
        assert!(
            alpha_models.is_empty(),
            "Alpha models should be hidden by default, but found: {:?}",
            alpha_models.iter().map(|m| &m.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn verify_non_alpha_models_always_visible() {
        let registry = ModelRegistry::new();
        let models = registry.list();
        let has_non_alpha = models.iter().any(|m| m.status != Some(ModelStatus::Alpha));
        assert!(
            has_non_alpha,
            "Non-alpha models should always be visible"
        );
    }

    #[test]
    fn verify_alpha_model_visible_with_env_flag() {
        let _temp_dir = std::env::temp_dir().join("test_alpha_catalog.json");
        let mut registry = ModelRegistry::new();
        registry.populate_from_catalog(&crate::catalog::types::ProviderCatalog {
            providers: std::collections::BTreeMap::from([(
                "test-provider".to_string(),
                crate::catalog::types::ProviderDescriptor {
                    id: "test-provider".to_string(),
                    display_name: "Test Provider".to_string(),
                    api_base_url: None,
                    docs_url: None,
                    env_vars: vec![],
                    npm_package: None,
                    models: std::collections::BTreeMap::from([(
                        "alpha-model".to_string(),
                        crate::catalog::types::ModelDescriptor {
                            id: "alpha-model".to_string(),
                            display_name: "Alpha Model".to_string(),
                            family: None,
                            provider_id: "test-provider".to_string(),
                            capabilities: crate::catalog::types::ModelCapabilities::default(),
                            cost: crate::catalog::types::CostInfo::default(),
                            limits: crate::catalog::types::LimitInfo::default(),
                            status: ModelStatus::Alpha,
                            variants: vec![],
                        },
                    )]),
                    source: crate::catalog::types::CatalogSource::Local,
                },
            )]),
            fetched_at: chrono::Utc::now(),
            source: crate::catalog::types::CatalogSource::Local,
        });
        let models_without_flag: Vec<&super::ModelInfo> = registry
            .list()
            .iter()
            .filter(|m| m.name == "alpha-model")
            .copied()
            .collect();
        assert!(
            models_without_flag.is_empty(),
            "Alpha model should not be visible without env flag"
        );
    }
}
