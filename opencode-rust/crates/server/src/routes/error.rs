use actix_web::HttpResponse;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: u16,
}

#[allow(dead_code)]
impl FieldError {
    pub(crate) fn new(field: impl Into<String>, message: impl Into<String>, code: u16) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code,
        }
    }

    pub(crate) fn with_code(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: 7001,
        }
    }

    pub(crate) fn required(field: impl Into<String>) -> Self {
        let f = field.into();
        Self::new(f.clone(), format!("Field '{}' is required", f), 7002)
    }

    pub(crate) fn format(field: impl Into<String>, expected: impl Into<String>) -> Self {
        let f = field.into();
        Self::new(
            f.clone(),
            format!(
                "Field '{}' has invalid format. Expected: {}",
                f,
                expected.into()
            ),
            7003,
        )
    }

    pub(crate) fn out_of_range(
        field: impl Into<String>,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Self {
        let f = field.into();
        let message = match (min, max) {
            (Some(min), Some(max)) => format!("Field '{}' must be between {} and {}", f, min, max),
            (Some(min), None) => format!("Field '{}' must be at least {}", f, min),
            (None, Some(max)) => format!("Field '{}' must be at most {}", f, max),
            (None, None) => format!("Field '{}' has invalid value", f),
        };
        Self::new(f, message, 7001)
    }

    pub(crate) fn too_long(field: impl Into<String>, max_length: usize) -> Self {
        let f = field.into();
        Self::new(
            f.clone(),
            format!(
                "Field '{}' exceeds maximum length of {} characters",
                f, max_length
            ),
            7001,
        )
    }

    pub(crate) fn too_short(field: impl Into<String>, min_length: usize) -> Self {
        let f = field.into();
        Self::new(
            f.clone(),
            format!("Field '{}' must be at least {} characters", f, min_length),
            7001,
        )
    }

    pub(crate) fn invalid_enum(field: impl Into<String>, valid_values: &[&str]) -> Self {
        let f = field.into();
        Self::new(
            f.clone(),
            format!(
                "Field '{}' has invalid value. Valid values: {}",
                f,
                valid_values.join(", ")
            ),
            7001,
        )
    }
}

#[derive(Error, Debug, Clone)]
pub enum RouteError {
    #[error("Token expired")]
    TokenExpired { detail: Option<String> },

    #[error("Invalid token")]
    InvalidToken { detail: Option<String> },

    #[error("Missing credentials")]
    MissingCredentials { detail: Option<String> },

    #[error("Insufficient permissions")]
    InsufficientPermissions { detail: Option<String> },

    #[error("Access denied")]
    AccessDenied { detail: Option<String> },

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Provider authentication failed: {0}")]
    ProviderAuthFailed(String),

    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution timeout: {tool} ({timeout_ms}ms)")]
    ToolTimeout { tool: String, timeout_ms: u64 },

    #[error("Invalid tool arguments: {0}")]
    ToolInvalidArgs(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session expired: {0}")]
    SessionExpired(String),

    #[error("Session corrupted: {0}")]
    SessionCorrupted(String),

    #[error("Config missing: {0}")]
    ConfigMissing(String),

    #[error("Config invalid: {0}")]
    ConfigInvalid(String),

    #[error("Config load failed: {0}")]
    ConfigLoadFailed(String),

    #[error("Invalid parameter: {field} - {message}")]
    ValidationError { field: String, message: String },

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Format mismatch: {0}")]
    FormatMismatch(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Resource not found: {resource} ({id})")]
    ResourceNotFound { resource: String, id: String },

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Checkpoint error: {0}")]
    CheckpointError(String),

    #[error("Share error: {0}")]
    ShareError(String),

    #[error("Summary error: {0}")]
    SummaryError(String),

    #[error("Fork error: {0}")]
    ForkError(String),

    #[error("Command execution failed: {0}")]
    CommandError(String),

    #[error("Revert failed: {0}")]
    RevertError(String),

    #[error("Credentials not found: {0}")]
    CredentialsNotFound(String),

    #[error("Credentials invalid: {0}")]
    CredentialsInvalid(String),

    #[error("ACP error: {0}")]
    AcpError(String),
}

impl RouteError {
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
            Self::ConfigMissing(_) => 6001,
            Self::ConfigInvalid(_) => 6002,
            Self::ConfigLoadFailed(_) => 6003,
            Self::ValidationError { .. } => 7001,
            Self::MissingRequiredField(_) => 7002,
            Self::FormatMismatch(_) => 7003,
            Self::InternalError(_) => 9001,
            Self::ServiceUnavailable(_) => 9002,
            Self::ResourceNotFound { .. } => 4040,
            Self::BadRequest(_) => 4001,
            Self::StorageError(_) => 5002,
            Self::CheckpointError(_) => 5002,
            Self::ShareError(_) => 4001,
            Self::SummaryError(_) => 4001,
            Self::ForkError(_) => 4001,
            Self::CommandError(_) => 4001,
            Self::RevertError(_) => 4001,
            Self::CredentialsNotFound(_) => 4040,
            Self::CredentialsInvalid(_) => 401,
            Self::AcpError(_) => 6001,
        }
    }

    pub fn http_status(&self) -> actix_web::http::StatusCode {
        match self {
            Self::TokenExpired { .. }
            | Self::InvalidToken { .. }
            | Self::MissingCredentials { .. }
            | Self::CredentialsInvalid(_) => actix_web::http::StatusCode::UNAUTHORIZED,

            Self::InsufficientPermissions { .. } | Self::AccessDenied { .. } => {
                actix_web::http::StatusCode::FORBIDDEN
            }

            Self::ProviderNotFound(_)
            | Self::ProviderAuthFailed(_)
            | Self::ProviderUnavailable(_)
            | Self::SessionNotFound(_)
            | Self::SessionExpired(_)
            | Self::SessionCorrupted(_)
            | Self::ResourceNotFound { .. }
            | Self::CredentialsNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,

            Self::ToolNotFound(_)
            | Self::ToolTimeout { .. }
            | Self::ToolInvalidArgs(_)
            | Self::BadRequest(_)
            | Self::ShareError(_)
            | Self::SummaryError(_)
            | Self::ForkError(_)
            | Self::CommandError(_)
            | Self::RevertError(_) => actix_web::http::StatusCode::BAD_REQUEST,

            Self::ConfigMissing(_)
            | Self::ConfigInvalid(_)
            | Self::ConfigLoadFailed(_)
            | Self::ValidationError { .. }
            | Self::MissingRequiredField(_)
            | Self::FormatMismatch(_)
            | Self::AcpError(_) => actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,

            Self::InternalError(_)
            | Self::ServiceUnavailable(_)
            | Self::StorageError(_)
            | Self::CheckpointError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Self::TokenExpired { .. } => "token_expired",
            Self::InvalidToken { .. } => "invalid_token",
            Self::MissingCredentials { .. } => "missing_credentials",
            Self::InsufficientPermissions { .. } => "insufficient_permissions",
            Self::AccessDenied { .. } => "access_denied",
            Self::ProviderNotFound(_) => "provider_not_found",
            Self::ProviderAuthFailed(_) => "provider_auth_failed",
            Self::ProviderUnavailable(_) => "provider_unavailable",
            Self::ToolNotFound(_) => "tool_not_found",
            Self::ToolTimeout { .. } => "tool_timeout",
            Self::ToolInvalidArgs(_) => "tool_invalid_args",
            Self::SessionNotFound(_) => "session_not_found",
            Self::SessionExpired(_) => "session_expired",
            Self::SessionCorrupted(_) => "session_corrupted",
            Self::ConfigMissing(_) => "config_missing",
            Self::ConfigInvalid(_) => "config_invalid",
            Self::ConfigLoadFailed(_) => "config_load_failed",
            Self::ValidationError { .. } => "validation_error",
            Self::MissingRequiredField(_) => "missing_required_field",
            Self::FormatMismatch(_) => "format_mismatch",
            Self::InternalError(_) => "internal_error",
            Self::ServiceUnavailable(_) => "service_unavailable",
            Self::ResourceNotFound { .. } => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::StorageError(_) => "storage_error",
            Self::CheckpointError(_) => "checkpoint_error",
            Self::ShareError(_) => "share_error",
            Self::SummaryError(_) => "summary_error",
            Self::ForkError(_) => "fork_error",
            Self::CommandError(_) => "command_error",
            Self::RevertError(_) => "revert_error",
            Self::CredentialsNotFound(_) => "credentials_not_found",
            Self::CredentialsInvalid(_) => "credentials_invalid",
            Self::AcpError(_) => "acp_error",
        }
    }

    pub fn to_response(&self) -> HttpResponse {
        let error_response =
            ErrorResponse::new(self.error_type(), self.user_message(), self.code());
        HttpResponse::build(self.http_status()).json(error_response)
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::TokenExpired { detail } => detail
                .clone()
                .unwrap_or_else(|| "Your session has expired. Please log in again.".to_string()),
            Self::InvalidToken { detail } => detail
                .clone()
                .unwrap_or_else(|| "Invalid authentication token.".to_string()),
            Self::MissingCredentials { detail } => detail
                .clone()
                .unwrap_or_else(|| "Authentication credentials are missing.".to_string()),
            Self::InsufficientPermissions { detail, .. } => detail
                .clone()
                .unwrap_or_else(|| "Insufficient permissions.".to_string()),
            Self::AccessDenied { detail } => detail
                .clone()
                .unwrap_or_else(|| "Access denied.".to_string()),
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
            Self::ResourceNotFound { resource, id } => format!("{} not found: {}", resource, id),
            Self::BadRequest(msg) => msg.clone(),
            Self::StorageError(msg) => format!("Storage error: {}", msg),
            Self::CheckpointError(msg) => format!("Checkpoint error: {}", msg),
            Self::ShareError(msg) => format!("Share error: {}", msg),
            Self::SummaryError(msg) => format!("Summary error: {}", msg),
            Self::ForkError(msg) => format!("Fork error: {}", msg),
            Self::CommandError(msg) => format!("Command error: {}", msg),
            Self::RevertError(msg) => format!("Revert error: {}", msg),
            Self::CredentialsNotFound(provider) => {
                format!("No credentials found for provider: {}", provider)
            }
            Self::CredentialsInvalid(provider) => format!(
                "Credentials for provider {} are expired or invalid",
                provider
            ),
            Self::AcpError(msg) => format!("ACP error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<FieldError>,
}

#[allow(dead_code)]
impl ErrorResponse {
    pub(crate) fn new(error: impl Into<String>, message: impl Into<String>, code: u16) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code,
            details: Vec::new(),
        }
    }

    pub(crate) fn with_details(mut self, details: impl Into<Vec<FieldError>>) -> Self {
        self.details = details.into();
        self
    }

    pub(crate) fn authentication_error(message: impl Into<String>) -> Self {
        Self::new("authentication_error", message, 1001)
    }

    pub(crate) fn authorization_error(message: impl Into<String>) -> Self {
        Self::new("authorization_error", message, 2001)
    }

    pub(crate) fn not_found_error(resource: &str, id: &str) -> Self {
        Self::new("not_found", format!("{} not found: {}", resource, id), 4040)
    }

    pub(crate) fn session_not_found(id: &str) -> Self {
        Self::new(
            "session_not_found",
            format!("Session not found: {}", id),
            5001,
        )
    }

    pub(crate) fn validation_error(message: impl Into<String>) -> Self {
        Self::new("validation_error", message, 7001)
    }

    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalid_request", message, 4001)
    }

    pub(crate) fn config_error(message: impl Into<String>) -> Self {
        Self::new("config_error", message, 6001)
    }

    pub(crate) fn storage_error(message: impl Into<String>) -> Self {
        Self::new("storage_error", message, 5002)
    }

    pub(crate) fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message, 9001)
    }

    pub(crate) fn bad_request(error_type: &str, message: impl Into<String>) -> Self {
        Self::new(error_type, message, 4001)
    }

    pub(crate) fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("unauthorized", message, 1001)
    }

    pub(crate) fn forbidden(message: impl Into<String>) -> Self {
        Self::new("forbidden", message, 2001)
    }

    pub(crate) fn permission_denied(message: impl Into<String>) -> Self {
        Self::new("permission_denied", message, 2002)
    }

    pub(crate) fn to_response(&self, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(self)
    }
}

pub(crate) fn json_error(
    status: actix_web::http::StatusCode,
    error: &str,
    message: impl Into<String>,
) -> HttpResponse {
    HttpResponse::build(status).json(ErrorResponse::new(error, message, status.as_u16()))
}

pub(crate) fn bad_request(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::BAD_REQUEST).json(ErrorResponse::new(
        "bad_request",
        message,
        4001,
    ))
}

pub(crate) fn not_found(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::NOT_FOUND).json(ErrorResponse::new(
        "not_found",
        message,
        4040,
    ))
}

pub(crate) fn internal_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        .json(ErrorResponse::internal_error(message))
}

#[allow(dead_code)]
pub(crate) fn validation_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
        .json(ErrorResponse::validation_error(message))
}

pub(crate) fn unauthorized_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::UNAUTHORIZED)
        .json(ErrorResponse::unauthorized(message))
}

#[allow(dead_code)]
pub(crate) fn permission_denied_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::FORBIDDEN)
        .json(ErrorResponse::permission_denied(message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_without_details() {
        let err = ErrorResponse::session_not_found("test-id");
        let json = serde_json::to_string(&err).unwrap();
        assert!(!json.contains("\"details\""));
        assert!(json.contains("\"error\":\"session_not_found\""));
        assert!(json.contains("\"code\":5001"));
    }

    #[test]
    fn test_error_response_with_details() {
        let err = ErrorResponse::validation_error("validation failed").with_details(vec![
            FieldError::required("email"),
            FieldError::format("phone", "valid phone number"),
        ]);
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"details\""));
        assert!(json.contains("\"field\":\"email\""));
        assert!(json.contains("\"field\":\"phone\""));
    }

    #[test]
    fn test_field_error_required() {
        let err = FieldError::required("name");
        assert_eq!(err.field, "name");
        assert!(err.message.contains("required"));
        assert_eq!(err.code, 7002);
    }

    #[test]
    fn test_field_error_format() {
        let err = FieldError::format("email", "valid email");
        assert_eq!(err.field, "email");
        assert!(err.message.contains("invalid format"));
        assert_eq!(err.code, 7003);
    }

    #[test]
    fn test_error_response_to_response() {
        let err = ErrorResponse::session_not_found("123");
        let resp = err.to_response(actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_all_error_codes_are_consistent() {
        assert_eq!(ErrorResponse::authentication_error("test").code, 1001);
        assert_eq!(ErrorResponse::authorization_error("test").code, 2001);
        assert_eq!(ErrorResponse::validation_error("test").code, 7001);
        assert_eq!(ErrorResponse::invalid_request("test").code, 4001);
        assert_eq!(ErrorResponse::config_error("test").code, 6001);
        assert_eq!(ErrorResponse::storage_error("test").code, 5002);
        assert_eq!(ErrorResponse::internal_error("test").code, 9001);
        assert_eq!(ErrorResponse::session_not_found("test").code, 5001);
    }

    #[test]
    fn test_json_error_helper() {
        let resp = json_error(
            actix_web::http::StatusCode::BAD_REQUEST,
            "test_error",
            "Test message",
        );
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_bad_request_helper() {
        let resp = bad_request("Invalid input");
        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_helper() {
        let resp = not_found("Resource not found");
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_internal_error_helper() {
        let resp = internal_error("Something went wrong");
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_validation_error_helper() {
        let resp = validation_error("Validation failed");
        assert_eq!(
            resp.status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn api_error_all_have_same_json_structure() {
        let errors = vec![
            ErrorResponse::authentication_error("auth failed"),
            ErrorResponse::authorization_error("not allowed"),
            ErrorResponse::session_not_found("abc"),
            ErrorResponse::validation_error("invalid"),
            ErrorResponse::invalid_request("bad input"),
            ErrorResponse::config_error("config issue"),
            ErrorResponse::storage_error("storage issue"),
            ErrorResponse::internal_error("oops"),
            ErrorResponse::unauthorized("not authorized"),
            ErrorResponse::forbidden("forbidden"),
        ];

        for err in errors {
            let json = serde_json::to_string(&err).unwrap();
            assert!(
                json.contains("\"error\""),
                "Missing 'error' field in: {}",
                json
            );
            assert!(
                json.contains("\"message\""),
                "Missing 'message' field in: {}",
                json
            );
            assert!(
                json.contains("\"code\""),
                "Missing 'code' field in: {}",
                json
            );
        }
    }

    #[test]
    fn api_error_details_skipped_when_empty() {
        let err = ErrorResponse::session_not_found("test-id");
        let json = serde_json::to_string(&err).unwrap();
        assert!(
            !json.contains("\"details\""),
            "details should not be in JSON when empty: {}",
            json
        );
    }

    #[test]
    fn api_error_details_included_when_present() {
        let err = ErrorResponse::validation_error("multiple errors").with_details(vec![
            FieldError::required("field1"),
            FieldError::required("field2"),
        ]);
        let json = serde_json::to_string(&err).unwrap();
        assert!(
            json.contains("\"details\""),
            "details should be in JSON when present: {}",
            json
        );
        assert!(
            json.contains("\"field1\""),
            "field1 should be in details: {}",
            json
        );
        assert!(
            json.contains("\"field2\""),
            "field2 should be in details: {}",
            json
        );
    }

    #[test]
    fn api_error_code_ranges_are_consistent() {
        assert_eq!(ErrorResponse::authentication_error("test").code, 1001);
        assert!(
            ErrorResponse::authentication_error("test").code >= 1000
                && ErrorResponse::authentication_error("test").code < 2000
        );

        assert_eq!(ErrorResponse::authorization_error("test").code, 2001);
        assert!(
            ErrorResponse::authorization_error("test").code >= 2000
                && ErrorResponse::authorization_error("test").code < 3000
        );

        assert!(
            ErrorResponse::invalid_request("test").code >= 4000
                && ErrorResponse::invalid_request("test").code < 5000
        );
        assert!(
            ErrorResponse::validation_error("test").code >= 7000
                && ErrorResponse::validation_error("test").code < 8000
        );

        assert!(
            ErrorResponse::session_not_found("test").code >= 5000
                && ErrorResponse::session_not_found("test").code < 6000
        );
        assert!(
            ErrorResponse::storage_error("test").code >= 5000
                && ErrorResponse::storage_error("test").code < 6000
        );

        assert!(
            ErrorResponse::config_error("test").code >= 6000
                && ErrorResponse::config_error("test").code < 7000
        );

        assert!(
            ErrorResponse::internal_error("test").code >= 9000
                && ErrorResponse::internal_error("test").code < 10000
        );
    }

    #[test]
    fn api_error_field_error_has_consistent_structure() {
        let field_errors = vec![
            FieldError::required("field1"),
            FieldError::format("field2", "email"),
            FieldError::out_of_range("field3", Some(1), Some(10)),
            FieldError::too_long("field4", 100),
            FieldError::too_short("field5", 5),
            FieldError::invalid_enum("field6", &["a", "b"]),
        ];

        for err in field_errors {
            assert!(err.field.len() > 0, "Field name should not be empty");
            assert!(err.message.len() > 0, "Error message should not be empty");
            assert!(
                err.code >= 7001 && err.code <= 7003,
                "Validation error code should be 7001-7003, got {}",
                err.code
            );
        }
    }

    #[test]
    fn api_error_response_serialization_is_deterministic() {
        let err = ErrorResponse::validation_error("test message")
            .with_details(vec![FieldError::required("field1")]);

        let json1 = serde_json::to_string(&err).unwrap();
        let json2 = serde_json::to_string(&err).unwrap();
        assert_eq!(json1, json2, "Serialization should be deterministic");
    }

    #[test]
    fn test_route_error_authentication_maps_to_401() {
        let err = RouteError::TokenExpired { detail: None };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.code(), 1001);

        let err = RouteError::InvalidToken { detail: None };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.code(), 1002);

        let err = RouteError::MissingCredentials { detail: None };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.code(), 1003);
    }

    #[test]
    fn test_route_error_authorization_maps_to_403() {
        let err = RouteError::InsufficientPermissions { detail: None };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::FORBIDDEN);
        assert_eq!(err.code(), 2001);

        let err = RouteError::AccessDenied { detail: None };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::FORBIDDEN);
        assert_eq!(err.code(), 2002);
    }

    #[test]
    fn test_route_error_provider_maps_to_404() {
        let err = RouteError::ProviderNotFound("openai".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 3001);

        let err = RouteError::ProviderAuthFailed("openai".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 3002);

        let err = RouteError::ProviderUnavailable("openai".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 3003);
    }

    #[test]
    fn test_route_error_tool_maps_to_400() {
        let err = RouteError::ToolNotFound("read".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), 4001);

        let err = RouteError::ToolTimeout {
            tool: "read".into(),
            timeout_ms: 5000,
        };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), 4002);

        let err = RouteError::ToolInvalidArgs("invalid".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), 4003);
    }

    #[test]
    fn test_route_error_session_maps_to_404() {
        let err = RouteError::SessionNotFound("abc".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 5001);

        let err = RouteError::SessionExpired("abc".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 5002);

        let err = RouteError::SessionCorrupted("abc".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 5003);
    }

    #[test]
    fn test_route_error_config_maps_to_422() {
        let err = RouteError::ConfigMissing("key".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 6001);

        let err = RouteError::ConfigInvalid("key".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 6002);

        let err = RouteError::ConfigLoadFailed("key".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 6003);
    }

    #[test]
    fn test_route_error_validation_maps_to_422() {
        let err = RouteError::ValidationError {
            field: "name".into(),
            message: "required".into(),
        };
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 7001);

        let err = RouteError::MissingRequiredField("email".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 7002);

        let err = RouteError::FormatMismatch("json".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 7003);
    }

    #[test]
    fn test_route_error_internal_maps_to_500() {
        let err = RouteError::InternalError("test".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(err.code(), 9001);

        let err = RouteError::ServiceUnavailable("test".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(err.code(), 9002);
    }

    #[test]
    fn test_route_error_bad_request_maps_to_400() {
        let err = RouteError::BadRequest("invalid input".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), 4001);

        let err = RouteError::ShareError("share failed".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), 4001);

        let err = RouteError::ForkError("fork failed".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert_eq!(err.code(), 4001);
    }

    #[test]
    fn test_route_error_not_found_maps_to_404() {
        let err = RouteError::ResourceNotFound {
            resource: "session".into(),
            id: "123".into(),
        };
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 4040);

        let err = RouteError::CredentialsNotFound("provider".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        assert_eq!(err.code(), 4040);
    }

    #[test]
    fn test_route_error_credentials_invalid_maps_to_401() {
        let err = RouteError::CredentialsInvalid("provider".into());
        assert_eq!(err.http_status(), actix_web::http::StatusCode::UNAUTHORIZED);
        assert_eq!(err.code(), 401);
    }

    #[test]
    fn test_route_error_to_response() {
        let err = RouteError::SessionNotFound("test-id".into());
        let resp = err.to_response();
        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_route_error_user_message() {
        let err = RouteError::TokenExpired { detail: None };
        assert!(err.user_message().contains("expired"));

        let err = RouteError::ProviderNotFound("openai".into());
        assert!(err.user_message().contains("openai"));

        let err = RouteError::SessionNotFound("abc".into());
        assert!(err.user_message().contains("abc"));
    }

    #[test]
    fn test_route_error_error_type() {
        assert_eq!(
            RouteError::TokenExpired { detail: None }.error_type(),
            "token_expired"
        );
        assert_eq!(
            RouteError::InvalidToken { detail: None }.error_type(),
            "invalid_token"
        );
        assert_eq!(
            RouteError::SessionNotFound("x".into()).error_type(),
            "session_not_found"
        );
        assert_eq!(
            RouteError::ValidationError {
                field: "x".into(),
                message: "y".into()
            }
            .error_type(),
            "validation_error"
        );
    }

    #[test]
    fn test_route_error_all_variants_have_valid_http_status() {
        let variants = vec![
            RouteError::TokenExpired { detail: None },
            RouteError::InvalidToken { detail: None },
            RouteError::MissingCredentials { detail: None },
            RouteError::InsufficientPermissions { detail: None },
            RouteError::AccessDenied { detail: None },
            RouteError::ProviderNotFound("x".into()),
            RouteError::ProviderAuthFailed("x".into()),
            RouteError::ProviderUnavailable("x".into()),
            RouteError::ToolNotFound("x".into()),
            RouteError::ToolTimeout {
                tool: "x".into(),
                timeout_ms: 5000,
            },
            RouteError::ToolInvalidArgs("x".into()),
            RouteError::SessionNotFound("x".into()),
            RouteError::SessionExpired("x".into()),
            RouteError::SessionCorrupted("x".into()),
            RouteError::ConfigMissing("x".into()),
            RouteError::ConfigInvalid("x".into()),
            RouteError::ConfigLoadFailed("x".into()),
            RouteError::ValidationError {
                field: "x".into(),
                message: "y".into(),
            },
            RouteError::MissingRequiredField("x".into()),
            RouteError::FormatMismatch("x".into()),
            RouteError::InternalError("x".into()),
            RouteError::ServiceUnavailable("x".into()),
            RouteError::ResourceNotFound {
                resource: "x".into(),
                id: "y".into(),
            },
            RouteError::BadRequest("x".into()),
            RouteError::StorageError("x".into()),
            RouteError::CheckpointError("x".into()),
            RouteError::ShareError("x".into()),
            RouteError::SummaryError("x".into()),
            RouteError::ForkError("x".into()),
            RouteError::CommandError("x".into()),
            RouteError::RevertError("x".into()),
            RouteError::CredentialsNotFound("x".into()),
            RouteError::CredentialsInvalid("x".into()),
            RouteError::AcpError("x".into()),
        ];

        for err in variants {
            let status = err.http_status();
            assert!(
                status.is_success() == false && status.as_u16() >= 400 && status.as_u16() < 600,
                "HTTP status for {:?} should be in 4xx-5xx range, got: {}",
                err,
                status
            );
        }
    }

    #[test]
    fn test_route_error_all_variants_have_valid_codes() {
        let variants = vec![
            RouteError::TokenExpired { detail: None },
            RouteError::InvalidToken { detail: None },
            RouteError::MissingCredentials { detail: None },
            RouteError::InsufficientPermissions { detail: None },
            RouteError::AccessDenied { detail: None },
            RouteError::ProviderNotFound("x".into()),
            RouteError::ProviderAuthFailed("x".into()),
            RouteError::ProviderUnavailable("x".into()),
            RouteError::ToolNotFound("x".into()),
            RouteError::ToolTimeout {
                tool: "x".into(),
                timeout_ms: 5000,
            },
            RouteError::ToolInvalidArgs("x".into()),
            RouteError::SessionNotFound("x".into()),
            RouteError::SessionExpired("x".into()),
            RouteError::SessionCorrupted("x".into()),
            RouteError::ConfigMissing("x".into()),
            RouteError::ConfigInvalid("x".into()),
            RouteError::ConfigLoadFailed("x".into()),
            RouteError::ValidationError {
                field: "x".into(),
                message: "y".into(),
            },
            RouteError::MissingRequiredField("x".into()),
            RouteError::FormatMismatch("x".into()),
            RouteError::InternalError("x".into()),
            RouteError::ServiceUnavailable("x".into()),
            RouteError::ResourceNotFound {
                resource: "x".into(),
                id: "y".into(),
            },
            RouteError::BadRequest("x".into()),
            RouteError::StorageError("x".into()),
            RouteError::CheckpointError("x".into()),
            RouteError::ShareError("x".into()),
            RouteError::SummaryError("x".into()),
            RouteError::ForkError("x".into()),
            RouteError::CommandError("x".into()),
            RouteError::RevertError("x".into()),
            RouteError::CredentialsNotFound("x".into()),
            RouteError::CredentialsInvalid("x".into()),
            RouteError::AcpError("x".into()),
        ];

        for err in variants {
            let code = err.code();
            let http_status = err.http_status();
            assert!(
                (code >= 1000 && code < 10000) || code == 401 || code == 4040,
                "Error code for {:?} should be in 1xxx-9xxx range or 401/4040, got: {}",
                err,
                code
            );
            assert!(
                http_status.as_u16() >= 400 && http_status.as_u16() < 600,
                "HTTP status for {:?} should be in 4xx-5xx range, got: {}",
                err,
                http_status
            );
        }
    }

    #[test]
    fn test_route_error_display_format() {
        let err = RouteError::TokenExpired { detail: None };
        assert!(err.to_string().contains("Token expired"));

        let err = RouteError::ProviderNotFound("openai".into());
        assert!(err.to_string().contains("Provider not found"));
        assert!(err.to_string().contains("openai"));

        let err = RouteError::ToolTimeout {
            tool: "bash".into(),
            timeout_ms: 30000,
        };
        assert!(err.to_string().contains("Tool execution timeout"));
        assert!(err.to_string().contains("bash"));
        assert!(err.to_string().contains("30000"));
    }

    #[test]
    fn test_route_error_storage_and_checkpoint_map_to_500() {
        let err = RouteError::StorageError("db connection failed".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(err.code(), 5002);

        let err = RouteError::CheckpointError("checkpoint failed".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(err.code(), 5002);
    }

    #[test]
    fn test_route_error_acp_maps_to_422() {
        let err = RouteError::AcpError("acp is disabled".into());
        assert_eq!(
            err.http_status(),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(err.code(), 6001);
    }
}
