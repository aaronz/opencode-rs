use clap::{Args, Subcommand};
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ModelsArgs {
    #[arg(short, long)]
    pub provider: Option<String>,

    #[arg(short, long)]
    pub json: bool,

    #[arg(short, long)]
    pub visibility: Option<String>,

    #[arg(long)]
    pub switch: Option<String>,

    #[command(subcommand)]
    pub action: Option<ModelsAction>,
}

#[derive(Subcommand, Debug)]
pub enum ModelsAction {
    Visibility {
        #[arg(short, long)]
        hide: Option<String>,

        #[arg(short, long)]
        show: Option<String>,

        #[arg(long)]
        list_hidden: bool,
    },
}

#[derive(Debug, Serialize)]
struct ModelRow {
    id: String,
    name: String,
    provider: String,
    supports_streaming: bool,
    supports_functions: bool,
    max_input_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_args_default() {
        let args = ModelsArgs {
            provider: None,
            json: false,
            visibility: None,
            switch: None,
            action: None,
        };
        assert!(args.provider.is_none());
        assert!(!args.json);
        assert!(args.visibility.is_none());
        assert!(args.switch.is_none());
        assert!(args.action.is_none());
    }

    #[test]
    fn test_models_args_with_provider() {
        let args = ModelsArgs {
            provider: Some("openai".to_string()),
            json: false,
            visibility: None,
            switch: None,
            action: None,
        };
        assert_eq!(args.provider.as_deref(), Some("openai"));
    }

    #[test]
    fn test_models_args_with_json() {
        let args = ModelsArgs {
            provider: None,
            json: true,
            visibility: None,
            switch: None,
            action: None,
        };
        assert!(args.json);
    }

    #[test]
    fn test_models_args_with_visibility() {
        let args = ModelsArgs {
            provider: None,
            json: false,
            visibility: Some("visible".to_string()),
            switch: None,
            action: None,
        };
        assert_eq!(args.visibility.as_deref(), Some("visible"));
    }

    #[test]
    fn test_models_args_with_switch() {
        let args = ModelsArgs {
            provider: None,
            json: false,
            visibility: None,
            switch: Some("gpt-4o".to_string()),
            action: None,
        };
        assert_eq!(args.switch.as_deref(), Some("gpt-4o"));
    }

    #[test]
    fn test_models_args_full() {
        let args = ModelsArgs {
            provider: Some("anthropic".to_string()),
            json: true,
            visibility: Some("hidden".to_string()),
            switch: Some("claude-3-5-sonnet".to_string()),
            action: None,
        };
        assert_eq!(args.provider.as_deref(), Some("anthropic"));
        assert!(args.json);
        assert_eq!(args.visibility.as_deref(), Some("hidden"));
        assert_eq!(args.switch.as_deref(), Some("claude-3-5-sonnet"));
    }

    #[test]
    fn test_models_action_visibility_hide() {
        let action = ModelsAction::Visibility {
            hide: Some("gpt-4".to_string()),
            show: None,
            list_hidden: false,
        };
        match &action {
            ModelsAction::Visibility {
                hide,
                show,
                list_hidden,
            } => {
                assert_eq!(hide.as_deref(), Some("gpt-4"));
                assert!(show.is_none());
                assert!(!*list_hidden);
            }
        }
    }

    #[test]
    fn test_models_action_visibility_show() {
        let action = ModelsAction::Visibility {
            hide: None,
            show: Some("gpt-4".to_string()),
            list_hidden: false,
        };
        match &action {
            ModelsAction::Visibility {
                hide,
                show,
                list_hidden,
            } => {
                assert!(hide.is_none());
                assert_eq!(show.as_deref(), Some("gpt-4"));
                assert!(!*list_hidden);
            }
        }
    }

    #[test]
    fn test_models_action_visibility_list_hidden() {
        let action = ModelsAction::Visibility {
            hide: None,
            show: None,
            list_hidden: true,
        };
        match &action {
            ModelsAction::Visibility {
                hide,
                show,
                list_hidden,
            } => {
                assert!(hide.is_none());
                assert!(show.is_none());
                assert!(*list_hidden);
            }
        }
    }

    #[test]
    fn test_model_row_serialization() {
        let row = ModelRow {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            provider: "openai".to_string(),
            supports_streaming: true,
            supports_functions: true,
            max_input_tokens: 128000,
        };
        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("gpt-4o"));
        assert!(json.contains("openai"));
        assert!(json.contains("128000"));
    }

    #[test]
    fn test_load_hidden_models_empty_when_no_file() {
        std::env::remove_var("OPENCODE_CONFIG_DIR");
        let temp_dir = std::env::temp_dir();
        std::env::set_var("OPENCODE_CONFIG_DIR", temp_dir.to_string_lossy().as_ref());
        let hidden_file = PathBuf::from(temp_dir).join("opencode-hidden-models.json");
        let _ = std::fs::remove_file(hidden_file);
        let hidden = load_hidden_models();
        assert!(hidden.is_empty());
        std::env::remove_var("OPENCODE_CONFIG_DIR");
    }

    #[test]
    fn test_load_hidden_models_with_existing_file() {
        let temp_dir = std::env::temp_dir();
        std::env::set_var("OPENCODE_CONFIG_DIR", temp_dir.to_string_lossy().as_ref());
        let hidden_file = PathBuf::from(temp_dir).join("opencode-hidden-models.json");
        std::fs::write(&hidden_file, r#"["gpt-4", "claude-3"]"#).unwrap();
        let hidden = load_hidden_models();
        assert!(hidden.contains("gpt-4"));
        assert!(hidden.contains("claude-3"));
        let _ = std::fs::remove_file(hidden_file);
        std::env::remove_var("OPENCODE_CONFIG_DIR");
    }

    #[test]
    fn test_save_and_load_hidden_models() {
        let temp_dir = std::env::temp_dir();
        std::env::set_var("OPENCODE_CONFIG_DIR", temp_dir.to_string_lossy().as_ref());
        let hidden_file = PathBuf::from(temp_dir).join("opencode-hidden-models.json");
        let _ = std::fs::remove_file(&hidden_file);

        let mut hidden = BTreeSet::new();
        hidden.insert("model-1".to_string());
        hidden.insert("model-2".to_string());
        save_hidden_models(&hidden);

        let loaded = load_hidden_models();
        assert!(loaded.contains("model-1"));
        assert!(loaded.contains("model-2"));

        let _ = std::fs::remove_file(&hidden_file);
        std::env::remove_var("OPENCODE_CONFIG_DIR");
    }
}

fn hidden_models_path() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENCODE_CONFIG_DIR") {
        let path = PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("hidden-models.json");
    }

    std::env::temp_dir().join("opencode-hidden-models.json")
}

fn load_hidden_models() -> BTreeSet<String> {
    let path = hidden_models_path();
    if !path.exists() {
        return BTreeSet::new();
    }

    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<String>>(&content).ok())
        .map(|models| models.into_iter().collect())
        .unwrap_or_default()
}

fn save_hidden_models(hidden_models: &BTreeSet<String>) {
    let path = hidden_models_path();
    let payload = hidden_models.iter().cloned().collect::<Vec<_>>();
    let serialized =
        serde_json::to_string_pretty(&payload).expect("failed to serialize hidden models");
    std::fs::write(&path, serialized).expect("failed to write hidden models file");
}

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

pub fn run(args: ModelsArgs) {
    match args.action {
        Some(ModelsAction::Visibility {
            hide,
            show,
            list_hidden,
        }) => {
            let mut hidden_models = load_hidden_models();
            if let Some(model_id) = hide {
                hidden_models.insert(model_id.clone());
                save_hidden_models(&hidden_models);
                println!("Hiding model: {}", model_id);
            } else if let Some(model_id) = show {
                hidden_models.remove(&model_id);
                save_hidden_models(&hidden_models);
                println!("Showing model: {}", model_id);
            } else if list_hidden {
                println!("Hidden models:");
                for model_id in hidden_models {
                    println!("  {}", model_id);
                }
            } else {
                println!("Visibility action requires --hide, --show, or --list-hidden");
            }
        }
        None => {
            if let Some(model_id) = args.switch.clone() {
                let registry = ModelRegistry::default();
                if registry.get(&model_id).is_none() {
                    eprintln!("Unknown model: {}", model_id);
                    std::process::exit(1);
                }

                let path = Config::config_path();
                let mut config = load_config();
                config.model = Some(model_id.clone());
                if let Err(error) = config.save(&path) {
                    eprintln!("Failed to save config: {}", error);
                    std::process::exit(1);
                }

                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "active_model": model_id,
                        }))
                        .expect("failed to serialize JSON output")
                    );
                } else {
                    println!("Switched model to {}", model_id);
                }
                return;
            }

            let hidden_models = load_hidden_models();
            let registry = ModelRegistry::default();
            let model_infos = match args.provider.as_deref() {
                Some(provider) => registry.list_by_provider(provider),
                None => registry.list(),
            };

            let mut models = model_infos
                .into_iter()
                .map(|model| ModelRow {
                    id: model.name.clone(),
                    name: model.name.clone(),
                    provider: model.provider.clone(),
                    supports_streaming: model.supports_streaming,
                    supports_functions: model.supports_functions,
                    max_input_tokens: model.max_input_tokens,
                })
                .collect::<Vec<_>>();

            if let Some(visibility) = args.visibility.as_deref() {
                match visibility {
                    "visible" => models.retain(|model| !hidden_models.contains(&model.id)),
                    "hidden" => models.retain(|model| hidden_models.contains(&model.id)),
                    _ => {}
                }
            }

            models.sort_by(|left, right| left.id.cmp(&right.id));

            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "action": "list",
                        "models": models,
                    }))
                    .expect("failed to serialize JSON output")
                );
            } else if let Some(vis) = args.visibility {
                println!("Models with visibility: {}", vis);
                for model in models {
                    println!("  {}", model.id);
                }
            } else {
                for model in models {
                    println!(
                        "{}\t{}\t{}",
                        model.provider, model.id, model.max_input_tokens
                    );
                }
            }
        }
    }
}
