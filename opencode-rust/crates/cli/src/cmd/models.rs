use clap::{Args, Subcommand};
use opencode_core::Config;
use opencode_llm::catalog::fetcher::ProviderCatalogFetcher;
use opencode_llm::ModelRegistry;
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use crate::cmd::load_config;

#[allow(dead_code)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Args, Debug)]
pub(crate) struct ModelsArgs {
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
pub(crate) enum ModelsAction {
    Visibility {
        #[arg(short, long)]
        hide: Option<String>,

        #[arg(short, long)]
        show: Option<String>,

        #[arg(long)]
        list_hidden: bool,
    },
    Refresh,
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

#[allow(clippy::items_after_test_module)]
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
            ModelsAction::Refresh => panic!("Expected Visibility variant"),
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
            ModelsAction::Refresh => panic!("Expected Visibility variant"),
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
            ModelsAction::Refresh => panic!("Expected Visibility variant"),
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
    fn test_save_and_load_hidden_models() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var(
            "OPENCODE_CONFIG_DIR",
            temp_dir.path().to_string_lossy().as_ref(),
        );

        let mut hidden = HashSet::new();
        hidden.insert("model-1".to_string());
        hidden.insert("model-2".to_string());
        save_hidden_models(&hidden);

        let loaded = load_hidden_models();
        assert!(loaded.contains("model-1"));
        assert!(loaded.contains("model-2"));

        std::env::remove_var("OPENCODE_CONFIG_DIR");
    }

    #[test]
    fn test_model_visibility_is_configurable() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var(
            "OPENCODE_CONFIG_DIR",
            temp_dir.path().to_string_lossy().as_ref(),
        );

        let mut hidden = HashSet::new();
        hidden.insert("gpt-4o".to_string());
        hidden.insert("claude-3-5-sonnet".to_string());
        save_hidden_models(&hidden);

        let loaded = load_hidden_models();
        assert!(loaded.contains("gpt-4o"));
        assert!(loaded.contains("claude-3-5-sonnet"));
        assert!(!loaded.contains("gpt-3.5-turbo"));

        std::env::remove_var("OPENCODE_CONFIG_DIR");
    }

    #[test]
    fn test_hidden_models_filtered_from_list() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::env::set_var(
            "OPENCODE_CONFIG_DIR",
            temp_dir.path().to_string_lossy().as_ref(),
        );

        let mut hidden = HashSet::new();
        hidden.insert("gpt-4o".to_string());
        save_hidden_models(&hidden);

        let hidden_models = load_hidden_models();
        assert!(hidden_models.contains("gpt-4o"));

        let registry = ModelRegistry::default();
        let all_models = registry.list();
        let visible_models: Vec<_> = all_models
            .into_iter()
            .filter(|m| !hidden_models.contains(&m.name))
            .collect();

        assert!(!visible_models.iter().any(|m| m.name == "gpt-4o"));

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

fn load_hidden_models() -> HashSet<String> {
    let mut hidden: HashSet<String> = HashSet::new();
    let flat_file_path = hidden_models_path();
    let mut migrated_from_flat_file = false;

    if flat_file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&flat_file_path) {
            if let Ok(models) = serde_json::from_str::<Vec<String>>(&content) {
                for model in models {
                    hidden.insert(model);
                }
                migrated_from_flat_file = true;
            }
        }
    }

    if migrated_from_flat_file {
        tracing::warn!(
            "Deprecated hidden-models.json file detected at {}. \
            Model visibility is now stored in config.json. \
            The flat file will be ignored in future versions.",
            flat_file_path.display()
        );
    }

    let path = Config::config_path();
    if let Ok(config) = Config::load(&path) {
        if let Some(models) = config.hidden_models {
            for model in models {
                hidden.insert(model);
            }
        }
    }

    hidden
}

fn save_hidden_models(hidden_models: &HashSet<String>) {
    let path = Config::config_path();
    let mut config = Config::load(&path).unwrap_or_default();
    config.hidden_models = Some(hidden_models.iter().cloned().collect::<Vec<_>>());
    if let Err(error) = config.save(&path) {
        eprintln!("Failed to save config: {}", error);
    }
}

fn default_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("opencode")
        .join("models.json")
}

pub(crate) fn run_refresh() {
    let cache_path = default_cache_path();
    let fetcher = Arc::new(ProviderCatalogFetcher::new(cache_path));

    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let result = rt.block_on(fetcher.force_refresh());

    match result {
        Ok(catalog) => {
            println!("Refresh successful");
            println!("Providers: {}", catalog.providers.len());
            for (id, provider) in &catalog.providers {
                println!("  {}: {} models", id, provider.models.len());
            }
        }
        Err(e) => {
            eprintln!("Refresh failed: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn run(args: ModelsArgs) {
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
                eprintln!("Visibility action requires --hide, --show, or --list-hidden");
                std::process::exit(1);
            }
        }
        Some(ModelsAction::Refresh) => {
            run_refresh();
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
