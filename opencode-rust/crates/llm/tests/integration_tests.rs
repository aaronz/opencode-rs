use opencode_llm::model_selection::{ModelSelection, ProviderType, UserModelConfig};
use opencode_llm::provider_abstraction::{
    ProviderIdentity, ProviderManager, ProviderSpec, ReasoningBudget,
};
use opencode_llm::{ChatMessage, LmStudioProvider, OllamaProvider, Provider};

#[test]
fn test_integration_provider_abstraction_works() {
    let manager = ProviderManager::new();

    let openai_spec = ProviderSpec::OpenAI {
        api_key: "test-key".to_string(),
        model: "gpt-4o".to_string(),
        base_url: None,
    };
    let provider = manager.create_provider(&openai_spec).unwrap();

    assert_eq!(provider.provider_name(), "openai");
    assert_eq!(provider.identity().provider_type, "openai");
    assert_eq!(provider.identity().model, Some("gpt-4o".to_string()));
}

#[test]
fn test_integration_provider_identity_factory_pattern() {
    let manager = ProviderManager::new();

    let identity = ProviderIdentity::anthropic("claude-3-5-sonnet");
    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.provider_name(), "anthropic");
    assert!(provider.identity().model.is_some());
}

#[test]
fn test_integration_multiple_providers_different_types() {
    let manager = ProviderManager::new();

    let providers = vec![
        (
            "openai",
            "gpt-4o",
            ProviderSpec::OpenAI {
                api_key: "key".to_string(),
                model: "gpt-4o".to_string(),
                base_url: None,
            },
        ),
        (
            "anthropic",
            "claude-3-5-sonnet",
            ProviderSpec::Anthropic {
                api_key: "key".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                base_url: None,
            },
        ),
        (
            "google",
            "gemini-1.5-pro",
            ProviderSpec::Google {
                api_key: "key".to_string(),
                model: "gemini-1.5-pro".to_string(),
            },
        ),
    ];

    for (provider_type, expected_model, spec) in providers {
        let provider = manager.create_provider(&spec).unwrap();
        assert_eq!(
            provider.provider_name(),
            provider_type,
            "Provider type mismatch for {}",
            provider_type
        );
        let models = provider.get_models();
        assert!(!models.is_empty(), "No models for {}", provider_type);
        assert_eq!(
            models[0].id, expected_model,
            "Model mismatch for {}",
            provider_type
        );
    }
}

#[test]
fn test_integration_dyn_provider_with_reasoning_budget() {
    let manager = ProviderManager::new();

    let identity =
        ProviderIdentity::openai("o1-preview").with_reasoning_budget(ReasoningBudget::High);

    let provider = manager.create_provider_by_identity(&identity).unwrap();

    assert_eq!(provider.reasoning_budget(), Some(ReasoningBudget::High));
    let config = provider.reasoning_config();
    assert!(config.is_some());
}

#[test]
fn test_integration_model_selection_precedence_explicit_override() {
    let selection = ModelSelection::new(ProviderType::OpenAI);

    let model = selection.resolve_model(Some("gpt-4o-mini"));

    assert_eq!(
        model, "gpt-4o-mini",
        "Explicit override should take highest precedence"
    );
}

#[test]
fn test_integration_model_selection_precedence_user_config_provider() {
    let mut config = UserModelConfig::default();
    config
        .provider_defaults
        .insert("openai".to_string(), "gpt-4o-mini".to_string());

    let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

    let model = selection.resolve_model(None);

    assert_eq!(
        model, "gpt-4o-mini",
        "User config provider default should win over global"
    );
}

#[test]
fn test_integration_model_selection_precedence_user_config_global() {
    let mut config = UserModelConfig::default();
    config.global_default = Some("gpt-4o".to_string());

    let selection = ModelSelection::with_user_config(ProviderType::Ollama, config);

    let model = selection.resolve_model(None);

    assert_eq!(
        model, "gpt-4o",
        "Global default should be used when no provider default"
    );
}

#[test]
fn test_integration_model_selection_precedence_provider_default() {
    let config = UserModelConfig::default();

    let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

    let model = selection.resolve_model(None);

    assert_eq!(
        model, "gpt-4o",
        "Provider default should be used when no user config"
    );
}

#[test]
fn test_integration_model_selection_precedence_full_chain() {
    let mut config = UserModelConfig::default();
    config.global_default = Some("fallback-global".to_string());
    config
        .provider_defaults
        .insert("openai".to_string(), "fallback-provider".to_string());

    let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

    assert_eq!(
        selection.resolve_model(Some("explicit-model")),
        "explicit-model",
        "Explicit should take precedence over everything"
    );

    let mut config_no_override = UserModelConfig::default();
    config_no_override.global_default = Some("fallback-global".to_string());
    config_no_override
        .provider_defaults
        .insert("openai".to_string(), "fallback-provider".to_string());
    let selection_no_override =
        ModelSelection::with_user_config(ProviderType::OpenAI, config_no_override);
    assert_eq!(
        selection_no_override.resolve_model(None),
        "fallback-provider",
        "Provider default should take precedence over global default"
    );
}

#[test]
fn test_integration_model_selection_precedence_all_providers() {
    let providers = vec![
        (ProviderType::OpenAI, "gpt-4o"),
        (ProviderType::Anthropic, "claude-sonnet-4-20250514"),
        (ProviderType::Google, "gemini-1.5-pro"),
        (ProviderType::Ollama, "llama3"),
        (ProviderType::LmStudio, "llama3"),
        (ProviderType::Mistral, "mistral-large-latest"),
        (ProviderType::Groq, "llama-3.1-70b-versatile"),
    ];

    for (provider_type, expected_default) in providers {
        let selection = ModelSelection::new(provider_type);
        let model = selection.resolve_model(None);
        assert_eq!(
            model,
            expected_default,
            "Provider {} should have default model {}",
            format!("{:?}", provider_type),
            expected_default
        );
    }
}

#[test]
fn test_integration_local_provider_ollama_creation() {
    let provider = OllamaProvider::new(
        "llama3".to_string(),
        Some("http://localhost:11434".to_string()),
    );

    assert_eq!(provider.provider_name(), "ollama");
}

#[test]
fn test_integration_local_provider_lmstudio_creation() {
    let provider = LmStudioProvider::new(
        "llama3".to_string(),
        Some("http://localhost:1234".to_string()),
    );

    assert_eq!(provider.provider_name(), "lmstudio");
}

#[test]
fn test_integration_local_provider_manager_has_local_providers() {
    let manager = ProviderManager::new();

    assert!(
        manager.has_provider("ollama"),
        "Manager should have ollama provider"
    );
    assert!(
        manager.has_provider("lmstudio"),
        "Manager should have lmstudio provider"
    );
    assert!(
        manager.has_provider("local"),
        "Manager should have local provider"
    );
}

#[test]
fn test_integration_local_provider_ollama_via_manager() {
    let manager = ProviderManager::new();

    let spec = ProviderSpec::Ollama {
        base_url: Some("http://localhost:11434".to_string()),
        model: "llama3".to_string(),
    };

    let result = manager.create_provider(&spec);
    assert!(result.is_ok(), "Should create ollama provider successfully");
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "ollama");
}

#[test]
fn test_integration_local_provider_lmstudio_via_manager() {
    let manager = ProviderManager::new();

    let spec = ProviderSpec::LmStudio {
        base_url: Some("http://localhost:1234".to_string()),
        model: "llama3".to_string(),
    };

    let result = manager.create_provider(&spec);
    assert!(
        result.is_ok(),
        "Should create lmstudio provider successfully"
    );
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "lmstudio");
}

#[test]
fn test_integration_local_provider_local_inference_via_manager() {
    let manager = ProviderManager::new();

    let spec = ProviderSpec::LocalInference {
        base_url: "http://localhost:8080".to_string(),
        model: "llama3".to_string(),
    };

    let result = manager.create_provider(&spec);
    assert!(
        result.is_ok(),
        "Should create local inference provider successfully"
    );
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "local");
}

#[tokio::test]
async fn test_integration_local_provider_ollama_complete_fails_gracefully() {
    let provider = OllamaProvider::new(
        "llama3".to_string(),
        Some("http://localhost:19999".to_string()),
    );

    let result = provider.complete("hello", None).await;
    assert!(
        result.is_err(),
        "Ollama complete should fail without server"
    );
}

#[tokio::test]
async fn test_integration_local_provider_lmstudio_chat_fails_gracefully() {
    let provider = LmStudioProvider::new(
        "llama3".to_string(),
        Some("http://localhost:19999".to_string()),
    );

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "hello".to_string(),
    }];

    let result = provider.chat(&messages).await;
    assert!(result.is_err(), "LM Studio chat should fail without server");
}

#[test]
fn test_integration_provider_spec_serialization_roundtrip() {
    let specs = vec![
        ProviderSpec::OpenAI {
            api_key: "key".to_string(),
            model: "gpt-4o".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
        },
        ProviderSpec::Anthropic {
            api_key: "key".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            base_url: None,
        },
        ProviderSpec::Google {
            api_key: "key".to_string(),
            model: "gemini-1.5-pro".to_string(),
        },
        ProviderSpec::Ollama {
            base_url: Some("http://localhost:11434".to_string()),
            model: "llama3".to_string(),
        },
        ProviderSpec::LmStudio {
            base_url: Some("http://localhost:1234".to_string()),
            model: "llama3".to_string(),
        },
        ProviderSpec::LocalInference {
            base_url: "http://localhost:8080".to_string(),
            model: "llama3".to_string(),
        },
    ];

    for spec in specs {
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: ProviderSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(
            spec.provider_type(),
            deserialized.provider_type(),
            "Provider type should be preserved after roundtrip"
        );
        assert_eq!(
            spec.model(),
            deserialized.model(),
            "Model should be preserved after roundtrip"
        );
    }
}

#[test]
fn test_integration_provider_manager_set_and_get_default() {
    let mut manager = ProviderManager::new();

    assert!(
        manager.get_default().is_none(),
        "Should have no default initially"
    );

    manager.set_default("openai");
    assert_eq!(
        manager.get_default(),
        Some("openai"),
        "Default should be set to openai"
    );

    manager.set_default("anthropic");
    assert_eq!(
        manager.get_default(),
        Some("anthropic"),
        "Default should be updated to anthropic"
    );
}

#[test]
fn test_integration_provider_manager_list_providers() {
    let manager = ProviderManager::new();

    let providers = manager.list_providers();

    assert!(
        providers.contains(&"openai".to_string()),
        "Should contain openai"
    );
    assert!(
        providers.contains(&"anthropic".to_string()),
        "Should contain anthropic"
    );
    assert!(
        providers.contains(&"google".to_string()),
        "Should contain google"
    );
    assert!(
        providers.contains(&"ollama".to_string()),
        "Should contain ollama"
    );
    assert!(
        providers.contains(&"lmstudio".to_string()),
        "Should contain lmstudio"
    );
}

#[test]
fn test_integration_reasoning_budget_for_all_providers() {
    let budgets = vec![
        (ReasoningBudget::None, "none"),
        (ReasoningBudget::Minimal, "minimal"),
        (ReasoningBudget::Low, "low"),
        (ReasoningBudget::Medium, "medium"),
        (ReasoningBudget::High, "high"),
        (ReasoningBudget::XHigh, "xhigh"),
        (ReasoningBudget::Max, "max"),
    ];

    for (budget, name) in budgets {
        assert_eq!(
            ReasoningBudget::from_str(name),
            Some(budget),
            "Should parse {} correctly",
            name
        );
    }

    assert_eq!(
        ReasoningBudget::from_str("invalid"),
        None,
        "Invalid budget should return None"
    );
}

#[test]
fn test_integration_reasoning_budget_for_anthropic_provider() {
    let budget = ReasoningBudget::High;
    let config = budget.for_provider("anthropic");

    assert!(
        config.is_some(),
        "Anthropic should support reasoning budget"
    );
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Anthropic { thinking } => {
            assert!(
                thinking.is_some(),
                "Anthropic thinking config should be set for High budget"
            );
        }
        _ => panic!("Expected Anthropic reasoning config"),
    }
}

#[test]
fn test_integration_reasoning_budget_for_openai_provider() {
    let budget = ReasoningBudget::Medium;
    let config = budget.for_provider("openai");

    assert!(config.is_some(), "OpenAI should support reasoning budget");
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::OpenAI { reasoning_effort } => {
            assert_eq!(reasoning_effort, Some("medium".to_string()));
        }
        _ => panic!("Expected OpenAI reasoning config"),
    }
}

#[test]
fn test_integration_reasoning_budget_for_google_provider() {
    let budget = ReasoningBudget::Low;
    let config = budget.for_provider("google");

    assert!(config.is_some(), "Google should support reasoning budget");
    match config.unwrap() {
        opencode_llm::ProviderReasoningConfig::Google { thinking_throttle } => {
            assert_eq!(thinking_throttle, Some("low".to_string()));
        }
        _ => panic!("Expected Google reasoning config"),
    }
}

#[test]
fn test_integration_reasoning_budget_unsupported_provider() {
    let budget = ReasoningBudget::High;

    assert!(
        budget.for_provider("ollama").is_none(),
        "Ollama should not support reasoning budget"
    );
    assert!(
        budget.for_provider("lmstudio").is_none(),
        "LM Studio should not support reasoning budget"
    );
}

#[test]
fn test_integration_provider_type_from_str() {
    let tests = vec![
        ("openai", ProviderType::OpenAI),
        ("OPENAI", ProviderType::OpenAI),
        ("anthropic", ProviderType::Anthropic),
        ("ANTHROPIC", ProviderType::Anthropic),
        ("google", ProviderType::Google),
        ("ollama", ProviderType::Ollama),
        ("lmstudio", ProviderType::LmStudio),
        ("lm_studio", ProviderType::LmStudio),
        ("lm-studio", ProviderType::LmStudio),
        ("local", ProviderType::LocalInference),
        ("local-inference", ProviderType::LocalInference),
        ("unknown", ProviderType::Custom),
    ];

    for (input, expected) in tests {
        assert_eq!(
            ProviderType::from_str(input),
            expected,
            "ProviderType::from_str({}) should return {:?}",
            input,
            expected
        );
    }
}

#[test]
fn test_integration_user_config_model_preferences() {
    let config = UserModelConfig {
        provider_defaults: std::collections::HashMap::from([
            ("openai".to_string(), "gpt-4o-mini".to_string()),
            ("anthropic".to_string(), "claude-haiku-3".to_string()),
        ]),
        global_default: Some("gpt-4o".to_string()),
    };

    let openai_selection = ModelSelection::with_user_config(ProviderType::OpenAI, config.clone());
    assert_eq!(openai_selection.resolve_model(None), "gpt-4o-mini");

    let anthropic_selection =
        ModelSelection::with_user_config(ProviderType::Anthropic, config.clone());
    assert_eq!(anthropic_selection.resolve_model(None), "claude-haiku-3");

    let ollama_selection = ModelSelection::with_user_config(ProviderType::Ollama, config.clone());
    assert_eq!(
        ollama_selection.resolve_model(None),
        "gpt-4o",
        "Should use global default for unknown provider"
    );
}
