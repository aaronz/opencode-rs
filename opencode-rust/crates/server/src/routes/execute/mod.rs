//! Execute route handler module for session execution endpoint.
//!
//! Provides the `POST /api/sessions/{id}/execute` endpoint for full agent execution
//! with tool access via HTTP API.

pub mod integration;
pub mod stream;
pub mod types;

use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::routes::error::{json_error, unauthorized_error, validation_error};
use crate::routes::validation::{validate_session_id, RequestValidator};
use crate::ServerState;

use integration::{execute_agent_loop, system_prompt_for_mode, ExecutionContext};
use stream::{execute_event_stream, format_sse_event};
use types::{ExecuteEvent, ExecuteMode, ExecuteRequest};

fn check_auth(req: &HttpRequest, state: &ServerState) -> Result<(), HttpResponse> {
    let config_guard = match state.config.read() {
        Ok(cfg) => cfg,
        Err(_) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_error",
                "Failed to access server config",
            ));
        }
    };

    let Some(expected_key) = config_guard.api_key.as_ref() else {
        return Ok(());
    };

    if expected_key.is_empty() {
        return Ok(());
    }

    let authorized = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|provided| provided == expected_key)
        .unwrap_or(false);

    if authorized {
        Ok(())
    } else {
        Err(unauthorized_error("Invalid or missing API key"))
    }
}

/// Execute session endpoint handler.
/// POST /api/sessions/{id}/execute
///
/// Accepts a prompt and executes the agent with tool access.
/// Returns SSE stream of execution events.
pub async fn execute_session(
    state: web::Data<ServerState>,
    req: HttpRequest,
    id: web::Path<String>,
    body: web::Json<ExecuteRequest>,
) -> impl Responder {
    if let Err(resp) = check_auth(&req, &state) {
        return resp;
    }

    let session_id = id.as_str();

    if let Err(errors) = validate_session_id(session_id) {
        return errors.to_response();
    }

    let mut validator = RequestValidator::new();
    validator.validate_required_string("prompt", Some(&body.prompt));
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    let mut session = match state.storage.load_session(session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return json_error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                format!("Session not found: {}", session_id),
            );
        }
        Err(e) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            );
        }
    };

    // Get provider from state - use config to determine provider
    let config = match state.config.read() {
        Ok(cfg) => cfg.clone(),
        Err(_) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_error",
                "Failed to access server config",
            );
        }
    };

    let model = config.model.clone().unwrap_or_else(|| "gpt-4o".to_string());
    let provider_id = "openai".to_string(); // Default provider

    // Build provider - this is simplified; full impl would need provider resolution
    let provider = match crate::routes::run::build_provider(&provider_id, &model, &config) {
        Ok(p) => p,
        Err(message) => {
            return json_error(StatusCode::BAD_REQUEST, "provider_init_error", message);
        }
    };

    // Get agent type from mode
    let agent_type = match body.mode.unwrap_or(ExecuteMode::General) {
        ExecuteMode::Build => opencode_agent::AgentType::Build,
        ExecuteMode::Plan => opencode_agent::AgentType::Plan,
        ExecuteMode::General => opencode_agent::AgentType::General,
    };

    // Create execution context - use tool_registry from application state
    let ctx = ExecutionContext::new(
        state.tool_registry.clone(),
        std::sync::Arc::from(provider),
        agent_type,
    );

    // Create agent
    let agent = ctx.create_agent();

    // Add user message to session
    session.add_message(opencode_core::Message::user(&body.prompt));

    // Execute the agent loop
    let events: Vec<ExecuteEvent> = match execute_agent_loop(
        &mut session,
        agent.as_ref(),
        ctx.provider.as_ref(),
        ctx.tool_registry.as_ref(),
        ctx.max_iterations,
        ctx.max_tool_results_per_iteration,
    )
    .await
    {
        Ok(response) => {
            vec![
                ExecuteEvent::message("assistant", &response.content),
                ExecuteEvent::complete(serde_json::json!({
                    "session_id": session.id.to_string(),
                    "message_count": session.messages.len(),
                })),
            ]
        }
        Err(opencode_core::OpenCodeError::InternalError(msg)) => {
            vec![ExecuteEvent::error("INTERNAL_ERROR", msg)]
        }
        Err(e) => {
            vec![ExecuteEvent::error("EXECUTION_ERROR", e.to_string())]
        }
    };

    // Save session after execution
    if let Err(e) = state.storage.save_session(&session).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            format!("Failed to save session: {}", e),
        );
    }

    // Return SSE stream
    let stream = execute_event_stream(events);
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}

/// Health check for execute endpoint - returns 200 if module is loaded.
pub async fn execute_health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "endpoint": "execute"
    }))
}

/// Configure routes for execute module.
/// Registers the execute route under /{id}/execute.
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/execute", web::post().to(execute_session));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create ExecuteRequest for testing
    fn create_test_request(prompt: &str, mode: Option<ExecuteMode>) -> ExecuteRequest {
        ExecuteRequest {
            prompt: prompt.to_string(),
            mode,
            stream: Some(true),
        }
    }

    #[test]
    fn test_execute_request_parsing_minimal() {
        // Test minimal request - just prompt
        let json = r#"{"prompt": "Hello, world!"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.prompt, "Hello, world!");
        assert!(req.mode.is_none());
        assert_eq!(req.stream, Some(true));
    }

    #[test]
    fn test_execute_request_parsing_with_mode() {
        // Test with mode specified
        let json = r#"{"prompt": "Test prompt", "mode": "build"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.prompt, "Test prompt");
        assert_eq!(req.mode, Some(ExecuteMode::Build));

        let json = r#"{"prompt": "Test prompt", "mode": "plan"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.mode, Some(ExecuteMode::Plan));

        let json = r#"{"prompt": "Test prompt", "mode": "general"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.mode, Some(ExecuteMode::General));
    }

    #[test]
    fn test_execute_request_parsing_with_stream() {
        // Test stream field
        let json = r#"{"prompt": "Test", "stream": true}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.stream, Some(true));

        let json = r#"{"prompt": "Test", "stream": false}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.stream, Some(false));
    }

    #[test]
    fn test_execute_request_parsing_all_fields() {
        // Test with all fields
        let json = r#"{"prompt": "Full request", "mode": "build", "stream": false}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should parse");
        assert_eq!(req.prompt, "Full request");
        assert_eq!(req.mode, Some(ExecuteMode::Build));
        assert_eq!(req.stream, Some(false));
    }

    #[test]
    fn test_execute_request_serialization_roundtrip() {
        // Test that serialization -> deserialization works
        let req = create_test_request("Roundtrip test", Some(ExecuteMode::Plan));
        let json = serde_json::to_string(&req).expect("should serialize");
        let parsed: ExecuteRequest = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.prompt, req.prompt);
        assert_eq!(parsed.mode, req.mode);
        assert_eq!(parsed.stream, req.stream);
    }

    #[test]
    fn test_execute_mode_system_prompt() {
        // Verify system prompts are set correctly
        let build = ExecuteMode::Build;
        assert!(build.system_prompt().contains("BUILD"));

        let plan = ExecuteMode::Plan;
        assert!(plan.system_prompt().contains("PLAN"));

        let general = ExecuteMode::General;
        assert!(general.system_prompt().contains("GENERAL"));
    }

    #[test]
    fn test_execute_mode_from_agent_type() {
        use opencode_agent::AgentType;
        use std::convert::TryFrom;

        assert_eq!(
            AgentType::from(ExecuteMode::Build),
            opencode_agent::AgentType::Build
        );
        assert_eq!(
            AgentType::from(ExecuteMode::Plan),
            opencode_agent::AgentType::Plan
        );
        assert_eq!(
            AgentType::from(ExecuteMode::General),
            opencode_agent::AgentType::General
        );
    }

    #[test]
    fn test_execute_event_creation() {
        // Test creating various event types
        let tool_call =
            ExecuteEvent::tool_call("read", serde_json::json!({"path": "/test"}), "call-1");
        match tool_call {
            ExecuteEvent::ToolCall {
                tool,
                params,
                call_id,
            } => {
                assert_eq!(tool, "read");
                assert_eq!(call_id, "call-1");
            }
            _ => panic!("Expected ToolCall variant"),
        }

        let message = ExecuteEvent::message("assistant", "Hello");
        match message {
            ExecuteEvent::Message { role, content } => {
                assert_eq!(role, "assistant");
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected Message variant"),
        }

        let error = ExecuteEvent::error("ERR_CODE", "Error message");
        match error {
            ExecuteEvent::Error { code, message } => {
                assert_eq!(code, "ERR_CODE");
                assert_eq!(message, "Error message");
            }
            _ => panic!("Expected Error variant"),
        }

        let complete = ExecuteEvent::complete(serde_json::json!({"status": "done"}));
        match complete {
            ExecuteEvent::Complete { session_state } => {
                assert_eq!(session_state["status"], "done");
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    #[test]
    fn test_execute_event_serialization() {
        // Test that events serialize to JSON correctly
        let event = ExecuteEvent::tool_call("grep", serde_json::json!({"pattern": "test"}), "c1");
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains(r#""event":"tool_call""#));
        assert!(json.contains(r#""tool":"grep""#));
        assert!(json.contains(r#""call_id":"c1""#));
    }

    #[test]
    fn test_system_prompt_for_mode() {
        use integration::system_prompt_for_mode;

        assert!(system_prompt_for_mode(ExecuteMode::Build).contains("BUILD"));
        assert!(system_prompt_for_mode(ExecuteMode::Plan).contains("PLAN"));
        assert!(system_prompt_for_mode(ExecuteMode::General).contains("GENERAL"));
    }

    #[test]
    fn test_validate_session_id_format() {
        use crate::routes::validation::validate_session_id;

        // Valid UUIDs
        assert!(validate_session_id("550e8400-e29b-41d4-a716-446655440000").is_ok());

        // Invalid formats
        assert!(validate_session_id("").is_err());
        assert!(validate_session_id("not-a-uuid").is_err());
        assert!(validate_session_id("123").is_err());
    }

    #[test]
    fn test_error_responses() {
        // Test that error helper functions return correct status codes
        let not_found_resp = json_error(
            StatusCode::NOT_FOUND,
            "session_not_found",
            "Session not found",
        );
        assert_eq!(not_found_resp.status(), StatusCode::NOT_FOUND);

        let bad_req_resp = json_error(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Invalid request",
        );
        assert_eq!(bad_req_resp.status(), StatusCode::BAD_REQUEST);

        let server_err_resp = json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Internal error",
        );
        assert_eq!(server_err_resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_validation_error() {
        let resp = validation_error("Validation failed");
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn test_unauthorized_error() {
        let resp = unauthorized_error("Invalid or missing API key");
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_execute_request_validation_rejects_empty_prompt() {
        let mut validator = RequestValidator::new();
        validator.validate_required_string("prompt", Some(""));
        assert!(!validator.is_valid());
        assert!(validator.errors().len() == 1);
    }

    #[test]
    fn test_execute_request_validation_rejects_whitespace_prompt() {
        let mut validator = RequestValidator::new();
        validator.validate_required_string("prompt", Some("   \t\n  "));
        assert!(!validator.is_valid());
        assert!(validator.errors().len() == 1);
    }

    #[test]
    fn test_execute_request_validation_accepts_valid_prompt() {
        let mut validator = RequestValidator::new();
        validator.validate_required_string("prompt", Some("Hello, world!"));
        assert!(validator.is_valid());
    }

    #[test]
    fn test_execute_request_validation_rejects_missing_prompt() {
        let mut validator = RequestValidator::new();
        validator.validate_required_string("prompt", None);
        assert!(!validator.is_valid());
        assert!(validator.errors().len() == 1);
    }

    #[test]
    fn test_execute_request_validation_rejects_too_long_prompt() {
        let mut validator = RequestValidator::new();
        let long_prompt = "a".repeat(10001);
        validator.validate_required_string("prompt", Some(&long_prompt));
        assert!(!validator.is_valid());
    }

    #[test]
    fn test_execute_session_id_validation_rejects_invalid_uuid() {
        let result = validate_session_id("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_session_id_validation_accepts_valid_uuid() {
        let result = validate_session_id("550e8400-e29b-41d4-a716-446655440000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_session_id_validation_rejects_empty_string() {
        let result = validate_session_id("");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_session_id_validation_rejects_short_string() {
        let result = validate_session_id("123");
        assert!(result.is_err());
    }
}
