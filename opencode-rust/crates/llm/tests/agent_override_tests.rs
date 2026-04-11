use opencode_llm::provider_abstraction::{
    DynProvider, ProviderIdentity, ProviderManager, ProviderSpec,
};

#[test]
fn test_agent_override_takes_precedence_over_provider_default() {
    let manager = ProviderManager::new();

    let identity_with_override = ProviderIdentity::openai("gpt-4o-mini");
    let provider = manager
        .create_provider_by_identity(&identity_with_override)
        .unwrap();

    let models = provider.get_models();
    assert!(!models.is_empty());
    assert_eq!(models[0].id, "gpt-4o-mini");
}

#[test]
fn test_agent_override_is_passed_to_provider() {
    let manager = ProviderManager::new();

    let custom_model = "claude-3-5-sonnet-20241022";
    let identity = ProviderIdentity::anthropic(custom_model);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    let models = provider.get_models();
    assert!(!models.is_empty());
    assert_eq!(models[0].id, custom_model);
}

#[test]
fn test_provider_with_agent_override_has_correct_identity() {
    let manager = ProviderManager::new();

    let agent_model = "gemini-1.5-pro";
    let identity = ProviderIdentity::google(agent_model);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.identity().provider_type, "google");
    assert_eq!(provider.identity().model, Some(agent_model.to_string()));
}

#[test]
fn test_multiple_providers_with_different_agent_overrides() {
    let manager = ProviderManager::new();

    let openai_override = ProviderIdentity::openai("gpt-4o-mini");
    let anthropic_override = ProviderIdentity::anthropic("claude-3-5-sonnet");

    let openai_provider = manager
        .create_provider_by_identity(&openai_override)
        .unwrap();
    let anthropic_provider = manager
        .create_provider_by_identity(&anthropic_override)
        .unwrap();

    assert_eq!(openai_provider.provider_name(), "openai");
    assert_eq!(anthropic_provider.provider_name(), "anthropic");

    let openai_models = openai_provider.get_models();
    let anthropic_models = anthropic_provider.get_models();

    assert_eq!(openai_models[0].id, "gpt-4o-mini");
    assert_eq!(anthropic_models[0].id, "claude-3-5-sonnet");
}

#[test]
fn test_provider_identity_with_no_model_uses_default() {
    let manager = ProviderManager::new();

    let identity_no_model = ProviderIdentity::new("openai", None);
    let provider = manager
        .create_provider_by_identity(&identity_no_model)
        .unwrap();

    let models = provider.get_models();
    assert!(!models.is_empty());
}

#[test]
fn test_agent_override_with_various_providers() {
    let manager = ProviderManager::new();

    let providers_models = vec![
        ("openai", "gpt-4-turbo"),
        ("anthropic", "claude-opus-4-20250514"),
        ("google", "gemini-1.5-flash"),
        ("ollama", "llama3"),
    ];

    for (provider_type, model) in providers_models {
        let identity = ProviderIdentity::new(provider_type, Some(model));
        let provider = manager.create_provider_by_identity(&identity);

        assert!(
            provider.is_ok(),
            "Failed to create provider for {} with model {}",
            provider_type,
            model
        );

        let provider = provider.unwrap();
        let models = provider.get_models();
        assert_eq!(
            models[0].id, model,
            "Provider {} should have model {}",
            provider_type, model
        );
    }
}
