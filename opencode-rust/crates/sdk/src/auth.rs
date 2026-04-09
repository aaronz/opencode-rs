//! Authentication module for OpenCode SDK.
//!
//! Supports API key authentication for the OpenCode REST API.

use serde::{Deserialize, Serialize};

/// API Key authentication credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAuth {
    /// The API key used for authentication.
    /// Typically starts with `sk-` for OpenAI-compatible APIs.
    key: String,

    /// Optional API key prefix for display purposes.
    prefix: Option<String>,
}

impl ApiKeyAuth {
    /// Creates a new API key authentication with the given key.
    pub fn new(key: impl Into<String>) -> Self {
        let key_str = key.into();
        let prefix = if key_str.starts_with("sk-") {
            Some("sk-***".to_string())
        } else {
            None
        };

        Self {
            key: key_str,
            prefix,
        }
    }

    /// Returns the API key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns a masked version of the API key for logging.
    pub fn masked_key(&self) -> String {
        self.prefix.clone().unwrap_or_else(|| "***".to_string())
    }

    /// Returns the Authorization header value.
    pub fn authorization_header(&self) -> String {
        format!("Bearer {}", self.key)
    }
}

impl Default for ApiKeyAuth {
    fn default() -> Self {
        Self {
            key: std::env::var("OPENCODE_API_KEY").unwrap_or_default(),
            prefix: Some("sk-***".to_string()),
        }
    }
}

impl std::fmt::Display for ApiKeyAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKeyAuth({})", self.masked_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_auth_creation() {
        let auth = ApiKeyAuth::new("sk-1234567890abcdef");
        assert_eq!(auth.key(), "sk-1234567890abcdef");
        assert_eq!(auth.masked_key(), "sk-***");
    }

    #[test]
    fn test_api_key_auth_non_sk_prefix() {
        let auth = ApiKeyAuth::new("my-secret-key");
        assert_eq!(auth.key(), "my-secret-key");
        assert_eq!(auth.masked_key(), "***");
    }

    #[test]
    fn test_authorization_header() {
        let auth = ApiKeyAuth::new("sk-test");
        assert_eq!(auth.authorization_header(), "Bearer sk-test");
    }

    #[test]
    fn test_display() {
        let auth = ApiKeyAuth::new("sk-12345");
        assert_eq!(format!("{}", auth), "ApiKeyAuth(sk-***)");
    }
}
