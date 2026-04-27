use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

pub type EffectResult<T> = Result<T, EffectError>;

pub type EffectFuture<T> = Pin<Box<dyn Future<Output = EffectResult<T>> + Send>>;

pub type EffectRunner<T> = Box<dyn FnOnce() -> EffectFuture<T> + Send>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectError {
    Generic(String),
    Io(String),
    Network(String),
    Validation(String),
    NotFound(String),
    PermissionDenied(String),
    Timeout(String),
    Cancelled(String),
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::Generic(msg) => write!(f, "Generic error: {}", msg),
            EffectError::Io(msg) => write!(f, "IO error: {}", msg),
            EffectError::Network(msg) => write!(f, "Network error: {}", msg),
            EffectError::Validation(msg) => write!(f, "Validation error: {}", msg),
            EffectError::NotFound(msg) => write!(f, "Not found: {}", msg),
            EffectError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            EffectError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            EffectError::Cancelled(msg) => write!(f, "Cancelled: {}", msg),
        }
    }
}

impl std::error::Error for EffectError {}

impl From<std::io::Error> for EffectError {
    fn from(err: std::io::Error) -> Self {
        EffectError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for EffectError {
    fn from(err: serde_json::Error) -> Self {
        EffectError::Generic(err.to_string())
    }
}