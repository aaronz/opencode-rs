use clap::Args;
use opencode_core::Config;
use opencode_llm::{ModelRegistry, OpenAiBrowserAuthService, OpenAiBrowserAuthStore};
use serde::Serialize;
use serde_json::json;
use std::process::Command;

#[derive(Args, Debug)]
pub struct ProvidersArgs {
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
        "ollama" => "Ollama".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
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

pub fn run(args: ProvidersArgs) {
    let config = load_config();
    let registry = ModelRegistry::default();
    let providers = ["openai", "anthropic", "ollama"]
        .into_iter()
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
            let result = json!({
                "action": "login",
                "provider": provider_id,
                "method": if args.browser { "browser" } else { "api_key" },
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
            );
            return;
        }

        if provider_id != "openai" || !args.browser {
            eprintln!("Only 'providers --login openai --browser' is currently implemented");
            std::process::exit(1);
        }

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
        return;
    }

    if let Some(provider_id) = args.test_connection {
        let provider = providers.iter().find(|provider| provider.id == provider_id);
        match provider {
            Some(provider) => {
                println!(
                    "Provider {} connection status: {}",
                    provider.id, provider.status
                );
                return;
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
        return;
    }

    for provider in providers {
        println!(
            "{}\t{}\t{}",
            provider.id, provider.status, provider.model_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
