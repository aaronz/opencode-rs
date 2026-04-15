//! Execute routes module for agent execution endpoints.
//!
//! This module provides the `/api/sessions/{id}/execute` endpoint for full agent
//! execution with tool access via HTTP API.

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::routes::error::{bad_request, json_error};
use crate::routes::validation::RequestValidator;
use crate::ServerState;

/// Request body for execute endpoint
#[derive(Debug, Deserialize)]
pub struct ExecuteRequest {
    pub prompt: String,
    pub mode: Option<String>,
    #[serde(default = "default_stream")]
    pub stream: bool,
}

fn default_stream() -> bool {
    true
}

/// Response for execute endpoint
#[derive(Debug, Serialize)]
pub struct ExecuteResponse {
    pub session_id: String,
    pub status: String,
    pub message_count: usize,
}

/// Handler for POST /sessions/{id}/execute
///
/// Executes an agent prompt with full tool access.
pub async fn execute_session(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    req: web::Json<ExecuteRequest>,
) -> impl Responder {
    let session_id = id.into_inner();

    // Validate request
    let mut validator = RequestValidator::new();
    validator.validate_required_string("prompt", Some(&req.prompt));
    if let Some(ref mode) = req.mode {
        validator.validate_enum("mode", mode, &["build", "plan", "general"]);
    }
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    // Load session
    let mut session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return json_error(
                actix_web::http::StatusCode::NOT_FOUND,
                "session_not_found",
                format!("Session not found: {}", session_id),
            );
        }
        Err(e) => {
            return json_error(
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            );
        }
    };

    // Add user prompt as message
    let message = opencode_core::Message::user(req.prompt.clone());
    session.add_message(message);

    // Save updated session
    if let Err(e) = state.storage.save_session(&session).await {
        return json_error(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        );
    }

    // Return response (actual execution will be implemented in FR-024)
    HttpResponse::Ok().json(ExecuteResponse {
        session_id: session.id.to_string(),
        status: "pending_execution".to_string(),
        message_count: session.messages.len(),
    })
}

/// Configure execute routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/{id}/execute", web::post().to(execute_session));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_request_deserialization() {
        let json = r#"{"prompt": "hello world"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "hello world");
        assert_eq!(req.mode, None);
        assert!(req.stream);
    }

    #[test]
    fn test_execute_request_with_mode() {
        let json = r#"{"prompt": "hello", "mode": "build", "stream": false}"#;
        let req: ExecuteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "hello");
        assert_eq!(req.mode, Some("build".to_string()));
        assert!(!req.stream);
    }

    #[test]
    fn test_execute_response_serialization() {
        let response = ExecuteResponse {
            session_id: "test-123".to_string(),
            status: "pending_execution".to_string(),
            message_count: 5,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-123"));
        assert!(json.contains("pending_execution"));
    }
}