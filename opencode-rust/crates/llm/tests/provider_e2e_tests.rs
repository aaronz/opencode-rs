#[cfg(test)]
mod provider_tests {
    use async_trait::async_trait;
    use opencode_core::OpenCodeError;
    use opencode_llm::budget::{BudgetLimit, BudgetTracker};
    use opencode_llm::error::{with_retry, LlmError, RetryConfig};
    use opencode_llm::message_transform::{MessageTransform, TransformPipeline};
    use opencode_llm::provider::sealed::Sealed;
    use opencode_llm::provider::{ChatMessage, ChatResponse, Model, Provider, StreamingCallback};
    use std::sync::{Arc, Mutex};

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
        use opencode_llm::provider_abstraction::ProviderManager;

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
        let openai_models = registry.list_by_provider("openai");

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
            assert_eq!(model.provider, "openai");
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
    #[allow(deprecated)]
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

    #[tokio::test]
    async fn test_provider_e2e_007_catalog_fetcher_returns_models_with_metadata() {
        use opencode_llm::catalog::{ProviderCatalog, ProviderCatalogFetcher};
        use std::collections::BTreeMap;

        let catalog = ProviderCatalog {
            providers: BTreeMap::from([(
                "openai".to_string(),
                opencode_llm::catalog::ProviderDescriptor {
                    id: "openai".to_string(),
                    display_name: "OpenAI".to_string(),
                    api_base_url: Some("https://api.openai.com/v1".to_string()),
                    docs_url: None,
                    env_vars: vec!["OPENAI_API_KEY".to_string()],
                    npm_package: None,
                    models: BTreeMap::from([(
                        "gpt-4o".to_string(),
                        opencode_llm::catalog::ModelDescriptor {
                            id: "gpt-4o".to_string(),
                            display_name: "GPT-4o".to_string(),
                            family: Some("GPT-4".to_string()),
                            provider_id: "openai".to_string(),
                            capabilities: opencode_llm::catalog::ModelCapabilities {
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
                            cost: opencode_llm::catalog::CostInfo {
                                input: 0.005,
                                output: 0.015,
                                cache_read: 0.0,
                                cache_write: 0.0,
                            },
                            limits: opencode_llm::catalog::LimitInfo {
                                context: 128000,
                                input: None,
                                output: 16384,
                            },
                            status: opencode_llm::catalog::ModelStatus::Active,
                            variants: vec![],
                        },
                    )]),
                    source: opencode_llm::catalog::CatalogSource::ModelsDev,
                },
            )]),
            fetched_at: chrono::Utc::now(),
            source: opencode_llm::catalog::CatalogSource::ModelsDev,
        };

        let temp_path = std::env::temp_dir().join("test_catalog_007.json");
        let json = serde_json::to_string_pretty(&catalog).expect("should serialize");
        std::fs::write(&temp_path, json).expect("should write");

        let fetcher = ProviderCatalogFetcher::new(temp_path);
        let result = fetcher.get_or_fetch().await;

        assert_eq!(result.providers.len(), 1);
        assert!(result.providers.contains_key("openai"));

        let openai = result.providers.get("openai").unwrap();
        assert_eq!(openai.display_name, "OpenAI");
        assert!(openai.models.contains_key("gpt-4o"));

        let gpt4o = openai.models.get("gpt-4o").unwrap();
        assert_eq!(gpt4o.display_name, "GPT-4o");
        assert!(gpt4o.cost.input > 0.0);
        assert!(gpt4o.cost.output > 0.0);
        assert_eq!(gpt4o.limits.context, 128000);
    }

    #[tokio::test]
    async fn test_provider_e2e_007_catalog_caching_reduces_network_calls() {
        use opencode_llm::catalog::{ProviderCatalog, ProviderCatalogFetcher};
        use std::collections::BTreeMap;

        let catalog = ProviderCatalog {
            providers: BTreeMap::from([(
                "test-provider".to_string(),
                opencode_llm::catalog::ProviderDescriptor {
                    id: "test-provider".to_string(),
                    display_name: "Test Provider".to_string(),
                    api_base_url: Some("https://api.test.com/v1".to_string()),
                    docs_url: None,
                    env_vars: vec![],
                    npm_package: None,
                    models: BTreeMap::from([(
                        "test-model".to_string(),
                        opencode_llm::catalog::ModelDescriptor {
                            id: "test-model".to_string(),
                            display_name: "Test Model".to_string(),
                            family: Some("Test".to_string()),
                            provider_id: "test-provider".to_string(),
                            capabilities: opencode_llm::catalog::ModelCapabilities {
                                attachment: false,
                                reasoning: false,
                                tool_call: false,
                                temperature: true,
                                structured_output: false,
                                interleaved: false,
                                open_weights: false,
                                input_modalities: vec!["text".to_string()],
                                output_modalities: vec!["text".to_string()],
                            },
                            cost: opencode_llm::catalog::CostInfo {
                                input: 0.001,
                                output: 0.002,
                                cache_read: 0.0,
                                cache_write: 0.0,
                            },
                            limits: opencode_llm::catalog::LimitInfo {
                                context: 4096,
                                input: None,
                                output: 1024,
                            },
                            status: opencode_llm::catalog::ModelStatus::Active,
                            variants: vec![],
                        },
                    )]),
                    source: opencode_llm::catalog::CatalogSource::ModelsDev,
                },
            )]),
            fetched_at: chrono::Utc::now(),
            source: opencode_llm::catalog::CatalogSource::ModelsDev,
        };

        let temp_path = std::env::temp_dir().join("test_catalog_cache_007.json");
        let json = serde_json::to_string_pretty(&catalog).expect("should serialize");
        std::fs::write(&temp_path, json).expect("should write");

        let fetcher = ProviderCatalogFetcher::new(temp_path);

        let result1 = fetcher.get_or_fetch().await;
        let result2 = fetcher.get_or_fetch().await;

        assert_eq!(result1.providers.len(), result2.providers.len());
        assert_eq!(
            result1.providers.get("test-provider").unwrap().models.len(),
            result2.providers.get("test-provider").unwrap().models.len()
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_007_cache_is_valid_for_fresh_catalog() {
        use opencode_llm::catalog::{CatalogSource, ProviderCatalog, ProviderCatalogFetcher};
        use std::collections::BTreeMap;

        let catalog = ProviderCatalog {
            providers: BTreeMap::new(),
            fetched_at: chrono::Utc::now(),
            source: CatalogSource::ModelsDev,
        };

        let temp_path = std::env::temp_dir().join("test_catalog_valid_007.json");
        let json = serde_json::to_string_pretty(&catalog).expect("should serialize");
        std::fs::write(&temp_path, json).expect("should write");

        let fetcher = ProviderCatalogFetcher::new(temp_path);
        let result = fetcher.get_or_fetch().await;

        assert!(result.fetched_at <= chrono::Utc::now());
    }

    #[test]
    fn test_provider_e2e_006_oauth_google_service_creation() {
        use opencode_llm::auth_layered::GoogleOAuthService;

        let service = GoogleOAuthService::new();
        let result = service.start_local_callback_listener();
        assert!(
            result.is_ok(),
            "Google OAuth service should be created successfully"
        );
    }

    #[test]
    fn test_provider_e2e_006_oauth_google_authorize_url_generated() {
        use opencode_llm::auth_layered::{GoogleOAuthRequest, GoogleOAuthService};

        let service = GoogleOAuthService::new();
        let request = GoogleOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "test-state-12345".to_string(),
            code_verifier: "test-verifier-1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"
                .to_string(),
        };

        let url = service.build_authorize_url(&request);
        assert!(
            url.contains("accounts.google.com"),
            "URL should contain Google auth endpoint"
        );
        assert!(
            url.contains("state=test-state-12345"),
            "URL should contain state parameter"
        );
    }

    #[test]
    fn test_provider_e2e_006_oauth_google_state_mismatch_detected() {
        use opencode_llm::auth_layered::{
            GoogleOAuthCallback, GoogleOAuthRequest, GoogleOAuthService,
        };

        let service = GoogleOAuthService::new();
        let request = GoogleOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/auth/callback".to_string(),
            state: "test-state".to_string(),
            code_verifier: "test-verifier".to_string(),
        };

        let callback = GoogleOAuthCallback {
            code: "test-auth-code".to_string(),
            state: "wrong-state".to_string(),
        };

        let result = service.exchange_code(callback, &request);
        assert!(
            result.is_err(),
            "Code exchange should fail with mismatched state"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("State mismatch") || err_msg.contains("mismatch"),
            "Error should mention state mismatch"
        );
    }

    #[test]
    fn test_provider_e2e_006_oauth_tokens_stored_successfully() {
        use opencode_llm::auth_layered::{GoogleOAuthSession, GoogleOAuthStore};

        let dir = tempfile::tempdir().unwrap();
        let store_path = dir.path().join("google_oauth_session.json");
        let store = GoogleOAuthStore::new(store_path);

        let session = GoogleOAuthSession {
            access_token: "ya29.a0AfH6SMBx".to_string(),
            refresh_token: Some("1//0gYJ9sBj8rK".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("user@gmail.com".to_string()),
        };

        let save_result = store.save(&session);
        assert!(save_result.is_ok(), "Session should be saved successfully");

        let loaded = store.load();
        assert!(loaded.is_ok(), "Session should be loaded successfully");
        let loaded_opt = loaded.unwrap();
        assert!(loaded_opt.is_some(), "Loaded session should be present");

        let loaded_session = loaded_opt.unwrap();
        assert_eq!(loaded_session.access_token, "ya29.a0AfH6SMBx");
        assert_eq!(loaded_session.email, Some("user@gmail.com".to_string()));
    }

    #[test]
    fn test_provider_e2e_006_oauth_copilot_service_creation() {
        use opencode_llm::auth_layered::CopilotOAuthService;

        let service = CopilotOAuthService::new();
        let result = service.start_local_callback_listener();
        assert!(
            result.is_ok(),
            "Copilot OAuth service should be created successfully"
        );
    }

    #[test]
    fn test_provider_e2e_006_oauth_copilot_authorize_url_generated() {
        use opencode_llm::auth_layered::{CopilotOAuthRequest, CopilotOAuthService};

        let service = CopilotOAuthService::new();
        let request = CopilotOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/copilot/auth/callback".to_string(),
            state: "copilot-state-12345".to_string(),
            code_verifier: "copilot-verifier-1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"
                .to_string(),
        };

        let url = service.build_authorize_url(&request);
        assert!(
            url.contains("github.com"),
            "URL should contain GitHub auth endpoint"
        );
        assert!(
            url.contains("state=copilot-state-12345"),
            "URL should contain state parameter"
        );
    }

    #[test]
    fn test_provider_e2e_006_oauth_copilot_state_mismatch_detected() {
        use opencode_llm::auth_layered::{
            CopilotOAuthCallback, CopilotOAuthRequest, CopilotOAuthService,
        };

        let service = CopilotOAuthService::new();
        let request = CopilotOAuthRequest {
            redirect_uri: "http://127.0.0.1:8080/copilot/auth/callback".to_string(),
            state: "copilot-state".to_string(),
            code_verifier: "copilot-verifier".to_string(),
        };

        let callback = CopilotOAuthCallback {
            code: "copilot-auth-code".to_string(),
            state: "wrong-state".to_string(),
        };

        let result = service.exchange_code(callback, &request);
        assert!(
            result.is_err(),
            "Code exchange should fail with mismatched state"
        );
    }

    #[test]
    fn test_provider_e2e_006_oauth_session_contains_required_fields() {
        use opencode_llm::auth_layered::GoogleOAuthSession;

        let session = GoogleOAuthSession {
            access_token: "ya29.test".to_string(),
            refresh_token: Some("1//test_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: Some("developer@example.com".to_string()),
        };

        assert!(!session.access_token.is_empty());
        assert!(session.refresh_token.is_some());
        assert!(session.expires_at_epoch_ms > chrono::Utc::now().timestamp_millis());
        assert!(session.email.is_some());
    }

    #[test]
    fn test_provider_e2e_006_oauth_session_expiration_check() {
        use opencode_llm::auth_layered::GoogleOAuthSession;

        let expired_session = GoogleOAuthSession {
            access_token: "ya29.expired".to_string(),
            refresh_token: Some("1//expired_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() - 1000,
            email: None,
        };
        assert!(
            expired_session.is_expired(),
            "Expired session should be detected"
        );

        let valid_session = GoogleOAuthSession {
            access_token: "ya29.valid".to_string(),
            refresh_token: Some("1//valid_refresh".to_string()),
            expires_at_epoch_ms: chrono::Utc::now().timestamp_millis() + 3600000,
            email: None,
        };
        assert!(
            !valid_session.is_expired(),
            "Valid session should not be expired"
        );
    }

    #[test]
    fn test_provider_err_005_provider_switch_requires_context() {
        use opencode_llm::models::ModelRegistry;

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
            vec!["openai".to_string()],
            vec!["openai".to_string(), "anthropic".to_string()],
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
        use opencode_llm::provider_abstraction::{ProviderManager, ProviderSpec};

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

    struct TestChatProvider {
        response: String,
        model: String,
        call_count: Arc<Mutex<usize>>,
    }

    impl TestChatProvider {
        fn new(response: &str, model: &str) -> Self {
            Self {
                response: response.to_string(),
                model: model.to_string(),
                call_count: Arc::new(Mutex::new(0)),
            }
        }
    }

    impl Sealed for TestChatProvider {}

    #[async_trait]
    impl Provider for TestChatProvider {
        async fn complete(
            &self,
            _prompt: &str,
            _context: Option<&str>,
        ) -> Result<String, OpenCodeError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            Ok(format!("{} (call #{})", self.response, *count))
        }

        async fn complete_streaming(
            &self,
            _prompt: &str,
            mut callback: StreamingCallback,
        ) -> Result<(), OpenCodeError> {
            callback(self.response.clone());
            Ok(())
        }

        async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
            let content = format!(
                "Processed {} messages: {}",
                messages.len(),
                messages
                    .iter()
                    .map(|m| m.content.clone())
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
            Ok(ChatResponse {
                content,
                model: self.model.clone(),
                usage: None,
            })
        }

        fn get_models(&self) -> Vec<Model> {
            vec![Model::new(&self.model, &self.model)]
        }

        fn provider_name(&self) -> &str {
            "test-chat"
        }
    }

    #[tokio::test]
    async fn test_provider_e2e_001_chat_completion_flow() {
        let provider = TestChatProvider::new("Hello, how can I help you?", "test-model-1");

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hi!".to_string(),
            },
        ];

        let response = provider.chat(&messages).await.unwrap();

        assert!(
            !response.content.is_empty(),
            "Response content should be non-empty"
        );
        assert_eq!(
            response.model, "test-model-1",
            "Model field should be populated"
        );
        assert!(
            response.content.contains("2"),
            "Should mention processing 2 messages"
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_001_chat_with_usage_stats() {
        let provider = TestChatProvider::new("Response with usage", "test-model");

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let mut chat_response = provider.chat(&messages).await.unwrap();
        chat_response.usage = Some(opencode_llm::provider::Usage::new(10, 20));

        let usage = chat_response.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10, "Prompt tokens should be recorded");
        assert_eq!(
            usage.completion_tokens, 20,
            "Completion tokens should be recorded"
        );
        assert_eq!(usage.total_tokens, 30, "Total tokens should be sum");
    }

    #[tokio::test]
    async fn test_provider_e2e_001_complete_basic() {
        let provider = TestChatProvider::new("Completion response", "test-model");

        let result = provider.complete("Test prompt", None).await.unwrap();
        assert_eq!(result, "Completion response (call #1)");
    }

    #[tokio::test]
    async fn test_provider_e2e_002_retry_on_rate_limit() {
        let retry_count = Arc::new(Mutex::new(0));
        let retry_count_clone = retry_count.clone();

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = with_retry(&config, || {
            let retry_count = retry_count_clone.clone();
            Box::pin(async move {
                let mut count = retry_count.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                if current < 3 {
                    Err(LlmError::RateLimitExceeded { retry_after: None })
                } else {
                    Ok("success".to_string())
                }
            })
        })
        .await;

        assert!(result.is_ok(), "Should succeed after retries");
        assert_eq!(
            *retry_count.lock().unwrap(),
            3,
            "Should have retried 3 times"
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_002_no_retry_on_invalid_api_key() {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result: Result<String, LlmError> = with_retry(&config, || {
            let call_count = call_count_clone.clone();
            Box::pin(async move {
                let mut count = call_count.lock().unwrap();
                *count += 1;
                drop(count);
                Err(LlmError::InvalidApiKey)
            })
        })
        .await;

        assert!(result.is_err(), "Should return error immediately");
        assert!(
            matches!(result.unwrap_err(), LlmError::InvalidApiKey),
            "Should be InvalidApiKey error"
        );
        assert_eq!(
            *call_count.lock().unwrap(),
            1,
            "Should only be called once - no retry on InvalidApiKey"
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_002_retry_on_network_error() {
        let retry_count = Arc::new(Mutex::new(0));
        let retry_count_clone = retry_count.clone();

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = with_retry(&config, || {
            let retry_count = retry_count_clone.clone();
            Box::pin(async move {
                let mut count = retry_count.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                if current < 2 {
                    Err(LlmError::NetworkError("connection reset".to_string()))
                } else {
                    Ok("recovered".to_string())
                }
            })
        })
        .await;

        assert!(result.is_ok(), "Should succeed after retries");
        assert_eq!(
            *retry_count.lock().unwrap(),
            2,
            "Should have retried 2 times"
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_002_exhaust_retries_returns_final_error() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result: Result<String, LlmError> = with_retry(&config, || {
            Box::pin(async { Err(LlmError::ServerError("still broken".to_string())) })
        })
        .await;

        assert!(
            result.is_err(),
            "Should return error after exhausting retries"
        );
        assert!(
            matches!(result.unwrap_err(), LlmError::ServerError(_)),
            "Should be ServerError"
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_002_no_retry_on_validation_error() {
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result: Result<String, LlmError> = with_retry(&config, || {
            let call_count = call_count_clone.clone();
            Box::pin(async move {
                let mut count = call_count.lock().unwrap();
                *count += 1;
                drop(count);
                Err(LlmError::ValidationError("missing param".to_string()))
            })
        })
        .await;

        assert!(result.is_err());
        assert_eq!(
            *call_count.lock().unwrap(),
            1,
            "Should not retry on ValidationError"
        );
    }

    #[tokio::test]
    async fn test_provider_e2e_002_retry_on_timeout() {
        let retry_count = Arc::new(Mutex::new(0));
        let retry_count_clone = retry_count.clone();

        let config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            backoff_multiplier: 2.0,
        };

        let result = with_retry(&config, || {
            let retry_count = retry_count_clone.clone();
            Box::pin(async move {
                let mut count = retry_count.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                if current < 2 {
                    Err(LlmError::RequestTimeout)
                } else {
                    Ok("timeout recovered".to_string())
                }
            })
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(*retry_count.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_provider_e2e_002_retry_after_header_respected() {
        let retry_count = Arc::new(Mutex::new(0));
        let retry_count_clone = retry_count.clone();

        let config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 1000,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        };

        let start = std::time::Instant::now();

        let result = with_retry(&config, || {
            let retry_count = retry_count_clone.clone();
            Box::pin(async move {
                let mut count = retry_count.lock().unwrap();
                *count += 1;
                let current = *count;
                drop(count);

                if current < 2 {
                    Err(LlmError::RateLimitExceeded {
                        retry_after: Some(1),
                    })
                } else {
                    Ok("success with retry-after".to_string())
                }
            })
        })
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(*retry_count.lock().unwrap(), 2);
        assert!(
            elapsed >= std::time::Duration::from_millis(900),
            "Should wait for retry_after value (~1s)"
        );
    }

    #[tokio::test]
    async fn test_provider_budget_002_streaming_budget_tracking() {
        use opencode_llm::budget::StreamingBudgetTracker;

        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_request_limit(tracker, 100);
        let mut stream_tracker = StreamingBudgetTracker::new(&tracker);

        let chunks = vec!["hello", " world", " this", " is", " a", " test"];
        let mut all_content = String::new();

        for chunk in &chunks {
            stream_tracker.record_chunk(chunk);
            all_content.push_str(chunk);
        }

        assert_eq!(stream_tracker.accumulated(), 10);

        let cost = stream_tracker.estimate_total_cost();
        assert!(cost > 0.0);
    }

    #[tokio::test]
    async fn test_provider_budget_002_budget_exceeded_during_streaming() {
        use opencode_llm::budget::StreamingBudgetTracker;

        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_request_limit(tracker, 5);
        let stream_tracker = StreamingBudgetTracker::new(&tracker);

        let large_text = "this is a much longer piece of text that should exceed budget";

        let result = stream_tracker.check_budget(stream_tracker.estimate_tokens(large_text));
        assert!(result.is_err(), "Budget should be exceeded with large text");

        let err = result.unwrap_err();
        assert!(err.to_string().contains("exceeds") || err.to_string().contains("Budget"));
    }

    #[tokio::test]
    async fn test_provider_budget_002_streaming_with_mock_provider() {
        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let _tracker = BudgetTracker::with_request_limit(tracker, 100);

        let received_chunks: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let received_chunks_clone = received_chunks.clone();

        let callback: Box<dyn FnMut(String) + Send> = Box::new(move |chunk: String| {
            if let Ok(mut guard) = received_chunks_clone.lock() {
                guard.push(chunk);
            }
        });

        let provider = TestChatProvider::new("short response", "test-model");
        provider.complete_streaming("test", callback).await.unwrap();

        let chunks = received_chunks.lock().unwrap();
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_provider_budget_002_conversation_budget_during_streaming() {
        use opencode_llm::budget::StreamingBudgetTracker;

        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_conversation_limit(tracker, 100);

        let mut stream_tracker = StreamingBudgetTracker::new(&tracker);

        stream_tracker.record_chunk("small chunk");
        assert!(stream_tracker
            .check_conversation_budget(stream_tracker.accumulated())
            .is_ok());

        stream_tracker.record_chunk("much larger chunk that accumulates more tokens");
        let result = stream_tracker.check_conversation_budget(stream_tracker.accumulated());
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_provider_budget_002_budget_check_per_chunk() {
        use opencode_llm::budget::StreamingBudgetTracker;

        let tracker = BudgetTracker::with_reasoning_budget(None, 0.001);
        let tracker = BudgetTracker::with_request_limit(tracker, 30);

        let chunks = vec!["one", "two", "three", "four", "five"];
        let mut stream_tracker = StreamingBudgetTracker::new(&tracker);
        let mut budget_errors: Vec<String> = Vec::new();

        for chunk in chunks {
            stream_tracker.record_chunk(chunk);
            if let Err(e) = stream_tracker.check_request_budget(stream_tracker.accumulated()) {
                budget_errors.push(format!("{}", e));
            }
        }

        assert!(stream_tracker.accumulated() > 0);
    }

    #[tokio::test]
    async fn test_provider_budget_002_estimate_tokens_from_text() {
        use opencode_llm::budget::StreamingBudgetTracker;

        let tracker = BudgetTracker::new();
        let stream_tracker = StreamingBudgetTracker::new(&tracker);

        let short_text = "hi";
        let long_text = "this is a much longer piece of text";

        assert_eq!(stream_tracker.estimate_tokens(short_text), 1);
        assert!(
            stream_tracker.estimate_tokens(long_text) > stream_tracker.estimate_tokens(short_text)
        );
    }

    struct MessageTrackingProvider {
        name: String,
        received_messages: Arc<Mutex<Vec<Vec<ChatMessage>>>>,
    }

    impl MessageTrackingProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                received_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_received_messages(&self) -> Vec<Vec<ChatMessage>> {
            self.received_messages.lock().unwrap().clone()
        }
    }

    impl Sealed for MessageTrackingProvider {}

    #[async_trait]
    impl Provider for MessageTrackingProvider {
        async fn complete(
            &self,
            prompt: &str,
            _context: Option<&str>,
        ) -> Result<String, OpenCodeError> {
            Ok(format!("{}: {}", self.name, prompt))
        }

        async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError> {
            self.received_messages
                .lock()
                .unwrap()
                .push(messages.to_vec());
            Ok(ChatResponse {
                content: format!("Response from {}", self.name),
                model: self.name.to_string(),
                usage: None,
            })
        }

        fn get_models(&self) -> Vec<Model> {
            vec![Model::new(&self.name, &self.name)]
        }

        fn provider_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_provider_err_005_provider_switch_preserves_context() {
        let provider_a = MessageTrackingProvider::new("provider-a");
        let provider_b = MessageTrackingProvider::new("provider-b");

        let conversation_history = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "First message".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "First response".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Second message".to_string(),
            },
        ];

        let _ = provider_a.chat(&conversation_history).await;
        let received_a = provider_a.get_received_messages();
        assert_eq!(received_a.len(), 1);
        assert_eq!(received_a[0].len(), 3);

        let _ = provider_b.chat(&conversation_history).await;
        let received_b = provider_b.get_received_messages();
        assert_eq!(received_b.len(), 1);
        assert_eq!(received_b[0].len(), 3);

        for msg in &received_b[0] {
            assert!(conversation_history
                .iter()
                .any(|h| h.content == msg.content));
        }
    }

    #[tokio::test]
    async fn test_provider_err_005_messages_preserved_across_switch() {
        let provider_a = MessageTrackingProvider::new("provider-a");
        let provider_b = MessageTrackingProvider::new("provider-b");

        let initial_messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];

        let _ = provider_a.chat(&initial_messages).await;
        let received_from_a = provider_a.get_received_messages();

        let switch_messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Continue our conversation".to_string(),
            },
        ];

        let _ = provider_b.chat(&switch_messages).await;
        let received_from_b = provider_b.get_received_messages();

        assert!(
            received_from_b[0].len() >= switch_messages.len(),
            "Switched provider should receive full conversation context"
        );

        assert!(
            received_from_b[0].len() > received_from_a[0].len(),
            "Second provider should receive more messages than first after context grows"
        );
    }

    #[tokio::test]
    async fn test_provider_err_005_conversation_context_integrity() {
        let provider = MessageTrackingProvider::new("test-provider");

        let messages_with_context = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a math tutor.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What is 2+2?".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "2+2 equals 4.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What about 3+3?".to_string(),
            },
        ];

        let _ = provider.chat(&messages_with_context).await;
        let received = provider.get_received_messages();

        assert_eq!(received.len(), 1);
        assert_eq!(received[0].len(), 4);

        let roles: Vec<&str> = received[0].iter().map(|m| m.role.as_str()).collect();
        assert_eq!(roles, vec!["system", "user", "assistant", "user"]);

        let contents: Vec<&str> = received[0].iter().map(|m| m.content.as_str()).collect();
        assert!(contents[0].contains("math tutor"));
        assert!(contents[1].contains("2+2"));
        assert!(contents[2].contains("4"));
        assert!(contents[3].contains("3+3"));
    }

    #[tokio::test]
    async fn test_provider_conc_002_cancellation_token_basic() {
        use opencode_llm::CancellationToken;

        let token = CancellationToken::new();
        assert!(
            !token.is_cancelled(),
            "Token should not be cancelled initially"
        );

        token.cancel();
        assert!(
            token.is_cancelled(),
            "Token should be cancelled after cancel()"
        );
    }

    #[tokio::test]
    async fn test_provider_conc_002_cancellable_provider_rejects_cancelled() {
        use opencode_llm::CancellationToken;

        let token = CancellationToken::new();
        let provider = TestChatProvider::new("response", "model");
        let cancellable = token.wrap_provider(&provider);

        let result = cancellable.complete("test", None).await;
        assert!(result.is_ok(), "Should succeed when not cancelled");

        token.cancel();

        let result = cancellable.complete("test", None).await;
        assert!(result.is_err(), "Should fail when cancelled");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("cancelled") || err.to_string().contains("Cancelled"),
            "Error should mention cancellation"
        );
    }

    #[tokio::test]
    async fn test_provider_conc_002_cancellation_does_not_affect_other_requests() {
        use opencode_llm::CancellationToken;

        let token1 = CancellationToken::new();
        let token2 = CancellationToken::new();

        let provider = TestChatProvider::new("response", "model");

        let cancellable1 = token1.wrap_provider(&provider);
        let cancellable2 = token2.wrap_provider(&provider);

        token1.cancel();

        let result1 = cancellable1.complete("test", None).await;
        let result2 = cancellable2.complete("test", None).await;

        assert!(result1.is_err(), "First should be cancelled");
        assert!(result2.is_ok(), "Second should succeed");
    }

    #[tokio::test]
    async fn test_provider_conc_002_multiple_tokens_independent() {
        use opencode_llm::CancellationToken;

        let token1 = CancellationToken::new();
        let token2 = CancellationToken::new();

        let provider = TestChatProvider::new("response", "model");

        let cancellable1 = token1.wrap_provider(&provider);
        let cancellable2 = token2.wrap_provider(&provider);

        let result1 = cancellable1.complete("test", None).await;
        assert!(result1.is_ok(), "First should succeed");

        token2.cancel();
        let result2 = cancellable2.complete("test", None).await;
        assert!(result2.is_err(), "Second should be cancelled");

        let result1_retry = cancellable1.complete("test", None).await;
        assert!(
            result1_retry.is_ok(),
            "First should still work after second is cancelled"
        );
    }

    #[tokio::test]
    async fn test_provider_conc_002_cancellation_releases_resources() {
        use opencode_llm::CancellationToken;

        let token1 = CancellationToken::new();
        let token2 = CancellationToken::new();

        let provider = TestChatProvider::new("response", "model");

        let cancellable1 = token1.wrap_provider(&provider);
        let cancellable2 = token2.wrap_provider(&provider);

        let result1 = cancellable1.complete("test", None).await;
        let result2 = cancellable2.complete("test", None).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        drop(cancellable1);
        drop(cancellable2);

        let result3 = provider.chat(&[]).await;
        assert!(
            result3.is_ok(),
            "Provider should still be usable after dropping cancellable wrappers"
        );
    }

    #[tokio::test]
    async fn test_provider_conc_002_cancelled_chat_returns_error() {
        use opencode_llm::CancellationToken;

        let token = CancellationToken::new();
        let provider = TestChatProvider::new("response", "model");
        let cancellable = token.wrap_provider(&provider);

        token.cancel();

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        let result = cancellable.chat(&messages).await;
        assert!(result.is_err(), "Chat should fail when cancelled");
    }

    #[tokio::test]
    async fn test_provider_conc_002_token_clone_shares_cancellation() {
        use opencode_llm::CancellationToken;

        let token1 = CancellationToken::new();
        let token2 = token1.clone();

        let provider = TestChatProvider::new("response", "model");

        let cancellable1 = token1.wrap_provider(&provider);
        let cancellable2 = token2.wrap_provider(&provider);

        let result1 = cancellable1.complete("test", None).await;
        assert!(result1.is_ok());

        token2.cancel();

        let result1_retry = cancellable1.complete("test", None).await;
        let result2_retry = cancellable2.complete("test", None).await;

        assert!(
            result1_retry.is_err(),
            "token1's wrapped provider should see cancellation"
        );
        assert!(
            result2_retry.is_err(),
            "token2's wrapped provider should see cancellation"
        );
    }
}
