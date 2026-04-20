//! Provider Configuration Example
//!
//! This example demonstrates how to configure different LLM providers
//! with OpenCode, including OpenAI, Anthropic, and Ollama.
//!
//! It also shows the environment variable configuration pattern.

use std::collections::HashMap;

use opencode_llm::anthropic::AnthropicProvider;
use opencode_llm::ollama::OllamaProvider;
use opencode_llm::openai::OpenAiProvider;
use opencode_llm::provider::{Provider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OpenCode Provider Configuration Example");
    println!("=======================================\n");

    example_openai_config()?;
    example_anthropic_config()?;
    example_ollama_config()?;
    example_env_var_configuration()?;

    println!("\nAll provider configuration examples completed!");
    Ok(())
}

fn example_openai_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. OpenAI Provider Configuration");
    println!("----------------------------------");

    let api_key =
        std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-your-openai-api-key".to_string());

    let config = ProviderConfig {
        model: "gpt-4o".to_string(),
        api_key: api_key.clone(),
        temperature: 0.7,
        headers: HashMap::new(),
    };

    let provider = OpenAiProvider::new(api_key, config.model.clone());
    println!("  Model: {}", config.model);
    println!("  Temperature: {}", config.temperature);
    println!(
        "  API Key: {}",
        if config.api_key.is_empty() {
            "not set"
        } else {
            "***"
        }
    );
    println!("  Provider name: {}", provider.provider_name());
    println!(
        "  Available models: {:?}",
        provider
            .get_models()
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    );

    println!("  Environment variables:");
    println!("    OPENAI_API_KEY=<your-api-key>");
    println!("    OPENAI_BASE_URL=https://api.openai.com/v1 (default)\n");

    Ok(())
}

fn example_anthropic_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Anthropic Provider Configuration");
    println!("-----------------------------------");

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .unwrap_or_else(|_| "sk-ant-your-anthropic-api-key".to_string());

    let config = ProviderConfig {
        model: "claude-sonnet-4-20250514".to_string(),
        api_key: api_key.clone(),
        temperature: 0.7,
        headers: HashMap::new(),
    };

    let provider = AnthropicProvider::new(api_key, config.model.clone());
    println!("  Model: {}", config.model);
    println!("  Temperature: {}", config.temperature);
    println!("  Provider name: {}", provider.provider_name());
    println!(
        "  Available models: {:?}",
        provider
            .get_models()
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    );

    println!("  Environment variables:");
    println!("    ANTHROPIC_API_KEY=<your-api-key>");
    println!("    ANTHROPIC_API_URL=https://api.anthropic.com (default)\n");

    Ok(())
}

fn example_ollama_config() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Ollama Provider Configuration (Local)");
    println!("---------------------------------------");

    let base_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let config = ProviderConfig {
        model: "llama2".to_string(),
        api_key: String::new(),
        temperature: 0.8,
        headers: HashMap::new(),
    };

    let provider = OllamaProvider::new(config.model.clone(), Some(base_url.clone()));
    println!("  Model: {}", config.model);
    println!("  Temperature: {}", config.temperature);
    println!("  Base URL: {}", base_url);
    println!("  Provider name: {}", provider.provider_name());
    println!(
        "  Available models: {:?}",
        provider
            .get_models()
            .iter()
            .map(|m| m.id.clone())
            .collect::<Vec<_>>()
    );

    println!("  Environment variables:");
    println!("    OLLAMA_BASE_URL=http://localhost:11434 (default)");
    println!("    OLLAMA_HOST=0.0.0.0:11434 (optional)\n");

    Ok(())
}

fn example_env_var_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Environment Variable Configuration Pattern");
    println!("----------------------------------------------");

    println!("  Standard environment variables for provider configuration:");
    println!();
    println!("  | Variable              | Description                          | Default                |");
    println!("  |----------------------|--------------------------------------|------------------------|");
    println!("  | OPENAI_API_KEY       | OpenAI API key                       | -                      |");
    println!("  | ANTHROPIC_API_KEY    | Anthropic API key                    | -                      |");
    println!("  | OLLAMA_BASE_URL      | Ollama server URL                    | http://localhost:11434 |");
    println!("  | OPENCODE_LLM_PROVIDER| Default LLM provider name            | openai                 |");
    println!();

    println!("  Code pattern for reading configuration from environment:");
    println!("  ```");
    println!("  let api_key = std::env::var(\"OPENAI_API_KEY\")");
    println!("      .unwrap_or_else(|_| \"default-key\".to_string());");
    println!("  ```");
    println!();

    let active_provider =
        std::env::var("OPENCODE_LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    println!("  Active provider: {}", active_provider);

    let openai_key = std::env::var("OPENAI_API_KEY").ok();
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
    let ollama_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    println!("  Configured providers:");
    if openai_key.is_some() {
        println!("    - OpenAI: API key present");
    } else {
        println!("    - OpenAI: API key not set");
    }
    if anthropic_key.is_some() {
        println!("    - Anthropic: API key present");
    } else {
        println!("    - Anthropic: API key not set");
    }
    println!("    - Ollama: {} (always local)", ollama_url);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_config_structure() {
        let config = ProviderConfig {
            model: "gpt-4o".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            headers: HashMap::new(),
        };
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_anthropic_config_structure() {
        let config = ProviderConfig {
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: "test-key".to_string(),
            temperature: 0.7,
            headers: HashMap::new(),
        };
        assert_eq!(config.model, "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_ollama_config_structure() {
        let config = ProviderConfig {
            model: "llama2".to_string(),
            api_key: String::new(),
            temperature: 0.8,
            headers: HashMap::new(),
        };
        assert_eq!(config.model, "llama2");
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_provider_config_sanitize() {
        let config = ProviderConfig {
            model: "gpt-4o".to_string(),
            api_key: "super-secret-key".to_string(),
            temperature: 0.7,
            headers: HashMap::new(),
        };
        let sanitized = config.sanitize_for_logging();
        assert_eq!(sanitized.api_key, "***REDACTED***");
        assert_eq!(sanitized.model, "gpt-4o");
    }

    #[test]
    fn test_env_var_defaults() {
        assert_eq!(
            std::env::var("NONEXISTENT_VAR")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            "http://localhost:11434"
        );
    }
}
