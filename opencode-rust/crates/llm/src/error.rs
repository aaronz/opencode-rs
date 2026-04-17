use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LlmError {
    #[error("Rate limit exceeded. Consider reducing request frequency or upgrading your plan.")]
    RateLimitExceeded { retry_after: Option<u64> },
    #[error("Invalid API key. Please check your API key in config or OPENAI_API_KEY environment variable.")]
    InvalidApiKey,
    #[error(
        "Model '{0}' not found. Available models vary by provider - check your configuration."
    )]
    ModelNotFound(String),
    #[error(
        "Request timeout. The server took too long to respond. Check your network connection."
    )]
    RequestTimeout,
    #[error("LLM server error: {0}. This is usually a temporary issue on the provider's side.")]
    ServerError(String),
    #[error(
        "Network error: {0}. Check your internet connection and proxy settings if applicable."
    )]
    NetworkError(String),
    #[error("Request validation failed: {0}. The request parameters may be invalid.")]
    ValidationError(String),
    #[error("Failed to parse response: {0}. The model returned an unexpected format.")]
    Parse(String),
    #[error("Provider error: {0}. Check your provider configuration and API key.")]
    Provider(String),
    #[error(
        "Authentication failed: {0}. Verify your API key is valid and has necessary permissions."
    )]
    Auth(String),
    #[error("Unexpected error: {0}. If this persists, check provider status.")]
    Other(String),
}

impl From<LlmError> for OpenCodeError {
    fn from(err: LlmError) -> Self {
        OpenCodeError::Llm(err.to_string())
    }
}

pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

#[derive(Error, Debug, Clone)]
pub enum RetryConfigError {
    #[error("max_retries must be greater than 0, got {0}")]
    InvalidMaxRetries(u32),
    #[error("initial_delay_ms must be greater than 0, got {0}")]
    InvalidInitialDelay(u64),
    #[error("max_delay_ms must be greater than or equal to initial_delay_ms: {0} < {1}")]
    InvalidMaxDelay(u64, u64),
    #[error("backoff_multiplier must be greater than 1.0, got {0}")]
    InvalidBackoffMultiplier(f64),
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn validate(&self) -> Result<(), RetryConfigError> {
        if self.max_retries == 0 {
            return Err(RetryConfigError::InvalidMaxRetries(self.max_retries));
        }
        if self.initial_delay_ms == 0 {
            return Err(RetryConfigError::InvalidInitialDelay(self.initial_delay_ms));
        }
        if self.max_delay_ms < self.initial_delay_ms {
            return Err(RetryConfigError::InvalidMaxDelay(
                self.max_delay_ms,
                self.initial_delay_ms,
            ));
        }
        if self.backoff_multiplier <= 1.0 {
            return Err(RetryConfigError::InvalidBackoffMultiplier(
                self.backoff_multiplier,
            ));
        }
        Ok(())
    }
}

pub async fn with_retry<F, T>(config: &RetryConfig, f: F) -> Result<T, LlmError>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, LlmError>> + Send>>,
{
    let mut retries = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                retries += 1;
                if retries >= config.max_retries {
                    return Err(err);
                }

                match &err {
                    LlmError::RateLimitExceeded { retry_after } => {
                        if let Some(seconds) = retry_after {
                            tokio::time::sleep(tokio::time::Duration::from_secs(*seconds)).await;
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                        }
                    }
                    LlmError::NetworkError(_)
                    | LlmError::ServerError(_)
                    | LlmError::RequestTimeout => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    }
                    _ => return Err(err),
                }

                delay = (delay as f64 * config.backoff_multiplier) as u64;
                delay = delay.min(config.max_delay_ms);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_llm_error_implements_std_error() {
        fn assert_implements_error<T: std::error::Error>() {}
        assert_implements_error::<LlmError>();
    }

    #[test]
    fn test_retry_config_error_implements_std_error() {
        fn assert_implements_error<T: std::error::Error>() {}
        assert_implements_error::<RetryConfigError>();
    }

    #[test]
    fn test_retry_config_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RetryConfigError>();
    }

    #[test]
    fn test_invalid_max_retries_display() {
        let err = RetryConfigError::InvalidMaxRetries(0);
        let msg = err.to_string();
        assert!(msg.contains("max_retries"));
        assert!(msg.contains("0"));
    }

    #[test]
    fn test_invalid_initial_delay_display() {
        let err = RetryConfigError::InvalidInitialDelay(0);
        let msg = err.to_string();
        assert!(msg.contains("initial_delay_ms"));
        assert!(msg.contains("0"));
    }

    #[test]
    fn test_invalid_max_delay_display() {
        let err = RetryConfigError::InvalidMaxDelay(100, 500);
        let msg = err.to_string();
        assert!(msg.contains("max_delay_ms"));
        assert!(msg.contains("100"));
        assert!(msg.contains("500"));
    }

    #[test]
    fn test_invalid_backoff_multiplier_display() {
        let err = RetryConfigError::InvalidBackoffMultiplier(1.0);
        let msg = err.to_string();
        assert!(msg.contains("backoff_multiplier"));
        assert!(msg.contains("1"));
    }

    #[test]
    fn test_retry_config_validate_success() {
        let config = RetryConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_retry_config_validate_invalid_max_retries() {
        let config = RetryConfig {
            max_retries: 0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate().unwrap_err(),
            RetryConfigError::InvalidMaxRetries(0)
        ));
    }

    #[test]
    fn test_retry_config_validate_invalid_initial_delay() {
        let config = RetryConfig {
            initial_delay_ms: 0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate().unwrap_err(),
            RetryConfigError::InvalidInitialDelay(0)
        ));
    }

    #[test]
    fn test_retry_config_validate_invalid_max_delay() {
        let config = RetryConfig {
            max_delay_ms: 100,
            initial_delay_ms: 500,
            ..Default::default()
        };
        assert!(matches!(
            config.validate().unwrap_err(),
            RetryConfigError::InvalidMaxDelay(100, 500)
        ));
    }

    #[test]
    fn test_retry_config_validate_invalid_backoff_multiplier() {
        let config = RetryConfig {
            backoff_multiplier: 1.0,
            ..Default::default()
        };
        assert!(matches!(
            config.validate().unwrap_err(),
            RetryConfigError::InvalidBackoffMultiplier(1.0)
        ));
    }

    #[test]
    fn test_retry_config_error_clone() {
        let err = RetryConfigError::InvalidMaxRetries(5);
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_llm_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LlmError>();
    }

    #[test]
    fn test_rate_limit_exceeded_display() {
        let err = LlmError::RateLimitExceeded {
            retry_after: Some(60),
        };
        let msg = err.to_string();
        assert!(msg.contains("Rate limit exceeded"));
        assert!(msg.contains("Consider reducing request frequency"));
    }

    #[test]
    fn test_rate_limit_exceeded_without_retry_display() {
        let err = LlmError::RateLimitExceeded { retry_after: None };
        let msg = err.to_string();
        assert!(msg.contains("Rate limit exceeded"));
    }

    #[test]
    fn test_invalid_api_key_display() {
        let err = LlmError::InvalidApiKey;
        let msg = err.to_string();
        assert!(msg.contains("Invalid API key"));
        assert!(msg.contains("OPENAI_API_KEY"));
    }

    #[test]
    fn test_model_not_found_display() {
        let err = LlmError::ModelNotFound("gpt-5".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Model 'gpt-5' not found"));
        assert!(msg.contains("Available models vary by provider"));
    }

    #[test]
    fn test_request_timeout_display() {
        let err = LlmError::RequestTimeout;
        let msg = err.to_string();
        assert!(msg.contains("Request timeout"));
        assert!(msg.contains("network connection"));
    }

    #[test]
    fn test_server_error_display() {
        let err = LlmError::ServerError("Internal error".to_string());
        let msg = err.to_string();
        assert!(msg.contains("LLM server error: Internal error"));
        assert!(msg.contains("temporary issue"));
    }

    #[test]
    fn test_network_error_display() {
        let err = LlmError::NetworkError("Connection refused".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Network error: Connection refused"));
        assert!(msg.contains("proxy settings"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = LlmError::ValidationError("invalid parameter".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Request validation failed: invalid parameter"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = LlmError::Parse("unexpected token".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse response: unexpected token"));
        assert!(msg.contains("unexpected format"));
    }

    #[test]
    fn test_provider_error_display() {
        let err = LlmError::Provider("Rate limit".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Provider error: Rate limit"));
        assert!(msg.contains("provider configuration"));
    }

    #[test]
    fn test_auth_error_display() {
        let err = LlmError::Auth("Token expired".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Authentication failed: Token expired"));
        assert!(msg.contains("necessary permissions"));
    }

    #[test]
    fn test_other_error_display() {
        let err = LlmError::Other("Unknown error".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Unexpected error: Unknown error"));
        assert!(msg.contains("provider status"));
    }

    #[test]
    fn test_error_source_chain() {
        let err = LlmError::Provider("test".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_clone() {
        let err = LlmError::ModelNotFound("test".to_string());
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }

    #[test]
    fn test_error_serialize_deserialize() {
        let err = LlmError::ModelNotFound("gpt-4".to_string());
        let json = serde_json::to_string(&err).unwrap();
        let decoded: LlmError = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.to_string(), err.to_string());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }
}
