use opencode_llm::models::ModelRegistry;

// The catalog currently has 2311 models; keep minimums well below actual count
// so this test passes even if a handful of models are removed, but catches large regressions.
const MINIMUM_MODEL_COUNT: usize = 1000;
const BASELINE_MODEL_COUNT: usize = 2000;

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

    // These are the core providers that must always be present.
    // Provider IDs are now canonical (no "models-dev-" prefix).
    let expected_providers = vec![
        "anthropic",
        "google",
        "groq",
        "mistral",
        "openai",
        "openrouter",
        "xai",
        "cohere",
        "deepinfra",
        "togetherai",
        "cerebras",
        "github-copilot",
        "perplexity",
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
fn verify_key_provider_model_counts_are_reasonable() {
    let registry = ModelRegistry::new();

    // Verify a selection of well-known providers have a reasonable lower bound.
    // These are deliberately conservative so minor catalog changes don't break CI.
    let provider_min_counts = vec![
        ("openai", 10),
        ("anthropic", 3),
        ("google", 5),
        ("github-copilot", 5),
        ("mistral", 10),
        ("groq", 5),
        ("openrouter", 20),
        ("xai", 5),
    ];

    for (provider, min_count) in provider_min_counts {
        let actual_count = registry.list_by_provider(provider).len();
        assert!(
            actual_count >= min_count,
            "Provider '{}' should have at least {} models, but has {}",
            provider,
            min_count,
            actual_count
        );
    }
}
