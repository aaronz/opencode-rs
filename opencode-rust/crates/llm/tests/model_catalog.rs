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