use opencode_llm::models::ModelRegistry;

const KNOWN_CONTEXT_LENGTHS: &[(&str, u32)] = &[
    ("gpt-4o", 128000),
    ("gpt-4o-mini", 128000),
    ("gpt-4-turbo", 128000),
    ("claude-sonnet-4-20250514", 200000),
    ("claude-haiku-3", 200000),
    ("claude-opus-4-20250514", 200000),
    ("llama3", 8192),
    ("codellama", 16384),
    ("gpt-4o-azure", 128000),
    ("gemini-1.5-pro", 2000000),
    ("gemini-1.5-flash", 1000000),
    ("openrouter/gpt-4o", 128000),
    ("grok-2", 131072),
    ("mistral-large-latest", 128000),
    ("llama-3.1-70b-versatile", 32768),
    ("deepinfra/llama-3.1-70b", 32768),
    ("cerebras/llama-3.1-70b", 32768),
    ("cohere-command-r-plus", 128000),
    ("togetherai/llama-3.1-70b", 32768),
    ("perplexity/llama-3.1-sonar-large", 127072),
    ("github-copilot/gpt-4o", 128000),
    ("github-copilot/gpt-4o-mini", 128000),
    ("github-copilot/claude-sonnet-4", 200000),
    ("github-copilot/claude-haiku-3", 200000),
    ("github-copilot/o1", 128000),
    ("github-copilot/o1-mini", 131072),
    ("github-copilot/o1-preview", 131072),
    ("opencode/gpt-5-nano", 128000),
    ("opencode/minimax-m2.5-free", 1000000),
    ("opencode/nemotron-3-super-free", 131072),
    ("google/antigravity-1", 1000000),
    ("google/antigravity-2", 2000000),
    ("google/antigravity-3", 2000000),
    ("google/antigravity-ultra", 2000000),
    ("kimi/kimi-2.5", 128000),
    ("kimi/kimi-2", 128000),
    ("kimi/kimi-1.5", 128000),
    ("kimi/kimi-latest", 128000),
    ("kimi/moonshot-turbo", 128000),
    ("kimi/moonshot-v1-128k", 131072),
    ("z.ai/z-1", 128000),
    ("z.ai/z-1-mini", 128000),
    ("z.ai/z-1-flash", 128000),
    ("z.ai/z-1-preview", 128000),
    ("z.ai/llama-3.1-70b", 32768),
    ("z.ai/llama-3.1-8b", 32768),
    ("z.ai/codellama-70b", 16384),
    ("z.ai/mistral-7b", 32768),
    ("z.ai/mixtral-8x7b", 32768),
    ("openai/o1", 128000),
    ("openai/o1-mini", 131072),
    ("openai/o1-preview", 131072),
    ("openai/gpt-4o-2024-08-13", 128000),
    ("openai/gpt-4o-mini-2024-07-18", 128000),
    ("anthropic/claude-sonnet-4-20250514", 200000),
    ("anthropic/claude-opus-4-20250514", 200000),
    ("anthropic/claude-3-5-sonnet-latest", 200000),
    ("anthropic/claude-3-5-haiku-latest", 200000),
    ("google/gemini-2.0-flash", 1000000),
    ("google/gemini-2.0-flash-exp", 1000000),
    ("google/gemini-1.5-pro-latest", 2000000),
    ("google/gemini-1.5-flash-latest", 1000000),
    ("google/gemini-exp-1206", 2000000),
];

#[test]
fn verify_all_models_have_context_length_greater_than_zero() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    assert!(!models.is_empty(), "Model registry should not be empty");

    let mut failures = Vec::new();
    for model in &models {
        if model.max_input_tokens == 0 {
            failures.push(format!(
                "Model '{}' (provider: '{}') has max_input_tokens = 0",
                model.name, model.provider
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "All models must have context_length > 0. Failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_all_models_have_max_tokens_greater_than_zero() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    assert!(!models.is_empty(), "Model registry should not be empty");

    let mut failures = Vec::new();
    for model in &models {
        if model.max_tokens == 0 {
            failures.push(format!(
                "Model '{}' (provider: '{}') has max_tokens = 0",
                model.name, model.provider
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "All models must have max_tokens > 0. Failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_context_lengths_match_known_provider_specifications() {
    let registry = ModelRegistry::new();

    let mut failures = Vec::new();
    for (model_name, expected_context) in KNOWN_CONTEXT_LENGTHS {
        match registry.get(model_name) {
            Some(model) => {
                if model.max_input_tokens != *expected_context {
                    failures.push(format!(
                        "Model '{}': expected context_length = {}, got {}",
                        model_name, expected_context, model.max_input_tokens
                    ));
                }
            }
            None => {
                failures.push(format!("Model '{}' not found in registry", model_name));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Context lengths must match known specifications. Failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_no_model_has_default_unset_context_length() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    const DEFAULT_CONTEXT_VALUE: u32 = 4096;

    let mut failures = Vec::new();
    for model in &models {
        if model.max_input_tokens == DEFAULT_CONTEXT_VALUE {
            failures.push(format!(
                "Model '{}' (provider: '{}') appears to have default/unset context_length = {}",
                model.name, model.provider, DEFAULT_CONTEXT_VALUE
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "No model should have default context_length. Models with suspicious values:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_context_length_is_reasonable() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let mut failures = Vec::new();
    for model in &models {
        if model.max_input_tokens > 2_000_000 {
            failures.push(format!(
                "Model '{}' (provider: '{}') has suspiciously large context_length = {}",
                model.name, model.provider, model.max_input_tokens
            ));
        }
        if model.max_input_tokens < 256 {
            failures.push(format!(
                "Model '{}' (provider: '{}') has suspiciously small context_length = {}",
                model.name, model.provider, model.max_input_tokens
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Context lengths should be reasonable (256 to 2M). Failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_max_tokens_is_reasonable() {
    let registry = ModelRegistry::new();
    let models = registry.list();

    let mut failures = Vec::new();
    for model in &models {
        if model.max_tokens > 256_000 {
            failures.push(format!(
                "Model '{}' (provider: '{}') has suspiciously large max_tokens = {}",
                model.name, model.provider, model.max_tokens
            ));
        }
        if model.max_tokens < 100 {
            failures.push(format!(
                "Model '{}' (provider: '{}') has suspiciously small max_tokens = {}",
                model.name, model.provider, model.max_tokens
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Max tokens should be reasonable (100 to 256K). Failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_each_provider_has_models_with_context_lengths() {
    let registry = ModelRegistry::new();
    let providers = registry.list_providers();

    assert!(
        !providers.is_empty(),
        "There should be at least one provider"
    );

    let mut failures = Vec::new();
    for provider in providers {
        let models = registry.list_by_provider(&provider);
        if models.is_empty() {
            failures.push(format!("Provider '{}' has no models", provider));
            continue;
        }

        for model in models {
            if model.max_input_tokens == 0 {
                failures.push(format!(
                    "Provider '{}' model '{}' has max_input_tokens = 0",
                    provider, model.name
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Each provider should have valid models. Failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn verify_model_context_lengths_are_consistent_with_provider_type() {
    let registry = ModelRegistry::new();

    let anthropic_models = registry.list_by_provider("anthropic");
    for model in anthropic_models {
        assert!(
            model.max_input_tokens >= 100_000,
            "Anthropic models should have context >= 100K, '{}' has {}",
            model.name,
            model.max_input_tokens
        );
    }

    let openai_models = registry.list_by_provider("openai");
    for model in openai_models {
        assert!(
            model.max_input_tokens >= 100_000,
            "OpenAI models should have context >= 100K, '{}' has {}",
            model.name,
            model.max_input_tokens
        );
    }

    let google_models = registry.list_by_provider("google");
    for model in google_models {
        assert!(
            model.max_input_tokens >= 100_000,
            "Google models should have context >= 100K, '{}' has {}",
            model.name,
            model.max_input_tokens
        );
    }
}
