use clap::{ArgAction, Args};
use opencode_core::Config;

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
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

    #[arg(long)]
    pub migrate: bool,

    #[arg(long)]
    pub remove: bool,

    pub value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_args_default() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: false,
            providers: false,
            set: None,
            migrate: false,
            remove: false,
            value: None,
        };
        assert_eq!(args.json, 0);
        assert!(!args.keybinds);
        assert!(!args.models);
        assert!(!args.providers);
        assert!(args.set.is_none());
        assert!(!args.migrate);
        assert!(!args.remove);
        assert!(args.value.is_none());
    }

    #[test]
    fn test_config_args_with_json() {
        let args = ConfigArgs {
            json: 1,
            keybinds: false,
            models: false,
            providers: false,
            set: None,
            migrate: false,
            remove: false,
            value: None,
        };
        assert_eq!(args.json, 1);
    }

    #[test]
    fn test_config_args_with_keybinds() {
        let args = ConfigArgs {
            json: 0,
            keybinds: true,
            models: false,
            providers: false,
            set: None,
            migrate: false,
            remove: false,
            value: None,
        };
        assert!(args.keybinds);
    }

    #[test]
    fn test_config_args_with_models() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: true,
            providers: false,
            set: None,
            migrate: false,
            remove: false,
            value: None,
        };
        assert!(args.models);
    }

    #[test]
    fn test_config_args_with_providers() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: false,
            providers: true,
            set: None,
            migrate: false,
            remove: false,
            value: None,
        };
        assert!(args.providers);
    }

    #[test]
    fn test_config_args_with_migrate() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: false,
            providers: false,
            set: None,
            migrate: true,
            remove: false,
            value: None,
        };
        assert!(args.migrate);
        assert!(!args.remove);
    }

    #[test]
    fn test_config_args_with_remove() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: false,
            providers: false,
            set: None,
            migrate: true,
            remove: true,
            value: None,
        };
        assert!(args.migrate);
        assert!(args.remove);
    }

    #[test]
    fn test_config_args_with_value() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: false,
            providers: false,
            set: None,
            migrate: false,
            remove: false,
            value: Some("test_value".to_string()),
        };
        assert_eq!(args.value.as_deref(), Some("test_value"));
    }

    #[test]
    fn test_config_args_multiple_flags() {
        let args = ConfigArgs {
            json: 2,
            keybinds: true,
            models: true,
            providers: true,
            set: None,
            migrate: false,
            remove: false,
            value: None,
        };
        assert_eq!(args.json, 2);
        assert!(args.keybinds);
        assert!(args.models);
        assert!(args.providers);
    }

    #[test]
    fn test_config_args_with_show_value() {
        let args = ConfigArgs {
            json: 0,
            keybinds: false,
            models: false,
            providers: false,
            set: None,
            migrate: false,
            remove: false,
            value: Some("show".to_string()),
        };
        assert_eq!(args.value.as_deref(), Some("show"));
    }
}

pub(crate) fn run(args: ConfigArgs) {
    if args.set.is_some() {
        eprintln!("Invalid setting key");
        std::process::exit(1);
    }

    if args.migrate {
        eprintln!("TOML configuration format is no longer supported.");
        eprintln!("Please manually convert your config.toml to config.jsonc format.");
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
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return;
    }

    println!("Config path: {}", path.display());
    if let Some(model) = config.model {
        println!("Model: {}", model);
    }
}
