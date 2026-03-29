use clap::Args;
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use serde::Serialize;
use serde_json::json;

#[derive(Args, Debug)]
pub struct ProvidersArgs {
    #[arg(short, long)]
    pub json: bool,

    #[arg(long = "test-connection")]
    pub test_connection: Option<String>,
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
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    for provider in providers {
        println!(
            "{}\t{}\t{}",
            provider.id, provider.status, provider.model_count
        );
    }
}
