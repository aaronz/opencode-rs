use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: u16,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>, code: u16) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code,
        }
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
