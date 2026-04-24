use clap::Args;
use opencode_core::Config;
use opencode_tui::{App, OutputFormat};

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

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

pub(crate) fn run(args: RunArgs) {
    if let Some(prompt) = args.prompt.clone() {
        let config = load_config();
        let model = args
            .model
            .clone()
            .or(config.model)
            .unwrap_or_else(|| "gpt-4o".to_string());

        match args.format {
            OutputFormat::Ndjson => {
                let mut serializer = crate::output::NdjsonSerializer::stdout();
                serializer.write_start(&model).ok();
                serializer
                    .write_message("system", "Mode: non-interactive")
                    .ok();
                serializer.write_message("user", &prompt).ok();
                serializer.flush().ok();
            }
            OutputFormat::Json => {
                let response = serde_json::json!({
                    "model": model,
                    "prompt": prompt,
                    "response": "This is a placeholder response. Use TUI mode for actual LLM interaction."
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&response)
                        .expect("failed to serialize JSON output")
                );
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
