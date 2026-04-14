use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;
use thiserror::Error;

use super::{TuiConfig, ValidationError, ValidationSeverity};

pub fn validate_tui_schema(value: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(obj) = value.as_object() else {
        errors.push("$: expected object".to_string());
        return errors;
    };

    for (key, val) in obj {
        match key.as_str() {
            "scroll_speed" | "scrollSpeed" => {
                if !val.is_number() {
                    errors.push(format!("$.{}: expected number", key));
                }
            }
            "scroll_acceleration" | "scrollAcceleration" => {
                if !val.is_object() {
                    errors.push(format!("$.{}: expected object", key));
                } else if let Some(accel) = val.as_object() {
                    if let Some(enabled) = accel.get("enabled") {
                        if !enabled.is_boolean() {
                            errors.push(format!("$.{}.enabled: expected boolean", key));
                        }
                    }
                    if let Some(speed) = accel.get("speed") {
                        if !speed.is_number() {
                            errors.push(format!("$.{}.speed: expected number", key));
                        }
                    }
                }
            }
            "diff_style" | "diffStyle" => {
                if !val.is_string() {
                    errors.push(format!("$.{}: expected string", key));
                }
            }
            "theme" => {
                if !val.is_object() {
                    errors.push("$.theme: expected object".to_string());
                }
            }
            "keybinds" => {
                if !val.is_object() {
                    errors.push("$.keybinds: expected object".to_string());
                } else if let Some(keybinds) = val.as_object() {
                    for (action, binding) in keybinds {
                        if !binding.is_string() {
                            errors.push(format!("$.keybinds.{}: expected string", action));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    errors
}

#[derive(Debug, Clone)]
pub struct SchemaValidationError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct SchemaValidationResult {
    pub valid: bool,
    pub errors: Vec<SchemaValidationError>,
}

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("Invalid schema URL: {0}")]
    InvalidUrl(String),
    #[error("HTTP client error: {0}")]
    HttpClient(String),
    #[error("HTTP request failed: {0}")]
    HttpRequest(String),
    #[error("Schema parse failed: {0}")]
    Parse(String),
}

pub fn fetch_schema(url: &str) -> Result<serde_json::Value, SchemaError> {
    let parsed = reqwest::Url::parse(url).map_err(|e| SchemaError::InvalidUrl(e.to_string()))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(SchemaError::InvalidUrl(
            "only http/https schema URLs are supported".to_string(),
        ));
    }

    let url_clone = parsed.clone();
    std::thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| SchemaError::HttpClient(e.to_string()))?;

        let response = client
            .get(url_clone)
            .send()
            .map_err(|e| SchemaError::HttpRequest(e.to_string()))?
            .error_for_status()
            .map_err(|e| SchemaError::HttpRequest(e.to_string()))?;

        response
            .json::<Value>()
            .map_err(|e| SchemaError::Parse(e.to_string()))
    })
    .join()
    .map_err(|e| SchemaError::HttpClient(format!("Thread panicked: {:?}", e)))?
}

pub fn cache_schema(url: &str, schema: &Value) {
    let cache_dir = schema_cache_dir();
    if let Err(err) = std::fs::create_dir_all(&cache_dir) {
        tracing::warn!(
            "failed to create schema cache directory {:?}: {}",
            cache_dir,
            err
        );
        return;
    }

    let cache_path = schema_cache_path(url);
    let payload = match serde_json::to_vec_pretty(schema) {
        Ok(data) => data,
        Err(err) => {
            tracing::warn!("failed to serialize schema for cache: {}", err);
            return;
        }
    };

    if let Err(err) = std::fs::write(&cache_path, payload) {
        tracing::warn!(
            "failed to write schema cache file {:?}: {}",
            cache_path,
            err
        );
    }
}

pub fn load_cached_schema(url: &str) -> Option<Value> {
    let cache_path = schema_cache_path(url);
    let data = std::fs::read_to_string(&cache_path).ok()?;
    match serde_json::from_str(&data) {
        Ok(value) => Some(value),
        Err(err) => {
            tracing::warn!("failed to parse cached schema {:?}: {}", cache_path, err);
            None
        }
    }
}

pub fn get_builtin_schema() -> Value {
    serde_json::from_str(include_str!("builtin_config.schema.json")).unwrap_or_else(|err| {
        tracing::warn!("failed to parse built-in schema: {}", err);
        serde_json::json!({"type": "object"})
    })
}

pub fn validate_json_schema(config: &Value, schema_url: &str) -> super::ValidationResult {
    let detailed = validate_json_schema_detailed(config, schema_url);
    let errors = detailed
        .errors
        .into_iter()
        .map(|e| super::ValidationError {
            field: e.path,
            message: e.message,
            severity: super::ValidationSeverity::Error,
        })
        .collect::<Vec<_>>();

    super::ValidationResult {
        valid: detailed.valid,
        errors,
    }
}

fn validate_json_schema_detailed(config: &Value, schema_url: &str) -> SchemaValidationResult {
    let resolved_url = if schema_url.trim().is_empty() {
        config
            .get("$schema")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        Some(schema_url.trim().to_string())
    };

    let Some(schema_url) = resolved_url else {
        return SchemaValidationResult {
            valid: true,
            errors: Vec::new(),
        };
    };

    let schema = match fetch_schema(&schema_url) {
        Ok(schema) => {
            cache_schema(&schema_url, &schema);
            schema
        }
        Err(err) => {
            tracing::warn!("failed to fetch remote schema {}: {}", schema_url, err);
            if let Some(cached) = load_cached_schema(&schema_url) {
                cached
            } else {
                tracing::warn!("using built-in schema fallback for {}", schema_url);
                get_builtin_schema()
            }
        }
    };

    let config_owned = config.clone();
    let schema_owned = schema.clone();
    std::thread::spawn(move || {
        let validator = match jsonschema::validator_for(&schema_owned) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!("failed to compile JSON schema ({}): {}", schema_url, err);
                return SchemaValidationResult {
                    valid: true,
                    errors: Vec::new(),
                };
            }
        };

        let errors = validator
            .iter_errors(&config_owned)
            .map(|error| SchemaValidationError {
                path: pointer_to_path(&error.instance_path.to_string()),
                message: error.to_string(),
            })
            .collect::<Vec<_>>();

        SchemaValidationResult {
            valid: errors.is_empty(),
            errors,
        }
    })
    .join()
    .map_err(|e| {
        tracing::warn!("schema validation panicked: {:?}", e);
        SchemaValidationResult {
            valid: true,
            errors: Vec::new(),
        }
    })
    .unwrap_or_else(|_| SchemaValidationResult {
        valid: true,
        errors: Vec::new(),
    })
}

fn pointer_to_path(pointer: &str) -> String {
    if pointer.is_empty() {
        return "$".to_string();
    }

    pointer
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>()
        .join(".")
}

fn schema_cache_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("OPENCODE_SCHEMA_CACHE_DIR") {
        return PathBuf::from(override_dir);
    }

    if let Some(home) = dirs::home_dir() {
        return home.join(".config").join("opencode").join("schemas");
    }

    PathBuf::from(".opencode/schemas")
}

fn schema_cache_path(url: &str) -> PathBuf {
    schema_cache_dir().join(cache_file_name(url))
}

fn cache_file_name(url: &str) -> String {
    let candidate = reqwest::Url::parse(url)
        .ok()
        .and_then(|u| {
            let file_component = Path::new(u.path())
                .file_name()
                .and_then(|part| part.to_str())
                .filter(|part| !part.is_empty())
                .map(ToOwned::to_owned);
            file_component.or_else(|| u.host_str().map(ToOwned::to_owned))
        })
        .unwrap_or_else(|| "schema".to_string());

    let mut sanitized = candidate
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    if !sanitized.ends_with(".json") {
        sanitized.push_str(".json");
    }
    sanitized
}

#[allow(dead_code)]
pub fn get_official_schema_url() -> &'static str {
    "https://opencode.ai/config.json"
}

#[allow(dead_code)]
pub fn validate_tui_config(tui_config: &TuiConfig) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if let Some(scroll_speed) = &tui_config.scroll_speed {
        if *scroll_speed == 0 {
            errors.push(ValidationError {
                field: "tui.scrollSpeed".to_string(),
                message: "scrollSpeed must be greater than 0".to_string(),
                severity: ValidationSeverity::Error,
            });
        }
        if *scroll_speed > 1000 {
            errors.push(ValidationError {
                field: "tui.scrollSpeed".to_string(),
                message: "scrollSpeed seems excessively high (max recommended: 1000)".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }
    }

    if let Some(scroll_accel) = &tui_config.scroll_acceleration {
        if let Some(speed) = &scroll_accel.speed {
            if *speed < 0.0 {
                errors.push(ValidationError {
                    field: "tui.scrollAcceleration.speed".to_string(),
                    message: "scrollAcceleration.speed must be non-negative".to_string(),
                    severity: ValidationSeverity::Error,
                });
            }
            if *speed > 10.0 {
                errors.push(ValidationError {
                    field: "tui.scrollAcceleration.speed".to_string(),
                    message:
                        "scrollAcceleration.speed seems excessively high (max recommended: 10.0)"
                            .to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn fetch_schema_network_failure_is_graceful() {
        let result = fetch_schema("http://127.0.0.1:9/config.schema.json");
        assert!(result.is_err());
    }

    #[test]
    fn cache_round_trip_works() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("OPENCODE_SCHEMA_CACHE_DIR", temp_dir.path());

        let url = "https://example.com/config.schema.json";
        let schema = serde_json::json!({"type": "object", "properties": {"x": {"type": "number"}}});
        cache_schema(url, &schema);
        let loaded = load_cached_schema(url);

        assert_eq!(loaded, Some(schema));
        std::env::remove_var("OPENCODE_SCHEMA_CACHE_DIR");
    }

    #[test]
    fn validation_errors_include_field_paths() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("OPENCODE_SCHEMA_CACHE_DIR", temp_dir.path());

        let url = "http://127.0.0.1:9/config.schema.json";
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "object",
                    "properties": {
                        "port": {"type": "integer", "minimum": 1}
                    }
                }
            }
        });
        cache_schema(url, &schema);

        let config = serde_json::json!({"$schema": url, "server": {"port": 0}});
        let result = validate_json_schema(&config, "");

        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.field.contains("server.port")));

        std::env::remove_var("OPENCODE_SCHEMA_CACHE_DIR");
    }

    #[test]
    fn offline_mode_falls_back_cache_then_builtin() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("OPENCODE_SCHEMA_CACHE_DIR", temp_dir.path());

        let url = "http://127.0.0.1:9/config.schema.json";
        let cached_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "object",
                    "properties": {
                        "port": {"type": "integer", "minimum": 1}
                    }
                }
            }
        });
        cache_schema(url, &cached_schema);

        let cached_config = serde_json::json!({"$schema": url, "server": {"port": 0}});
        let cached_result = validate_json_schema(&cached_config, "");
        assert!(!cached_result.valid);

        let cached_path = schema_cache_path(url);
        std::fs::remove_file(cached_path).unwrap();

        let builtin_config = serde_json::json!({"$schema": url, "temperature": 4.2});
        let builtin_result = validate_json_schema(&builtin_config, "");
        assert!(!builtin_result.valid);

        std::env::remove_var("OPENCODE_SCHEMA_CACHE_DIR");
    }

    #[test]
    fn invalid_schema_url_does_not_panic_or_fail_loading_semantics() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("OPENCODE_SCHEMA_CACHE_DIR", temp_dir.path());

        let config = serde_json::json!({"$schema": "not a valid url", "temperature": 0.7});
        let result = validate_json_schema(&config, "");
        assert!(result.valid);

        std::env::remove_var("OPENCODE_SCHEMA_CACHE_DIR");
    }

    #[test]
    fn validate_tui_schema_reports_type_errors() {
        let value = serde_json::json!({
            "scroll_speed": "fast",
            "scroll_acceleration": { "enabled": "yes", "speed": "1.2" },
            "keybinds": { "commands": 1 }
        });

        let errors = validate_tui_schema(&value);
        assert!(errors.iter().any(|e| e.contains("$.scroll_speed")));
        assert!(errors
            .iter()
            .any(|e| e.contains("$.scroll_acceleration.enabled")));
        assert!(errors.iter().any(|e| e.contains("$.keybinds.commands")));
    }
}
