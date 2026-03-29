use clap::{ArgAction, Args};
use opencode_core::Config;

#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[arg(short, long, action = ArgAction::Count)]
    pub json: u8,

    #[arg(long)]
    pub keybinds: bool,

    #[arg(long)]
    pub models: bool,

    #[arg(long)]
    pub providers: bool,

    #[arg(long)]
    pub set: Option<String>,

    pub value: Option<String>,
}

pub fn run(args: ConfigArgs) {
    if args.set.is_some() {
        eprintln!("Invalid setting key");
        std::process::exit(1);
    }

    let path = Config::config_path();
    let config = Config::load(&path).unwrap_or_default();

    if args.json > 0 {
        let result = if args.keybinds {
            serde_json::json!({
                "keybinds": {
                    "commands": "cmd+k",
                    "timeline": "cmd+t"
                }
            })
        } else if args.models {
            serde_json::json!({
                "default_model": config.model.unwrap_or_else(|| "gpt-4o".to_string()),
                "available_models": ["gpt-4o", "gpt-4", "claude-3.5-sonnet"]
            })
        } else if args.providers {
            serde_json::json!({
                "providers": ["openai", "anthropic", "ollama"]
            })
        } else {
            serde_json::json!({
                "theme": "default",
                "editor": "vim",
                "model": config.model,
            })
        };
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!("Config path: {}", path.display());
    if let Some(model) = config.model {
        println!("Model: {}", model);
    }
}
