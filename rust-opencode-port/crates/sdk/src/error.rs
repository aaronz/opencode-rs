//! SDK Error types with error code mapping (FR-222, ERR-001-012).
//!
//! Error codes follow the unified numbering scheme:
//! - 1xxx: Authentication errors
//! - 2xxx: Authorization errors
//! - 3xxx: Provider errors
//! - 4xxx: Tool errors
//! - 5xxx: Session errors
//! - 6xxx: Config errors
//! - 7xxx: Validation errors
//! - 9xxx: Internal errors

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// SDK Result type alias
pub type SdkResult<T> = Result<T, SdkError>;

/// Unified SDK Error type with error codes per FR-222 and ERR-001-012.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SdkError {
    // --- Authentication (1xxx) ---
    #[error("Authentication failed: {detail}")]
    AuthenticationFailed { detail: String, code: u16 },

    #[error("Invalid API key")]
    InvalidApiKey { detail: Option<String>, code: u16 },

    #[error("Token expired")]
    TokenExpired { detail: Option<String>, code: u16 },

    // --- Authorization (2xxx) ---
    #[error("Access denied: {detail}")]
    AccessDenied { detail: String, code: u16 },

    #[error("Insufficient permissions")]
    InsufficientPermissions {
        detail: String,
        required_role: Option<String>,
        code: u16,
    },

    // --- Provider (3xxx) ---
    #[error("Provider error: {detail}")]
    ProviderError {
        provider: String,
        detail: String,
        code: u16,
    },

    #[error("Provider not found: {name}")]
    ProviderNotFound { name: String, code: u16 },

    // --- Tool (4xxx) ---
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String, code: u16 },

    #[error("Tool execution failed: {detail}")]
    ToolExecutionFailed {
        tool: String,
        detail: String,
        code: u16,
    },

    #[error("Tool timeout: {tool} ({timeout_ms}ms)")]
    ToolTimeout {
        tool: String,
        timeout_ms: u64,
        code: u16,
    },

    #[error("Invalid tool arguments: {detail}")]
    InvalidToolArgs { detail: String, code: u16 },

    // --- Session (5xxx) ---
    #[error("Session not found: {id}")]
    SessionNotFound { id: String, code: u16 },

    #[error("Session error: {detail}")]
    SessionError { detail: String, code: u16 },

    #[error("Session expired: {id}")]
    SessionExpired { id: String, code: u16 },

    // --- Config (6xxx) ---
    #[error("Configuration error: {detail}")]
    ConfigError { detail: String, code: u16 },

    #[error("Missing configuration: {field}")]
    MissingConfig { field: String, code: u16 },

    // --- Validation (7xxx) ---
    #[error("Validation error: {field} - {message}")]
    ValidationError {
        field: String,
        message: String,
        code: u16,
    },

    #[error("Invalid request: {detail}")]
    InvalidRequest { detail: String, code: u16 },

    // --- Internal (9xxx) ---
    #[error("Internal error: {detail}")]
    InternalError { detail: String, code: u16 },

    #[error("Network error: {detail}")]
    NetworkError { detail: String, code: u16 },

    #[error("IO error: {detail}")]
    IoError { detail: String, code: u16 },

    // --- HTTP Error (API error from server) ---
    #[error("API error: {status_code} - {message}")]
    ApiError {
        status_code: u16,
        message: String,
        code: u16,
    },
}

impl SdkError {
    /// Returns the error code for this error.
    pub fn code(&self) -> u16 {
        match self {
            Self::AuthenticationFailed { code, .. } => *code,
            Self::InvalidApiKey { code, .. } => *code,
            Self::TokenExpired { code, .. } => *code,
            Self::AccessDenied { code, .. } => *code,
            Self::InsufficientPermissions { code, .. } => *code,
            Self::ProviderError { code, .. } => *code,
            Self::ProviderNotFound { code, .. } => *code,
            Self::ToolNotFound { code, .. } => *code,
            Self::ToolExecutionFailed { code, .. } => *code,
            Self::ToolTimeout { code, .. } => *code,
            Self::InvalidToolArgs { code, .. } => *code,
            Self::SessionNotFound { code, .. } => *code,
            Self::SessionError { code, .. } => *code,
            Self::SessionExpired { code, .. } => *code,
            Self::ConfigError { code, .. } => *code,
            Self::MissingConfig { code, .. } => *code,
            Self::ValidationError { code, .. } => *code,
            Self::InvalidRequest { code, .. } => *code,
            Self::InternalError { code, .. } => *code,
            Self::NetworkError { code, .. } => *code,
            Self::IoError { code, .. } => *code,
            Self::ApiError { code, .. } => *code,
        }
    }

    /// Maps an HTTP status code to an SDK error code.
    pub fn from_http_status(status: u16, message: &str) -> Self {
        match status {
            400 => Self::InvalidRequest {
                detail: message.to_string(),
                code: 4001,
            },
            401 | 403 => Self::AuthenticationFailed {
                detail: message.to_string(),
                code: 1001,
            },
            404 => Self::SessionNotFound {
                id: message.to_string(),
                code: 5001,
            },
            422 => Self::ValidationError {
                field: "request".to_string(),
                message: message.to_string(),
                code: 7001,
            },
            500..=599 => Self::InternalError {
                detail: message.to_string(),
                code: 9001,
            },
            _ => Self::ApiError {
                status_code: status,
                message: message.to_string(),
                code: 9001,
            },
        }
    }

    /// Creates an authentication error.
    pub fn auth_failed(detail: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            detail: detail.into(),
            code: 1001,
        }
    }

    /// Creates an invalid API key error.
    pub fn invalid_api_key(detail: Option<String>) -> Self {
        Self::InvalidApiKey { detail, code: 1002 }
    }

    /// Creates a session not found error.
    pub fn session_not_found(id: impl Into<String>) -> Self {
        Self::SessionNotFound {
            id: id.into(),
            code: 5001,
        }
    }

    /// Creates a tool not found error.
    pub fn tool_not_found(name: impl Into<String>) -> Self {
        Self::ToolNotFound {
            name: name.into(),
            code: 4001,
        }
    }

    /// Creates a tool execution failed error.
    pub fn tool_execution_failed(tool: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ToolExecutionFailed {
            tool: tool.into(),
            detail: detail.into(),
            code: 4003,
        }
    }

    /// Creates a network error.
    pub fn network_error(detail: impl Into<String>) -> Self {
        Self::NetworkError {
            detail: detail.into(),
            code: 9001,
        }
    }

    /// Creates an internal error.
    pub fn internal_error(detail: impl Into<String>) -> Self {
        Self::InternalError {
            detail: detail.into(),
            code: 9001,
        }
    }

    /// Creates a config error.
    pub fn config_error(detail: impl Into<String>) -> Self {
        Self::ConfigError {
            detail: detail.into(),
            code: 6002,
        }
    }

    /// Creates a missing config error.
    pub fn missing_config(field: impl Into<String>) -> Self {
        Self::MissingConfig {
            field: field.into(),
            code: 6001,
        }
    }

    /// Returns the error category based on code range.
    pub fn category(&self) -> &'static str {
        match self.code() {
            1000..=1999 => "Authentication",
            2000..=2999 => "Authorization",
            3000..=3999 => "Provider",
            4000..=4999 => "Tool",
            5000..=5999 => "Session",
            6000..=6999 => "Config",
            7000..=7999 => "Validation",
            9000..=9999 => "Internal",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(SdkError::auth_failed("test").code(), 1001);
        assert_eq!(SdkError::invalid_api_key(None).code(), 1002);
        assert_eq!(SdkError::session_not_found("x").code(), 5001);
        assert_eq!(SdkError::tool_not_found("x").code(), 4001);
        assert_eq!(SdkError::tool_execution_failed("x", "y").code(), 4003);
        assert_eq!(SdkError::config_error("x").code(), 6002);
        assert_eq!(SdkError::missing_config("x").code(), 6001);
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(SdkError::auth_failed("test").category(), "Authentication");
        assert_eq!(SdkError::session_not_found("x").category(), "Session");
        assert_eq!(SdkError::tool_not_found("x").category(), "Tool");
    }

    #[test]
    fn test_http_status_mapping() {
        let err = SdkError::from_http_status(401, "Unauthorized");
        assert_eq!(err.code(), 1001);
        assert_eq!(err.category(), "Authentication");

        let err = SdkError::from_http_status(404, "Not found");
        assert_eq!(err.code(), 5001);
        assert_eq!(err.category(), "Session");

        let err = SdkError::from_http_status(500, "Internal error");
        assert_eq!(err.code(), 9001);
        assert_eq!(err.category(), "Internal");
    }
}
