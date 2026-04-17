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

    let copilot_gpt4o = registry.get("github-copilot/gpt-4o");
    assert!(
        copilot_gpt4o.is_some(),
        "github-copilot/gpt-4o should exist"
    );
    let model = copilot_gpt4o.unwrap();
    assert_eq!(model.provider, "github-copilot");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 128000);
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

    let expected_opencode_models = vec![
        "opencode/gpt-5-nano",
        "opencode/minimax-m2.5-free",
        "opencode/nemotron-3-super-free",
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

    let gpt5_nano = registry.get("opencode/gpt-5-nano");
    assert!(
        gpt5_nano.is_some(),
        "opencode/gpt-5-nano should exist"
    );
    let model = gpt5_nano.unwrap();
    assert_eq!(model.provider, "opencode");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 128000);

    let minimax_free = registry.get("opencode/minimax-m2.5-free");
    assert!(
        minimax_free.is_some(),
        "opencode/minimax-m2.5-free should exist"
    );
    let model = minimax_free.unwrap();
    assert_eq!(model.provider, "opencode");
    assert!(!model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 8192);
    assert_eq!(model.max_input_tokens, 1000000);

    let nemotron = registry.get("opencode/nemotron-3-super-free");
    assert!(
        nemotron.is_some(),
        "opencode/nemotron-3-super-free should exist"
    );
    let model = nemotron.unwrap();
    assert_eq!(model.provider, "opencode");
    assert!(!model.supports_functions);
    assert!(!model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 8192);
    assert_eq!(model.max_input_tokens, 131072);
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
        .filter(|m| m.provider == "kimi")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !kimi_models.is_empty(),
        "kimi models should be included in model list"
    );

    let expected_kimi_models = vec![
        "kimi/kimi-2.5",
        "kimi/kimi-2",
        "kimi/kimi-1.5",
        "kimi/kimi-latest",
        "kimi/moonshot-turbo",
        "kimi/moonshot-v1-128k",
    ];

    for model_name in expected_kimi_models {
        assert!(
            kimi_models.contains(&model_name),
            "Expected kimi model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_kimi_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"kimi".to_string()),
        "kimi should be in the providers list"
    );
}

#[test]
fn verify_kimi_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    let kimi_25 = registry.get("kimi/kimi-2.5");
    assert!(
        kimi_25.is_some(),
        "kimi/kimi-2.5 should exist"
    );
    let model = kimi_25.unwrap();
    assert_eq!(model.provider, "kimi");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 32768);
    assert_eq!(model.max_input_tokens, 128000);

    let moonshot_turbo = registry.get("kimi/moonshot-turbo");
    assert!(
        moonshot_turbo.is_some(),
        "kimi/moonshot-turbo should exist"
    );
    let model = moonshot_turbo.unwrap();
    assert_eq!(model.provider, "kimi");
    assert!(!model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 128000);

    let moonshot_v1_128k = registry.get("kimi/moonshot-v1-128k");
    assert!(
        moonshot_v1_128k.is_some(),
        "kimi/moonshot-v1-128k should exist"
    );
    let model = moonshot_v1_128k.unwrap();
    assert_eq!(model.provider, "kimi");
    assert!(!model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 131072);
}

#[test]
fn verify_model_count_increases_by_kimi_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    let kimi_model_count = registry
        .list_by_provider("kimi")
        .len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        kimi_model_count >= 6,
        "kimi should have at least 6 models, but has {}",
        kimi_model_count
    );

    assert!(
        model_count > kimi_model_count,
        "Total model count ({}) should be greater than kimi model count ({})",
        model_count,
        kimi_model_count
    );
}

#[test]
fn verify_z_ai_models_included_in_model_list() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let z_ai_models: Vec<_> = models
        .iter()
        .filter(|m| m.provider == "z.ai")
        .map(|m| m.name.as_str())
        .collect();

    assert!(
        !z_ai_models.is_empty(),
        "z.ai models should be included in model list"
    );

    let expected_z_ai_models = vec![
        "z.ai/z-1",
        "z.ai/z-1-mini",
        "z.ai/z-1-flash",
        "z.ai/z-1-preview",
        "z.ai/llama-3.1-70b",
        "z.ai/llama-3.1-8b",
        "z.ai/codellama-70b",
        "z.ai/mistral-7b",
        "z.ai/mixtral-8x7b",
    ];

    for model_name in expected_z_ai_models {
        assert!(
            z_ai_models.contains(&model_name),
            "Expected z.ai model '{}' should be in the model list",
            model_name
        );
    }
}

#[test]
fn verify_z_ai_provider_in_providers_list() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        providers.contains(&"z.ai".to_string()),
        "z.ai should be in the providers list"
    );
}

#[test]
fn verify_z_ai_models_have_correct_attributes() {
    let registry = ModelRegistry::new();

    let z1 = registry.get("z.ai/z-1");
    assert!(
        z1.is_some(),
        "z.ai/z-1 should exist"
    );
    let model = z1.unwrap();
    assert_eq!(model.provider, "z.ai");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 128000);

    let z1_mini = registry.get("z.ai/z-1-mini");
    assert!(
        z1_mini.is_some(),
        "z.ai/z-1-mini should exist"
    );
    let model = z1_mini.unwrap();
    assert_eq!(model.provider, "z.ai");
    assert!(model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 16384);
    assert_eq!(model.max_input_tokens, 128000);

    let z1_flash = registry.get("z.ai/z-1-flash");
    assert!(
        z1_flash.is_some(),
        "z.ai/z-1-flash should exist"
    );
    let model = z1_flash.unwrap();
    assert_eq!(model.provider, "z.ai");
    assert!(!model.supports_functions);
    assert!(model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 8192);
    assert_eq!(model.max_input_tokens, 128000);

    let llama_70b = registry.get("z.ai/llama-3.1-70b");
    assert!(
        llama_70b.is_some(),
        "z.ai/llama-3.1-70b should exist"
    );
    let model = llama_70b.unwrap();
    assert_eq!(model.provider, "z.ai");
    assert!(!model.supports_functions);
    assert!(!model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 8192);
    assert_eq!(model.max_input_tokens, 32768);

    let codellama_70b = registry.get("z.ai/codellama-70b");
    assert!(
        codellama_70b.is_some(),
        "z.ai/codellama-70b should exist"
    );
    let model = codellama_70b.unwrap();
    assert_eq!(model.provider, "z.ai");
    assert!(!model.supports_functions);
    assert!(!model.supports_vision);
    assert!(model.supports_streaming);
    assert_eq!(model.max_tokens, 4096);
    assert_eq!(model.max_input_tokens, 16384);
}

#[test]
fn verify_model_count_increases_by_z_ai_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    let z_ai_model_count = registry
        .list_by_provider("z.ai")
        .len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        z_ai_model_count >= 9,
        "z.ai should have at least 9 models, but has {}",
        z_ai_model_count
    );

    assert!(
        model_count > z_ai_model_count,
        "Total model count ({}) should be greater than z.ai model count ({})",
        model_count,
        z_ai_model_count
    );
}