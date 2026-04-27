use thiserror::Error;

/// OpenCodeError - unified error type with error code numbering (FR-118).
///
/// Error code ranges:
/// - 1xxx: Authentication (token expired, invalid, missing)
/// - 2xxx: Authorization (insufficient permissions, access denied)
/// - 3xxx: Provider (not found, auth failed, unavailable)
/// - 4xxx: Tool (not found, timeout, invalid args)
/// - 5xxx: Session (not found, expired, corrupted)
/// - 6xxx: Config (missing, invalid, load failed)
/// - 7xxx: Validation (invalid parameter, missing field, format mismatch)
/// - 9xxx: Internal (internal error, service unavailable, database error)
///
/// Legacy variants (Network, Parse, Llm, Tui, Storage) are preserved for
/// backward compatibility and mapped to the closest error code range.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum OpenCodeError {
    // --- Wrapper errors (From implementations) ---
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    // --- Legacy variants (backward compatible) ---
    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Storage error: {0}")]
    Storage(String),

    // --- FR-118: Structured authentication errors (1xxx) ---
    #[error("Token expired")]
    TokenExpired { detail: Option<String> },

    #[error("Invalid token")]
    InvalidToken { detail: Option<String> },

    #[error("Missing credentials")]
    MissingCredentials { detail: Option<String> },

    // --- FR-118: Structured authorization errors (2xxx) ---
    #[error("Insufficient permissions")]
    InsufficientPermissions {
        detail: Option<String>,
        required_role: Option<String>,
    },

    #[error("Access denied")]
    AccessDenied { detail: Option<String> },

    // --- FR-118: Structured provider errors (3xxx) ---
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Provider authentication failed: {0}")]
    ProviderAuthFailed(String),

    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    // --- FR-118: Structured tool errors (4xxx) ---
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution timeout: {tool} ({timeout_ms}ms)")]
    ToolTimeout { tool: String, timeout_ms: u64 },

    #[error("Invalid tool arguments: {0}")]
    ToolInvalidArgs(String),

    // --- FR-118: Structured session errors (5xxx) ---
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session expired: {0}")]
    SessionExpired(String),

    #[error("Session corrupted: {0}")]
    SessionCorrupted(String),

    #[error("Version mismatch: {0}")]
    VersionMismatchError(String),

    // --- FR-118: Structured config errors (6xxx) ---
    #[error("Config missing: {0}")]
    ConfigMissing(String),

    #[error("Config invalid: {0}")]
    ConfigInvalid(String),

    #[error("Config load failed: {0}")]
    ConfigLoadFailed(String),

    // --- FR-118: Structured validation errors (7xxx) ---
    #[error("Invalid parameter: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Format mismatch: {0}")]
    FormatMismatch(String),

    // --- Workspace path validation errors (71xx) ---
    #[error("Workspace path does not exist: {0}")]
    WorkspacePathNotFound(String),

    #[error("Workspace path is not accessible: {0}")]
    WorkspacePathNotAccessible(String),

    #[error("Workspace path is not a directory: {0}")]
    WorkspacePathNotDirectory(String),

    #[error("Workspace path is not readable: {0}")]
    WorkspacePathNotReadable(String),

    // --- FR-118: Structured internal errors (9xxx) ---
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl OpenCodeError {
    pub fn code(&self) -> u16 {
        match self {
            Self::TokenExpired { .. } => 1001,
            Self::InvalidToken { .. } => 1002,
            Self::MissingCredentials { .. } => 1003,
            Self::InsufficientPermissions { .. } => 2001,
            Self::AccessDenied { .. } => 2002,
            Self::ProviderNotFound(_) => 3001,
            Self::ProviderAuthFailed(_) => 3002,
            Self::ProviderUnavailable(_) => 3003,
            Self::ToolNotFound(_) => 4001,
            Self::ToolTimeout { .. } => 4002,
            Self::ToolInvalidArgs(_) => 4003,
            Self::SessionNotFound(_) => 5001,
            Self::SessionExpired(_) => 5002,
            Self::SessionCorrupted(_) => 5003,
            Self::VersionMismatchError(_) => 5004,
            Self::ConfigMissing(_) => 6001,
            Self::ConfigInvalid(_) => 6002,
            Self::ConfigLoadFailed(_) => 6003,
            Self::ValidationError { .. } => 7001,
            Self::MissingRequiredField(_) => 7002,
            Self::FormatMismatch(_) => 7003,
            Self::WorkspacePathNotFound(_) => 7011,
            Self::WorkspacePathNotAccessible(_) => 7012,
            Self::WorkspacePathNotDirectory(_) => 7013,
            Self::WorkspacePathNotReadable(_) => 7014,
            Self::InternalError(_) => 9001,
            Self::ServiceUnavailable(_) => 9002,
            Self::Network(_) => 9001,
            Self::Parse(_) => 7003,
            Self::Config(_) => 6002,
            Self::Session(_) => 5001,
            Self::Tool(_) => 4001,
            Self::Llm(_) => 9001,
            Self::Tui(_) => 9001,
            Self::Provider(_) => 3001,
            Self::Storage(_) => 9003,
            Self::Io(_) => 9003,
            Self::Json(_) => 9003,
            Self::Sqlite(_) => 9003,
        }
    }

    pub fn http_status(&self) -> u16 {
        let code = self.code();
        match code {
            1000..=1999 => 401,
            2000..=2999 => 403,
            3000..=3999 => 404,
            4000..=4999 => 400,
            5000..=5999 => 404,
            6000..=6999 => 422,
            7000..=7999 => 422,
            _ => 500,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::TokenExpired { .. } => {
                "Your session has expired. Please log in again.".to_string()
            }
            Self::InvalidToken { .. } => "Invalid authentication token.".to_string(),
            Self::MissingCredentials { .. } => {
                "Authentication credentials are missing.".to_string()
            }
            Self::InsufficientPermissions { required_role, .. } => {
                if let Some(role) = required_role {
                    format!("Insufficient permissions. Required role: {}", role)
                } else {
                    "Insufficient permissions.".to_string()
                }
            }
            Self::AccessDenied { .. } => "Access denied.".to_string(),
            Self::ProviderNotFound(name) => format!("Provider '{}' not found.", name),
            Self::ProviderAuthFailed(name) => format!("Provider '{}' authentication failed.", name),
            Self::ProviderUnavailable(name) => format!("Provider '{}' is unavailable.", name),
            Self::ToolNotFound(name) => format!("Tool '{}' not found.", name),
            Self::ToolTimeout { tool, .. } => format!("Tool '{}' execution timed out.", tool),
            Self::ToolInvalidArgs(msg) => format!("Invalid tool arguments: {}", msg),
            Self::SessionNotFound(id) => format!("Session '{}' not found.", id),
            Self::SessionExpired(id) => format!("Session '{}' has expired.", id),
            Self::SessionCorrupted(id) => format!("Session '{}' is corrupted.", id),
            Self::ConfigMissing(key) => format!("Configuration '{}' is missing.", key),
            Self::ConfigInvalid(msg) => format!("Invalid configuration: {}", msg),
            Self::ConfigLoadFailed(msg) => format!("Failed to load configuration: {}", msg),
            Self::ValidationError { field, message } => {
                format!("Invalid value for '{}': {}", field, message)
            }
            Self::MissingRequiredField(field) => {
                format!("Missing required field: {}", field)
            }
            Self::FormatMismatch(msg) => format!("Format mismatch: {}", msg),
            Self::InternalError(_) => "An internal error occurred.".to_string(),
            Self::ServiceUnavailable(msg) => format!("Service unavailable: {}", msg),
            _ => self.to_string(),
        }
    }

    pub fn to_api_response(&self, debug_mode: bool) -> serde_json::Value {
        let mut error_obj = serde_json::Map::new();
        error_obj.insert("code".to_string(), serde_json::json!(self.code()));
        error_obj.insert(
            "message".to_string(),
            serde_json::json!(self.user_message()),
        );
        if debug_mode {
            error_obj.insert("detail".to_string(), serde_json::json!(self.to_string()));
        }
        serde_json::json!({ "error": error_obj })
    }
}

pub mod tests {
    use super::*;

    #[test]
    fn test_error_display_io() {
        let err = OpenCodeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert!(err.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_display_json() {
        let err =
            OpenCodeError::Json(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err());
        assert!(err.to_string().contains("JSON error"));
    }

    #[test]
    fn test_error_display_network() {
        let err = OpenCodeError::Network("connection refused".to_string());
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn test_error_display_config() {
        let err = OpenCodeError::Config("missing key".to_string());
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_error_display_session() {
        let err = OpenCodeError::Session("not found".to_string());
        assert!(err.to_string().contains("Session error"));
    }

    #[test]
    fn test_error_display_tool() {
        let err = OpenCodeError::Tool("execution failed".to_string());
        assert!(err.to_string().contains("Tool error"));
    }

    #[test]
    fn test_error_display_llm() {
        let err = OpenCodeError::Llm("api error".to_string());
        assert!(err.to_string().contains("LLM error"));
    }

    #[test]
    fn test_error_display_provider() {
        let err = OpenCodeError::Provider("invalid response".to_string());
        assert!(err.to_string().contains("Provider error"));
    }

    #[test]
    fn test_error_code_auth() {
        assert_eq!(OpenCodeError::TokenExpired { detail: None }.code(), 1001);
        assert_eq!(OpenCodeError::InvalidToken { detail: None }.code(), 1002);
        assert_eq!(
            OpenCodeError::MissingCredentials { detail: None }.code(),
            1003
        );
    }

    #[test]
    fn test_error_code_authz() {
        assert_eq!(
            OpenCodeError::InsufficientPermissions {
                detail: None,
                required_role: None
            }
            .code(),
            2001
        );
        assert_eq!(OpenCodeError::AccessDenied { detail: None }.code(), 2002);
    }

    #[test]
    fn test_error_code_provider() {
        assert_eq!(OpenCodeError::ProviderNotFound("x".into()).code(), 3001);
        assert_eq!(OpenCodeError::ProviderAuthFailed("x".into()).code(), 3002);
        assert_eq!(OpenCodeError::ProviderUnavailable("x".into()).code(), 3003);
    }

    #[test]
    fn test_error_code_tool() {
        assert_eq!(OpenCodeError::ToolNotFound("x".into()).code(), 4001);
        assert_eq!(
            OpenCodeError::ToolTimeout {
                tool: "x".into(),
                timeout_ms: 5000
            }
            .code(),
            4002
        );
        assert_eq!(OpenCodeError::ToolInvalidArgs("x".into()).code(), 4003);
    }

    #[test]
    fn test_error_code_session() {
        assert_eq!(OpenCodeError::SessionNotFound("x".into()).code(), 5001);
        assert_eq!(OpenCodeError::SessionExpired("x".into()).code(), 5002);
        assert_eq!(OpenCodeError::SessionCorrupted("x".into()).code(), 5003);
    }

    #[test]
    fn test_error_code_config() {
        assert_eq!(OpenCodeError::ConfigMissing("x".into()).code(), 6001);
        assert_eq!(OpenCodeError::ConfigInvalid("x".into()).code(), 6002);
        assert_eq!(OpenCodeError::ConfigLoadFailed("x".into()).code(), 6003);
    }

    #[test]
    fn test_error_code_validation() {
        assert_eq!(
            OpenCodeError::ValidationError {
                field: "x".into(),
                message: "y".into()
            }
            .code(),
            7001
        );
        assert_eq!(OpenCodeError::MissingRequiredField("x".into()).code(), 7002);
        assert_eq!(OpenCodeError::FormatMismatch("x".into()).code(), 7003);
    }

    #[test]
    fn test_error_code_internal() {
        assert_eq!(OpenCodeError::InternalError("x".into()).code(), 9001);
        assert_eq!(OpenCodeError::ServiceUnavailable("x".into()).code(), 9002);
    }

    #[test]
    fn test_error_code_legacy_mapping() {
        assert_eq!(OpenCodeError::Network("x".into()).code(), 9001);
        assert_eq!(OpenCodeError::Parse("x".into()).code(), 7003);
        assert_eq!(OpenCodeError::Config("x".into()).code(), 6002);
        assert_eq!(OpenCodeError::Session("x".into()).code(), 5001);
        assert_eq!(OpenCodeError::Tool("x".into()).code(), 4001);
        assert_eq!(OpenCodeError::Llm("x".into()).code(), 9001);
        assert_eq!(OpenCodeError::Tui("x".into()).code(), 9001);
        assert_eq!(OpenCodeError::Provider("x".into()).code(), 3001);
        assert_eq!(OpenCodeError::Storage("x".into()).code(), 9003);
    }

    #[test]
    fn test_http_status_mapping() {
        assert_eq!(
            OpenCodeError::TokenExpired { detail: None }.http_status(),
            401
        );
        assert_eq!(
            OpenCodeError::InsufficientPermissions {
                detail: None,
                required_role: None
            }
            .http_status(),
            403
        );
        assert_eq!(
            OpenCodeError::ProviderNotFound("x".into()).http_status(),
            404
        );
        assert_eq!(OpenCodeError::ToolNotFound("x".into()).http_status(), 400);
        assert_eq!(
            OpenCodeError::SessionNotFound("x".into()).http_status(),
            404
        );
        assert_eq!(OpenCodeError::ConfigMissing("x".into()).http_status(), 422);
        assert_eq!(
            OpenCodeError::ValidationError {
                field: "x".into(),
                message: "y".into()
            }
            .http_status(),
            422
        );
        assert_eq!(OpenCodeError::InternalError("x".into()).http_status(), 500);
    }

    #[test]
    fn test_api_response_format() {
        let err = OpenCodeError::TokenExpired { detail: None };
        let resp = err.to_api_response(false);
        let error_obj = resp.get("error").unwrap();
        assert_eq!(error_obj.get("code").unwrap().as_u64().unwrap(), 1001);
        assert!(error_obj.get("message").is_some());
        assert!(error_obj.get("detail").is_none());

        let resp_debug = err.to_api_response(true);
        let error_obj_debug = resp_debug.get("error").unwrap();
        assert!(error_obj_debug.get("detail").is_some());
    }

    #[test]
    fn test_user_message_auth() {
        let err = OpenCodeError::TokenExpired { detail: None };
        assert!(err.user_message().contains("expired"));
    }

    #[test]
    fn test_user_message_provider() {
        let err = OpenCodeError::ProviderNotFound("openai".into());
        assert!(err.user_message().contains("openai"));
    }

    #[test]
    fn test_all_variants_display_consistency() {
        let variants: Vec<(&str, OpenCodeError)> = vec![
            (
                "IO error",
                OpenCodeError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            ),
            (
                "JSON error",
                OpenCodeError::Json(
                    serde_json::from_str::<serde_json::Value>("invalid").unwrap_err(),
                ),
            ),
            (
                "Sqlite error",
                OpenCodeError::Sqlite(rusqlite::Error::InvalidQuery),
            ),
            ("Network error", OpenCodeError::Network("test".to_string())),
            ("Parse error", OpenCodeError::Parse("test".to_string())),
            (
                "Configuration error",
                OpenCodeError::Config("test".to_string()),
            ),
            ("Session error", OpenCodeError::Session("test".to_string())),
            ("Tool error", OpenCodeError::Tool("test".to_string())),
            ("LLM error", OpenCodeError::Llm("test".to_string())),
            ("TUI error", OpenCodeError::Tui("test".to_string())),
            (
                "Provider error",
                OpenCodeError::Provider("test".to_string()),
            ),
            ("Storage error", OpenCodeError::Storage("test".to_string())),
            (
                "Token expired",
                OpenCodeError::TokenExpired { detail: None },
            ),
            (
                "Invalid token",
                OpenCodeError::InvalidToken { detail: None },
            ),
            (
                "Missing credentials",
                OpenCodeError::MissingCredentials { detail: None },
            ),
            (
                "Insufficient permissions",
                OpenCodeError::InsufficientPermissions {
                    detail: None,
                    required_role: None,
                },
            ),
            (
                "Access denied",
                OpenCodeError::AccessDenied { detail: None },
            ),
            (
                "Provider not found",
                OpenCodeError::ProviderNotFound("openai".to_string()),
            ),
            (
                "Provider authentication failed",
                OpenCodeError::ProviderAuthFailed("openai".to_string()),
            ),
            (
                "Provider unavailable",
                OpenCodeError::ProviderUnavailable("openai".to_string()),
            ),
            (
                "Tool not found",
                OpenCodeError::ToolNotFound("read".to_string()),
            ),
            (
                "Tool execution timeout",
                OpenCodeError::ToolTimeout {
                    tool: "read".to_string(),
                    timeout_ms: 5000,
                },
            ),
            (
                "Invalid tool arguments",
                OpenCodeError::ToolInvalidArgs("test".to_string()),
            ),
            (
                "Session not found",
                OpenCodeError::SessionNotFound("abc".to_string()),
            ),
            (
                "Session expired",
                OpenCodeError::SessionExpired("abc".to_string()),
            ),
            (
                "Session corrupted",
                OpenCodeError::SessionCorrupted("abc".to_string()),
            ),
            (
                "Version mismatch",
                OpenCodeError::VersionMismatchError("1.0".to_string()),
            ),
            (
                "Config missing",
                OpenCodeError::ConfigMissing("key".to_string()),
            ),
            (
                "Config invalid",
                OpenCodeError::ConfigInvalid("key".to_string()),
            ),
            (
                "Config load failed",
                OpenCodeError::ConfigLoadFailed("key".to_string()),
            ),
            (
                "Invalid parameter",
                OpenCodeError::ValidationError {
                    field: "name".to_string(),
                    message: "required".to_string(),
                },
            ),
            (
                "Missing required field",
                OpenCodeError::MissingRequiredField("email".to_string()),
            ),
            (
                "Format mismatch",
                OpenCodeError::FormatMismatch("json".to_string()),
            ),
            (
                "Workspace path does not exist",
                OpenCodeError::WorkspacePathNotFound("/path".to_string()),
            ),
            (
                "Workspace path is not accessible",
                OpenCodeError::WorkspacePathNotAccessible("/path".to_string()),
            ),
            (
                "Workspace path is not a directory",
                OpenCodeError::WorkspacePathNotDirectory("/path".to_string()),
            ),
            (
                "Workspace path is not readable",
                OpenCodeError::WorkspacePathNotReadable("/path".to_string()),
            ),
            (
                "Internal error",
                OpenCodeError::InternalError("test".to_string()),
            ),
            (
                "Service unavailable",
                OpenCodeError::ServiceUnavailable("test".to_string()),
            ),
        ];

        for (expected_prefix, err) in variants {
            let display = err.to_string();
            assert!(
                display.contains(expected_prefix),
                "Error {:?} display should contain '{}', got: {}",
                err,
                expected_prefix,
                display
            );
            assert!(
                !display.is_empty(),
                "Error {:?} should not have empty display",
                err
            );
        }
    }

    #[test]
    fn test_all_variants_have_valid_codes() {
        let variants: Vec<OpenCodeError> = vec![
            OpenCodeError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            OpenCodeError::Json(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err()),
            OpenCodeError::Sqlite(rusqlite::Error::InvalidQuery),
            OpenCodeError::Network("test".to_string()),
            OpenCodeError::Parse("test".to_string()),
            OpenCodeError::Config("test".to_string()),
            OpenCodeError::Session("test".to_string()),
            OpenCodeError::Tool("test".to_string()),
            OpenCodeError::Llm("test".to_string()),
            OpenCodeError::Tui("test".to_string()),
            OpenCodeError::Provider("test".to_string()),
            OpenCodeError::Storage("test".to_string()),
            OpenCodeError::TokenExpired { detail: None },
            OpenCodeError::InvalidToken { detail: None },
            OpenCodeError::MissingCredentials { detail: None },
            OpenCodeError::InsufficientPermissions {
                detail: None,
                required_role: None,
            },
            OpenCodeError::AccessDenied { detail: None },
            OpenCodeError::ProviderNotFound("openai".to_string()),
            OpenCodeError::ProviderAuthFailed("openai".to_string()),
            OpenCodeError::ProviderUnavailable("openai".to_string()),
            OpenCodeError::ToolNotFound("read".to_string()),
            OpenCodeError::ToolTimeout {
                tool: "read".to_string(),
                timeout_ms: 5000,
            },
            OpenCodeError::ToolInvalidArgs("test".to_string()),
            OpenCodeError::SessionNotFound("abc".to_string()),
            OpenCodeError::SessionExpired("abc".to_string()),
            OpenCodeError::SessionCorrupted("abc".to_string()),
            OpenCodeError::VersionMismatchError("1.0".to_string()),
            OpenCodeError::ConfigMissing("key".to_string()),
            OpenCodeError::ConfigInvalid("key".to_string()),
            OpenCodeError::ConfigLoadFailed("key".to_string()),
            OpenCodeError::ValidationError {
                field: "name".to_string(),
                message: "required".to_string(),
            },
            OpenCodeError::MissingRequiredField("email".to_string()),
            OpenCodeError::FormatMismatch("json".to_string()),
            OpenCodeError::WorkspacePathNotFound("/path".to_string()),
            OpenCodeError::WorkspacePathNotAccessible("/path".to_string()),
            OpenCodeError::WorkspacePathNotDirectory("/path".to_string()),
            OpenCodeError::WorkspacePathNotReadable("/path".to_string()),
            OpenCodeError::InternalError("test".to_string()),
            OpenCodeError::ServiceUnavailable("test".to_string()),
        ];

        for err in variants {
            let code = err.code();
            assert!(
                (1000..10000).contains(&code),
                "Error code for {:?} should be in 1xxx-9xxx range, got: {}",
                err,
                code
            );
        }
    }

    #[test]
    fn test_all_variants_have_http_status() {
        let variants: Vec<OpenCodeError> = vec![
            OpenCodeError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
            OpenCodeError::Json(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err()),
            OpenCodeError::Sqlite(rusqlite::Error::InvalidQuery),
            OpenCodeError::Network("test".to_string()),
            OpenCodeError::Parse("test".to_string()),
            OpenCodeError::Config("test".to_string()),
            OpenCodeError::Session("test".to_string()),
            OpenCodeError::Tool("test".to_string()),
            OpenCodeError::Llm("test".to_string()),
            OpenCodeError::Tui("test".to_string()),
            OpenCodeError::Provider("test".to_string()),
            OpenCodeError::Storage("test".to_string()),
            OpenCodeError::TokenExpired { detail: None },
            OpenCodeError::InvalidToken { detail: None },
            OpenCodeError::MissingCredentials { detail: None },
            OpenCodeError::InsufficientPermissions {
                detail: None,
                required_role: None,
            },
            OpenCodeError::AccessDenied { detail: None },
            OpenCodeError::ProviderNotFound("openai".to_string()),
            OpenCodeError::ProviderAuthFailed("openai".to_string()),
            OpenCodeError::ProviderUnavailable("openai".to_string()),
            OpenCodeError::ToolNotFound("read".to_string()),
            OpenCodeError::ToolTimeout {
                tool: "read".to_string(),
                timeout_ms: 5000,
            },
            OpenCodeError::ToolInvalidArgs("test".to_string()),
            OpenCodeError::SessionNotFound("abc".to_string()),
            OpenCodeError::SessionExpired("abc".to_string()),
            OpenCodeError::SessionCorrupted("abc".to_string()),
            OpenCodeError::VersionMismatchError("1.0".to_string()),
            OpenCodeError::ConfigMissing("key".to_string()),
            OpenCodeError::ConfigInvalid("key".to_string()),
            OpenCodeError::ConfigLoadFailed("key".to_string()),
            OpenCodeError::ValidationError {
                field: "name".to_string(),
                message: "required".to_string(),
            },
            OpenCodeError::MissingRequiredField("email".to_string()),
            OpenCodeError::FormatMismatch("json".to_string()),
            OpenCodeError::WorkspacePathNotFound("/path".to_string()),
            OpenCodeError::WorkspacePathNotAccessible("/path".to_string()),
            OpenCodeError::WorkspacePathNotDirectory("/path".to_string()),
            OpenCodeError::WorkspacePathNotReadable("/path".to_string()),
            OpenCodeError::InternalError("test".to_string()),
            OpenCodeError::ServiceUnavailable("test".to_string()),
        ];

        for err in variants {
            let status = err.http_status();
            assert!(
                (400..600).contains(&status),
                "HTTP status for {:?} should be in 4xx-5xx range, got: {}",
                err,
                status
            );
        }
    }
}