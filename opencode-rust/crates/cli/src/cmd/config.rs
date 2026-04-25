use clap::{ArgAction, Args};
use opencode_core::Config;
use std::path::PathBuf;

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

#[allow(clippy::items_after_test_module)]
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

    #[test]
    fn test_parse_primitive_value_string() {
        let result = parse_primitive_value("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_primitive_value_bool_true() {
        let result = parse_primitive_value("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Bool(true));
    }

    #[test]
    fn test_parse_primitive_value_bool_false() {
        let result = parse_primitive_value("false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Bool(false));
    }

    #[test]
    fn test_parse_primitive_value_integer() {
        let result = parse_primitive_value("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Number(42.into()));
    }

    #[test]
    fn test_parse_primitive_value_float() {
        let result = parse_primitive_value("3.14");
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(val.is_number());
    }

    #[test]
    fn test_parse_primitive_value_null() {
        let result = parse_primitive_value("null");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Null);
    }

    #[test]
    fn test_parse_array_value() {
        let result = parse_array_value("[a, b, c]");
        assert!(result.is_ok());
        let arr = result.unwrap();
        assert!(arr.is_array());
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_parse_array_value_empty() {
        let result = parse_array_value("[]");
        assert!(result.is_ok());
        let arr = result.unwrap();
        assert!(arr.is_array());
        assert!(arr.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_parse_array_value_malformed() {
        let result = parse_array_value("not an array");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_nested_value_simple_key() {
        let json = serde_json::json!({"name": "old"});
        let result = set_nested_value(json, &["name"], "new");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get("name").unwrap(), "new");
    }

    #[test]
    fn test_set_nested_value_nested_key() {
        let json = serde_json::json!({"agent": {"model": "old"}});
        let result = set_nested_value(json, &["agent", "model"], "new");
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.pointer("/agent/model").unwrap(), "new");
    }

    #[test]
    fn test_set_nested_value_invalid_path() {
        let json = serde_json::json!({"name": "value"});
        let result = set_nested_value(json, &["name", "path"], "value");
        assert!(result.is_err(), "Should fail when navigating through non-object value");
    }

    #[test]
    fn test_parse_value_with_type_string_to_string() {
        let current = serde_json::Value::String("old".to_string());
        let result = parse_value_with_type(&current, "new");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::String("new".to_string()));
    }

    #[test]
    fn test_parse_value_with_type_number_to_number() {
        let current = serde_json::Value::Number(42.into());
        let result = parse_value_with_type(&current, "100");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Number(100.into()));
    }

    #[test]
    fn test_parse_value_with_type_number_to_invalid() {
        let current = serde_json::Value::Number(42.into());
        let result = parse_value_with_type(&current, "not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_value_with_type_bool_to_invalid() {
        let current = serde_json::Value::Bool(true);
        let result = parse_value_with_type(&current, "not_a_boolean");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_value_with_type_bool_true() {
        let current = serde_json::Value::Bool(true);
        let result = parse_value_with_type(&current, "false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::Value::Bool(false));
    }

    #[test]
    fn test_parse_value_with_type_array() {
        let current = serde_json::Value::Array(vec![serde_json::Value::String("a".to_string()), serde_json::Value::String("b".to_string())]);
        let result = parse_value_with_type(&current, "[x, y, z]");
        assert!(result.is_ok());
        let arr = result.unwrap();
        assert!(arr.is_array());
        assert_eq!(arr.as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_migrate_from_toml_basic() {
        let toml_content = r#"
model = "gpt-4o"
logLevel = "debug"
temperature = 0.7
"#;
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert_eq!(config.model, Some("gpt-4o".to_string()));
        assert_eq!(config.log_level, Some(opencode_core::config::LogLevel::Debug));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_migrate_from_toml_snake_case_keys() {
        let toml_content = r#"
model = "gpt-4"
small_model = "gpt-3.5"
default_agent = "build"
api_key = "secret-key"
max_tokens = 1000
"#;
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert_eq!(config.model, Some("gpt-4".to_string()));
        assert_eq!(config.small_model, Some("gpt-3.5".to_string()));
        assert_eq!(config.default_agent, Some("build".to_string()));
        assert_eq!(config.api_key, Some("secret-key".to_string()));
        assert_eq!(config.max_tokens, Some(1000));
    }

    #[test]
    fn test_migrate_from_toml_with_providers() {
        let toml_content = r#"
[providers.openai]
apiKey = "openai-key"
baseUrl = "https://api.openai.com"

[providers.anthropic]
apiKey = "anthropic-key"
"#;
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert!(config.provider.is_some());
        let providers = config.provider.unwrap();
        assert!(providers.contains_key("openai"));
        assert!(providers.contains_key("anthropic"));
    }

    #[test]
    fn test_migrate_from_toml_share_mode() {
        let toml_content = r#"
share = "auto"
"#;
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert_eq!(config.share, Some(opencode_core::config::ShareMode::Auto));
    }

    #[test]
    fn test_migrate_from_toml_autoupdate() {
        let toml_content = r#"
autoUpdate = true
"#;
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert_eq!(config.autoupdate, Some(opencode_core::config::AutoUpdate::Bool(true)));
    }

    #[test]
    fn test_migrate_from_toml_disabled_providers() {
        let toml_content = r#"
disabledProviders = ["ollama", "local"]
"#;
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert_eq!(
            config.disabled_providers,
            Some(vec!["ollama".to_string(), "local".to_string()])
        );
    }

    #[test]
    fn test_migrate_from_toml_invalid_toml() {
        let toml_content = r#"
model = "gpt-4
"#;
        let toml_value: Result<toml::Value, _> = toml_content.parse();
        assert!(toml_value.is_err());
    }

    #[test]
    fn test_migrate_from_toml_empty() {
        let toml_content = "";
        let toml_value: toml::Value = toml_content.parse().unwrap();
        let config = migrate_from_toml(toml_value).unwrap();
        assert!(config.model.is_none());
        assert!(config.log_level.is_none());
    }

    #[test]
    fn test_keybinds_from_config_defaults() {
        let config = Config::load(&PathBuf::from("/nonexistent/path")).unwrap_or_default();
        let keybinds = config.tui.as_ref().and_then(|t| t.keybinds.as_ref());
        let commands = keybinds.and_then(|k| k.commands.as_deref());
        let timeline = keybinds.and_then(|k| k.timeline.as_deref());
        assert_eq!(commands.unwrap_or("cmd+k"), "cmd+k");
        assert_eq!(timeline.unwrap_or("cmd+t"), "cmd+t");
    }

    #[test]
    fn test_keybinds_from_config_custom_values() {
        let mut config = Config::default();
        config.tui = Some(opencode_core::config::TuiConfig {
            keybinds: Some(opencode_core::config::KeybindConfig {
                commands: Some("ctrl+x".to_string()),
                timeline: Some("ctrl+t".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        });
        let keybinds = config.tui.as_ref().and_then(|t| t.keybinds.as_ref());
        let commands = keybinds.and_then(|k| k.commands.as_deref());
        let timeline = keybinds.and_then(|k| k.timeline.as_deref());
        assert_eq!(commands.unwrap_or("cmd+k"), "ctrl+x");
        assert_eq!(timeline.unwrap_or("cmd+t"), "ctrl+t");
    }
}

pub(crate) fn run(args: ConfigArgs) {
    if args.migrate {
        run_migrate();
        return;
    }

    let path = Config::config_path();

    if let Some(key) = &args.set {
        let value = args.value.as_deref().unwrap_or("");
        if let Err(e) = set_config_value(&path, key, value) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let config = Config::load(&path).unwrap_or_default();

    if args.json > 0 {
        let result = if args.keybinds {
            let keybinds = config.tui.as_ref().and_then(|t| t.keybinds.as_ref());
            let commands = keybinds.and_then(|k| k.commands.as_deref());
            let timeline = keybinds.and_then(|k| k.timeline.as_deref());
            serde_json::json!({
                "keybinds": {
                    "commands": commands.unwrap_or("cmd+k"),
                    "timeline": timeline.unwrap_or("cmd+t")
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

fn run_migrate() {
    let config_dir = Config::config_path().parent().map(|p| p.to_path_buf());

    let Some(config_dir) = config_dir else {
        eprintln!("Error: Could not determine config directory");
        std::process::exit(1);
    };

    let toml_path = config_dir.join("config.toml");

    if !toml_path.exists() {
        eprintln!(
            "No config.toml found at {}. Nothing to migrate.",
            toml_path.display()
        );
        return;
    }

    eprintln!("Found TOML config at {}. Starting migration...", toml_path.display());

    let toml_content = match std::fs::read_to_string(&toml_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading config.toml: {}", e);
            std::process::exit(1);
        }
    };

    let toml_value: toml::Value = match toml_content.parse() {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error parsing config.toml: {}", e);
            std::process::exit(1);
        }
    };

    let config = match migrate_from_toml(toml_value) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error migrating config: {}", e);
            std::process::exit(1);
        }
    };

    let jsonc_path = config_dir.join("config.jsonc");

    if let Err(e) = config.save(&jsonc_path) {
        eprintln!("Error saving migrated config to {}: {}", jsonc_path.display(), e);
        std::process::exit(1);
    }

    eprintln!(
        "Successfully migrated config from {} to {}",
        toml_path.display(),
        jsonc_path.display()
    );

    let backup_path = config_dir.join("config.toml.bak");
    if let Err(e) = std::fs::rename(&toml_path, &backup_path) {
        eprintln!(
            "Warning: Could not backup old config to {}: {}",
            backup_path.display(),
            e
        );
    } else {
        eprintln!("Backed up old config to {}", backup_path.display());
    }
}

fn migrate_from_toml(toml_value: toml::Value) -> Result<Config, String> {
    let table = toml_value
        .as_table()
        .ok_or_else(|| "TOML root must be a table".to_string())?;

    let mut config = Config::default();

    if let Some(v) = table.get("schema").and_then(|v| v.as_str()) {
        config.schema = Some(v.to_string());
    }

    if let Some(v) = table.get("logLevel").or_else(|| table.get("log_level")).and_then(|v| v.as_str()) {
        config.log_level = match v.to_lowercase().as_str() {
            "trace" => Some(opencode_core::config::LogLevel::Trace),
            "debug" => Some(opencode_core::config::LogLevel::Debug),
            "info" => Some(opencode_core::config::LogLevel::Info),
            "warn" | "warning" => Some(opencode_core::config::LogLevel::Warn),
            "error" => Some(opencode_core::config::LogLevel::Error),
            _ => None,
        };
    }

    if let Some(v) = table.get("model").and_then(|v| v.as_str()) {
        config.model = Some(v.to_string());
    }

    if let Some(v) = table.get("smallModel").or_else(|| table.get("small_model")).and_then(|v| v.as_str()) {
        config.small_model = Some(v.to_string());
    }

    if let Some(v) = table.get("defaultAgent").or_else(|| table.get("default_agent")).and_then(|v| v.as_str()) {
        config.default_agent = Some(v.to_string());
    }

    if let Some(v) = table.get("username").and_then(|v| v.as_str()) {
        config.username = Some(v.to_string());
    }

    if let Some(v) = table.get("apiKey").or_else(|| table.get("api_key")).and_then(|v| v.as_str()) {
        config.api_key = Some(v.to_string());
    }

    if let Some(v) = table.get("temperature").and_then(|v| v.as_float()) {
        config.temperature = Some(v as f32);
    }

    if let Some(v) = table.get("maxTokens").or_else(|| table.get("max_tokens")).and_then(|v| v.as_integer()) {
        config.max_tokens = Some(v as u32);
    }

    if let Some(share) = table.get("share").and_then(|v| v.as_str()) {
        config.share = match share.to_lowercase().as_str() {
            "manual" => Some(opencode_core::config::ShareMode::Manual),
            "auto" => Some(opencode_core::config::ShareMode::Auto),
            "disabled" => Some(opencode_core::config::ShareMode::Disabled),
            _ => None,
        };
    }

    if let Some(autoupdate) = table.get("autoUpdate").or_else(|| table.get("auto_update")) {
        if let Some(b) = autoupdate.as_bool() {
            config.autoupdate = Some(opencode_core::config::AutoUpdate::Bool(b));
        } else if let Some(s) = autoupdate.as_str() {
            config.autoupdate = Some(opencode_core::config::AutoUpdate::Notify(s.to_string()));
        }
    }

    if let Some(v) = table.get("snapshot").and_then(|v| v.as_bool()) {
        config.snapshot = Some(v);
    }

    if let Some(v) = table.get("autoshare").and_then(|v| v.as_bool()) {
        config.autoshare = Some(v);
    }

    if let Some(disabled) = table.get("disabledProviders").or_else(|| table.get("disabled_providers")).and_then(|v| v.as_array()) {
        config.disabled_providers = Some(
            disabled
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        );
    }

    if let Some(enabled) = table.get("enabledProviders").or_else(|| table.get("enabled_providers")).and_then(|v| v.as_array()) {
        config.enabled_providers = Some(
            enabled
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        );
    }

    if let Some(providers) = table.get("providers").and_then(|v| v.as_table()) {
        let mut provider_map = std::collections::HashMap::new();
        for (name, provider_json) in providers {
            if let Some(provider_obj) = provider_json.as_table() {
                let mut provider_config = opencode_core::config::ProviderConfig {
                    id: Some(name.clone()),
                    ..Default::default()
                };

                let mut options = opencode_core::config::ProviderOptions::default();
                if let Some(api_key) = provider_obj.get("apiKey").or_else(|| provider_obj.get("api_key")).and_then(|v| v.as_str()) {
                    options.api_key = Some(api_key.to_string());
                }
                if let Some(base_url) = provider_obj.get("baseUrl").or_else(|| provider_obj.get("base_url")).and_then(|v| v.as_str()) {
                    options.base_url = Some(base_url.to_string());
                }
                provider_config.options = Some(options);

                provider_map.insert(name.clone(), provider_config);
            }
        }
        config.provider = Some(provider_map);
    }

    Ok(config)
}

fn set_config_value(path: &PathBuf, key: &str, value: &str) -> Result<(), ConfigSetError> {
    let mut config = Config::load(path).unwrap_or_default();

    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() || parts.iter().any(|s| s.is_empty()) {
        return Err(ConfigSetError::InvalidKey(key.to_string()));
    }

    if !is_valid_config_key(parts[0]) {
        return Err(ConfigSetError::InvalidKey(format!(
            "Unknown config key: '{}'. Valid keys include: model, temperature, agent, provider, server, etc.",
            parts[0]
        )));
    }

    let json_value = serde_json::to_value(&config)
        .map_err(|e| ConfigSetError::SerializationError(e.to_string()))?;

    let updated_json = set_nested_value(json_value, &parts, value)?;

    config = serde_json::from_value(updated_json)
        .map_err(|e| ConfigSetError::InvalidValue(format!("Type mismatch: {}", e)))?;

    config.save(path)
        .map_err(|e| ConfigSetError::SaveError(e.to_string()))?;

    let _ = Config::load(path).map(|c| config = c);

    println!("Set {} to {}", key, value);
    Ok(())
}

fn is_valid_config_key(key: &str) -> bool {
    matches!(
        key,
        "schema"
            | "logLevel"
            | "server"
            | "command"
            | "skills"
            | "watcher"
            | "plugin"
            | "snapshot"
            | "share"
            | "autoshare"
            | "autoupdate"
            | "disabledProviders"
            | "enabledProviders"
            | "model"
            | "smallModel"
            | "defaultAgent"
            | "username"
            | "agent"
            | "provider"
            | "mcp"
            | "formatter"
            | "lsp"
            | "instructions"
            | "agentsMd"
            | "permission"
            | "enterprise"
            | "compaction"
            | "experimental"
            | "tui"
            | "apiKey"
            | "temperature"
            | "maxTokens"
    )
}

fn set_nested_value(
    mut json: serde_json::Value,
    path: &[&str],
    value: &str,
) -> Result<serde_json::Value, ConfigSetError> {
    if path.is_empty() {
        return parse_and_validate_value(&json, value).map_err(ConfigSetError::InvalidValue);
    }

    let key = path[0];
    let remaining = &path[1..];

    if remaining.is_empty() {
        let current_value = json.get(key);
        let parsed_value = match current_value {
            Some(current) => parse_value_with_type(current, value)?,
            None => parse_primitive_value(value).map_err(ConfigSetError::InvalidValue)?,
        };
        json[key] = parsed_value;
        return Ok(json);
    }

    if json.get(key).is_none() {
        json[key] = serde_json::Value::Object(serde_json::Map::new());
    }

    let child = json.get_mut(key).ok_or_else(|| ConfigSetError::InvalidKey(key.to_string()))?;

    if !child.is_object() && !child.is_array() {
        return Err(ConfigSetError::InvalidKey(format!(
            "Cannot navigate through non-object value at '{}'",
            key
        )));
    }

    *child = set_nested_value(child.take(), remaining, value)?;
    Ok(json)
}

fn parse_value_with_type(current: &serde_json::Value, value: &str) -> Result<serde_json::Value, String> {
    match current {
        serde_json::Value::Null => parse_primitive_value(value),
        serde_json::Value::Bool(_) => {
            if value.eq_ignore_ascii_case("true") {
                Ok(serde_json::Value::Bool(true))
            } else if value.eq_ignore_ascii_case("false") {
                Ok(serde_json::Value::Bool(false))
            } else {
                Err(format!("Expected boolean, got '{}'", value))
            }
        }
        serde_json::Value::Number(_) => {
            if let Ok(n) = value.parse::<i64>() {
                Ok(serde_json::Value::Number(n.into()))
            } else if let Ok(n) = value.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| format!("Invalid number: {}", value))
            } else {
                Err(format!("Expected number, got '{}'", value))
            }
        }
        serde_json::Value::String(_) => Ok(serde_json::Value::String(value.to_string())),
        serde_json::Value::Array(_) => parse_array_value(value),
        serde_json::Value::Object(_) => Err("Cannot determine type for object".to_string()),
    }
}

fn parse_and_validate_value(current: &serde_json::Value, value: &str) -> Result<serde_json::Value, String> {
    match current {
        serde_json::Value::Null => parse_primitive_value(value),
        serde_json::Value::Bool(_) => parse_value_with_type(current, value),
        serde_json::Value::Number(_) => parse_value_with_type(current, value),
        serde_json::Value::String(_) => Ok(serde_json::Value::String(value.to_string())),
        serde_json::Value::Array(_) => parse_array_value(value),
        serde_json::Value::Object(_) => Err("Cannot set object value directly".to_string()),
    }
}

fn parse_primitive_value(value: &str) -> Result<serde_json::Value, String> {
    if value.eq_ignore_ascii_case("null") {
        return Ok(serde_json::Value::Null);
    }
    if value.eq_ignore_ascii_case("true") {
        return Ok(serde_json::Value::Bool(true));
    }
    if value.eq_ignore_ascii_case("false") {
        return Ok(serde_json::Value::Bool(false));
    }
    if let Ok(n) = value.parse::<i64>() {
        return Ok(serde_json::Value::Number(n.into()));
    }
    if let Ok(n) = value.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(n) {
            return Ok(serde_json::Value::Number(num));
        }
    }
    Ok(serde_json::Value::String(value.to_string()))
}

fn parse_array_value(value: &str) -> Result<serde_json::Value, String> {
    if !value.starts_with('[') || !value.ends_with(']') {
        return Err("Array must be in [item1,item2,...] format".to_string());
    }
    let inner = &value[1..value.len() - 1];
    if inner.trim().is_empty() {
        return Ok(serde_json::Value::Array(vec![]));
    }
    let items: Result<Vec<serde_json::Value>, _> = inner
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            parse_primitive_value(trimmed)
        })
        .collect();
    items.map(serde_json::Value::Array)
}

#[derive(Debug)]
enum ConfigSetError {
    InvalidKey(String),
    InvalidValue(String),
    SerializationError(String),
    SaveError(String),
}

impl std::fmt::Display for ConfigSetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigSetError::InvalidKey(key) => write!(f, "Invalid setting key: {}", key),
            ConfigSetError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ConfigSetError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ConfigSetError::SaveError(msg) => write!(f, "Failed to save config: {}", msg),
        }
    }
}

impl std::error::Error for ConfigSetError {}

impl From<String> for ConfigSetError {
    fn from(s: String) -> Self {
        ConfigSetError::InvalidValue(s)
    }
}
