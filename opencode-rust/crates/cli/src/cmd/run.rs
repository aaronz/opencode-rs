use clap::Args;
use opencode_core::Config;
use opencode_llm::provider_abstraction::ProviderManager;
use opencode_llm::ProviderSpec;
use opencode_tui::{App, OutputFormat};

use crate::cmd::load_config;

#[derive(Args, Debug)]
pub(crate) struct RunArgs {
    #[arg(short, long)]
    pub prompt: Option<String>,

    #[arg(short, long)]
    pub agent: Option<String>,

    #[arg(short, long)]
    pub model: Option<String>,

    #[arg(short, long)]
    pub continue_session: Option<String>,

    #[arg(short = 'j', long)]
    pub attach: Option<String>,

    #[arg(short = 'y', long)]
    pub yes: bool,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(short, long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_args_default() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: None,
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Text,
        };
        assert!(args.prompt.is_none());
        assert!(args.agent.is_none());
        assert!(args.model.is_none());
        assert!(!args.yes);
        matches!(args.format, OutputFormat::Text);
    }

    #[test]
    fn test_run_args_with_prompt() {
        let args = RunArgs {
            prompt: Some("Hello world".to_string()),
            agent: None,
            model: None,
            continue_session: None,
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Text,
        };
        assert_eq!(args.prompt.as_deref(), Some("Hello world"));
    }

    #[test]
    fn test_run_args_with_agent() {
        let args = RunArgs {
            prompt: None,
            agent: Some("expert".to_string()),
            model: None,
            continue_session: None,
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Text,
        };
        assert_eq!(args.agent.as_deref(), Some("expert"));
    }

    #[test]
    fn test_run_args_with_model() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: Some("gpt-4o".to_string()),
            continue_session: None,
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Text,
        };
        assert_eq!(args.model.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn test_run_args_with_continue_session() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: Some("session-123".to_string()),
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Text,
        };
        assert_eq!(args.continue_session.as_deref(), Some("session-123"));
    }

    #[test]
    fn test_run_args_with_attach() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: None,
            attach: Some("attach-id".to_string()),
            yes: false,
            title: None,
            format: OutputFormat::Text,
        };
        assert_eq!(args.attach.as_deref(), Some("attach-id"));
    }

    #[test]
    fn test_run_args_with_yes() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: None,
            attach: None,
            yes: true,
            title: None,
            format: OutputFormat::Text,
        };
        assert!(args.yes);
    }

    #[test]
    fn test_run_args_with_title() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: None,
            attach: None,
            yes: false,
            title: Some("My Task".to_string()),
            format: OutputFormat::Text,
        };
        assert_eq!(args.title.as_deref(), Some("My Task"));
    }

    #[test]
    fn test_run_args_with_json_format() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: None,
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Json,
        };
        matches!(args.format, OutputFormat::Json);
    }

    #[test]
    fn test_run_args_with_ndjson_format() {
        let args = RunArgs {
            prompt: None,
            agent: None,
            model: None,
            continue_session: None,
            attach: None,
            yes: false,
            title: None,
            format: OutputFormat::Ndjson,
        };
        matches!(args.format, OutputFormat::Ndjson);
    }

    #[test]
    fn test_run_args_full() {
        let args = RunArgs {
            prompt: Some("Test prompt".to_string()),
            agent: Some("review".to_string()),
            model: Some("claude-3-5-sonnet".to_string()),
            continue_session: Some("session-456".to_string()),
            attach: Some("attach-789".to_string()),
            yes: true,
            title: Some("Review PR".to_string()),
            format: OutputFormat::Json,
        };
        assert_eq!(args.prompt.as_deref(), Some("Test prompt"));
        assert_eq!(args.agent.as_deref(), Some("review"));
        assert_eq!(args.model.as_deref(), Some("claude-3-5-sonnet"));
        assert_eq!(args.continue_session.as_deref(), Some("session-456"));
        assert_eq!(args.attach.as_deref(), Some("attach-789"));
        assert!(args.yes);
        assert_eq!(args.title.as_deref(), Some("Review PR"));
        matches!(args.format, OutputFormat::Json);
    }
}

pub(crate) fn run(args: RunArgs) {
    if let Some(prompt) = args.prompt.clone() {
        let config = load_config();
        let model = args
            .model
            .clone()
            .or_else(|| config.get("agent.model"))
            .unwrap_or_else(|| "gpt-4o".to_string());

        match args.format {
            OutputFormat::Ndjson | OutputFormat::Json => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(run_llm(&config, &model, &prompt, args.format));
            }
            _ => {
                println!("Mode: non-interactive");
                println!("Model: {}", model);
                println!("Prompt: {}", prompt);
            }
        }
        return;
    }

    let mut app = App::new();

    if let Some(agent) = args.agent {
        app.agent = agent;
    }

    if let Err(e) = app.run() {
        eprintln!("Error running TUI: {}", e);
    }
}

async fn run_llm(config: &Config, model: &str, prompt: &str, format: OutputFormat) {
    let provider_manager = ProviderManager::new();

    let (provider_type, model_name) = if model.contains('/') {
        let parts: Vec<&str> = model.split('/').collect();
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("openai".to_string(), model.to_string())
    };

    fn get_api_key(config: &Config, provider: &str) -> String {
        std::env::var(format!("{}_API_KEY", provider.to_uppercase()))
            .ok()
            .or_else(|| {
                config
                    .get_provider(provider)
                    .and_then(|p| p.options.as_ref())
                    .and_then(|o| o.api_key.clone())
            })
            .unwrap_or_default()
    }

    let spec = match provider_type.as_str() {
        "openai" => ProviderSpec::OpenAI {
            api_key: get_api_key(config, "openai"),
            model: model_name,
            base_url: config
                .get_provider("openai")
                .and_then(|p| p.options.as_ref())
                .and_then(|o| o.base_url.clone()),
        },
        "anthropic" => ProviderSpec::Anthropic {
            api_key: get_api_key(config, "anthropic"),
            model: model_name,
            base_url: config
                .get_provider("anthropic")
                .and_then(|p| p.options.as_ref())
                .and_then(|o| o.base_url.clone()),
        },
        "google" => ProviderSpec::Google {
            api_key: get_api_key(config, "google"),
            model: model_name,
        },
        "ollama" => ProviderSpec::Ollama {
            base_url: config
                .get_provider("ollama")
                .and_then(|p| p.options.as_ref())
                .and_then(|o| o.base_url.clone()),
            model: model_name,
        },
        "lmstudio" => ProviderSpec::LmStudio {
            base_url: config
                .get_provider("lmstudio")
                .and_then(|p| p.options.as_ref())
                .and_then(|o| o.base_url.clone()),
            model: model_name,
        },
        _ => ProviderSpec::OpenAI {
            api_key: get_api_key(config, "openai"),
            model: model.to_string(),
            base_url: None,
        },
    };

    let provider = match provider_manager.create_provider(&spec) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create provider: {}", e);
            return;
        }
    };

    match format {
        OutputFormat::Json => {
            use std::sync::{Arc, Mutex};
            let chunks = Arc::new(Mutex::new(Vec::<String>::new()));
            let chunks_clone = chunks.clone();
            let result = provider
                .complete_streaming(
                    prompt,
                    Box::new(move |chunk| {
                        chunks_clone.lock().unwrap().push(chunk);
                    }),
                )
                .await;

            match result {
                Ok(_) => {
                    let content = chunks.lock().unwrap().join("");
                    let response = serde_json::json!({
                        "event": "done",
                        "model": model,
                        "content": content
                    });
                    println!("{}", serde_json::to_string(&response).unwrap());
                }
                Err(e) => {
                    let response = serde_json::json!({
                        "event": "error",
                        "error": e.to_string()
                    });
                    println!("{}", serde_json::to_string(&response).unwrap());
                }
            }
        }
        OutputFormat::Ndjson => {
            use std::sync::{Arc, Mutex};

            let serializer = Arc::new(Mutex::new(crate::output::NdjsonSerializer::stdout()));
            serializer.lock().unwrap().write_start(model).ok();

            let serializer_clone = serializer.clone();
            let result = provider
                .complete_streaming(
                    prompt,
                    Box::new(move |chunk| {
                        serializer_clone.lock().unwrap().write_chunk(&chunk).ok();
                    }),
                )
                .await;

            match result {
                Ok(_) => {
                    serializer.lock().unwrap().write_done().ok();
                }
                Err(e) => {
                    serializer.lock().unwrap().write_error(&e.to_string()).ok();
                }
            }

            serializer.lock().unwrap().flush().ok();
        }
        _ => {
            let result = provider
                .complete_streaming(
                    prompt,
                    Box::new(|chunk| {
                        print!("{}", chunk);
                    }),
                )
                .await;

            if result.is_err() {
                eprintln!("Error: {}", result.err().unwrap());
            }
        }
    }
}
