use opencode_llm::models::ModelRegistry;

#[test]
fn verify_github_copilot_models_included_in_model_list() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let copilot_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == "github-copilot")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !copilot_models.is_empty(),
        "github-copilot models should be included in model list"
    );

    let expected_copilot_models = vec![
        "github-copilot/gpt-4o",
        "github-copilot/gpt-4o-mini",
        "github-copilot/claude-sonnet-4",
        "github-copilot/claude-haiku-3",
        "github-copilot/o1",
        "github-copilot/o1-mini",
        "github-copilot/o1-preview",
    ];

    for model_name in expected_copilot_models {
        assert!(
            copilot_models.contains(&model_name),
            "Expected github-copilot model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_model_count_increases_by_copilot_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    let copilot_model_count = registry
        .list_by_provider("github-copilot")
        .len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        copilot_model_count >= 7,
        "github-copilot should have at least 7 models, but has {}",
        copilot_model_count
    );

    assert!(
        model_count > copilot_model_count,
        "Total model count ({}) should be greater than copilot model count ({})",
        model_count,
        copilot_model_count
    );
}

#[test]
fn verify_github_copilot_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"github-copilot".to_string()),
        "github-copilot should be in the providers list"
    );
}

#[test]
fn verify_github_copilot_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    // Use actual model from snapshot
    let copilot_gpt5 = registry.get("gpt-5-mini");
    assert!(copilot_gpt5.is_some(), "gpt-5-mini should exist");
    let model = copilot_gpt5.unwrap();
    assert_eq!(model.provider, "github-copilot");
    assert!(model.supports_streaming);
}

#[test]
fn verify_opencode_models_included_in_model_list() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let opencode_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == "opencode")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !opencode_models.is_empty(),
        "opencode models should be included in model list"
    );

    // Use actual opencode model IDs from snapshot
    let expected_opencode_models = vec![
        "gpt-5.1-codex-max",
        "claude-haiku-4-5",
        "kimi-k2.5",
    ];

    for model_name in expected_opencode_models {
        assert!(
            opencode_models.contains(&model_name),
            "Expected opencode model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_opencode_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"opencode".to_string()),
        "opencode should be in the providers list"
    );
}

#[test]
fn verify_opencode_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    // Use actual model from snapshot
    let gpt5 = registry.get("gpt-5.1-codex-max");
    assert!(gpt5.is_some(), "gpt-5.1-codex-max should exist");
    let model = gpt5.unwrap();
    assert_eq!(model.provider, "opencode");
    assert!(model.supports_streaming);
}

#[test]
fn verify_google_antigravity_models_included_in_model_list() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let google_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == "google")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !google_models.is_empty(),
        "google models should be included in model list"
    );

    let expected_antigravity_models = vec![
        "google/antigravity-1",
        "google/antigravity-2",
        "google/antigravity-3",
        "google/antigravity-ultra",
    ];

    for model_name in expected_antigravity_models {
        assert!(
            google_models.contains(&model_name),
            "Expected google model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_google_antigravity_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"google".to_string()),
        "google should be in the providers list"
    );
}

#[test]
fn verify_google_antigravity_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    let antigravity1 = registry.get("google/antigravity-1");
    assert!(
        antigravity1.is_some(),
        "google/antigravity-1 should exist"
    );
    let model = antigravity1.unwrap();
    assert_eq!(model.provider, "google");
    assert!(!model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 8192);
    assert_eq!(model.max_input_tokens, 1000000);

    let antigravity2 = registry.get("google/antigravity-2");
    assert!(
        antigravity2.is_some(),
        "google/antigravity-2 should exist"
    );
    let model = antigravity2.unwrap();
    assert_eq!(model.provider, "google");
    assert!(!model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 2000000);

    let antigravity3 = registry.get("google/antigravity-3");
    assert!(
        antigravity3.is_some(),
        "google/antigravity-3 should exist"
    );
    let model = antigravity3.unwrap();
    assert_eq!(model.provider, "google");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 2000000);

    let antigravity_ultra = registry.get("google/antigravity-ultra");
    assert!(
        antigravity_ultra.is_some(),
        "google/antigravity-ultra should exist"
    );
    let model = antigravity_ultra.unwrap();
    assert_eq!(model.provider, "google");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 32768);
    assert_eq!(model.max_input_tokens, 2000000);
}

#[test]
fn verify_model_count_increases_by_google_antigravity_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    let google_antigravity_count = registry
        .list_by_provider("google")
        .len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        google_antigravity_count >= 8,
        "google should have at least 8 models (including gemini variants), but has {}",
        google_antigravity_count
    );

    let antigravity_count = registry
        .list_by_provider("google")
        .iter()
        .filter(|m| m.name.starts_with("google/antigravity"))
        .count();

    assert!(
        antigravity_count >= 4,
        "google should have at least 4 antigravity models, but has {}",
        antigravity_count
    );
}

#[test]
fn verify_kimi_models_included_in_model_list() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let kimi_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == "kimi-for-coding")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !kimi_models.is_empty(),
        "kimi-for-coding models should be included in model list"
    );

    // Use actual kimi-for-coding model IDs from snapshot
    let expected_kimi_models = vec![
        "k2p5",
        "kimi-k2-thinking",
    ];

    for model_name in expected_kimi_models {
        assert!(
            kimi_models.contains(&model_name),
            "Expected kimi-for-coding model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_kimi_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"kimi-for-coding".to_string()),
        "kimi-for-coding should be in the providers list"
    );
}

#[test]
fn verify_kimi_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    // Use actual model from snapshot
    let kimi = registry.get("k2p5");
    assert!(kimi.is_some(), "k2p5 should exist");
    let model = kimi.unwrap();
    assert_eq!(model.provider, "kimi-for-coding");
    assert!(model.supports_streaming);
}

#[test]
fn verify_model_count_increases_by_kimi_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    let kimi_model_count = registry
        .list_by_provider("kimi-for-coding")
        .len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        kimi_model_count >= 1,
        "kimi-for-coding should have at least 1 model, but has {}",
        kimi_model_count
    );

    assert!(
        model_count > kimi_model_count,
        "Total model count ({}) should be greater than kimi-for-coding model count ({})",
        model_count,
        kimi_model_count
    );
}

#[test]
fn verify_zai_models_included_in_model_list() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let zai_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == "zai")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !zai_models.is_empty(),
        "zai models should be included in model list"
    );

    let expected_zai_models = vec![
        "glm-5v-turbo",
        "glm-4.7",
        "glm-5",
    ];

    for model_name in expected_zai_models {
        assert!(
            zai_models.contains(&model_name),
            "Expected zai model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_zai_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"zai".to_string()),
        "zai should be in the providers list"
    );
}

#[test]
fn verify_zai_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    let glm5 = registry.get("glm-5");
    assert!(glm5.is_some(), "glm-5 should exist");
    let model = glm5.unwrap();
    assert_eq!(model.provider, "zai");
    assert!(model.supports_streaming);
}

#[test]
fn verify_model_count_increases_by_zai_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    let zai_model_count = registry
        .list_by_provider("zai")
        .len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        zai_model_count >= 9,
        "zai should have at least 9 models, but has {}",
        zai_model_count
    );

    assert!(
        model_count > zai_model_count,
        "Total model count ({}) should be greater than zai model count ({})",
        model_count,
        zai_model_count
    );
}
#[test]
fn debug_zai_direct() {
    use opencode_llm::catalog::snapshot::get_snapshot;
    
    let snapshot = get_snapshot().unwrap();
    
    // Check zai provider
    if let Some(zai) = snapshot.providers.get("zai") {
        println!("Found zai in snapshot providers");
        println!("  id: {}", zai.id);
        println!("  models: {}", zai.models.len());
        
        // Get first model's provider_id
        for (m_id, m) in zai.models.iter().take(3) {
            println!("  Model {}: provider_id={}", m_id, m.provider_id);
        }
    } else {
        println!("zai NOT in snapshot providers!");
    }
    
    // Also check what the registry sees
    use opencode_llm::models::ModelRegistry;
    let registry = ModelRegistry::new();
    
    // Get all unique providers from models
    let all: std::collections::HashSet<String> = registry.list().iter().map(|m| m.provider.clone()).collect();
    let zai_like: Vec<_> = all.iter().filter(|p| p.contains("zai")).collect();
    println!("\nProviders containing 'zai' in registry: {:?}", zai_like);
}

#[test]
fn debug_zai_models_actual() {
    use opencode_llm::models::ModelRegistry;
    
    let registry = ModelRegistry::new();
    
    // Get all models from zai-like providers  
    let all = registry.list();
    
    // Find models with name starting with "glm-" (zai models are glm-*)
    let glm_models: Vec<_> = all.iter()
        .filter(|m| m.name.starts_with("glm-"))
        .collect();
    
    println!("Total glm-* models: {}", glm_models.len());
    
    // Get unique providers for glm models
    let providers: std::collections::HashSet<String> = glm_models.iter().map(|m| m.provider.clone()).collect();
    println!("Unique providers for glm-* models: {:?}", providers);
    
    // Show first 5 glm models with their providers
    for m in glm_models.iter().take(10) {
        println!("  {} -> provider: {}", m.name, m.provider);
    }
}

#[test]
fn debug_zai_raw_snapshot() {
    use opencode_llm::catalog::snapshot::get_snapshot;
    
    let snapshot = get_snapshot().unwrap();
    
    // Get zai provider
    let zai_provider = snapshot.providers.get("zai").unwrap();
    println!("zai provider id: {}", zai_provider.id);
    
    // Get first model from zai
    let first_model = zai_provider.models.values().next().unwrap();
    println!("First model id: {}", first_model.id);
    println!("First model provider_id: {}", first_model.provider_id);
    
    // Check the model_key that would be used in ModelRegistry::new()
    println!("\nModel key would be: {}", first_model.id);
    println!("Model provider would be: {}", first_model.provider_id);
}
