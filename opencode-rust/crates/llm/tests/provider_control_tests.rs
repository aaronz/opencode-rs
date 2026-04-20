use std::collections::HashMap;

use opencode_core::config::{ProviderConfig as CoreProviderConfig, ProviderOptions};
use opencode_llm::{ModelRegistry, ProviderConfig, ProviderFilter};

#[test]
fn provider_filter_blacklist_priority_over_whitelist() {
    let filter = ProviderFilter::new(
        vec!["openai".to_string()],
        vec!["openai".to_string(), "anthropic".to_string()],
    );

    assert!(!filter.is_allowed("openai"));
    assert!(filter.is_allowed("anthropic"));
}

#[test]
fn provider_filter_empty_whitelist_allows_non_blacklisted() {
    let filter = ProviderFilter::new(vec!["openai".to_string()], vec![]);

    assert!(!filter.is_allowed("openai"));
    assert!(filter.is_allowed("anthropic"));
}

#[test]
fn provider_filter_both_set_blacklist_wins() {
    let filter = ProviderFilter::new(
        vec!["anthropic".to_string()],
        vec!["openai".to_string(), "anthropic".to_string()],
    );

    assert!(filter.is_allowed("openai"));
    assert!(!filter.is_allowed("anthropic"));
}

#[test]
fn model_registry_respects_provider_filter() {
    let mut registry = ModelRegistry::default();
    registry.set_provider_filter(ProviderFilter::new(
        vec!["openai".to_string()],
        vec!["openai".to_string(), "anthropic".to_string()],
    ));

    assert!(registry.get("gpt-4o").is_none());
    assert!(registry.get("claude-haiku-3").is_some());
    assert!(registry.list_by_provider("openai").is_empty());
    assert!(!registry.list_by_provider("anthropic").is_empty());
}

#[test]
fn model_registry_returns_next_available_provider_for_failover() {
    let mut registry = ModelRegistry::default();
    registry.set_provider_filter(ProviderFilter::new(
        vec!["openai".to_string()],
        vec!["anthropic".to_string(), "ollama".to_string()],
    ));

    assert_eq!(
        registry.get_next_available_provider("anthropic"),
        Some("ollama".to_string())
    );
    assert_eq!(
        registry.get_next_available_provider("ollama"),
        Some("anthropic".to_string())
    );
}

#[test]
fn api_key_sanitization_redacts_provider_api_keys() {
    let llm_provider_config = ProviderConfig {
        model: "gpt-4o".to_string(),
        api_key: "super-secret-key".to_string(),
        temperature: 0.7,
        headers: HashMap::new(),
    };
    let sanitized_llm = llm_provider_config.sanitize_for_logging();
    assert_eq!(sanitized_llm.api_key, "***REDACTED***");
    assert!(!format!("{llm_provider_config:?}").contains("super-secret-key"));

    let core_provider_config = CoreProviderConfig {
        options: Some(ProviderOptions {
            api_key: Some("another-secret".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let sanitized_core = core_provider_config.sanitize_for_logging();
    assert_eq!(
        sanitized_core
            .options
            .as_ref()
            .and_then(|options| options.api_key.as_deref()),
        Some("***REDACTED***")
    );
}
