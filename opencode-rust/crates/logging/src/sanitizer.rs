//! Sensitive data redaction for logging.

use std::collections::HashMap;

use crate::event::SanitizedValue;

const SECRET_PATTERNS: &[&str] = &[
    "api_key",
    "password",
    "token",
    "secret",
    "authorization",
    "credential",
    "private_key",
    "access_key",
];

pub struct Sanitizer;

impl Sanitizer {
    pub fn new() -> Self {
        Self
    }

    pub fn sanitize_value(&self, key: &str, value: &serde_json::Value) -> SanitizedValue {
        if self.is_secret_key(key) {
            return SanitizedValue::redacted(format!("{} detected", key));
        }

        match value {
            serde_json::Value::String(s) => SanitizedValue::Safe(s.clone()),
            serde_json::Value::Object(m) => {
                let mut sanitized_map = HashMap::new();
                for (k, v) in m {
                    sanitized_map.insert(k.clone(), self.sanitize_value(k, v));
                }
                SanitizedValue::Nested(sanitized_map)
            }
            _ => SanitizedValue::Safe(value.to_string()),
        }
    }

    pub fn is_secret_key(&self, key: &str) -> bool {
        let key_lower = key.to_lowercase();
        SECRET_PATTERNS.iter().any(|pattern| {
            key_lower.contains(pattern)
        }) || key_lower.ends_with("_key")
            || key_lower.ends_with("_token")
            || key_lower.ends_with("_secret")
    }

    pub fn sanitize_params(&self, params: &HashMap<String, serde_json::Value>) -> SanitizedValue {
        let mut sanitized = HashMap::new();
        for (key, value) in params {
            sanitized.insert(key.clone(), self.sanitize_value(key, value));
        }
        SanitizedValue::Nested(sanitized)
    }
}

impl Default for Sanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_is_secret_key() {
        let sanitizer = Sanitizer::new();
        assert!(sanitizer.is_secret_key("api_key"));
        assert!(sanitizer.is_secret_key("API_KEY"));
        assert!(sanitizer.is_secret_key("password"));
        assert!(sanitizer.is_secret_key("user_token"));
        assert!(sanitizer.is_secret_key("auth_secret"));
        assert!(!sanitizer.is_secret_key("username"));
        assert!(!sanitizer.is_secret_key("file_path"));
    }

    #[test]
    fn test_sanitize_params() {
        let sanitizer = Sanitizer::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), serde_json::json!("/tmp/test.txt"));
        params.insert("api_key".to_string(), serde_json::json!("sk-12345"));

        let result = sanitizer.sanitize_params(&params);
        assert!(matches!(result, SanitizedValue::Nested(_)));
    }
}