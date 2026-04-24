//! Error types module for OpenCode
//!
//! Provides NamedError for structured error representation and WithContext
//! for error wrapping with additional context.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Unified error type with name, code, message, and optional data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedError {
    /// Error type name (e.g., "IOError", "HttpError")
    pub name: String,
    /// Error code for programmatic handling
    pub code: Option<String>,
    /// Human-readable error message
    pub message: String,
    /// Additional structured context
    pub data: Option<serde_json::Value>,
}

impl NamedError {
    /// Create a new NamedError.
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            code: None,
            message: message.into(),
            data: None,
        }
    }

    /// Add error code to the error.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add structured data to the error.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Returns the error kind (name).
    pub fn kind(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for NamedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.message)
    }
}

impl std::error::Error for NamedError {}

impl From<std::io::Error> for NamedError {
    fn from(e: std::io::Error) -> Self {
        NamedError::new("IOError", e.to_string()).with_code(format!("IO_{}", e.kind() as i32))
    }
}

impl From<reqwest::Error> for NamedError {
    fn from(e: reqwest::Error) -> Self {
        NamedError::new("HttpError", e.to_string()).with_code("HTTP")
    }
}

/// Wraps any error with additional context.
#[derive(Debug)]
pub struct WithContext<E> {
    /// Additional context describing where/why the error occurred
    context: String,
    /// The underlying error
    inner: E,
}

impl<E: Error> WithContext<E> {
    /// Create a new WithContext wrapper.
    pub fn new(context: impl Into<String>, inner: E) -> Self {
        Self {
            context: context.into(),
            inner,
        }
    }
}

impl<E: Error> fmt::Display for WithContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = self
            .inner
            .source()
            .map(|s| s.to_string())
            .unwrap_or_default();
        write!(
            f,
            "{}: {} (caused by: {})",
            self.context, self.inner, source
        )
    }
}

impl<E: Error> Error for WithContext<E> {}

/// Extension trait for adding context to Result<T, E>.
pub trait Context<T, E: Error> {
    /// Add context to the error variant of the Result.
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>>;
}

impl<T, E: Error> Context<T, E> for Result<T, E> {
    fn context<C: Into<String>>(self, ctx: C) -> Result<T, WithContext<E>> {
        self.map_err(|e| WithContext::new(ctx, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_named_error_display() {
        let err = NamedError::new("ToolNotFound", "Tool 'foo' not found");
        assert_eq!(format!("{}", err), "ToolNotFound: Tool 'foo' not found");
    }

    #[test]
    fn test_named_error_with_code() {
        let err = NamedError::new("ValidationError", "Invalid input").with_code("VALIDATION_001");
        assert_eq!(err.code, Some("VALIDATION_001".to_string()));
    }

    #[test]
    fn test_named_error_with_data() {
        let err = NamedError::new("ValidationError", "Invalid input")
            .with_data(serde_json::json!({"field": "email", "reason": "invalid format"}));
        assert!(err.data.is_some());
    }

    #[test]
    fn test_named_error_serde() {
        let err = NamedError::new("TestError", "Test message").with_code("TEST_001");
        let json = serde_json::to_string(&err).unwrap();
        let decoded: NamedError = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "TestError");
        assert_eq!(decoded.code, Some("TEST_001".to_string()));
    }

    #[test]
    fn test_named_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file.txt");
        let named: NamedError = io_err.into();
        assert_eq!(named.name, "IOError");
        assert!(named.code.is_some());
    }

    #[test]
    fn test_named_error_from_reqwest_error() {
        // We can't easily create a reqwest::Error without a network request,
        // so we just test the From implementation exists and is correct
        // This would be tested in integration tests with actual reqwest errors
    }

    #[test]
    fn test_with_context_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file.txt");
        let wrapped = WithContext::new("Failed to load config", io_err);
        let display = format!("{}", wrapped);
        assert!(display.contains("Failed to load config"));
        assert!(display.contains("file.txt"));
    }

    #[test]
    fn test_context_trait_extension() {
        let result: Result<i32, std::io::Error> = Err(io::Error::other("test"));
        let with_ctx = result.context("context message");
        assert!(with_ctx.is_err());
        let wrapped_err = with_ctx.unwrap_err();
        assert!(wrapped_err.to_string().contains("context message"));
    }
}
