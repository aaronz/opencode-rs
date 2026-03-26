use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub type EffectResult<T> = Result<T, EffectError>;

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

pub struct Effect<T> {
    run: Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = EffectResult<T>> + Send>> + Send>,
}

impl<T: Send + 'static> Effect<T> {
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = EffectResult<T>> + Send + 'static,
    {
        Self {
            run: Box::new(move || Box::pin(f())),
        }
    }

    pub async fn run(self) -> EffectResult<T> {
        (self.run)().await
    }

    pub fn map<U: Send + 'static>(self, f: impl FnOnce(T) -> U + Send + 'static) -> Effect<U> {
        Effect::new(move || async move {
            match self.run().await {
                Ok(val) => Ok(f(val)),
                Err(e) => Err(e),
            }
        })
    }

    pub fn and_then<U: Send + 'static>(self, f: impl FnOnce(T) -> Effect<U> + Send + 'static) -> Effect<U> {
        Effect::new(move || async move {
            match self.run().await {
                Ok(val) => f(val).run().await,
                Err(e) => Err(e),
            }
        })
    }

    pub fn success(value: T) -> Self {
        Effect::new(move || async move { Ok(value) })
    }

    pub fn failure(error: EffectError) -> Self {
        Effect::new(move || async move { Err(error) })
    }
}

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
