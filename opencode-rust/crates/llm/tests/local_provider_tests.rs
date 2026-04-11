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
