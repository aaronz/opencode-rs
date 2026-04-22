use opencode_logging::sanitizer::Sanitizer;
use opencode_logging::event::SanitizedValue;
use std::collections::HashMap;

#[test]
fn test_sanitize_normal_value() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("hello");
    let result = sanitizer.sanitize_value("username", &value);
    match result {
        SanitizedValue::Safe(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected Safe variant"),
    }
}

#[test]
fn test_sanitize_api_key() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("sk-1234567890");
    let result = sanitizer.sanitize_value("api_key", &value);
    assert!(matches!(result, SanitizedValue::Redacted(_)));
}

#[test]
fn test_sanitize_password() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("secret123");
    let result = sanitizer.sanitize_value("user_password", &value);
    assert!(matches!(result, SanitizedValue::Redacted(_)));
}

#[test]
fn test_sanitize_token() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("Bearer token123");
    let result = sanitizer.sanitize_value("authorization_token", &value);
    assert!(matches!(result, SanitizedValue::Redacted(_)));
}

#[test]
fn test_sanitize_secret() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("my_secret_value");
    let result = sanitizer.sanitize_value("app_secret", &value);
    assert!(matches!(result, SanitizedValue::Redacted(_)));
}

#[test]
fn test_is_secret_key_patterns() {
    let sanitizer = Sanitizer::new();
    assert!(sanitizer.is_secret_key("api_key"));
    assert!(sanitizer.is_secret_key("API_KEY"));
    assert!(sanitizer.is_secret_key("password"));
    assert!(sanitizer.is_secret_key("user_token"));
    assert!(sanitizer.is_secret_key("auth_secret"));
    assert!(sanitizer.is_secret_key("private_key"));
    assert!(sanitizer.is_secret_key("access_key"));
    assert!(sanitizer.is_secret_key("some_key"));
    assert!(sanitizer.is_secret_key("some_token"));
    assert!(sanitizer.is_secret_key("some_secret"));
    assert!(!sanitizer.is_secret_key("username"));
    assert!(!sanitizer.is_secret_key("file_path"));
    assert!(!sanitizer.is_secret_key("email"));
}

#[test]
fn test_sanitize_nested_object() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!({
        "username": "john",
        "api_key": "sk-12345"
    });
    let result = sanitizer.sanitize_value("params", &value);
    assert!(matches!(result, SanitizedValue::Nested(_)));
}

#[test]
fn test_sanitize_params() {
    let sanitizer = Sanitizer::new();
    let mut params = HashMap::new();
    params.insert("file_path".to_string(), serde_json::json!("/tmp/test.txt"));
    params.insert("api_key".to_string(), serde_json::json!("sk-12345"));
    params.insert("content".to_string(), serde_json::json!("some text content"));

    let result = sanitizer.sanitize_params(&params);
    assert!(matches!(result, SanitizedValue::Nested(_)));
}