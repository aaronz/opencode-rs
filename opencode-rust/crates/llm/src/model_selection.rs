use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProviderType {
    #[default]
    OpenAI,
    Anthropic,
    Google,
    Ollama,
    LmStudio,
    LocalInference,
    Azure,
    OpenRouter,
    Mistral,
    Groq,
    DeepInfra,
    Cerebras,
    Cohere,
    TogetherAI,
    Perplexity,
    XAI,
    HuggingFace,
    Bedrock,
    Vercel,
    Copilot,
    Ai21,
    SapAiCore,
    Vertex,
    Custom,
}

impl ProviderType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => ProviderType::OpenAI,
            "anthropic" => ProviderType::Anthropic,
            "google" => ProviderType::Google,
            "ollama" => ProviderType::Ollama,
            "lmstudio" | "lm_studio" | "lm-studio" => ProviderType::LmStudio,
            "local" | "local-inference" | "localinference" => ProviderType::LocalInference,
            "azure" => ProviderType::Azure,
            "openrouter" => ProviderType::OpenRouter,
            "mistral" => ProviderType::Mistral,
            "groq" => ProviderType::Groq,
            "deepinfra" => ProviderType::DeepInfra,
            "cerebras" => ProviderType::Cerebras,
            "cohere" => ProviderType::Cohere,
            "togetherai" => ProviderType::TogetherAI,
            "perplexity" => ProviderType::Perplexity,
            "xai" => ProviderType::XAI,
            "huggingface" => ProviderType::HuggingFace,
            "bedrock" => ProviderType::Bedrock,
            "vercel" => ProviderType::Vercel,
            "copilot" => ProviderType::Copilot,
            "ai21" => ProviderType::Ai21,
            "sapaicore" | "sap_aicore" => ProviderType::SapAiCore,
            "vertex" => ProviderType::Vertex,
            _ => ProviderType::Custom,
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "gpt-4o",
            ProviderType::Anthropic => "claude-sonnet-4-20250514",
            ProviderType::Google => "gemini-1.5-pro",
            ProviderType::Ollama => "llama3",
            ProviderType::LmStudio => "llama3",
            ProviderType::LocalInference => "llama3",
            ProviderType::Azure => "gpt-4o",
            ProviderType::OpenRouter => "openrouter/gpt-4o",
            ProviderType::Mistral => "mistral-large-latest",
            ProviderType::Groq => "llama-3.1-70b-versatile",
            ProviderType::DeepInfra => "deepinfra/llama-3.1-70b",
            ProviderType::Cerebras => "cerebras/llama-3.1-70b",
            ProviderType::Cohere => "cohere-command-r-plus",
            ProviderType::TogetherAI => "togetherai/llama-3.1-70b",
            ProviderType::Perplexity => "perplexity/llama-3.1-sonar-large",
            ProviderType::XAI => "grok-2",
            ProviderType::HuggingFace => "meta-llama/llama-3.1-70b-instruct",
            ProviderType::Bedrock => "anthropic.claude-3-sonnet-20240229-v1:0",
            ProviderType::Vercel => "gpt-4o",
            ProviderType::Copilot => "gpt-4o",
            ProviderType::Ai21 => "jamba-1.5-large",
            ProviderType::SapAiCore => "gemini-1.5-pro",
            ProviderType::Vertex => "gemini-1.5-pro",
            ProviderType::Custom => "gpt-4o",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserModelConfig {
    pub provider_defaults: HashMap<String, String>,
    pub global_default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelSelection {
    provider_type: ProviderType,
    user_config: Option<UserModelConfig>,
}

impl ModelSelection {
    pub fn new(provider_type: impl Into<ProviderType>) -> Self {
        Self {
            provider_type: provider_type.into(),
            user_config: None,
        }
    }

    pub fn with_user_config(
        provider_type: impl Into<ProviderType>,
        user_config: UserModelConfig,
    ) -> Self {
        Self {
            provider_type: provider_type.into(),
            user_config: Some(user_config),
        }
    }

    pub fn resolve_model(&self, explicit_override: Option<&str>) -> String {
        if let Some(model) = explicit_override {
            return model.to_string();
        }

        if let Some(ref config) = self.user_config {
            let provider_str = self.provider_str();

            if let Some(user_default) = config.provider_defaults.get(&provider_str) {
                return user_default.clone();
            }

            if let Some(ref global_default) = config.global_default {
                if !global_default.is_empty() {
                    return global_default.clone();
                }
            }
        }

        self.provider_type.default_model().to_string()
    }

    fn provider_str(&self) -> String {
        match self.provider_type {
            ProviderType::OpenAI => "openai".to_string(),
            ProviderType::Anthropic => "anthropic".to_string(),
            ProviderType::Google => "google".to_string(),
            ProviderType::Ollama => "ollama".to_string(),
            ProviderType::LmStudio => "lmstudio".to_string(),
            ProviderType::LocalInference => "local".to_string(),
            ProviderType::Azure => "azure".to_string(),
            ProviderType::OpenRouter => "openrouter".to_string(),
            ProviderType::Mistral => "mistral".to_string(),
            ProviderType::Groq => "groq".to_string(),
            ProviderType::DeepInfra => "deepinfra".to_string(),
            ProviderType::Cerebras => "cerebras".to_string(),
            ProviderType::Cohere => "cohere".to_string(),
            ProviderType::TogetherAI => "togetherai".to_string(),
            ProviderType::Perplexity => "perplexity".to_string(),
            ProviderType::XAI => "xai".to_string(),
            ProviderType::HuggingFace => "huggingface".to_string(),
            ProviderType::Bedrock => "bedrock".to_string(),
            ProviderType::Vercel => "vercel".to_string(),
            ProviderType::Copilot => "copilot".to_string(),
            ProviderType::Ai21 => "ai21".to_string(),
            ProviderType::SapAiCore => "sap_aicore".to_string(),
            ProviderType::Vertex => "vertex".to_string(),
            ProviderType::Custom => "custom".to_string(),
        }
    }

    pub fn system_default_model() -> &'static str {
        "gpt-4o"
    }

    pub fn documentation() -> &'static str {
        r#"## Model Selection Precedence

When selecting a model for a provider, the following precedence is applied (highest to lowest):

1. **Explicit Override**: A model explicitly specified when creating the provider (highest priority)
2. **User Config Provider Default**: User configuration default for the specific provider
3. **User Config Global Default**: User configuration global default (fallback if no provider-specific config)
4. **Provider-Specific Default**: Built-in default model for the provider type
5. **System-Wide Fallback**: The hardcoded fallback is `gpt-4o` (lowest priority)

### Provider Default Models

| Provider   | Default Model                    |
|------------|----------------------------------|
| OpenAI     | gpt-4o                           |
| Anthropic  | claude-sonnet-4-20250514         |
| Google     | gemini-1.5-pro                   |
| Ollama     | llama3                           |
| Azure      | gpt-4o                           |
| OpenRouter | openrouter/gpt-4o                |
| Mistral    | mistral-large-latest            |
| Groq       | llama-3.1-70b-versatile          |
| DeepInfra  | deepinfra/llama-3.1-70b          |
| Cerebras   | cerebras/llama-3.1-70b           |
| Cohere     | cohere-command-r-plus            |
| TogetherAI | togetherai/llama-3.1-70b         |
| Perplexity | perplexity/llama-3.1-sonar-large |
| XAI        | grok-2                           |
| HuggingFace| meta-llama/llama-3.1-70b-instruct|
| Bedrock    | anthropic.claude-3-sonnet...     |
| Vercel     | gpt-4o                           |
| Copilot    | gpt-4o                           |
| Ai21       | jamba-1.5-large                  |
| SapAiCore  | gemini-1.5-pro                   |
| Vertex     | gemini-1.5-pro                   |

### User Config Example

```json
{
  "model_preferences": {
    "provider_defaults": {
      "openai": "gpt-4o-mini",
      "anthropic": "claude-haiku-3"
    },
    "global_default": "gpt-4o"
  }
}
```

With this config:
- OpenAI requests use `gpt-4o-mini` unless overridden
- Anthropic requests use `claude-haiku-3` unless overridden
- All other providers use `gpt-4o` as fallback
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(ProviderType::from_str("openai"), ProviderType::OpenAI);
        assert_eq!(ProviderType::from_str("OPENAI"), ProviderType::OpenAI);
        assert_eq!(ProviderType::from_str("Anthropic"), ProviderType::Anthropic);
        assert_eq!(ProviderType::from_str("unknown"), ProviderType::Custom);
    }

    #[test]
    fn test_provider_default_models() {
        assert_eq!(ProviderType::OpenAI.default_model(), "gpt-4o");
        assert_eq!(
            ProviderType::Anthropic.default_model(),
            "claude-sonnet-4-20250514"
        );
        assert_eq!(ProviderType::Google.default_model(), "gemini-1.5-pro");
        assert_eq!(ProviderType::Ollama.default_model(), "llama3");
        assert_eq!(
            ProviderType::Mistral.default_model(),
            "mistral-large-latest"
        );
    }

    #[test]
    fn test_model_selection_explicit_override_wins() {
        let selection = ModelSelection::new(ProviderType::OpenAI);

        let model = selection.resolve_model(Some("gpt-4o-mini"));

        assert_eq!(model, "gpt-4o-mini");
    }

    #[test]
    fn test_model_selection_user_config_provider_default() {
        let mut config = UserModelConfig::default();
        config
            .provider_defaults
            .insert("openai".to_string(), "gpt-4o-mini".to_string());

        let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

        let model = selection.resolve_model(None);

        assert_eq!(model, "gpt-4o-mini");
    }

    #[test]
    fn test_model_selection_user_config_global_default() {
        let config = UserModelConfig {
            global_default: Some("gpt-4o".to_string()),
            ..Default::default()
        };

        let selection = ModelSelection::with_user_config(ProviderType::Ollama, config);

        let model = selection.resolve_model(None);

        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_model_selection_provider_default_wins_over_global() {
        let config = UserModelConfig {
            global_default: Some("gpt-4o".to_string()),
            provider_defaults: [("anthropic".to_string(), "claude-haiku-3".to_string())]
                .into_iter()
                .collect(),
        };

        let selection = ModelSelection::with_user_config(ProviderType::Anthropic, config);

        let model = selection.resolve_model(None);

        assert_eq!(model, "claude-haiku-3");
    }

    #[test]
    fn test_model_selection_falls_back_to_provider_default() {
        let config = UserModelConfig::default();

        let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

        let model = selection.resolve_model(None);

        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_model_selection_system_default_fallback() {
        let selection = ModelSelection::new(ProviderType::Custom);

        let model = selection.resolve_model(None);

        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_model_selection_precedence_order_is_respected() {
        let config = UserModelConfig {
            global_default: Some("fallback-model".to_string()),
            provider_defaults: [("openai".to_string(), "provider-default".to_string())]
                .into_iter()
                .collect(),
        };

        let selection = ModelSelection::with_user_config(ProviderType::OpenAI, config);

        assert_eq!(selection.resolve_model(Some("explicit")), "explicit");

        let selection_no_override =
            ModelSelection::with_user_config(ProviderType::OpenAI, UserModelConfig::default());
        assert_eq!(selection_no_override.resolve_model(None), "gpt-4o");

        let selection_with_config = ModelSelection::with_user_config(
            ProviderType::Anthropic,
            UserModelConfig {
                global_default: Some("global-fallback".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(selection_with_config.resolve_model(None), "global-fallback");
    }

    #[test]
    fn test_model_selection_anthropic_respects_user_config() {
        let config = UserModelConfig {
            provider_defaults: [("anthropic".to_string(), "claude-opus-4-20250514".to_string())]
                .into_iter()
                .collect(),
            ..Default::default()
        };

        let selection = ModelSelection::with_user_config(ProviderType::Anthropic, config);

        let model = selection.resolve_model(None);

        assert_eq!(model, "claude-opus-4-20250514");
    }

    #[test]
    fn test_model_selection_google_respects_user_config() {
        let mut config = UserModelConfig::default();
        config
            .provider_defaults
            .insert("google".to_string(), "gemini-1.5-flash".to_string());

        let selection = ModelSelection::with_user_config(ProviderType::Google, config);

        let model = selection.resolve_model(None);

        assert_eq!(model, "gemini-1.5-flash");
    }

    #[test]
    fn test_user_config_overrides_system_defaults() {
        let config = UserModelConfig {
            provider_defaults: HashMap::from([
                ("openai".to_string(), "gpt-4o-mini".to_string()),
                ("anthropic".to_string(), "claude-haiku-3".to_string()),
            ]),
            global_default: Some("custom-global-default".to_string()),
        };

        let openai_selection =
            ModelSelection::with_user_config(ProviderType::OpenAI, config.clone());
        assert_eq!(openai_selection.resolve_model(None), "gpt-4o-mini");

        let anthropic_selection =
            ModelSelection::with_user_config(ProviderType::Anthropic, config.clone());
        assert_eq!(anthropic_selection.resolve_model(None), "claude-haiku-3");
    }

    #[test]
    fn test_provider_specific_defaults_are_respected() {
        let config = UserModelConfig::default();
        let selection = ModelSelection::with_user_config(ProviderType::Ollama, config);

        assert_eq!(selection.resolve_model(None), "llama3");

        let selection2 =
            ModelSelection::with_user_config(ProviderType::Mistral, UserModelConfig::default());
        assert_eq!(selection2.resolve_model(None), "mistral-large-latest");

        let selection3 =
            ModelSelection::with_user_config(ProviderType::Groq, UserModelConfig::default());
        assert_eq!(selection3.resolve_model(None), "llama-3.1-70b-versatile");
    }

    #[test]
    fn test_model_selection_documentation_is_available() {
        let docs = ModelSelection::documentation();
        assert!(docs.contains("Model Selection Precedence"));
        assert!(docs.contains("Explicit Override"));
        assert!(docs.contains("Provider-Specific Default"));
        assert!(docs.contains("System-Wide Fallback"));
        assert!(docs.contains("gpt-4o"));
    }
}
