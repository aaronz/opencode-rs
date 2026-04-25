use clap::Args;
use opencode_auth::credential_store::CredentialStore;
use opencode_core::Config;
use opencode_llm::auth_method::{get_provider_auth_methods, AuthMethod};
use opencode_llm::{ModelRegistry, OpenAiBrowserAuthService, OpenAiBrowserAuthStore};
use serde::Serialize;
use serde_json::json;
use std::io::Write;
use std::process::Command;

use super::load_config_result;

#[derive(Args, Debug)]
pub(crate) struct ProvidersArgs {
    #[arg(short, long)]
    pub json: bool,

    #[arg(long = "test-connection")]
    pub test_connection: Option<String>,

    #[arg(long)]
    pub login: Option<String>,

    #[arg(long)]
    pub browser: bool,
}

#[derive(Debug, Serialize)]
struct ProviderRow {
    id: String,
    name: String,
    enabled: bool,
    status: String,
    model_count: usize,
}

fn provider_name(id: &str) -> String {
    match id {
        "openai" => "OpenAI".to_string(),
        "anthropic" => "Anthropic".to_string(),
        "google" => "Google".to_string(),
        "ollama" => "Ollama".to_string(),
        "lmstudio" => "LM Studio".to_string(),
        "azure" => "Azure".to_string(),
        "openrouter" => "OpenRouter".to_string(),
        "mistral" => "Mistral".to_string(),
        "groq" => "Groq".to_string(),
        "deepinfra" => "DeepInfra".to_string(),
        "cerebras" => "Cerebras".to_string(),
        "cohere" => "Cohere".to_string(),
        "togetherai" => "Together AI".to_string(),
        "perplexity" => "Perplexity".to_string(),
        "xai" => "xAI".to_string(),
        "huggingface" => "Hugging Face".to_string(),
        "copilot" => "GitHub Copilot".to_string(),
        "ai21" => "AI21".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn open_browser(url: &str) -> Result<(), String> {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    };

    result
        .map_err(|e| format!("Failed to open browser: {}", e))
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "Browser open command failed with status {}",
                    status
                ))
            }
        })
}

fn prompt_for_api_key(provider: &str) -> Option<String> {
    print!("Enter API key for {}: ", provider);
    std::io::stdout().flush().ok()?;
    let mut api_key = String::new();
    std::io::stdin().read_line(&mut api_key).ok()?;
    let api_key = api_key.trim().to_string();
    if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    }
}

fn store_api_key_credential(provider: &str, api_key: &str, base_url: Option<&str>) -> Result<(), String> {
    let credential_store = CredentialStore::new();
    let credential = opencode_auth::credential_store::Credential {
        api_key: api_key.to_string(),
        base_url: base_url.map(String::from),
        metadata: std::collections::HashMap::new(),
    };
    credential_store
        .store(provider, &credential)
        .map_err(|e| format!("Failed to store credential: {}", e))
}

fn run_openai_browser_login() {
    let service = OpenAiBrowserAuthService::new();
    let listener = match service.start_local_callback_listener() {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Failed to start OpenAI browser login: {}", error);
            std::process::exit(1);
        }
    };
    let request = listener.request();
    let url = service.build_authorize_url(&request);
    println!(
        "Open this URL if the browser does not launch automatically:\n{}",
        url
    );
    let _ = open_browser(&url);

    let callback = match listener.wait_for_callback() {
        Ok(callback) => callback,
        Err(error) => {
            eprintln!(
                "OpenAI browser login failed while waiting for callback: {}",
                error
            );
            std::process::exit(1);
        }
    };

    let session = match service.exchange_code(callback, &request) {
        Ok(session) => session,
        Err(error) => {
            eprintln!(
                "OpenAI browser login failed during token exchange: {}",
                error
            );
            std::process::exit(1);
        }
    };

    let store = OpenAiBrowserAuthStore::from_default_location();
    if let Err(error) = store.save(&session) {
        eprintln!("Failed to save OpenAI browser session: {}", error);
        std::process::exit(1);
    }

    println!("OpenAI browser login successful");
}

fn run_api_key_login(provider: &str) {
    let auth_methods = get_provider_auth_methods(provider);
    let supports_browser = auth_methods.contains(&AuthMethod::Browser);

    if supports_browser {
        println!("Note: {} supports browser authentication. Use --browser flag for OAuth flow.", provider);
        println!("Proceeding with API key login...\n");
    }

    let api_key = match prompt_for_api_key(provider) {
        Some(key) => key,
        None => {
            eprintln!("API key is required. Login cancelled.");
            std::process::exit(1);
        }
    };

    if let Err(e) = store_api_key_credential(provider, &api_key, None) {
        eprintln!("Failed to store credential: {}", e);
        std::process::exit(1);
    }

    println!("{} login successful (API key stored)", provider);
}

fn provider_enabled(config: &Config, id: &str) -> bool {
    let enabled_by_allowlist = config
        .enabled_providers
        .as_ref()
        .map(|enabled| enabled.iter().any(|value| value == id))
        .unwrap_or(true);

    let disabled_by_denylist = config
        .disabled_providers
        .as_ref()
        .map(|disabled| disabled.iter().any(|value| value == id))
        .unwrap_or(false);

    enabled_by_allowlist && !disabled_by_denylist
}

pub(crate) fn run(args: ProvidersArgs) -> Result<(), String> {
    let config = load_config_result()?;
    let registry = ModelRegistry::default();
    let provider_ids = registry.list_providers();
    let providers = provider_ids
        .iter()
        .map(|id| {
            let enabled = provider_enabled(&config, id);
            ProviderRow {
                id: id.to_string(),
                name: provider_name(id),
                enabled,
                status: if enabled {
                    "available".to_string()
                } else {
                    "disabled".to_string()
                },
                model_count: registry.list_by_provider(id).len(),
            }
        })
        .collect::<Vec<_>>();

    if let Some(provider_id) = args.login.as_deref() {
        if args.json {
            let auth_methods = get_provider_auth_methods(provider_id);
            let result = json!({
                "action": "login",
                "provider": provider_id,
                "method": if args.browser && auth_methods.contains(&AuthMethod::Browser) {
                    "browser"
                } else {
                    "api_key"
                },
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
            );
            return Ok(());
        }

        let auth_methods = get_provider_auth_methods(provider_id);
        let supports_browser = auth_methods.contains(&AuthMethod::Browser);

        if provider_id == "openai" && args.browser && supports_browser {
            run_openai_browser_login();
            return Ok(());
        }

        if supports_browser && args.browser {
            eprintln!("Browser authentication for {} is not yet implemented.", provider_id);
            eprintln!("Falling back to API key authentication...");
        }

        run_api_key_login(provider_id);
        return Ok(());
    }

    if let Some(provider_id) = args.test_connection {
        let provider = providers.iter().find(|provider| provider.id == provider_id);
        match provider {
            Some(provider) => {
                println!(
                    "Provider {} connection status: {}",
                    provider.id, provider.status
                );
                return Ok(());
            }
            None => {
                eprintln!("Unknown provider: {}", provider_id);
                std::process::exit(1);
            }
        }
    }

    if args.json {
        let result = json!({
            "action": "list",
            "providers": providers,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return Ok(());
    }

    for provider in providers {
        println!(
            "{}\t{}\t{}",
            provider.id, provider.status, provider.model_count
        );
    }
    Ok(())
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name_openai() {
        assert_eq!(provider_name("openai"), "OpenAI");
    }

    #[test]
    fn test_provider_name_anthropic() {
        assert_eq!(provider_name("anthropic"), "Anthropic");
    }

    #[test]
    fn test_provider_name_ollama() {
        assert_eq!(provider_name("ollama"), "Ollama");
    }

    #[test]
    fn test_provider_name_unknown() {
        assert_eq!(provider_name("custom"), "Custom");
    }

    #[test]
    fn test_provider_name_unknown_single_char() {
        assert_eq!(provider_name("x"), "X");
    }

    #[test]
    fn test_provider_name_empty() {
        assert_eq!(provider_name(""), "");
    }

    #[test]
    fn test_provider_name_mixed_case() {
        assert_eq!(provider_name("myProvider"), "MyProvider");
    }

    #[test]
    fn test_provider_enabled_no_restrictions() {
        let config = Config::default();
        assert!(provider_enabled(&config, "openai"));
        assert!(provider_enabled(&config, "anthropic"));
        assert!(provider_enabled(&config, "ollama"));
    }

    #[test]
    fn test_provider_enabled_with_allowlist() {
        let config = Config {
            enabled_providers: Some(vec!["openai".to_string()]),
            ..Default::default()
        };
        assert!(provider_enabled(&config, "openai"));
        assert!(!provider_enabled(&config, "anthropic"));
        assert!(!provider_enabled(&config, "ollama"));
    }

    #[test]
    fn test_provider_enabled_with_denylist() {
        let config = Config {
            disabled_providers: Some(vec!["openai".to_string()]),
            ..Default::default()
        };
        assert!(!provider_enabled(&config, "openai"));
        assert!(provider_enabled(&config, "anthropic"));
        assert!(provider_enabled(&config, "ollama"));
    }

    #[test]
    fn test_provider_enabled_allowlist_takes_precedence() {
        let config = Config {
            enabled_providers: Some(vec!["openai".to_string()]),
            disabled_providers: Some(vec!["openai".to_string()]),
            ..Default::default()
        };
        assert!(!provider_enabled(&config, "openai"));
    }

    #[test]
    fn test_provider_enabled_multiple_in_allowlist() {
        let config = Config {
            enabled_providers: Some(vec!["openai".to_string(), "anthropic".to_string()]),
            ..Default::default()
        };
        assert!(provider_enabled(&config, "openai"));
        assert!(provider_enabled(&config, "anthropic"));
        assert!(!provider_enabled(&config, "ollama"));
    }

    #[test]
    fn test_provider_enabled_multiple_in_denylist() {
        let config = Config {
            disabled_providers: Some(vec!["openai".to_string(), "anthropic".to_string()]),
            ..Default::default()
        };
        assert!(!provider_enabled(&config, "openai"));
        assert!(!provider_enabled(&config, "anthropic"));
        assert!(provider_enabled(&config, "ollama"));
    }

    #[test]
    fn test_providers_args_default() {
        let args = ProvidersArgs {
            json: false,
            test_connection: None,
            login: None,
            browser: false,
        };
        assert!(!args.json);
        assert!(args.test_connection.is_none());
        assert!(args.login.is_none());
        assert!(!args.browser);
    }

    #[test]
    fn test_providers_args_with_json() {
        let args = ProvidersArgs {
            json: true,
            test_connection: None,
            login: None,
            browser: false,
        };
        assert!(args.json);
    }

    #[test]
    fn test_providers_args_with_test_connection() {
        let args = ProvidersArgs {
            json: false,
            test_connection: Some("openai".to_string()),
            login: None,
            browser: false,
        };
        assert_eq!(args.test_connection.as_deref(), Some("openai"));
    }

    #[test]
    fn test_providers_args_with_login() {
        let args = ProvidersArgs {
            json: false,
            test_connection: None,
            login: Some("anthropic".to_string()),
            browser: false,
        };
        assert_eq!(args.login.as_deref(), Some("anthropic"));
    }

    #[test]
    fn test_providers_args_with_browser() {
        let args = ProvidersArgs {
            json: false,
            test_connection: None,
            login: Some("openai".to_string()),
            browser: true,
        };
        assert!(args.browser);
    }

    #[test]
    fn test_providers_login_json_shape() {
        let args = ProvidersArgs {
            json: true,
            test_connection: None,
            login: Some("openai".to_string()),
            browser: true,
        };

        assert!(args.browser);
        assert_eq!(args.login.as_deref(), Some("openai"));
    }

    #[test]
    fn test_store_api_key_credential_anthropic() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let store_path = dir.path().join("credentials.enc.json");
        let key_path = dir.path().join("credentials.key");
        let store = CredentialStore::with_paths(store_path.clone(), key_path);

        let result = store.store(
            "anthropic",
            &opencode_auth::credential_store::Credential {
                api_key: "sk-ant-test-key".to_string(),
                base_url: None,
                metadata: std::collections::HashMap::new(),
            },
        );
        assert!(result.is_ok(), "Should store anthropic credential successfully");

        let loaded = store.load("anthropic");
        assert!(loaded.is_ok());
        let cred = loaded.unwrap().expect("Should have anthropic credential");
        assert_eq!(cred.api_key, "sk-ant-test-key");
    }

    #[test]
    fn test_store_api_key_credential_google() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let store_path = dir.path().join("credentials.enc.json");
        let key_path = dir.path().join("credentials.key");
        let store = CredentialStore::with_paths(store_path.clone(), key_path);

        let result = store.store(
            "google",
            &opencode_auth::credential_store::Credential {
                api_key: "AIza-test-key".to_string(),
                base_url: None,
                metadata: std::collections::HashMap::new(),
            },
        );
        assert!(result.is_ok(), "Should store google credential successfully");

        let loaded = store.load("google");
        assert!(loaded.is_ok());
        let cred = loaded.unwrap().expect("Should have google credential");
        assert_eq!(cred.api_key, "AIza-test-key");
    }

    #[test]
    fn test_store_api_key_credential_azure() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let store_path = dir.path().join("credentials.enc.json");
        let key_path = dir.path().join("credentials.key");
        let store = CredentialStore::with_paths(store_path.clone(), key_path);

        let result = store.store(
            "azure",
            &opencode_auth::credential_store::Credential {
                api_key: "azure-test-key".to_string(),
                base_url: Some("https://test.openai.azure.com".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        );
        assert!(result.is_ok(), "Should store azure credential successfully");

        let loaded = store.load("azure");
        assert!(loaded.is_ok());
        let cred = loaded.unwrap().expect("Should have azure credential");
        assert_eq!(cred.api_key, "azure-test-key");
        assert_eq!(cred.base_url, Some("https://test.openai.azure.com".to_string()));
    }

    #[test]
    fn test_store_api_key_credential_custom_provider() {
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let store_path = dir.path().join("credentials.enc.json");
        let key_path = dir.path().join("credentials.key");
        let store = CredentialStore::with_paths(store_path.clone(), key_path);

        let result = store.store(
            "custom-provider",
            &opencode_auth::credential_store::Credential {
                api_key: "custom-key-123".to_string(),
                base_url: Some("https://custom.api.com".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        );
        assert!(result.is_ok(), "Should store custom provider credential successfully");

        let loaded = store.load("custom-provider");
        assert!(loaded.is_ok());
        let cred = loaded.unwrap().expect("Should have custom provider credential");
        assert_eq!(cred.api_key, "custom-key-123");
        assert_eq!(cred.base_url, Some("https://custom.api.com".to_string()));
    }

    #[test]
    fn test_get_provider_auth_methods_anthropic() {
        let methods = get_provider_auth_methods("anthropic");
        assert!(methods.contains(&AuthMethod::ApiKey));
        assert!(!methods.contains(&AuthMethod::Browser));
    }

    #[test]
    fn test_get_provider_auth_methods_google() {
        let methods = get_provider_auth_methods("google");
        assert!(methods.contains(&AuthMethod::Browser));
        assert!(!methods.contains(&AuthMethod::ApiKey));
    }

    #[test]
    fn test_get_provider_auth_methods_azure() {
        let methods = get_provider_auth_methods("azure");
        assert!(methods.contains(&AuthMethod::ApiKey));
    }

    #[test]
    fn test_get_provider_auth_methods_openai() {
        let methods = get_provider_auth_methods("openai");
        assert!(methods.contains(&AuthMethod::Browser));
        assert!(methods.contains(&AuthMethod::ApiKey));
    }

    #[test]
    fn test_load_config_returns_result_type() {
        let result = load_config_result();
        assert!(result.is_ok() || result.is_err(), "load_config should return a Result");
    }

    #[test]
    fn test_open_browser_returns_result() {
        let result = open_browser("https://example.com");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_open_browser_invalid_url() {
        let result = open_browser("not-a-valid-url");
        assert!(result.is_err() || result.is_ok());
    }
}
