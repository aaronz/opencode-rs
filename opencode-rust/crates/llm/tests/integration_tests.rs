use opencode_llm::budget::{
    BudgetExceededError, BudgetLimit, BudgetTracker, ConversationBudgetState, Usage, VariantCost,
};
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

#[allow(deprecated)]
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
    let config = UserModelConfig {
        global_default: Some("gpt-4o".to_string()),
        ..Default::default()
    };

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
    let mut config = UserModelConfig {
        global_default: Some("fallback-global".to_string()),
        ..Default::default()
    };
    config
        .provider_defaults
        .insert("openai".to_string(), "fallback-provider".to_string());

    let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

    assert_eq!(
        selection.resolve_model(Some("explicit-model")),
        "explicit-model",
        "Explicit should take precedence over everything"
    );

    let mut config_no_override = UserModelConfig {
        global_default: Some("fallback-global".to_string()),
        ..Default::default()
    };
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
            format_args!("{:?}", provider_type),
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

#[allow(deprecated)]
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

#[test]
fn test_budget_usage_new_and_cost_calculation() {
    let usage = Usage::new(1000, 500);
    assert_eq!(usage.prompt_tokens, 1000);
    assert_eq!(usage.completion_tokens, 500);
    assert_eq!(usage.total_tokens, 1500);

    let cost = usage.calculate_cost(0.005);
    assert!((cost - 0.0075).abs() < 0.0001);
}

#[test]
fn test_budget_tracker_records_usage() {
    let tracker = BudgetTracker::new();
    let usage = Usage::new(100, 50);

    let state = tracker.record_usage(&usage);

    assert_eq!(state.request_num, 1);
    assert_eq!(state.prompt_tokens, 100);
    assert_eq!(state.completion_tokens, 50);
    assert_eq!(state.total_tokens, 150);
}

#[test]
fn test_budget_tracker_accumulates_across_requests() {
    let tracker = BudgetTracker::new();
    tracker.record_usage(&Usage::new(100, 50));
    tracker.record_usage(&Usage::new(200, 100));

    let state = tracker.get_conversation_state();
    assert_eq!(state.total_requests, 2);
    assert_eq!(state.total_prompt_tokens, 300);
    assert_eq!(state.total_completion_tokens, 150);
    assert_eq!(state.total_tokens, 450);
}

#[test]
fn test_budget_tracker_variant_costs() {
    let tracker = BudgetTracker::new();
    tracker.record_variant_usage("variant_a", &Usage::new(100, 50));
    tracker.record_variant_usage("variant_b", &Usage::new(200, 100));

    let state = tracker.get_conversation_state();
    assert_eq!(state.variant_costs.len(), 2);
    assert_eq!(state.variant_costs[0].variant_id, "variant_a");
    assert_eq!(state.variant_costs[1].variant_id, "variant_b");
    assert_eq!(state.variant_costs[0].total_tokens, 150);
    assert_eq!(state.variant_costs[1].total_tokens, 300);
}

#[test]
fn test_budget_limit_per_request_enforcement() {
    let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
    let tracker = BudgetTracker::with_request_limit(tracker, 500);

    let small_usage = Usage::new(100, 100);
    assert!(tracker.check_request_budget(&small_usage).is_ok());

    let large_usage = Usage::new(1000, 1000);
    assert!(tracker.check_request_budget(&large_usage).is_err());
}

#[test]
fn test_budget_limit_combined_enforcement() {
    let limit = BudgetLimit::Combined {
        per_request: 0.005,
        per_conversation: 0.01,
    };

    assert!(limit.is_exceeded(0.006, 0.0));
    assert!(limit.is_exceeded(0.0, 0.015));
    assert!(!limit.is_exceeded(0.004, 0.009));
}

#[test]
fn test_budget_tracker_conversation_limit() {
    let tracker = BudgetTracker::with_reasoning_budget(None, 1.0);
    let tracker = BudgetTracker::with_conversation_limit(tracker, 1000);

    assert!(tracker.check_conversation_budget(0.0005).is_ok());
    assert!(tracker.check_conversation_budget(0.002).is_err());
}

#[test]
fn test_budget_tracker_with_reasoning_budget() {
    let tracker = BudgetTracker::with_reasoning_budget(Some(ReasoningBudget::High), 0.005);

    assert_eq!(tracker.reasoning_budget(), Some(ReasoningBudget::High));

    tracker.record_usage(&Usage::new(500, 250));

    let state = tracker.get_conversation_state();
    assert_eq!(state.total_tokens, 750);
}

#[test]
fn test_budget_tracker_cost_calculation() {
    let tracker = BudgetTracker::with_reasoning_budget(None, 0.01);

    tracker.record_usage(&Usage::new(1000, 500));

    let state = tracker.get_conversation_state();
    assert!((state.total_cost_usd - 0.015).abs() < 0.0001);
}

#[test]
fn test_budget_tracker_remaining_budget() {
    let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
    let tracker = BudgetTracker::with_request_limit(tracker, 1000);

    tracker.record_usage(&Usage::new(100, 100));

    let remaining = tracker.remaining_request_budget();
    assert!(remaining.is_some());
}

#[test]
fn test_budget_tracker_reset() {
    let tracker = BudgetTracker::new();
    tracker.record_usage(&Usage::new(100, 50));

    tracker.reset_conversation_budget();

    let state = tracker.get_conversation_state();
    assert_eq!(state.total_requests, 0);
    assert_eq!(state.total_prompt_tokens, 0);
    assert_eq!(state.total_completion_tokens, 0);
}

#[test]
fn test_budget_tracker_clone_shares_state() {
    let tracker = BudgetTracker::new();
    tracker.record_usage(&Usage::new(100, 50));

    let cloned = tracker.clone();
    cloned.record_usage(&Usage::new(200, 100));

    let state = tracker.get_conversation_state();
    assert_eq!(state.total_requests, 2);
    assert_eq!(cloned.get_conversation_state().total_requests, 2);
}

#[test]
fn test_budget_exceeded_error_display() {
    let error = BudgetExceededError {
        limit_type: BudgetLimit::PerRequest(0.01),
        request_cost: 0.015,
        conversation_cost: 0.0,
    };

    let display = format!("{}", error);
    assert!(display.contains("0.015"));
    assert!(display.contains("0.01"));
}

#[test]
fn test_conversation_budget_state_serialization() {
    let state = ConversationBudgetState {
        total_requests: 5,
        total_prompt_tokens: 1000,
        total_completion_tokens: 500,
        total_tokens: 1500,
        total_cost_usd: 0.015,
        variant_costs: vec![VariantCost {
            variant_id: "v1".to_string(),
            prompt_tokens: 500,
            completion_tokens: 250,
            total_tokens: 750,
            cost_usd: 0.0075,
        }],
    };

    let json = serde_json::to_string(&state).unwrap();
    let deserialized: ConversationBudgetState = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.total_requests, 5);
    assert_eq!(deserialized.variant_costs.len(), 1);
    assert_eq!(deserialized.variant_costs[0].variant_id, "v1");
}

#[test]
fn test_variant_cost_serialization() {
    let variant = VariantCost {
        variant_id: "test-variant".to_string(),
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
        cost_usd: 0.00075,
    };

    let json = serde_json::to_string(&variant).unwrap();
    let deserialized: VariantCost = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.variant_id, "test-variant");
    assert_eq!(deserialized.total_tokens, 150);
}

#[test]
fn test_budget_tracker_variants_within_budget() {
    let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
    let tracker = BudgetTracker::with_conversation_limit(tracker, 10000);

    tracker.record_variant_usage("v1", &Usage::new(1000, 500));
    tracker.record_variant_usage("v2", &Usage::new(1000, 500));
    tracker.record_variant_usage("v3", &Usage::new(1000, 500));

    let state = tracker.get_conversation_state();
    assert_eq!(state.variant_costs.len(), 3);

    let total_variant_tokens: u64 = state.variant_costs.iter().map(|v| v.total_tokens).sum();
    assert_eq!(total_variant_tokens, 4500);

    let total_variant_cost: f64 = state.variant_costs.iter().map(|v| v.cost_usd).sum();
    assert!((total_variant_cost - 0.0045).abs() < 0.0001);

    assert!((tracker.total_cost_usd() - 0.0).abs() < 0.0001);
}

#[test]
fn test_budget_tracker_request_budget_state() {
    let tracker = BudgetTracker::new();
    tracker.record_usage(&Usage::new(100, 50));

    let state = tracker.get_request_state();
    assert_eq!(state.request_num, 1);
    assert_eq!(state.prompt_tokens, 100);
    assert_eq!(state.completion_tokens, 50);
    assert_eq!(state.total_tokens, 150);
}

#[test]
fn test_budget_limit_none_never_exceeded() {
    let limit = BudgetLimit::None;
    assert!(!limit.is_exceeded(999999.0, 999999.0));
    assert!(limit.check_and_update(0.0, 0.0).is_ok());
}

#[test]
fn test_budget_tracker_tracks_both_token_types() {
    let tracker = BudgetTracker::new();
    tracker.record_usage(&Usage::new(1000, 0));
    tracker.record_usage(&Usage::new(0, 1000));

    let state = tracker.get_conversation_state();
    assert_eq!(state.total_prompt_tokens, 1000);
    assert_eq!(state.total_completion_tokens, 1000);
    assert_eq!(state.total_tokens, 2000);
}
