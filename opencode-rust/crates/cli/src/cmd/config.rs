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
}

pub(crate) fn run(args: ConfigArgs) {
    if args.migrate {
        eprintln!("TOML configuration format is no longer supported.");
        eprintln!("Please manually convert your config.toml to config.jsonc format.");
        std::process::exit(1);
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

fn set_config_value(path: &PathBuf, key: &str, value: &str) -> Result<(), ConfigSetError> {
    let mut config = Config::load(path).unwrap_or_default();

    let parts: Vec<&str> = key.split('.').collect();
    if parts.is_empty() || parts.iter().any(|s| s.is_empty()) {
        return Err(ConfigSetError::InvalidKey(key.to_string()));
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
