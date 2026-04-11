use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: u16,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>, code: u16) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code,
        }
    }

    pub fn with_code(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: 7001,
        }
    }

    pub fn required(field: impl Into<String>) -> Self {
        let f = field.into();
        Self::new(f.clone(), format!("Field '{}' is required", f), 7002)
    }

    pub fn format(field: impl Into<String>, expected: impl Into<String>) -> Self {
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

    pub fn out_of_range(field: impl Into<String>, min: Option<usize>, max: Option<usize>) -> Self {
        let f = field.into();
        let message = match (min, max) {
            (Some(min), Some(max)) => format!("Field '{}' must be between {} and {}", f, min, max),
            (Some(min), None) => format!("Field '{}' must be at least {}", f, min),
            (None, Some(max)) => format!("Field '{}' must be at most {}", f, max),
            (None, None) => format!("Field '{}' has invalid value", f),
        };
        Self::new(f, message, 7001)
    }

    pub fn too_long(field: impl Into<String>, max_length: usize) -> Self {
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

    pub fn too_short(field: impl Into<String>, min_length: usize) -> Self {
        let f = field.into();
        Self::new(
            f.clone(),
            format!("Field '{}' must be at least {} characters", f, min_length),
            7001,
        )
    }

    pub fn invalid_enum(field: impl Into<String>, valid_values: &[&str]) -> Self {
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

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<FieldError>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>, code: u16) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code,
            details: Vec::new(),
        }
    }

    pub fn with_details(mut self, details: impl Into<Vec<FieldError>>) -> Self {
        self.details = details.into();
        self
    }

    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self::new("authentication_error", message, 1001)
    }

    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::new("authorization_error", message, 2001)
    }

    pub fn not_found_error(resource: &str, id: &str) -> Self {
        Self::new("not_found", format!("{} not found: {}", resource, id), 4040)
    }

    pub fn session_not_found(id: &str) -> Self {
        Self::new(
            "session_not_found",
            format!("Session not found: {}", id),
            5001,
        )
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("validation_error", message, 7001)
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("invalid_request", message, 4001)
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::new("config_error", message, 6001)
    }

    pub fn storage_error(message: impl Into<String>) -> Self {
        Self::new("storage_error", message, 5002)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("internal_error", message, 9001)
    }

    pub fn bad_request(error_type: &str, message: impl Into<String>) -> Self {
        Self::new(error_type, message, 4001)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("unauthorized", message, 1001)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new("forbidden", message, 2001)
    }

    pub fn to_response(self, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(self)
    }
}

pub fn json_error(
    status: actix_web::http::StatusCode,
    error: &str,
    message: impl Into<String>,
) -> HttpResponse {
    HttpResponse::build(status).json(ErrorResponse::new(error, message, status.as_u16()))
}

pub fn bad_request(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::BAD_REQUEST).json(ErrorResponse::new(
        "bad_request",
        message,
        4001,
    ))
}

pub fn not_found(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::NOT_FOUND).json(ErrorResponse::new(
        "not_found",
        message,
        4040,
    ))
}

pub fn internal_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
        .json(ErrorResponse::internal_error(message))
}

pub fn validation_error(message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY)
        .json(ErrorResponse::validation_error(message))
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
}
