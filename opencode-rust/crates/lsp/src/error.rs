//! LSP-specific error types for robust failure handling.
//!
//! This module provides structured error types for LSP server failures including:
//! - Server crashes
//! - Request timeouts
//! - Protocol violations

use opencode_core::OpenCodeError;
use std::time::Duration;

/// LSP-specific error codes (following FR-118 convention, using 4xxx range for tool errors)
const LSP_ERROR_BASE: u16 = 4100;

/// Errors that can occur during LSP operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LspError {
    /// LSP server process crashed or was killed unexpectedly.
    #[error("LSP server crashed: {server_name} ({cause})")]
    ServerCrash {
        server_name: String,
        cause: CrashCause,
    },

    /// Request to LSP server timed out.
    #[error("LSP request timed out: {method} ({timeout_ms}ms)")]
    RequestTimeout { method: String, timeout_ms: u64 },

    /// LSP protocol violation detected.
    #[error("LSP protocol violation: {violation} ({detail})")]
    ProtocolViolation {
        violation: ProtocolViolationType,
        detail: String,
    },

    /// Server is not responding or unhealthy.
    #[error("LSP server unhealthy: {server_name} ({reason})")]
    ServerUnhealthy {
        server_name: String,
        reason: UnhealthyReason,
    },

    /// Failed to parse LSP message.
    #[error("Invalid LSP message: {0}")]
    InvalidMessage(String),

    /// Server does not support the requested capability.
    #[error("Capability not supported: {capability} ({server})")]
    CapabilityNotSupported { capability: String, server: String },
}

/// Cause of a server crash.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CrashCause {
    #[error("process exited with code {code}")]
    ProcessExited { code: i32 },

    #[error("process was killed")]
    Killed,

    #[error("process panic: {message}")]
    Panic { message: String },

    #[error("broken pipe")]
    BrokenPipe,

    #[error("connection refused")]
    ConnectionRefused,

    #[error("unknown: {0}")]
    Unknown(String),
}

/// Type of protocol violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolViolationType {
    /// Message is not valid JSON-RPC.
    InvalidJson,
    /// Message is missing required fields.
    MissingField,
    /// Message has incorrect type for the method.
    InvalidMessageType,
    /// Response ID does not match any pending request.
    ResponseIdMismatch,
    /// Content-Length header is missing or invalid.
    MissingContentLength,
    /// Invalid Content-Length value.
    InvalidContentLength,
    /// Message batch is malformed.
    InvalidBatch,
    /// Server sent a response to a notification.
    UnexpectedResponse,
}

impl std::fmt::Display for ProtocolViolationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolViolationType::InvalidJson => write!(f, "InvalidJson"),
            ProtocolViolationType::MissingField => write!(f, "MissingField"),
            ProtocolViolationType::InvalidMessageType => write!(f, "InvalidMessageType"),
            ProtocolViolationType::ResponseIdMismatch => write!(f, "ResponseIdMismatch"),
            ProtocolViolationType::MissingContentLength => write!(f, "MissingContentLength"),
            ProtocolViolationType::InvalidContentLength => write!(f, "InvalidContentLength"),
            ProtocolViolationType::InvalidBatch => write!(f, "InvalidBatch"),
            ProtocolViolationType::UnexpectedResponse => write!(f, "UnexpectedResponse"),
        }
    }
}

/// Reason why server is considered unhealthy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnhealthyReason {
    /// Server is not responding to health checks.
    NotResponding,
    /// Server has exceeded error threshold.
    ErrorThresholdExceeded,
    /// Server process is in zombie state.
    ZombieProcess,
}

impl std::fmt::Display for UnhealthyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnhealthyReason::NotResponding => write!(f, "NotResponding"),
            UnhealthyReason::ErrorThresholdExceeded => write!(f, "ErrorThresholdExceeded"),
            UnhealthyReason::ZombieProcess => write!(f, "ZombieProcess"),
        }
    }
}

impl LspError {
    /// Returns the numeric error code.
    pub fn code(&self) -> u16 {
        match self {
            Self::ServerCrash { .. } => LSP_ERROR_BASE,
            Self::RequestTimeout { .. } => LSP_ERROR_BASE + 1,
            Self::ProtocolViolation { .. } => LSP_ERROR_BASE + 2,
            Self::ServerUnhealthy { .. } => LSP_ERROR_BASE + 3,
            Self::InvalidMessage(_) => LSP_ERROR_BASE + 4,
            Self::CapabilityNotSupported { .. } => LSP_ERROR_BASE + 5,
        }
    }

    /// Returns the appropriate HTTP status code.
    pub fn http_status(&self) -> u16 {
        match self {
            Self::ServerCrash { .. } => 503,       // Service unavailable
            Self::RequestTimeout { .. } => 504,    // Gateway timeout
            Self::ProtocolViolation { .. } => 422, // Unprocessable entity
            Self::ServerUnhealthy { .. } => 503,
            Self::InvalidMessage(_) => 400,
            Self::CapabilityNotSupported { .. } => 501, // Not implemented
        }
    }

    /// Convert to OpenCodeError for unified error handling.
    pub fn into_opencode_error(self) -> OpenCodeError {
        match self {
            Self::ServerCrash { server_name, cause } => OpenCodeError::ServiceUnavailable(format!(
                "LSP server '{}' crashed: {}",
                server_name, cause
            )),
            Self::RequestTimeout { method, timeout_ms } => OpenCodeError::ToolTimeout {
                tool: format!("LSP:{}", method),
                timeout_ms,
            },
            Self::ProtocolViolation { violation, detail } => OpenCodeError::Tool(format!(
                "LSP protocol violation ({:?}): {}",
                violation, detail
            )),
            Self::ServerUnhealthy {
                server_name,
                reason,
            } => OpenCodeError::ServiceUnavailable(format!(
                "LSP server '{}' unhealthy: {:?}",
                server_name, reason
            )),
            Self::InvalidMessage(msg) => {
                OpenCodeError::Tool(format!("Invalid LSP message: {}", msg))
            }
            Self::CapabilityNotSupported { capability, server } => OpenCodeError::Tool(format!(
                "LSP capability '{}' not supported by '{}'",
                capability, server
            )),
        }
    }
}

impl From<LspError> for OpenCodeError {
    fn from(err: LspError) -> Self {
        err.into_opencode_error()
    }
}

/// Configuration for LSP failure handling.
#[derive(Debug, Clone)]
pub struct FailureHandlingConfig {
    /// Default timeout for LSP requests in milliseconds.
    pub default_request_timeout_ms: u64,

    /// Maximum number of consecutive errors before server is considered unhealthy.
    pub max_consecutive_errors: u32,

    /// Interval for health checks in milliseconds.
    pub health_check_interval_ms: u64,

    /// Whether to automatically restart crashed servers.
    pub auto_restart: bool,
}

impl Default for FailureHandlingConfig {
    fn default() -> Self {
        Self {
            default_request_timeout_ms: 30_000, // 30 seconds
            max_consecutive_errors: 5,
            health_check_interval_ms: 5_000, // 5 seconds
            auto_restart: true,
        }
    }
}

impl FailureHandlingConfig {
    /// Create a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the default request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.default_request_timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set the max consecutive errors threshold.
    pub fn with_max_consecutive_errors(mut self, max: u32) -> Self {
        self.max_consecutive_errors = max;
        self
    }

    /// Set whether to auto-restart crashed servers.
    pub fn with_auto_restart(mut self, enabled: bool) -> Self {
        self.auto_restart = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_error_codes() {
        assert_eq!(
            LspError::ServerCrash {
                server_name: "rust-analyzer".to_string(),
                cause: CrashCause::Killed,
            }
            .code(),
            4100
        );

        assert_eq!(
            LspError::RequestTimeout {
                method: "textDocument/definition".to_string(),
                timeout_ms: 5000,
            }
            .code(),
            4101
        );

        assert_eq!(
            LspError::ProtocolViolation {
                violation: ProtocolViolationType::InvalidJson,
                detail: "missing jsonrpc field".to_string(),
            }
            .code(),
            4102
        );
    }

    #[test]
    fn test_lsp_error_http_status() {
        assert_eq!(
            LspError::ServerCrash {
                server_name: "rust-analyzer".to_string(),
                cause: CrashCause::Killed,
            }
            .http_status(),
            503
        );

        assert_eq!(
            LspError::RequestTimeout {
                method: "textDocument/definition".to_string(),
                timeout_ms: 5000,
            }
            .http_status(),
            504
        );

        assert_eq!(
            LspError::InvalidMessage("test".to_string()).http_status(),
            400
        );
    }

    #[test]
    fn test_lsp_error_to_opencode() {
        let err = LspError::RequestTimeout {
            method: "textDocument/hover".to_string(),
            timeout_ms: 3000,
        };
        let oc_err: OpenCodeError = err.into();
        assert!(matches!(oc_err, OpenCodeError::ToolTimeout { .. }));
    }

    #[test]
    fn test_failure_handling_config_default() {
        let config = FailureHandlingConfig::default();
        assert_eq!(config.default_request_timeout_ms, 30_000);
        assert_eq!(config.max_consecutive_errors, 5);
        assert!(config.auto_restart);
    }

    #[test]
    fn test_failure_handling_config_builder() {
        let config = FailureHandlingConfig::new()
            .with_request_timeout(Duration::from_secs(60))
            .with_max_consecutive_errors(10)
            .with_auto_restart(false);

        assert_eq!(config.default_request_timeout_ms, 60_000);
        assert_eq!(config.max_consecutive_errors, 10);
        assert!(!config.auto_restart);
    }
}
