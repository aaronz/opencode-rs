use opencode_llm::models::ModelRegistry;

const MINIMUM_MODEL_COUNT: usize = 50;
const BASELINE_MODEL_COUNT: usize = 63;

#[test]
fn verify_total_model_count_meets_minimum_requirement() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    assert!(
        model_count >= MINIMUM_MODEL_COUNT,
        "Model catalog must contain at least {} models, but only contains {}",
        MINIMUM_MODEL_COUNT,
        model_count
    );
}

#[test]
fn verify_model_count_does_not_decrease() {
    let registry = ModelRegistry::new();
    let model_count = registry.list().len();

    assert!(
        model_count >= BASELINE_MODEL_COUNT,
        "Model count ({}) must not decrease below baseline ({}). \
         If models were removed, this is a regression.",
        model_count,
        BASELINE_MODEL_COUNT
    );
}

#[test]
fn verify_all_expected_providers_have_models() {
    let registry = ModelRegistry::new();

    let expected_providers = vec![
        "anthropic",
        "azure",
        "cerebras",
        "cohere",
        "deepinfra",
        "github-copilot",
        "google",
        "groq",
        "kimi",
        "mistral",
        "ollama",
        "openai",
        "opencode",
        "openrouter",
        "perplexity",
        "togetherai",
        "xai",
        "z.ai",
    ];

    for provider in expected_providers {
        let models = registry.list_by_provider(provider);
        assert!(
            !models.is_empty(),
            "Provider '{}' should have at least one model registered",
            provider
        );
    }
}

#[test]
fn verify_model_count_by_provider_distributions() {
    let registry = ModelRegistry::new();

    let provider_model_counts = vec![
        ("openai", 8),
        ("anthropic", 7),
        ("google", 11),
        ("github-copilot", 7),
        ("ollama", 2),
        ("azure", 1),
        ("openrouter", 1),
        ("xai", 1),
        ("mistral", 1),
        ("groq", 1),
        ("deepinfra", 1),
        ("cerebras", 1),
        ("cohere", 1),
        ("togetherai", 1),
        ("perplexity", 1),
        ("opencode", 3),
        ("kimi", 6),
        ("z.ai", 9),
    ];

    let total_expected: usize = provider_model_counts.iter().map(|(_, c)| c).sum();
    let actual_total = registry.list().len();

    assert_eq!(
        actual_total, total_expected,
        "Total model count ({}) should match sum of provider models ({})",
        actual_total, total_expected
    );

    for (provider, expected_count) in provider_model_counts {
        let actual_count = registry.list_by_provider(provider).len();
        assert_eq!(
            actual_count, expected_count,
            "Provider '{}' should have {} models, but has {}",
            provider, expected_count, actual_count
        );
    }
}
