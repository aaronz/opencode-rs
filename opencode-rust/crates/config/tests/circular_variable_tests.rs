use opencode_config::Config;

#[test]
fn test_variable_expansion_circular_reference() {
    let mut value = serde_json::json!({
        "a": "${b}",
        "b": "${a}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
    assert!(err_msg.contains("a") || err_msg.contains("b"));
}

#[test]
fn test_variable_expansion_self_reference() {
    let mut value = serde_json::json!({
        "name": "${name}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_variable_expansion_indirect_circular() {
    let mut value = serde_json::json!({
        "a": "${b}",
        "b": "${c}",
        "c": "${a}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_variable_expansion_basic() {
    let mut value = serde_json::json!({
        "model": "openai/gpt-4o",
        "api_key": "${model}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["api_key"], "openai/gpt-4o");
}

#[test]
fn test_variable_expansion_nested() {
    let mut value = serde_json::json!({
        "a": "first",
        "b": "${a}",
        "c": "${b}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["c"], "first");
}

#[test]
fn test_variable_expansion_undefined_variable() {
    let mut value = serde_json::json!({
        "key": "${undefined_var}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Undefined"));
    assert!(err_msg.contains("undefined_var"));
}

#[test]
fn test_variable_expansion_multiple_in_string() {
    let mut value = serde_json::json!({
        "host": "example.com",
        "url": "https://${host}/api"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["url"], "https://example.com/api");
}

#[test]
fn test_variable_expansion_in_array() {
    let mut value = serde_json::json!({
        "base": "value",
        "items": ["${base}", "other", "${base}"]
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["items"][0], "value");
    assert_eq!(value["items"][1], "other");
    assert_eq!(value["items"][2], "value");
}

#[test]
fn test_variable_expansion_in_nested_object() {
    let mut value = serde_json::json!({
        "outer": {
            "inner": {
                "ref": "${outer.value}"
            },
            "value": "deep"
        }
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["outer"]["inner"]["ref"], "deep");
}

#[test]
fn test_variable_expansion_non_string_no_change() {
    let mut value = serde_json::json!({
        "number": 42,
        "bool": true,
        "null": null
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["number"], 42);
    assert_eq!(value["bool"], true);
    assert_eq!(value["null"], serde_json::Value::Null);
}

#[test]
fn test_variable_expansion_partial_string_preserved() {
    let mut value = serde_json::json!({
        "model": "openai/gpt-4o",
        "prefix": "before-${model}-after"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_ok());
    assert_eq!(value["prefix"], "before-openai/gpt-4o-after");
}

#[test]
fn test_circular_error_message_shows_chain() {
    let mut value = serde_json::json!({
        "x": "${y}",
        "y": "${z}",
        "z": "${x}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
    assert!(err_msg.contains("x") && err_msg.contains("y") && err_msg.contains("z"));
}

#[test]
fn test_circular_error_message_shows_variables_in_cycle() {
    let mut value = serde_json::json!({
        "var_a": "${var_b}",
        "var_b": "${var_a}"
    });
    let result = Config::expand_variables(&mut value);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
    assert!(err_msg.contains("var_a") && err_msg.contains("var_b"));
}

#[test]
fn test_env_variable_circular_self_reference() {
    std::env::set_var("CIRCULAR_SELF", "{env:CIRCULAR_SELF}");
    let result = Config::substitute_variables("{env:CIRCULAR_SELF}", None);
    std::env::remove_var("CIRCULAR_SELF");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
    assert!(err_msg.contains("CIRCULAR_SELF"));
}

#[test]
fn test_env_variable_circular_indirect() {
    std::env::set_var("ENV_A", "{env:ENV_B}");
    std::env::set_var("ENV_B", "{env:ENV_A}");
    let result = Config::substitute_variables("{env:ENV_A}", None);
    std::env::remove_var("ENV_A");
    std::env::remove_var("ENV_B");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_env_variable_circular_chain() {
    std::env::set_var("CHAIN_A", "{env:CHAIN_B}");
    std::env::set_var("CHAIN_B", "{env:CHAIN_C}");
    std::env::set_var("CHAIN_C", "{env:CHAIN_A}");
    let result = Config::substitute_variables("{env:CHAIN_A}", None);
    std::env::remove_var("CHAIN_A");
    std::env::remove_var("CHAIN_B");
    std::env::remove_var("CHAIN_C");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
    assert!(
        err_msg.contains("CHAIN_A") && err_msg.contains("CHAIN_B") && err_msg.contains("CHAIN_C")
    );
}

#[test]
fn test_env_variable_expansion_normal() {
    std::env::set_var("NORMAL_VAR", "normal_value");
    let result = Config::substitute_variables("{env:NORMAL_VAR}", None);
    std::env::remove_var("NORMAL_VAR");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "normal_value");
}

#[test]
fn test_file_variable_circular_self_reference() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("self_ref.txt");
    std::fs::write(&file_path, "{file:./self_ref.txt}").unwrap();
    let result = Config::substitute_variables("{file:./self_ref.txt}", Some(temp_dir.path()));
    std::fs::remove_file(&file_path).ok();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_file_variable_circular_indirect() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_a = temp_dir.path().join("a.txt");
    let file_b = temp_dir.path().join("b.txt");
    std::fs::write(&file_a, "{file:./b.txt}").unwrap();
    std::fs::write(&file_b, "{file:./a.txt}").unwrap();
    let result = Config::substitute_variables("{file:./a.txt}", Some(temp_dir.path()));
    std::fs::remove_file(&file_a).ok();
    std::fs::remove_file(&file_b).ok();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_file_variable_circular_chain() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_x = temp_dir.path().join("x.txt");
    let file_y = temp_dir.path().join("y.txt");
    let file_z = temp_dir.path().join("z.txt");
    std::fs::write(&file_x, "{file:./y.txt}").unwrap();
    std::fs::write(&file_y, "{file:./z.txt}").unwrap();
    std::fs::write(&file_z, "{file:./x.txt}").unwrap();
    let result = Config::substitute_variables("{file:./x.txt}", Some(temp_dir.path()));
    std::fs::remove_file(&file_x).ok();
    std::fs::remove_file(&file_y).ok();
    std::fs::remove_file(&file_z).ok();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}

#[test]
fn test_file_variable_expansion_normal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("content.txt");
    std::fs::write(&file_path, "file_content").unwrap();
    let result = Config::substitute_variables("{file:./content.txt}", Some(temp_dir.path()));
    std::fs::remove_file(&file_path).ok();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "file_content");
}

#[test]
fn test_nested_env_in_file_circular() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("nested.txt");
    std::env::set_var("NESTED_OUTER", "{file:./nested.txt}");
    std::fs::write(&file_path, "{env:NESTED_OUTER}").unwrap();
    let result = Config::substitute_variables("{env:NESTED_OUTER}", Some(temp_dir.path()));
    std::env::remove_var("NESTED_OUTER");
    std::fs::remove_file(&file_path).ok();
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Circular"));
}
