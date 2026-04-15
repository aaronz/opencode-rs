use clap::Args;
use opencode_core::Config;
use opencode_tui::{App, OutputFormat};

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(short, long)]
    pub prompt: Option<String>,

    #[arg(short, long)]
    pub agent: Option<String>,

    #[arg(short, long)]
    pub model: Option<String>,

    #[arg(short, long)]
    pub continue_session: Option<String>,

    #[arg(short, long)]
    pub attach: Option<String>,

    #[arg(short = 'y', long)]
    pub yes: bool,

    #[arg(long)]
    pub title: Option<String>,

    #[arg(short, long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

pub fn run(args: RunArgs) {
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
