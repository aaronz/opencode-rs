use opencode_llm::{ModelRegistry, ProviderConfig, ProviderFilter};

#[test]
fn test_model_registry_default_has_providers() {
    let registry = ModelRegistry::default();
    let models = registry.list();
    assert!(!models.is_empty(), "Default registry should have models");
}

#[test]
fn test_model_registry_list_by_provider() {
    let registry = ModelRegistry::default();
    let openai_models = registry.list_by_provider("openai");
    assert!(!openai_models.is_empty(), "OpenAI should have models");
}

#[test]
fn test_provider_config_sanitize() {
    let config = ProviderConfig {
        model: "gpt-4o".to_string(),
        api_key: "secret-key-123".to_string(),
        temperature: 0.7,
    };
    let sanitized = config.sanitize_for_logging();
    assert_eq!(sanitized.api_key, "***REDACTED***");
    assert_eq!(sanitized.model, "gpt-4o");
}

#[test]
fn test_provider_filter_is_allowed() {
    let filter = ProviderFilter::new(vec![], vec!["openai".to_string()]);
    assert!(filter.is_allowed("openai"));
    assert!(!filter.is_allowed("anthropic"));
}

#[test]
fn test_provider_filter_blacklist() {
    let filter = ProviderFilter::new(vec!["openai".to_string()], vec![]);
    assert!(!filter.is_allowed("openai"));
    assert!(filter.is_allowed("anthropic"));
}

#[test]
fn test_model_registry_get_nonexistent() {
    let registry = ModelRegistry::default();
    assert!(registry.get("nonexistent-model-xyz").is_none());
}

#[test]
fn test_model_registry_set_and_get_filter() {
    let mut registry = ModelRegistry::default();
    let filter = ProviderFilter::new(vec!["openai".to_string()], vec!["openai".to_string()]);
    registry.set_provider_filter(filter.clone());
    assert!(!registry.get("gpt-4o").is_some() || registry.get("gpt-4o").is_none());
}

#[test]
fn test_provider_config_display() {
    let config = ProviderConfig {
        model: "test-model".to_string(),
        api_key: "key".to_string(),
        temperature: 0.5,
    };
    let display = format!("{config:?}");
    assert!(
        display.contains("test-model"),
        "Debug should contain model name"
    );
    assert!(
        display.contains("api_key"),
        "Debug should have api_key field"
    );
}
