use opencode_llm::{
    ChatMessage, LmStudioProvider, LmStudioProviderFactory, LocalInferenceProviderFactory,
    OllamaProvider, Provider, ProviderFactory, ProviderManager, ProviderSpec,
};

#[test]
fn test_ollama_provider_creation() {
    let provider = OllamaProvider::new(
        "llama2".to_string(),
        Some("http://localhost:11434".to_string()),
    );
    assert_eq!(provider.provider_name(), "ollama");
}

#[test]
fn test_lmstudio_provider_creation() {
    let provider = LmStudioProvider::new(
        "llama2".to_string(),
        Some("http://localhost:1234".to_string()),
    );
    assert_eq!(provider.provider_name(), "lmstudio");
}

#[test]
fn test_lmstudio_provider_default_url() {
    let provider = LmStudioProvider::new("llama2".to_string(), None);
    assert_eq!(provider.provider_name(), "lmstudio");
}

#[test]
fn test_provider_manager_has_local_providers() {
    let manager = ProviderManager::new();
    assert!(manager.has_provider("ollama"));
    assert!(manager.has_provider("lmstudio"));
    assert!(manager.has_provider("local"));
}

#[test]
fn test_provider_manager_create_lmstudio() {
    let manager = ProviderManager::new();
    let spec = ProviderSpec::LmStudio {
        base_url: Some("http://localhost:1234".to_string()),
        model: "llama2".to_string(),
    };

    let result = manager.create_provider(&spec);
    assert!(result.is_ok());
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "lmstudio");
}

#[test]
fn test_provider_manager_create_local_inference() {
    let manager = ProviderManager::new();
    let spec = ProviderSpec::LocalInference {
        base_url: "http://localhost:8080".to_string(),
        model: "llama2".to_string(),
    };

    let result = manager.create_provider(&spec);
    assert!(result.is_ok());
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "local");
}

#[test]
fn test_provider_manager_create_ollama() {
    let manager = ProviderManager::new();
    let spec = ProviderSpec::Ollama {
        base_url: Some("http://localhost:11434".to_string()),
        model: "llama2".to_string(),
    };

    let result = manager.create_provider(&spec);
    assert!(result.is_ok());
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "ollama");
}

#[test]
fn test_lmstudio_factory_supports() {
    let factory = LmStudioProviderFactory;
    let spec = ProviderSpec::LmStudio {
        base_url: Some("http://localhost:1234".to_string()),
        model: "llama2".to_string(),
    };
    assert!(factory.supports(&spec));
}

#[test]
fn test_local_inference_factory_supports() {
    let factory = LocalInferenceProviderFactory;
    let spec = ProviderSpec::LocalInference {
        base_url: "http://localhost:8080".to_string(),
        model: "llama2".to_string(),
    };
    assert!(factory.supports(&spec));
}

#[test]
fn test_lmstudio_factory_does_not_support_ollama() {
    let factory = LmStudioProviderFactory;
    let spec = ProviderSpec::Ollama {
        base_url: Some("http://localhost:11434".to_string()),
        model: "llama2".to_string(),
    };
    assert!(!factory.supports(&spec));
}

#[test]
fn test_local_inference_factory_does_not_support_lmstudio() {
    let factory = LocalInferenceProviderFactory;
    let spec = ProviderSpec::LmStudio {
        base_url: Some("http://localhost:1234".to_string()),
        model: "llama2".to_string(),
    };
    assert!(!factory.supports(&spec));
}

#[tokio::test]
async fn test_ollama_complete_fails_without_server() {
    let provider = OllamaProvider::new(
        "llama2".to_string(),
        Some("http://localhost:19999".to_string()),
    );
    let result: Result<String, _> = provider.complete("hello", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_lmstudio_chat_fails_without_server() {
    let provider = LmStudioProvider::new(
        "llama2".to_string(),
        Some("http://localhost:19999".to_string()),
    );
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "hello".to_string(),
    }];
    let result: Result<_, _> = provider.chat(&messages).await;
    assert!(result.is_err());
}

#[test]
fn test_provider_spec_lmstudio_serialization() {
    let spec = ProviderSpec::LmStudio {
        base_url: Some("http://localhost:1234".to_string()),
        model: "llama2".to_string(),
    };

    let json = serde_json::to_string(&spec).unwrap();
    assert!(json.contains("\"type\":\"lmstudio\""));

    let deserialized: ProviderSpec = serde_json::from_str(&json).unwrap();
    match deserialized {
        ProviderSpec::LmStudio { base_url, model } => {
            assert_eq!(base_url, Some("http://localhost:1234".to_string()));
            assert_eq!(model, "llama2");
        }
        _ => panic!("Expected LmStudio variant"),
    }
}

#[test]
fn test_provider_spec_local_inference_serialization() {
    let spec = ProviderSpec::LocalInference {
        base_url: "http://localhost:8080".to_string(),
        model: "llama2".to_string(),
    };

    let json = serde_json::to_string(&spec).unwrap();
    assert!(json.contains("\"type\":\"local\""));

    let deserialized: ProviderSpec = serde_json::from_str(&json).unwrap();
    match deserialized {
        ProviderSpec::LocalInference { base_url, model } => {
            assert_eq!(base_url, "http://localhost:8080");
            assert_eq!(model, "llama2");
        }
        _ => panic!("Expected LocalInference variant"),
    }
}

#[test]
fn test_create_provider_fallback_for_unknown_type() {
    let manager = ProviderManager::new();
    let spec = ProviderSpec::OpenAI {
        api_key: "test-key".to_string(),
        model: "gpt-4o".to_string(),
        base_url: None,
    };

    let result = manager.create_provider(&spec);
    assert!(result.is_ok());
}

#[test]
fn test_create_provider_fallback_for_custom_provider() {
    let manager = ProviderManager::new();

    let spec = ProviderSpec::Mistral {
        api_key: "mistral-key".to_string(),
        model: "mistral-large".to_string(),
    };

    let result = manager.create_provider_fallback(&spec);
    assert!(result.is_ok());
    let provider = result.unwrap();
    assert_eq!(provider.provider_name(), "mistral");
}

#[test]
fn test_dynamic_factory_supports_always_returns_true() {
    use opencode_llm::provider_abstraction::DynamicProviderFactory;

    let factory = DynamicProviderFactory::new();
    let spec = ProviderSpec::OpenAI {
        api_key: "test".to_string(),
        model: "gpt-4".to_string(),
        base_url: None,
    };
    assert!(factory.supports(&spec));
}

#[test]
fn test_populate_from_catalog_adds_models() {
    use opencode_llm::catalog::{
        CatalogSource, CostInfo, LimitInfo, ModelCapabilities, ModelDescriptor, ModelStatus,
        ProviderCatalog, ProviderDescriptor,
    };
    use opencode_llm::models::ModelRegistry;
    use std::collections::BTreeMap;

    let mut providers = BTreeMap::new();
    providers.insert(
        "minimax".to_string(),
        ProviderDescriptor {
            id: "minimax".to_string(),
            display_name: "MiniMax".to_string(),
            api_base_url: Some("https://api.minimax.chat/v1".to_string()),
            docs_url: None,
            env_vars: vec![],
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
                },
            )]),
            source: CatalogSource::ModelsDev,
        },
    );

    let catalog = ProviderCatalog {
        providers,
        fetched_at: chrono::Utc::now(),
        source: CatalogSource::ModelsDev,
    };

    let mut registry = ModelRegistry::new();
    let initial_count = registry.list().len();

    registry.populate_from_catalog(&catalog);

    let models = registry.list();
    assert!(models.len() > initial_count);
    assert!(registry.get("MiniMax-M2.7").is_some());
    let minimax_model = registry.get("MiniMax-M2.7").unwrap();
    assert_eq!(minimax_model.provider, "minimax");
    assert!(minimax_model.supports_streaming);
}

#[test]
fn test_populate_from_catalog_preserves_existing_models() {
    use opencode_llm::catalog::{CatalogSource, ProviderCatalog, ProviderDescriptor};
    use opencode_llm::models::ModelRegistry;
    use std::collections::BTreeMap;

    let mut registry = ModelRegistry::new();
    let original_gpt4o = registry.get("gpt-4o");
    assert!(original_gpt4o.is_some());

    let mut providers = BTreeMap::new();
    providers.insert(
        "newprovider".to_string(),
        ProviderDescriptor {
            id: "newprovider".to_string(),
            display_name: "New Provider".to_string(),
            api_base_url: None,
            docs_url: None,
            env_vars: vec![],
            npm_package: None,
            models: BTreeMap::new(),
            source: CatalogSource::ModelsDev,
        },
    );

    let catalog = ProviderCatalog {
        providers,
        fetched_at: chrono::Utc::now(),
        source: CatalogSource::ModelsDev,
    };

    registry.populate_from_catalog(&catalog);

    assert!(registry.get("gpt-4o").is_some());
}
