use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::catalog::snapshot_data::SNAPSHOT_JSON;
use crate::catalog::types::{CatalogSource, ProviderCatalog, ProviderDescriptor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotCatalog {
    pub providers: BTreeMap<String, ProviderDescriptor>,
    pub snapshot_version: String,
    pub generated_at: String,
}

pub fn is_snapshot_available() -> bool {
    !SNAPSHOT_JSON.is_empty() && serde_json::from_slice::<SnapshotCatalog>(SNAPSHOT_JSON).is_ok()
}

pub fn get_snapshot() -> Option<SnapshotCatalog> {
    if SNAPSHOT_JSON.is_empty() {
        return None;
    }
    serde_json::from_slice(SNAPSHOT_JSON).ok()
}

impl From<SnapshotCatalog> for ProviderCatalog {
    fn from(snapshot: SnapshotCatalog) -> Self {
        let providers = snapshot
            .providers
            .into_iter()
            .map(|(id, mut descriptor)| {
                let prefixed_id = format!("models-dev-{}", id);
                descriptor.id = prefixed_id.clone();
                descriptor
                    .models
                    .iter_mut()
                    .for_each(|(_, model)| model.provider_id = prefixed_id.clone());
                (prefixed_id, descriptor)
            })
            .collect();

        ProviderCatalog {
            providers,
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::Local,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::{
        CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
        ProviderDescriptor,
    };

    fn make_test_snapshot() -> SnapshotCatalog {
        let mut models = BTreeMap::new();
        models.insert(
            "gpt-4o".to_string(),
            ModelDescriptor {
                id: "gpt-4o".to_string(),
                display_name: "GPT-4o".to_string(),
                family: Some("GPT-4".to_string()),
                provider_id: "openai".to_string(),
                capabilities: ModelCapabilities {
                    attachment: true,
                    reasoning: false,
                    tool_call: true,
                    temperature: true,
                    structured_output: true,
                    interleaved: false,
                    open_weights: false,
                    input_modalities: vec!["text".to_string(), "image".to_string()],
                    output_modalities: vec!["text".to_string()],
                },
                cost: CostInfo {
                    input: 0.0025,
                    output: 0.01,
                    cache_read: 0.00125,
                    cache_write: 0.0,
                },
                limits: LimitInfo {
                    context: 128000,
                    input: None,
                    output: 16384,
                },
                status: ModelStatus::Active,
                variants: vec![],
            },
        );

        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderDescriptor {
                id: "openai".to_string(),
                display_name: "OpenAI".to_string(),
                api_base_url: Some("https://api.openai.com/v1".to_string()),
                docs_url: Some("https://platform.openai.com/docs".to_string()),
                env_vars: vec!["OPENAI_API_KEY".to_string()],
                npm_package: Some("@ai-sdk/openai".to_string()),
                source: CatalogSource::ModelsDev,
                models,
            },
        );

        SnapshotCatalog {
            providers,
            snapshot_version: "1".to_string(),
            generated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_is_snapshot_available_returns_true_when_snapshot_exists() {
        assert!(is_snapshot_available());
    }

    #[test]
    fn test_get_snapshot_returns_some_when_data_present() {
        let result = get_snapshot();
        assert!(result.is_some());
        let snapshot = result.unwrap();
        assert!(!snapshot.providers.is_empty());
        assert!(!snapshot.snapshot_version.is_empty());
    }

    #[test]
    fn test_get_snapshot_providers_are_populated() {
        let snapshot = get_snapshot().expect("snapshot should be present");
        assert!(snapshot.providers.contains_key("openai"));
        assert!(snapshot.providers.contains_key("anthropic"));
        let openai = &snapshot.providers["openai"];
        assert!(!openai.models.is_empty());
    }

    #[test]
    fn test_from_snapshot_catalog_for_provider_catalog() {
        let snapshot = make_test_snapshot();
        let catalog = ProviderCatalog::from(snapshot.clone());

        assert_eq!(catalog.source, CatalogSource::Local);
        assert_eq!(catalog.providers.len(), snapshot.providers.len());
        assert!(catalog.providers.contains_key("models-dev-openai"));

        let provider = &catalog.providers["models-dev-openai"];
        assert_eq!(provider.id, "models-dev-openai");
        assert_eq!(provider.display_name, "OpenAI");
        assert!(provider.models.contains_key("gpt-4o"));
    }

    #[test]
    fn test_from_snapshot_preserves_model_details() {
        let snapshot = make_test_snapshot();
        let catalog = ProviderCatalog::from(snapshot);

        let model = &catalog.providers["models-dev-openai"].models["gpt-4o"];
        assert_eq!(model.id, "gpt-4o");
        assert_eq!(model.display_name, "GPT-4o");
        assert_eq!(model.provider_id, "models-dev-openai");
        assert!(model.capabilities.tool_call);
        assert!(model.capabilities.attachment);
        assert_eq!(model.cost.input, 0.0025);
        assert_eq!(model.limits.context, 128000);
        assert_eq!(model.status, ModelStatus::Active);
    }

    #[test]
    fn test_snapshot_loading_handles_corrupted_data_gracefully() {
        let corrupted: &[u8] = b"{ not valid json [[[";
        let result: Option<SnapshotCatalog> = serde_json::from_slice(corrupted).ok();
        assert!(result.is_none());
    }

    #[test]
    fn test_snapshot_loading_handles_empty_data_gracefully() {
        let empty: &[u8] = b"";
        let result: Option<SnapshotCatalog> = if empty.is_empty() {
            None
        } else {
            serde_json::from_slice(empty).ok()
        };
        assert!(result.is_none());
    }

    #[test]
    fn test_snapshot_loading_handles_wrong_schema_gracefully() {
        let wrong_schema: &[u8] = b"{\"unexpected_field\": true}";
        let result: Option<SnapshotCatalog> = serde_json::from_slice(wrong_schema).ok();
        assert!(result.is_none());
    }

    #[test]
    fn test_snapshot_catalog_snapshot_version_is_valid() {
        let snapshot = get_snapshot().expect("snapshot should be present");
        assert!(!snapshot.snapshot_version.is_empty());
    }

    #[test]
    fn test_snapshot_catalog_generated_at_is_valid() {
        let snapshot = get_snapshot().expect("snapshot should be present");
        assert!(!snapshot.generated_at.is_empty());
    }

    #[test]
    fn test_from_empty_snapshot_catalog_for_provider_catalog() {
        let empty_snapshot = SnapshotCatalog {
            providers: BTreeMap::new(),
            snapshot_version: "1".to_string(),
            generated_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let catalog = ProviderCatalog::from(empty_snapshot);
        assert_eq!(catalog.source, CatalogSource::Local);
        assert!(catalog.providers.is_empty());
    }
}
