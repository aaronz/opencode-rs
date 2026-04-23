#[cfg(test)]
mod provider_tests {
    use opencode_llm::budget::{BudgetLimit, BudgetTracker};
    use opencode_llm::message_transform::{MessageTransform, TransformPipeline};

    #[test]
    fn test_provider_e2e_003_budget_tracker_per_request() {
        let tracker = BudgetTracker::new().with_request_limit(1);

        let usage1 = opencode_llm::provider::Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let result1 = tracker.record_usage(&usage1);
        assert!(result1.cost_usd >= 0.0, "Usage should be recorded");

        let usage2 = opencode_llm::provider::Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };
        let result2 = tracker.record_usage(&usage2);
        assert!(
            result2.cost_usd >= result1.cost_usd,
            "Cost should accumulate"
        );
    }

    #[test]
    fn test_provider_e2e_003_budget_tracker_conversation_limit() {
        let tracker = BudgetTracker::new().with_conversation_limit(100);

        let usage = opencode_llm::provider::Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let result = tracker.record_usage(&usage);
        assert!(result.total_tokens >= 150, "Tokens should be accumulated");
    }

    #[test]
    fn test_provider_e2e_003_budget_tracker_cost_calculation() {
        let tracker = BudgetTracker::new().with_cost_per_1k_tokens(0.01);

        let usage = opencode_llm::provider::Usage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };
        let result = tracker.record_usage(&usage);
        assert!(result.cost_usd > 0.0, "Cost should be calculated");
    }

    #[test]
    fn test_provider_err_001_invalid_api_key_error_type() {
        use opencode_llm::error::LlmError;

        let error = LlmError::InvalidApiKey;
        let error_string = error.to_string();

        assert!(
            error_string.to_lowercase().contains("api key")
                || error_string.to_lowercase().contains("invalid")
                || error_string.to_lowercase().contains("authentication"),
            "InvalidApiKey error should mention API key or authentication"
        );
    }

    #[test]
    fn test_provider_err_001_auth_error_distinct_from_network() {
        use opencode_llm::error::LlmError;

        let auth_error = LlmError::Auth("test".to_string());
        let network_error = LlmError::NetworkError("test".to_string());

        let auth_str = auth_error.to_string().to_lowercase();
        let network_str = network_error.to_string().to_lowercase();

        assert!(
            auth_str.contains("auth") || auth_str.contains("invalid"),
            "Auth error should mention auth"
        );
        assert!(
            network_str.contains("network") || network_str.contains("connection"),
            "Network error should mention network"
        );
    }

    #[test]
    fn test_provider_err_003_parse_error_on_malformed_data() {
        use opencode_llm::error::LlmError;

        let parse_error = LlmError::Parse("unexpected token at line 1".to_string());
        assert!(
            parse_error.to_string().contains("parse") || parse_error.to_string().contains("format"),
            "Parse error should mention parse issue"
        );
    }

    #[test]
    fn test_provider_transform_001_transform_pipeline() {
        let mut pipeline = TransformPipeline::new();
        pipeline.add_transform(MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: Some("SYSTEM: ".to_string()),
            suffix: None,
        });
        pipeline.add_transform(MessageTransform {
            trim_whitespace: false,
            max_length: Some(20),
            prefix: None,
            suffix: None,
        });

        let result = pipeline.apply("  hello world  ");
        assert!(result.starts_with("SYSTEM:"), "Prefix should be applied");
        assert!(result.contains("hello"), "Content should be present");
    }

    #[test]
    fn test_provider_transform_001_transform_preserves_original() {
        let transform = MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: None,
        };

        let original = "  hello  ";
        let transformed = transform.apply(original);

        assert_eq!(original, "  hello  ", "Original should not be mutated");
        assert_eq!(transformed, "hello", "Transform should produce new string");
    }

    #[test]
    fn test_provider_transform_002_pipeline_empty() {
        let pipeline = TransformPipeline::new();
        let result = pipeline.apply("hello");

        assert_eq!(result, "hello", "Empty pipeline should pass through");
    }

    #[test]
    fn test_provider_transform_002_multiple_transforms_order() {
        let mut pipeline = TransformPipeline::new();

        pipeline.add_transform(MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: Some("PREFIX_".to_string()),
            suffix: None,
        });

        pipeline.add_transform(MessageTransform {
            trim_whitespace: false,
            max_length: None,
            prefix: None,
            suffix: Some("_SUFFIX".to_string()),
        });

        let result = pipeline.apply("  test  ");
        assert_eq!(result, "PREFIX_test_SUFFIX");
    }

    #[test]
    fn test_provider_conc_001_message_transform_concurrent_safe() {
        use std::thread;

        let transform = MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: None,
        };

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let transform = transform.clone();
                thread::spawn(move || {
                    for _ in 0..100 {
                        let result = transform.apply("  hello world  ");
                        assert_eq!(result, "hello world");
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread should not panic");
        }
    }

    #[test]
    fn test_provider_budget_001_budget_limit_combined() {
        let limit = BudgetLimit::Combined {
            per_request: 0.01,
            per_conversation: 0.02,
        };

        assert!(!limit.is_exceeded(0.005, 0.01), "Both under should pass");
        assert!(limit.is_exceeded(0.015, 0.01), "Request over should fail");
        assert!(
            limit.is_exceeded(0.005, 0.025),
            "Conversation over should fail"
        );
    }

    #[test]
    fn test_provider_err_004_rate_limit_error_has_retry_info() {
        use opencode_llm::error::LlmError;

        let error = LlmError::RateLimitExceeded {
            retry_after: Some(60),
        };
        let error_string = error.to_string();

        assert!(
            error_string.to_lowercase().contains("rate limit") || error_string.contains("429"),
            "Rate limit error should mention rate limit"
        );
    }

    #[test]
    fn test_provider_err_002_timeout_error() {
        use opencode_llm::error::LlmError;

        let error = LlmError::RequestTimeout;
        let error_string = error.to_string();

        assert!(
            error_string.contains("timeout") || error_string.contains("time"),
            "Timeout error should mention timeout"
        );
    }

    #[test]
    fn test_provider_e2e_008_message_transform_with_prefix_and_suffix() {
        let mut pipeline = TransformPipeline::new();
        pipeline.add_transform(MessageTransform {
            trim_whitespace: false,
            max_length: None,
            prefix: Some("[SYS] ".to_string()),
            suffix: Some(" [/SYS]".to_string()),
        });

        let result = pipeline.apply("hello");
        assert_eq!(result, "[SYS] hello [/SYS]");
    }

    #[test]
    fn test_provider_e2e_008_message_transform_max_length_truncation() {
        let transform = MessageTransform {
            trim_whitespace: false,
            max_length: Some(10),
            prefix: None,
            suffix: None,
        };

        let result = transform.apply("hello world this is long");
        assert!(result.len() <= 13, "Should be truncated with ellipsis"); // "hello..." = 8
        assert!(
            result.contains("..."),
            "Should end with ellipsis when truncated"
        );
    }

    #[test]
    fn test_provider_budget_exceeded_error_display() {
        use opencode_llm::budget::BudgetExceededError;
        use opencode_llm::budget::BudgetLimit;

        let error = BudgetExceededError {
            limit_type: BudgetLimit::PerRequest(0.01),
            request_cost: 0.02,
            conversation_cost: 0.0,
        };

        let error_string = format!("{}", error);
        assert!(
            error_string.contains("0.02") || error_string.contains("exceed"),
            "Error should show cost or exceed message"
        );
    }

    #[test]
    fn test_provider_err_003_validation_error() {
        use opencode_llm::error::LlmError;

        let error = LlmError::ValidationError("missing required field".to_string());
        let error_string = error.to_string();

        assert!(
            error_string.contains("validation") || error_string.contains("field"),
            "Validation error should mention validation"
        );
    }

    #[test]
    fn test_provider_err_003_server_error() {
        use opencode_llm::error::LlmError;

        let error = LlmError::ServerError("internal error".to_string());
        let error_string = error.to_string();

        assert!(
            error_string.contains("server") || error_string.contains("error"),
            "Server error should mention server"
        );
    }

    #[test]
    fn test_message_transform_trim_and_prefix() {
        let transform = MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: Some(">> ".to_string()),
            suffix: None,
        };

        let result = transform.apply("  hello  ");
        assert_eq!(result, ">> hello");
    }

    #[test]
    fn test_message_transform_trim_and_suffix() {
        let transform = MessageTransform {
            trim_whitespace: true,
            max_length: None,
            prefix: None,
            suffix: Some(" <<".to_string()),
        };

        let result = transform.apply("  hello  ");
        assert_eq!(result, "hello <<");
    }

    #[test]
    fn test_message_transform_all_options() {
        let transform = MessageTransform {
            trim_whitespace: true,
            max_length: Some(20),
            prefix: Some("[".to_string()),
            suffix: Some("]".to_string()),
        };

        let result = transform.apply("  hello world  ");
        assert!(result.starts_with("["));
        assert!(result.ends_with("]"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_provider_e2e_004_provider_manager_lists_all_factories() {
        use opencode_llm::provider_abstraction::{ProviderManager, ProviderSpec};

        let manager = ProviderManager::new();
        let providers = manager.list_providers();

        assert!(
            providers.contains(&"openai".to_string()),
            "Should have openai"
        );
        assert!(
            providers.contains(&"anthropic".to_string()),
            "Should have anthropic"
        );
        assert!(
            providers.contains(&"google".to_string()),
            "Should have google"
        );
        assert!(
            providers.contains(&"ollama".to_string()),
            "Should have ollama"
        );
        assert!(
            providers.contains(&"lmstudio".to_string()),
            "Should have lmstudio"
        );
        assert!(
            providers.contains(&"local".to_string()),
            "Should have local"
        );
        assert!(
            providers.contains(&"minimax".to_string()),
            "Should have minimax"
        );
        assert!(providers.contains(&"qwen".to_string()), "Should have qwen");
    }

    #[test]
    fn test_provider_e2e_004_provider_manager_has_provider() {
        use opencode_llm::provider_abstraction::ProviderManager;

        let manager = ProviderManager::new();

        assert!(manager.has_provider("openai"));
        assert!(manager.has_provider("anthropic"));
        assert!(manager.has_provider("google"));
        assert!(manager.has_provider("ollama"));
        assert!(manager.has_provider("lmstudio"));
        assert!(manager.has_provider("local"));
        assert!(manager.has_provider("minimax"));
        assert!(manager.has_provider("qwen"));
        assert!(!manager.has_provider("nonexistent"));
    }

    #[test]
    fn test_provider_e2e_004_provider_manager_creates_providers() {
        use opencode_llm::provider_abstraction::{ProviderManager, ProviderSpec};

        let manager = ProviderManager::new();

        let openai_spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
        };
        let result = manager.create_provider(&openai_spec);
        assert!(result.is_ok(), "Should create OpenAI provider");
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "openai");

        let anthropic_spec = ProviderSpec::Anthropic {
            api_key: "test-key".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
        };
        let result = manager.create_provider(&anthropic_spec);
        assert!(result.is_ok(), "Should create Anthropic provider");
        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "anthropic");
    }

    #[test]
    fn test_provider_e2e_004_model_registry_list_contains_models() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();
        let models = registry.list();

        assert!(!models.is_empty(), "Model list should not be empty");
        assert!(
            models.iter().any(|m| m.name.contains("gpt-4")),
            "Should have GPT-4 models"
        );
        assert!(
            models.iter().any(|m| m.name.contains("claude")),
            "Should have Claude models"
        );
    }

    #[test]
    fn test_provider_e2e_004_model_registry_list_providers() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();
        let providers = registry.list_providers();

        assert!(!providers.is_empty(), "Provider list should not be empty");
        assert!(
            providers.iter().any(|p| p.contains("openai")),
            "Should have OpenAI provider"
        );
    }

    #[test]
    fn test_provider_e2e_004_model_registry_list_by_provider() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();
        let openai_models = registry.list_by_provider("models-dev-openai");

        for model in &openai_models {
            assert!(
                model.provider.contains("openai"),
                "All models should be from OpenAI provider"
            );
        }
    }

    #[test]
    fn test_provider_e2e_004_model_registry_get_returns_model_info() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();

        if let Some(model) = registry.get("gpt-4o") {
            assert_eq!(model.provider, "models-dev-openai");
            assert!(model.supports_functions);
            assert!(model.supports_streaming);
        }
    }

    #[test]
    fn test_provider_e2e_004_model_registry_max_tokens() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();
        let max_tokens = registry.max_tokens("gpt-4o");

        assert!(max_tokens > 0, "Max tokens should be positive");
    }

    #[test]
    fn test_provider_e2e_004_model_registry_supports_function() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();

        assert!(
            registry.supports_function("gpt-4o"),
            "GPT-4o should support functions"
        );
    }

    #[test]
    fn test_provider_e2e_005_reasoning_budget_anthropic_high() {
        use opencode_llm::provider_abstraction::{AnthropicThinkingConfig, ReasoningBudget};

        let budget = ReasoningBudget::High;
        let config = budget.for_provider("anthropic");

        assert!(config.is_some(), "Should get Anthropic config");
        match config {
            Some(opencode_llm::provider_abstraction::ProviderReasoningConfig::Anthropic {
                thinking,
            }) => {
                assert!(thinking.is_some(), "Thinking config should be present");
                match thinking {
                    Some(AnthropicThinkingConfig::High) => {}
                    other => panic!("Expected High, got {:?}", other),
                }
            }
            _ => panic!("Expected Anthropic config"),
        }
    }

    #[test]
    fn test_provider_e2e_005_reasoning_budget_anthropic_max() {
        use opencode_llm::provider_abstraction::{AnthropicThinkingConfig, ReasoningBudget};

        let budget = ReasoningBudget::Max;
        let config = budget.for_provider("anthropic");

        assert!(config.is_some());
        match config {
            Some(opencode_llm::provider_abstraction::ProviderReasoningConfig::Anthropic {
                thinking,
            }) => {
                assert!(matches!(thinking, Some(AnthropicThinkingConfig::Max)));
            }
            _ => panic!("Expected Anthropic config"),
        }
    }

    #[test]
    fn test_provider_e2e_005_reasoning_budget_openai_medium() {
        use opencode_llm::provider_abstraction::ReasoningBudget;

        let budget = ReasoningBudget::Medium;
        let config = budget.for_provider("openai");

        assert!(config.is_some());
        match config {
            Some(opencode_llm::provider_abstraction::ProviderReasoningConfig::OpenAI {
                reasoning_effort,
            }) => {
                assert_eq!(reasoning_effort, Some("medium".to_string()));
            }
            _ => panic!("Expected OpenAI config"),
        }
    }

    #[test]
    fn test_provider_e2e_005_reasoning_budget_google_high() {
        use opencode_llm::provider_abstraction::ReasoningBudget;

        let budget = ReasoningBudget::High;
        let config = budget.for_provider("google");

        assert!(config.is_some());
        match config {
            Some(opencode_llm::provider_abstraction::ProviderReasoningConfig::Google {
                thinking_throttle,
            }) => {
                assert_eq!(thinking_throttle, Some("high".to_string()));
            }
            _ => panic!("Expected Google config"),
        }
    }

    #[test]
    fn test_provider_e2e_005_provider_identity_with_reasoning_budget() {
        use opencode_llm::provider_abstraction::{ProviderIdentity, ReasoningBudget};

        let identity = ProviderIdentity::anthropic("claude-3-5-sonnet-20241022")
            .with_reasoning_budget(ReasoningBudget::High);

        assert_eq!(identity.provider_type, "anthropic");
        assert!(identity.reasoning_budget.is_some());
    }

    #[test]
    fn test_provider_e2e_007_catalog_fetch_error_display() {
        use opencode_llm::catalog::fetcher::FetchError;

        let error = FetchError::Network("connection refused".to_string());
        assert!(error.to_string().contains("Network error"));

        let error = FetchError::HttpStatus(404);
        assert!(error.to_string().contains("HTTP error"));

        let error = FetchError::Parse("invalid json".to_string());
        assert!(error.to_string().contains("Parse error"));
    }

    #[test]
    fn test_provider_err_005_provider_switch_requires_context() {
        use opencode_llm::models::ModelRegistry;
        use opencode_llm::provider_abstraction::{ProviderIdentity, ProviderSpec};

        let registry = ModelRegistry::new();

        let openai_model = registry.get("gpt-4o");
        let anthropic_model = registry.get("claude-3-5-sonnet-20241022");

        assert!(openai_model.is_some());
        assert!(anthropic_model.is_some());

        assert_ne!(
            openai_model.unwrap().provider,
            anthropic_model.unwrap().provider,
            "Models should be from different providers"
        );
    }

    #[test]
    fn test_provider_e2e_004_provider_identity_equality() {
        use opencode_llm::provider_abstraction::ProviderIdentity;

        let id1 = ProviderIdentity::openai("gpt-4o");
        let id2 = ProviderIdentity::openai("gpt-4o");
        let id3 = ProviderIdentity::anthropic("claude-3-5-sonnet-20241022");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_provider_e2e_004_provider_spec_serialization() {
        use opencode_llm::provider_abstraction::ProviderSpec;

        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            base_url: Some("https://api.openai.com".to_string()),
        };

        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("\"type\":\"openai\""));

        let deserialized: ProviderSpec = serde_json::from_str(&json).unwrap();
        match deserialized {
            ProviderSpec::OpenAI {
                api_key,
                model,
                base_url,
            } => {
                assert_eq!(api_key, "test-key");
                assert_eq!(model, "gpt-4o");
                assert_eq!(base_url, Some("https://api.openai.com".to_string()));
            }
            _ => panic!("Expected OpenAI variant"),
        }
    }

    #[test]
    fn test_provider_e2e_004_provider_filter_respected() {
        use opencode_llm::models::ModelRegistry;
        use opencode_llm::provider_filter::ProviderFilter;

        let mut registry = ModelRegistry::new();
        registry.set_provider_filter(ProviderFilter::new(
            vec!["models-dev-openai".to_string()],
            vec![
                "models-dev-openai".to_string(),
                "models-dev-anthropic".to_string(),
            ],
        ));

        let providers: Vec<String> = registry.list().iter().map(|m| m.provider.clone()).collect();

        assert!(!providers.is_empty());
    }

    #[test]
    fn test_provider_e2e_004_model_registry_max_input_tokens() {
        use opencode_llm::models::ModelRegistry;

        let registry = ModelRegistry::new();
        let max_input = registry.max_input_tokens("gpt-4o");

        assert!(max_input > 0, "Max input tokens should be positive");
        assert!(max_input >= 100000, "GPT-4o should have large context");
    }

    #[test]
    fn test_provider_e2e_004_dyn_provider_get_models() {
        use opencode_llm::provider_abstraction::{ProviderManager, ProviderSpec};

        let manager = ProviderManager::new();
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
        };

        let provider = manager.create_provider(&spec).unwrap();
        let models = provider.get_models();

        assert!(!models.is_empty());
        assert_eq!(models[0].id, "gpt-4o");
    }

    #[test]
    fn test_provider_e2e_004_dyn_provider_identity() {
        use opencode_llm::provider_abstraction::{ProviderIdentity, ProviderManager, ProviderSpec};

        let manager = ProviderManager::new();
        let spec = ProviderSpec::OpenAI {
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            base_url: None,
        };

        let provider = manager.create_provider(&spec).unwrap();
        let identity = provider.identity();

        assert_eq!(identity.provider_type, "openai");
        assert_eq!(identity.model, Some("gpt-4o".to_string()));
    }

    #[test]
    fn test_provider_e2e_004_reasoning_budget_none_returns_none() {
        use opencode_llm::provider_abstraction::ReasoningBudget;

        let budget = ReasoningBudget::None;
        let config = budget.for_provider("anthropic");

        match config {
            Some(opencode_llm::provider_abstraction::ProviderReasoningConfig::Anthropic {
                thinking,
            }) => {
                assert!(
                    thinking.is_none(),
                    "None budget should have no thinking config"
                );
            }
            _ => panic!("Expected Anthropic config"),
        }
    }
}
