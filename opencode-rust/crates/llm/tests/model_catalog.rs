use opencode_llm::models::ModelRegistry;

// Provider IDs are canonical — no "models-dev-" prefix after the catalog fix.

#[test]
fn verify_model_catalog_contains_50_plus_models() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();
    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );
}

#[test]
fn verify_github_copilot_provider_exists() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"github-copilot".to_string()),
        "github-copilot should be in the providers list"
    );
}

#[test]
fn verify_github_copilot_has_claude_models() {
    let registry = ModelRegistry::new();
    let copilot_models = registry.list_by_provider("github-copilot");

    assert!(
        !copilot_models.is_empty(),
        "github-copilot models should be included in model list"
    );

    assert!(
        copilot_models.len() >= 7,
        "github-copilot should have at least 7 models, but has {}",
        copilot_models.len()
    );

    let model_names: Vec<_> = copilot_models.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"claude-opus-41"),
        "github-copilot should have claude-opus-41 model"
    );
    assert!(
        model_names.contains(&"claude-opus-4.6"),
        "github-copilot should have claude-opus-4.6 model"
    );
}

#[test]
fn verify_github_copilot_model_attributes() {
    let registry = ModelRegistry::new();
    let model = registry.get("claude-opus-41");
    assert!(model.is_some(), "claude-opus-41 should exist");
    let model = model.unwrap();
    assert_eq!(model.provider, "github-copilot");
    assert!(model.supports_streaming);
}

#[test]
fn verify_chinese_providers_exist() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    let chinese_providers = vec![
        "kimi-for-coding",
        "minimax-coding-plan",
        "302ai",
        "zhipuai-coding-plan",
    ];

    for provider in chinese_providers {
        assert!(
            providers.contains(&provider.to_string()),
            "{} should be in the providers list",
            provider
        );
    }
}

#[test]
fn verify_kimi_for_coding_provider() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"kimi-for-coding".to_string()),
        "kimi-for-coding should be in the providers list"
    );
}

#[test]
fn verify_kimi_for_coding_has_k2p5_model() {
    let registry = ModelRegistry::new();
    let kimi_models = registry.list_by_provider("kimi-for-coding");

    assert!(
        !kimi_models.is_empty(),
        "kimi-for-coding models should be included in model list"
    );

    let model_names: Vec<_> = kimi_models.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"k2p5"),
        "kimi-for-coding should have k2p5 model"
    );
}

#[test]
fn verify_kimi_model_attributes() {
    let registry = ModelRegistry::new();
    let model = registry.get("k2p5");
    assert!(model.is_some(), "k2p5 should exist");
    let model = model.unwrap();
    assert_eq!(model.provider, "kimi-for-coding");
    assert!(model.supports_streaming);
}

#[test]
fn verify_minimax_coding_plan_provider() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"minimax-coding-plan".to_string()),
        "minimax-coding-plan should be in the providers list"
    );
}

#[test]
fn verify_minimax_coding_plan_has_models() {
    let registry = ModelRegistry::new();
    let minimax_models = registry.list_by_provider("minimax-coding-plan");

    assert!(
        !minimax_models.is_empty(),
        "minimax-coding-plan models should be included in model list"
    );

    let model_names: Vec<_> = minimax_models.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"MiniMax-M2.7"),
        "minimax-coding-plan should have MiniMax-M2.7 model"
    );
    assert!(
        model_names.contains(&"MiniMax-M2.7-highspeed"),
        "minimax-coding-plan should have MiniMax-M2.7-highspeed model"
    );
}

#[test]
fn verify_302ai_provider() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"302ai".to_string()),
        "302ai should be in the providers list"
    );
}

#[test]
fn verify_302ai_has_models() {
    let registry = ModelRegistry::new();
    let models_302ai = registry.list_by_provider("302ai");

    assert!(
        !models_302ai.is_empty(),
        "302ai models should be included in model list"
    );

    assert!(
        models_302ai.len() >= 10,
        "302ai should have at least 10 models, but has {}",
        models_302ai.len()
    );

    let model_names: Vec<_> = models_302ai.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"deepseek-v3.2-thinking"),
        "302ai should have deepseek-v3.2-thinking model"
    );
}

#[test]
fn verify_zhipuai_coding_plan_provider() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"zhipuai-coding-plan".to_string()),
        "zhipuai-coding-plan should be in the providers list"
    );
}

#[test]
fn verify_zhipuai_coding_plan_has_glm_models() {
    let registry = ModelRegistry::new();
    let zhipuai_models = registry.list_by_provider("zhipuai-coding-plan");

    assert!(
        !zhipuai_models.is_empty(),
        "zhipuai-coding-plan models should be included in model list"
    );

    let model_names: Vec<_> = zhipuai_models.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"glm-5v-turbo"),
        "zhipuai-coding-plan should have glm-5v-turbo model"
    );
    assert!(
        model_names.contains(&"glm-5"),
        "zhipuai-coding-plan should have glm-5 model"
    );
}

#[test]
fn verify_deepseek_models_exist_across_providers() {
    let registry = ModelRegistry::new();

    let all_models = registry.list();
    let deepseek_models: Vec<_> = all_models
        .iter()
        .filter(|m| m.name.contains("deepseek"))
        .collect();

    assert!(
        !deepseek_models.is_empty(),
        "deepseek models should exist across various providers"
    );

    assert!(
        deepseek_models.len() >= 10,
        "should have at least 10 deepseek models, but has {}",
        deepseek_models.len()
    );
}

#[test]
fn verify_openai_provider_exists() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"openai".to_string()),
        "openai should be in the providers list"
    );
}

#[test]
fn verify_openai_has_common_models() {
    let registry = ModelRegistry::new();
    let openai_models = registry.list_by_provider("openai");

    assert!(
        !openai_models.is_empty(),
        "openai models should be included in model list"
    );

    let model_names: Vec<_> = openai_models.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"gpt-4o"),
        "openai should have gpt-4o model"
    );
    assert!(
        model_names.contains(&"gpt-4o-mini"),
        "openai should have gpt-4o-mini model"
    );
}

#[test]
fn verify_anthropic_provider_exists() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();
    assert!(
        providers.contains(&"anthropic".to_string()),
        "anthropic should be in the providers list"
    );
}

#[test]
fn verify_anthropic_has_claude_models() {
    let registry = ModelRegistry::new();
    let anthropic_models = registry.list_by_provider("anthropic");

    assert!(
        !anthropic_models.is_empty(),
        "anthropic models should be included in model list"
    );

    let model_names: Vec<_> = anthropic_models.iter().map(|m| m.name.as_str()).collect();
    assert!(
        model_names.contains(&"claude-3-5-haiku-latest"),
        "anthropic should have claude-3-5-haiku-latest model"
    );
    assert!(
        model_names.contains(&"claude-opus-4-0"),
        "anthropic should have claude-opus-4-0 model"
    );
}

#[test]
fn verify_total_model_count_is_sufficient() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();
    let provider_count = registry.list_providers().len();

    assert!(
        model_count >= 50,
        "Model catalog should contain at least 50 models, but only contains {}",
        model_count
    );

    assert!(
        provider_count >= 50,
        "Model catalog should contain at least 50 providers, but only contains {}",
        provider_count
    );
}
