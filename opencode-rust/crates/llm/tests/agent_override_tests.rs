use opencode_llm::provider_abstraction::{
    DynProvider, ProviderIdentity, ProviderManager, ProviderSpec, ReasoningBudget,
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

#[test]
fn test_reasoning_budget_from_str() {
    assert_eq!(
        ReasoningBudget::from_str("none"),
        Some(ReasoningBudget::None)
    );
    assert_eq!(
        ReasoningBudget::from_str("minimal"),
        Some(ReasoningBudget::Minimal)
    );
    assert_eq!(ReasoningBudget::from_str("low"), Some(ReasoningBudget::Low));
    assert_eq!(
        ReasoningBudget::from_str("medium"),
        Some(ReasoningBudget::Medium)
    );
    assert_eq!(
        ReasoningBudget::from_str("high"),
        Some(ReasoningBudget::High)
    );
    assert_eq!(
        ReasoningBudget::from_str("xhigh"),
        Some(ReasoningBudget::XHigh)
    );
    assert_eq!(ReasoningBudget::from_str("max"), Some(ReasoningBudget::Max));
    assert_eq!(ReasoningBudget::from_str("invalid"), None);
}

#[test]
fn test_reasoning_budget_case_insensitive() {
    assert_eq!(
        ReasoningBudget::from_str("HIGH"),
        Some(ReasoningBudget::High)
    );
    assert_eq!(
        ReasoningBudget::from_str("High"),
        Some(ReasoningBudget::High)
    );
    assert_eq!(ReasoningBudget::from_str("Max"), Some(ReasoningBudget::Max));
}

#[test]
fn test_provider_identity_with_reasoning_budget() {
    let identity = ProviderIdentity::openai("gpt-4o").with_reasoning_budget(ReasoningBudget::High);

    assert_eq!(identity.provider_type, "openai");
    assert_eq!(identity.model, Some("gpt-4o".to_string()));
    assert_eq!(identity.reasoning_budget, Some(ReasoningBudget::High));
}

#[test]
fn test_provider_identity_with_variant() {
    let identity = ProviderIdentity::anthropic("claude-3-5-sonnet").with_variant("high");

    assert_eq!(identity.provider_type, "anthropic");
    assert_eq!(identity.model, Some("claude-3-5-sonnet".to_string()));
    assert_eq!(identity.variant, Some("high".to_string()));
}

#[test]
fn test_provider_identity_with_variant_and_reasoning_budget() {
    let identity = ProviderIdentity::openai("o1-preview")
        .with_variant("high")
        .with_reasoning_budget(ReasoningBudget::Max);

    assert_eq!(identity.provider_type, "openai");
    assert_eq!(identity.model, Some("o1-preview".to_string()));
    assert_eq!(identity.variant, Some("high".to_string()));
    assert_eq!(identity.reasoning_budget, Some(ReasoningBudget::Max));
}

#[test]
fn test_dyn_provider_has_reasoning_budget() {
    let manager = ProviderManager::new();

    let identity = ProviderIdentity::anthropic("claude-3-5-sonnet")
        .with_reasoning_budget(ReasoningBudget::High);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.reasoning_budget(), Some(ReasoningBudget::High));
    assert_eq!(provider.variant(), None);
}

#[test]
fn test_dyn_provider_has_variant() {
    let manager = ProviderManager::new();

    let identity = ProviderIdentity::google("gemini-1.5-pro").with_variant("low");
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.variant(), Some("low"));
}

#[test]
fn test_reasoning_budget_for_anthropic() {
    let budget = ReasoningBudget::High;
    let config = budget.for_provider("anthropic");

    assert!(config.is_some());
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Anthropic { thinking } => {
            assert!(thinking.is_some());
        }
        _ => panic!("Expected Anthropic reasoning config"),
    }
}

#[test]
fn test_reasoning_budget_for_openai() {
    let budget = ReasoningBudget::Medium;
    let config = budget.for_provider("openai");

    assert!(config.is_some());
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::OpenAI { reasoning_effort } => {
            assert_eq!(reasoning_effort, Some("medium".to_string()));
        }
        _ => panic!("Expected OpenAI reasoning config"),
    }
}

#[test]
fn test_reasoning_budget_for_google() {
    let budget = ReasoningBudget::Low;
    let config = budget.for_provider("google");

    assert!(config.is_some());
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Google { thinking_throttle } => {
            assert_eq!(thinking_throttle, Some("low".to_string()));
        }
        _ => panic!("Expected Google reasoning config"),
    }
}

#[test]
fn test_reasoning_budget_none_for_unsupported_provider() {
    let budget = ReasoningBudget::High;
    let config = budget.for_provider("ollama");

    assert!(config.is_none());
}

#[test]
fn test_reasoning_config_conversion() {
    let manager = ProviderManager::new();
    let identity = ProviderIdentity::anthropic("claude-3-5-sonnet")
        .with_reasoning_budget(ReasoningBudget::Max);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    let config = provider.reasoning_config();
    assert!(config.is_some());

    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Anthropic { thinking } => {
            assert!(thinking.is_some());
        }
        _ => panic!("Expected Anthropic reasoning config"),
    }
}

#[test]
fn test_openai_provider_factory_applies_reasoning_effort() {
    let manager = ProviderManager::new();

    let identity =
        ProviderIdentity::openai("o1-preview").with_reasoning_budget(ReasoningBudget::High);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.provider_name(), "openai");
    assert_eq!(provider.reasoning_budget(), Some(ReasoningBudget::High));

    let config = provider.reasoning_config();
    assert!(config.is_some());
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::OpenAI { reasoning_effort } => {
            assert_eq!(reasoning_effort, Some("high".to_string()));
        }
        _ => panic!("Expected OpenAI reasoning config"),
    }
}

#[test]
fn test_anthropic_provider_factory_applies_thinking_budget() {
    let manager = ProviderManager::new();

    let identity = ProviderIdentity::anthropic("claude-3-5-sonnet-20241022")
        .with_reasoning_budget(ReasoningBudget::Max);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.provider_name(), "anthropic");
    assert_eq!(provider.reasoning_budget(), Some(ReasoningBudget::Max));

    let config = provider.reasoning_config();
    assert!(config.is_some());
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Anthropic { thinking } => {
            assert!(thinking.is_some());
        }
        _ => panic!("Expected Anthropic reasoning config"),
    }
}

#[test]
fn test_google_provider_factory_applies_thinking_throttle() {
    let manager = ProviderManager::new();

    let identity =
        ProviderIdentity::google("gemini-1.5-pro").with_reasoning_budget(ReasoningBudget::Low);
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.provider_name(), "google");
    assert_eq!(provider.reasoning_budget(), Some(ReasoningBudget::Low));

    let config = provider.reasoning_config();
    assert!(config.is_some());
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Google { thinking_throttle } => {
            assert_eq!(thinking_throttle, Some("low".to_string()));
        }
        _ => panic!("Expected Google reasoning config"),
    }
}

#[test]
fn test_reasoning_budget_all_levels_openai() {
    let levels = vec![
        (ReasoningBudget::None, None as Option<String>),
        (ReasoningBudget::Minimal, Some("minimal".to_string())),
        (ReasoningBudget::Low, Some("low".to_string())),
        (ReasoningBudget::Medium, Some("medium".to_string())),
        (ReasoningBudget::High, Some("high".to_string())),
        (ReasoningBudget::XHigh, Some("xhigh".to_string())),
        (ReasoningBudget::Max, Some("xhigh".to_string())),
    ];

    for (budget, expected_effort) in levels {
        let config = budget.for_provider("openai");
        match config {
            Some(opencode_llm::ProviderReasoningConfig::OpenAI { reasoning_effort }) => {
                assert_eq!(reasoning_effort, expected_effort, "Failed for {:?}", budget);
            }
            _ => panic!("Expected OpenAI config for {:?}", budget),
        }
    }
}

#[test]
fn test_reasoning_budget_all_levels_anthropic() {
    let levels = vec![
        ReasoningBudget::None,
        ReasoningBudget::Minimal,
        ReasoningBudget::Low,
        ReasoningBudget::Medium,
        ReasoningBudget::High,
        ReasoningBudget::XHigh,
        ReasoningBudget::Max,
    ];

    for budget in levels {
        let config = budget.for_provider("anthropic");
        match config {
            Some(opencode_llm::ProviderReasoningConfig::Anthropic { thinking }) => {
                if budget == ReasoningBudget::None {
                    assert!(
                        thinking.is_none(),
                        "Expected None thinking for {:?}",
                        budget
                    );
                } else {
                    assert!(
                        thinking.is_some(),
                        "Expected Some thinking for {:?}",
                        budget
                    );
                }
            }
            _ => panic!("Expected Anthropic config for {:?}", budget),
        }
    }
}
