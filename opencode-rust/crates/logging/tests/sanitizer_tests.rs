use opencode_logging::sanitizer::Sanitizer;
use opencode_logging::event::SanitizedValue;
use std::collections::HashMap;

#[test]
fn test_sanitize_exact_api_key() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("sk-1234567890");
    let result = sanitizer.sanitize_value("api_key", &value);
    match result {
        SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
        _ => panic!("Expected Redacted variant"),
    }
}

#[test]
fn test_sanitize_exact_password() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("secret123");
    let result = sanitizer.sanitize_value("password", &value);
    match result {
        SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
        _ => panic!("Expected Redacted variant"),
    }
}

#[test]
fn test_sanitize_exact_token() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("token123");
    let result = sanitizer.sanitize_value("token", &value);
    match result {
        SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
        _ => panic!("Expected Redacted variant"),
    }
}

#[test]
fn test_sanitize_exact_authorization() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("Bearer eyJhbGc...");
    let result = sanitizer.sanitize_value("authorization", &value);
    match result {
        SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
        _ => panic!("Expected Redacted variant"),
    }
}

#[test]
fn test_sanitize_suffix_key_pattern() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("key_value_123");
    for key in vec!["my_key", "some_key", "private_key", "access_key"] {
        let result = sanitizer.sanitize_value(key, &value);
        match result {
            SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
            _ => panic!("Expected Redacted variant for key: {}", key),
        }
    }
}

#[test]
fn test_sanitize_suffix_token_pattern() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("token_value_456");
    for key in vec!["my_token", "some_token", "auth_token", "bearer_token"] {
        let result = sanitizer.sanitize_value(key, &value);
        match result {
            SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
            _ => panic!("Expected Redacted variant for key: {}", key),
        }
    }
}

#[test]
fn test_sanitize_suffix_secret_pattern() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!("secret_value_789");
    for key in vec!["my_secret", "some_secret", "app_secret", "client_secret"] {
        let result = sanitizer.sanitize_value(key, &value);
        match result {
            SanitizedValue::Redacted(s) => assert_eq!(s, "[REDACTED]"),
            _ => panic!("Expected Redacted variant for key: {}", key),
        }
    }
}

#[test]
fn test_sanitize_nested_structures_correctly() {
    let sanitizer = Sanitizer::new();
    let value = serde_json::json!({
        "username": "john",
        "api_key": "sk-12345",
        "nested": {
            "password": "secret_pass",
            "deep": {
                "token": "bearer_token_val"
            }
        }
    });
    let result = sanitizer.sanitize_value("params", &value);
    match result {
        SanitizedValue::Nested(map) => {
            match map.get("username") {
                Some(SanitizedValue::Safe(s)) => assert_eq!(s, "john"),
                _ => panic!("Expected Safe variant for username"),
            }
            match map.get("api_key") {
                Some(SanitizedValue::Redacted(s)) => assert_eq!(s, "[REDACTED]"),
                _ => panic!("Expected Redacted variant for api_key"),
            }
            match map.get("nested") {
                Some(SanitizedValue::Nested(nested_map)) => {
                    match nested_map.get("password") {
                        Some(SanitizedValue::Redacted(s)) => assert_eq!(s, "[REDACTED]"),
                        _ => panic!("Expected Redacted variant for nested password"),
                    }
                    match nested_map.get("deep") {
                        Some(SanitizedValue::Nested(deep_map)) => {
                            match deep_map.get("token") {
                                Some(SanitizedValue::Redacted(s)) => assert_eq!(s, "[REDACTED]"),
                                _ => panic!("Expected Redacted variant for deep token"),
                            }
                        }
                        _ => panic!("Expected Nested variant for deep"),
                    }
                }
                _ => panic!("Expected Nested variant for nested"),
            }
        }
        _ => panic!("Expected Nested variant at top level"),
    }
}

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