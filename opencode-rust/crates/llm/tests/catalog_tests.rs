use std::collections::BTreeMap;
use std::sync::Arc;

use opencode_config::ProviderConfig;
use tempfile::TempDir;
use tokio::fs;

use opencode_llm::catalog::{
    merge_catalogs, CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor,
    ModelStatus, ProviderCatalog, ProviderCatalogFetcher, ProviderDescriptor,
};

fn create_test_catalog() -> ProviderCatalog {
    let mut providers = BTreeMap::new();

    providers.insert(
        "openai".to_string(),
        ProviderDescriptor {
            id: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            api_base_url: Some("https://api.openai.com/v1".to_string()),
            docs_url: None,
            env_vars: vec!["OPENAI_API_KEY".to_string()],
            npm_package: None,
            models: BTreeMap::from([(
                "gpt-4o".to_string(),
                ModelDescriptor {
                    id: "gpt-4o".to_string(),
                    display_name: "GPT-4o".to_string(),
                    family: Some("GPT-4".to_string()),
                    provider_id: "openai".to_string(),
                    capabilities: ModelCapabilities {
                        attachment: false,
                        reasoning: false,
                        tool_call: true,
                        temperature: true,
                        structured_output: true,
                        interleaved: false,
                        open_weights: false,
                        input_modalities: vec!["text".to_string()],
                        output_modalities: vec!["text".to_string()],
                    },
                    cost: CostInfo {
                        input: 0.005,
                        output: 0.015,
                        cache_read: 0.0,
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
            )]),
            source: CatalogSource::ModelsDev,
        },
    );

    providers.insert(
        "minimax".to_string(),
        ProviderDescriptor {
            id: "minimax".to_string(),
            display_name: "MiniMax".to_string(),
            api_base_url: Some("https://api.minimax.chat/v1".to_string()),
            docs_url: None,
            env_vars: vec!["MINIMAX_API_KEY".to_string()],
            npm_package: None,
            models: BTreeMap::from([(
                "MiniMax-M2.7".to_string(),
                ModelDescriptor {
                    id: "MiniMax-M2.7".to_string(),
                    display_name: "MiniMax M2.7".to_string(),
                    family: Some("MiniMax".to_string()),
                    provider_id: "minimax".to_string(),
                    capabilities: ModelCapabilities {
                        attachment: false,
                        reasoning: true,
                        tool_call: false,
                        temperature: true,
                        structured_output: false,
                        interleaved: false,
                        open_weights: false,
                        input_modalities: vec!["text".to_string()],
                        output_modalities: vec!["text".to_string()],
                    },
                    cost: CostInfo {
                        input: 0.0,
                        output: 0.0,
                        cache_read: 0.0,
                        cache_write: 0.0,
                    },
                    limits: LimitInfo {
                        context: 1000000,
                        input: None,
                        output: 8192,
                    },
                    status: ModelStatus::Active,
                    variants: vec![],
                },
            )]),
            source: CatalogSource::ModelsDev,
        },
    );

    ProviderCatalog {
        providers,
        fetched_at: chrono::Utc::now(),
        source: CatalogSource::ModelsDev,
    }
}

#[test]
fn test_provider_catalog_serialization_roundtrip() {
    let catalog = create_test_catalog();

    let json = serde_json::to_string(&catalog).expect("should serialize");
    let deserialized: ProviderCatalog = serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(catalog.providers.len(), deserialized.providers.len());
    assert_eq!(catalog.source, deserialized.source);

    let original_gpt4o = catalog
        .providers
        .get("openai")
        .unwrap()
        .models
        .get("gpt-4o")
        .unwrap();
    let deser_gpt4o = deserialized
        .providers
        .get("openai")
        .unwrap()
        .models
        .get("gpt-4o")
        .unwrap();

    assert_eq!(original_gpt4o.id, deser_gpt4o.id);
    assert_eq!(original_gpt4o.cost.input, deser_gpt4o.cost.input);
    assert_eq!(
        original_gpt4o.capabilities.tool_call,
        deser_gpt4o.capabilities.tool_call
    );
}

#[test]
fn test_catalog_source_variants() {
    assert!(matches!(CatalogSource::ModelsDev, CatalogSource::ModelsDev));
    assert!(matches!(CatalogSource::Config, CatalogSource::Config));
    assert!(matches!(CatalogSource::Local, CatalogSource::Local));
}

#[test]
fn test_model_status_variants() {
    assert!(matches!(ModelStatus::Active, ModelStatus::Active));
    assert!(matches!(ModelStatus::Beta, ModelStatus::Beta));
    assert!(matches!(ModelStatus::Alpha, ModelStatus::Alpha));
    assert!(matches!(ModelStatus::Deprecated, ModelStatus::Deprecated));
}

#[tokio::test]
async fn test_fetcher_get_blocking_returns_none_when_no_cache() {
    let temp_dir = TempDir::new().expect("should create temp dir");
    let cache_path = temp_dir.path().join("catalog.json");

    let fetcher = ProviderCatalogFetcher::new(cache_path);
    let result = fetcher.get_blocking();

    assert!(result.is_none(), "should return None when no cache exists");
}

#[tokio::test]
async fn test_fetcher_get_blocking_returns_cached_data() {
    let temp_dir = TempDir::new().expect("should create temp dir");
    let cache_path = temp_dir.path().join("catalog.json");

    let catalog = create_test_catalog();
    let json = serde_json::to_string_pretty(&catalog).expect("should serialize");
    fs::write(&cache_path, json)
        .await
        .expect("should write cache");

    let fetcher = ProviderCatalogFetcher::new(cache_path);
    let result = fetcher.get_blocking().expect("should return cached data");

    assert_eq!(result.providers.len(), 2);
    assert!(result.providers.contains_key("openai"));
    assert!(result.providers.contains_key("minimax"));
}

#[tokio::test]
async fn test_fetcher_get_or_fetch_returns_stale_on_network_failure() {
    let temp_dir = TempDir::new().expect("should create temp dir");
    let cache_path = temp_dir.path().join("catalog.json");

    let catalog = create_test_catalog();
    let json = serde_json::to_string_pretty(&catalog).expect("should serialize");
    fs::write(&cache_path, json)
        .await
        .expect("should write cache");

    let fetcher = ProviderCatalogFetcher::new(cache_path);

    let result = fetcher.get_or_fetch().await;
    assert_eq!(result.providers.len(), 2);
}

#[tokio::test]
async fn test_fetcher_refresh_spawns_task() {
    let temp_dir = TempDir::new().expect("should create temp dir");
    let cache_path = temp_dir.path().join("catalog.json");

    let catalog = create_test_catalog();
    let json = serde_json::to_string_pretty(&catalog).expect("should serialize");
    fs::write(&cache_path, json)
        .await
        .expect("should write cache");

    let fetcher = Arc::new(ProviderCatalogFetcher::new(cache_path));

    fetcher.refresh();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

#[test]
fn test_merge_adds_local_providers_when_missing() {
    let catalog = create_test_catalog();

    let merged = merge_catalogs(catalog, &BTreeMap::new(), None, &[]);

    assert!(merged.providers.contains_key("ollama"));
    assert!(merged.providers.contains_key("lmstudio"));
    assert!(merged.providers.contains_key("local"));
    assert_eq!(
        merged.providers.get("ollama").unwrap().source,
        CatalogSource::Local
    );
}

#[test]
fn test_merge_respects_disabled_providers() {
    let catalog = create_test_catalog();

    let merged = merge_catalogs(catalog, &BTreeMap::new(), None, &["minimax".to_string()]);

    assert!(!merged.providers.contains_key("minimax"));
    assert!(merged.providers.contains_key("openai"));
}

#[test]
fn test_merge_empty_enabled_list_means_disable_all() {
    let catalog = create_test_catalog();

    let merged = merge_catalogs(catalog, &BTreeMap::new(), Some(&[]), &[]);

    assert_eq!(merged.providers.len(), 0);
}

#[test]
fn test_merge_config_overrides_provider_name() {
    let catalog = create_test_catalog();

    let mut config_providers = BTreeMap::new();
    config_providers.insert(
        "openai".to_string(),
        ProviderConfig {
            id: Some("openai".to_string()),
            name: Some("OpenAI Custom".to_string()),
            options: None,
            models: None,
            ..Default::default()
        },
    );

    let merged = merge_catalogs(catalog, &config_providers, None, &[]);

    let openai = merged.providers.get("openai").unwrap();
    assert_eq!(openai.display_name, "OpenAI Custom");
}

#[test]
fn test_merge_config_overrides_base_url() {
    let catalog = create_test_catalog();

    let mut config_providers = BTreeMap::new();
    config_providers.insert(
        "openai".to_string(),
        ProviderConfig {
            id: Some("openai".to_string()),
            name: None,
            options: Some(opencode_config::ProviderOptions {
                base_url: Some("https://custom.openai.com/v1".to_string()),
                ..Default::default()
            }),
            models: None,
            ..Default::default()
        },
    );

    let merged = merge_catalogs(catalog, &config_providers, None, &[]);

    let openai = merged.providers.get("openai").unwrap();
    assert_eq!(
        openai.api_base_url.as_ref().unwrap(),
        "https://custom.openai.com/v1"
    );
}

#[test]
fn test_merge_config_overrides_model_visibility() {
    let catalog = create_test_catalog();

    let mut config_providers = BTreeMap::new();
    config_providers.insert(
        "openai".to_string(),
        ProviderConfig {
            id: Some("openai".to_string()),
            name: None,
            models: Some(std::collections::HashMap::from([(
                "gpt-4o".to_string(),
                opencode_config::ModelConfig {
                    visible: Some(false),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        },
    );

    let merged = merge_catalogs(catalog, &config_providers, None, &[]);

    let openai = merged.providers.get("openai").unwrap();
    assert!(!openai.models.contains_key("gpt-4o"));
}

#[test]
fn test_merge_config_creates_synthetic_provider() {
    let catalog = create_test_catalog();

    let mut config_providers = BTreeMap::new();
    config_providers.insert(
        "custom".to_string(),
        ProviderConfig {
            id: Some("custom".to_string()),
            name: Some("Custom Provider".to_string()),
            options: Some(opencode_config::ProviderOptions {
                base_url: Some("https://custom.api.com/v1".to_string()),
                ..Default::default()
            }),
            models: None,
            ..Default::default()
        },
    );

    let merged = merge_catalogs(catalog, &config_providers, None, &[]);

    let custom = merged.providers.get("custom").unwrap();
    assert_eq!(custom.display_name, "Custom Provider");
    assert!(custom
        .api_base_url
        .as_ref()
        .unwrap()
        .contains("custom.api.com"));
}

#[test]
fn test_merge_enabled_filter_takes_precedence() {
    let catalog = create_test_catalog();

    let merged = merge_catalogs(
        catalog,
        &BTreeMap::new(),
        Some(&["openai".to_string()]),
        &["minimax".to_string()],
    );

    assert!(merged.providers.contains_key("openai"));
    assert!(!merged.providers.contains_key("minimax"));
}

#[test]
fn test_merge_with_both_enabled_and_disabled() {
    let catalog = create_test_catalog();

    let merged = merge_catalogs(
        catalog,
        &BTreeMap::new(),
        Some(&["openai".to_string(), "minimax".to_string()]),
        &["minimax".to_string()],
    );

    assert!(!merged.providers.contains_key("minimax"));
}

#[test]
fn test_catalog_merger_build() {
    use opencode_llm::catalog::merge::CatalogMerger;

    let catalog = create_test_catalog();
    let merger = CatalogMerger::new(catalog);
    let built = merger.build();

    assert!(built.providers.contains_key("openai"));
}

#[test]
fn test_catalog_merger_with_local_providers() {
    use opencode_llm::catalog::merge::CatalogMerger;

    let catalog = create_test_catalog();
    let merger = CatalogMerger::new(catalog).with_local_providers();
    let built = merger.build();

    assert!(built.providers.contains_key("ollama"));
    assert!(built.providers.contains_key("lmstudio"));
}

#[test]
fn test_catalog_merger_does_not_override_existing_local_providers() {
    use opencode_llm::catalog::merge::CatalogMerger;

    let mut catalog = create_test_catalog();
    catalog.providers.insert(
        "ollama".to_string(),
        ProviderDescriptor {
            id: "ollama".to_string(),
            display_name: "Custom Ollama".to_string(),
            api_base_url: Some("http://custom:11434".to_string()),
            docs_url: None,
            env_vars: vec![],
            npm_package: None,
            models: BTreeMap::new(),
            source: CatalogSource::Config,
        },
    );

    let merger = CatalogMerger::new(catalog).with_local_providers();
    let built = merger.build();

    assert_eq!(
        built.providers.get("ollama").unwrap().display_name,
        "Custom Ollama"
    );
}
