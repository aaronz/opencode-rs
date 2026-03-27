use serde::{Deserialize, Serialize};
use opencode_core::OpenCodeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmError {
    RateLimitExceeded { retry_after: Option<u64> },
    InvalidApiKey,
    ModelNotFound(String),
    RequestTimeout,
    ServerError(String),
    NetworkError(String),
    ValidationError(String),
    Parse(String),
    Provider(String),
    Auth(String),
    Other(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::RateLimitExceeded { retry_after } => {
                write!(f, "Rate limit exceeded")?;
                if let Some(seconds) = retry_after {
                    write!(f, ". Retry after {} seconds", seconds)?;
                }
                Ok(())
            }
            LlmError::InvalidApiKey => write!(f, "Invalid API key"),
            LlmError::ModelNotFound(model) => write!(f, "Model not found: {}", model),
            LlmError::RequestTimeout => write!(f, "Request timeout"),
            LlmError::ServerError(msg) => write!(f, "Server error: {}", msg),
            LlmError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            LlmError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            LlmError::Parse(msg) => write!(f, "Parse error: {}", msg),
            LlmError::Provider(msg) => write!(f, "Provider error: {}", msg),
            LlmError::Auth(msg) => write!(f, "Auth error: {}", msg),
            LlmError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
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
                    LlmError::NetworkError(_) | LlmError::ServerError(_) | LlmError::RequestTimeout => {
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
