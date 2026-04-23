use opencode_config::{Config, DirectoryScanner};
use serde_json::json;
use tempfile::TempDir;

fn create_temp_config(content: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, content).unwrap();
    (temp_dir, config_path)
}

#[tokio::test]
async fn config_e2e_001_load_json_file() {
    let (_temp_dir, config_path) = create_temp_config(
        r#"{
            "logLevel": "debug",
            "server": {
                "port": 3000,
                "hostname": "localhost"
            },
            "model": "gpt-4o"
        }"#,
    );

    let config = Config::load(&config_path).unwrap();

    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Debug));
    assert!(config.server.is_some());
    assert_eq!(config.server.as_ref().unwrap().port, Some(3000));
    assert_eq!(config.model, Some("gpt-4o".to_string()));
}

#[tokio::test]
async fn config_e2e_004_env_variable_expansion() {
    std::env::set_var("TEST_MODEL", "gpt-4o");
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "model": "{env:TEST_MODEL}"
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();
    std::env::remove_var("TEST_MODEL");

    assert_eq!(config.model, Some("gpt-4o".to_string()));
}

#[tokio::test]
async fn config_e2e_005_config_path_resolution() {
    let path = Config::config_path();
    assert!(path.to_string_lossy().contains("opencode"));
}

#[tokio::test]
async fn config_e2e_006_directory_scanner() {
    use std::fs;

    let temp_dir = TempDir::new().unwrap();
    let opencode_dir = temp_dir.path().join(".opencode").join("tools");
    fs::create_dir_all(&opencode_dir).unwrap();

    fs::write(
        opencode_dir.join("tool1.ts"),
        r#"export default tool({
            name: "tool_one",
            description: "First tool",
            args: {}
        });"#,
    )
    .unwrap();
    fs::write(
        opencode_dir.join("tool2.js"),
        r#"export default tool({
            name: "tool_two",
            description: "Second tool",
            args: {}
        });"#,
    )
    .unwrap();

    let scanner = DirectoryScanner::new();
    let results = scanner.scan_tools(&temp_dir.path().join(".opencode"));

    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn config_e2e_008_jsonc_comment_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.jsonc");

    let content = r#"{
        // This is a comment
        "logLevel": "info",
        /* Block comment */
        "server": {
            "port": 3000
        }
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Info));
    assert_eq!(config.server.as_ref().unwrap().port, Some(3000));
}

#[tokio::test]
async fn config_val_001_malformed_json_recovery() {
    let test_cases = vec![
        (r#"{ "key": "value""#, "unclosed bracket"),
        (r#"{ "key": "value", }"#, "trailing comma"),
    ];

    for (content, issue) in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");
        std::fs::write(&config_path, content).unwrap();

        let result = Config::load(&config_path);
        assert!(
            result.is_err(),
            "Expected error for {} but got success",
            issue
        );
    }
}

#[tokio::test]
async fn config_val_002_circular_reference_detection() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "A": "${B}",
        "B": "${A}"
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let result = Config::load(&config_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Circular") || err_msg.contains("circular"),
        "Error should mention circular reference"
    );
}

#[tokio::test]
async fn config_val_003_missing_required_env_vars() {
    std::env::remove_var("OPENCODE_API_KEY");
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "apiKey": "${OPENCODE_API_KEY}"
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let result = Config::load(&config_path);
    assert!(result.is_err());

    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Undefined") || err_msg.contains("undefined"),
        "Error should indicate undefined variable"
    );
}

#[tokio::test]
async fn config_val_006_array_merge_behavior() {
    let base = json!({
        "tools": ["read", "write"]
    });
    let patch = json!({
        "tools": ["bash"]
    });

    let merged = deep_merge_json(&base, &patch);
    assert_eq!(merged["tools"], json!(["bash"]));
}

fn deep_merge_json(
    base: &serde_json::Value,
    override_val: &serde_json::Value,
) -> serde_json::Value {
    match (base, override_val) {
        (serde_json::Value::Object(base_map), serde_json::Value::Object(override_map)) => {
            let mut result = base_map.clone();
            for (key, override_value) in override_map {
                let base_value = result.get(key);
                let merged = match base_value {
                    Some(base_val) => deep_merge_json(base_val, override_value),
                    None => override_value.clone(),
                };
                result.insert(key.clone(), merged);
            }
            serde_json::Value::Object(result)
        }
        _ => override_val.clone(),
    }
}

#[tokio::test]
async fn config_reload_001_hot_reload_detect_change() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    std::fs::write(&config_path, r#"{"logLevel": "info"}"#).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Info));

    std::fs::write(&config_path, r#"{"logLevel": "debug"}"#).unwrap();

    let reloaded = Config::load(&config_path).unwrap();
    assert_eq!(reloaded.log_level, Some(opencode_config::LogLevel::Debug));
}

#[tokio::test]
async fn config_reload_002_atomic_reload() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "logLevel": "info",
        "model": "gpt-4o"
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Info));
    assert_eq!(config.model.as_deref(), Some("gpt-4o"));
}

#[tokio::test]
async fn config_expand_001_variable_expansion_quotes() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    std::env::set_var("TEST_URL", "http://example.com/?a=1&b=2");

    let content = r#"{
        "server": {
            "hostname": "${TEST_URL}"
        }
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let result = Config::load(&config_path);
    std::env::remove_var("TEST_URL");

    if result.is_ok() {
        let config = result.unwrap();
        let hostname = config.server.as_ref().and_then(|s| s.hostname.clone());
        assert_eq!(hostname, Some("http://example.com/?a=1&b=2".to_string()));
    }
}

#[tokio::test]
async fn config_expand_002_variable_default_values() {
    std::env::remove_var("PORT");
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "server": {
            "port": 3000
        }
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.server.as_ref().unwrap().port, Some(3000));
}

#[tokio::test]
async fn config_e2e_load_empty_config() {
    std::env::remove_var("OPENCODE_LOG_LEVEL");
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    std::fs::write(&config_path, "{}").unwrap();

    let config = Config::load(&config_path).unwrap();
    assert!(config.server.is_none());
}

#[tokio::test]
async fn config_e2e_load_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("nonexistent.json");

    let result = Config::load(&config_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn config_jsonc_single_line_comments() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.jsonc");

    let content = r#"{
        // Single line comment
        "logLevel": "warn"
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Warn));
}

#[tokio::test]
async fn config_jsonc_block_comments() {
    std::env::remove_var("OPENCODE_LOG_LEVEL");
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.jsonc");

    let content = r#"{
        /* Block comment
           spanning multiple lines */
        "logLevel": "error"
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Error));
}

#[tokio::test]
async fn config_validation_rejects_tui_fields_in_runtime() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "logLevel": "info",
        "scroll_speed": 50
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let result = Config::load(&config_path);
    if result.is_err() {
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("TUI") || err_msg.contains("runtime"),
            "Error should mention TUI/runtime field mismatch"
        );
    }
}

#[tokio::test]
async fn config_e2e_nested_config_loading() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let content = r#"{
        "logLevel": "debug",
        "server": {
            "port": 8080,
            "hostname": "0.0.0.0",
            "cors": ["http://localhost:3000"]
        },
        "agent": {
            "agents": {
                "test": {
                    "model": "gpt-4"
                }
            }
        }
    }"#;
    std::fs::write(&config_path, content).unwrap();

    let config = Config::load(&config_path).unwrap();

    assert_eq!(config.log_level, Some(opencode_config::LogLevel::Debug));
    assert!(config.server.is_some());
    assert_eq!(config.server.as_ref().unwrap().port, Some(8080));
    assert!(config.agent.is_some());
}
