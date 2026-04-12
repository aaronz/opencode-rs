use opencode_core::config::Config;

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
