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
}

pub fn json_error(
    status: actix_web::http::StatusCode,
    error: &str,
    message: impl Into<String>,
) -> HttpResponse {
    HttpResponse::build(status).json(ErrorResponse::new(error, message, status.as_u16()))
}
