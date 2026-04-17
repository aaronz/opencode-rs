use crate::routes::error::json_error;
use crate::routes::error::RouteError;
use crate::routes::execute::integration::{execute_agent_loop, ExecutionContext};
use crate::routes::execute::stream::execute_event_stream;
use crate::routes::execute::types::ExecuteEvent;
use crate::routes::validation::RequestValidator;
use crate::ServerState;
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures::stream::Stream;
use opencode_agent::{Agent, AgentType};
use opencode_core::{Message, OpenCodeError, Session};
use opencode_llm::{AnthropicProvider, OllamaProvider, OpenAiProvider, Provider};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub stream: bool,
}

fn resolve_model_and_provider(
    state: &ServerState,
    configured_model: Option<String>,
) -> (String, String) {
    let mut model = configured_model.unwrap_or_else(|| "gpt-4o".to_string());

    if let Some((provider, model_name)) = model.split_once('/') {
        return (provider.to_string(), model_name.to_string());
    }

    if let Some(model_info) = state.models.get(&model) {
        return (model_info.provider.clone(), model);
    }

    ("openai".to_string(), std::mem::take(&mut model))
}

fn api_key_for_provider(config: &opencode_core::Config, provider_id: &str) -> Option<String> {
    config
        .get_provider(provider_id)
        .and_then(|provider| provider.options.as_ref())
        .and_then(|options| options.api_key.clone())
        .or_else(|| config.api_key.clone())
        .or_else(|| match provider_id {
            "openai" => std::env::var("OPENAI_API_KEY").ok(),
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
            _ => None,
        })
}

fn ollama_base_url(config: &opencode_core::Config) -> Option<String> {
    config
        .get_provider("ollama")
        .and_then(|provider| provider.options.as_ref())
        .and_then(|options| options.base_url.clone())
        .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
}

pub fn build_provider(
    provider_id: &str,
    model: &str,
    config: &opencode_core::Config,
) -> Result<Box<dyn Provider + Send + Sync>, RouteError> {
    match provider_id {
        "anthropic" => {
            let key = api_key_for_provider(config, "anthropic")
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| {
                    RouteError::ProviderAuthFailed("Missing Anthropic API key".into())
                })?;
            Ok(Box::new(AnthropicProvider::new(key, model.to_string())))
        }
        "ollama" => Ok(Box::new(OllamaProvider::new(
            model.to_string(),
            ollama_base_url(config),
        ))),
        _ => {
            let key = api_key_for_provider(config, "openai")
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| RouteError::ProviderAuthFailed("Missing OpenAI API key".into()))?;
            Ok(Box::new(OpenAiProvider::new(key, model.to_string())))
        }
    }
}

fn accepts_sse(req: &HttpRequest) -> bool {
    if let Some(accept) = req.headers().get("Accept") {
        if let Ok(accept_str) = accept.to_str() {
            return accept_str.contains("text/event-stream");
        }
    }
    false
}

fn agent_type_from_string(agent_str: &str) -> AgentType {
    match agent_str.to_ascii_lowercase().as_str() {
        "build" => AgentType::Build,
        "plan" => AgentType::Plan,
        "explore" => AgentType::Explore,
        "review" => AgentType::Review,
        "refactor" => AgentType::Refactor,
        "debug" => AgentType::Debug,
        _ => AgentType::General,
    }
}

async fn run_prompt_with_agent_execution(
    state: web::Data<ServerState>,
    req: web::Json<RunRequest>,
    session_id: String,
) -> Result<Vec<ExecuteEvent>, HttpResponse> {
    let config = match state.config.read() {
        Ok(cfg) => cfg.clone(),
        Err(_) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_lock_error",
                "Failed to access server config",
            ));
        }
    };

    let selected_model = req
        .model
        .clone()
        .or_else(|| config.model.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());
    let (provider_id, model_name) =
        resolve_model_and_provider(&state, Some(selected_model.clone()));
    let provider = match build_provider(&provider_id, &model_name, &config) {
        Ok(provider) => provider,
        Err(err) => {
            return Err(err.to_response());
        }
    };

    let selected_agent = req.agent.clone().unwrap_or_else(|| "general".to_string());
    let agent_type = agent_type_from_string(&selected_agent);

    let ctx = ExecutionContext::new(state.tool_registry.clone(), Arc::from(provider), agent_type);

    let agent = ctx.create_agent();

    let mut session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Err(json_error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                format!("Session not found: {}", session_id),
            ));
        }
        Err(e) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                e.to_string(),
            ));
        }
    };

    let mut events = Vec::new();

    match execute_agent_loop(
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
            events.push(ExecuteEvent::message("assistant", &response.content));
            events.push(ExecuteEvent::complete(serde_json::json!({
                "session_id": session.id.to_string(),
                "message_count": session.messages.len(),
            })));
        }
        Err(OpenCodeError::InternalError(msg)) => {
            events.push(ExecuteEvent::error("INTERNAL_ERROR", msg));
        }
        Err(e) => {
            events.push(ExecuteEvent::error("EXECUTION_ERROR", e.to_string()));
        }
    }

    if let Err(e) = state.storage.save_session(&session).await {
        return Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            format!("Failed to save session: {}", e),
        ));
    }

    Ok(events)
}

async fn run_prompt_streaming(
    state: web::Data<ServerState>,
    req: web::Json<RunRequest>,
    session_id: String,
) -> HttpResponse {
    let events_result = run_prompt_with_agent_execution(state, req, session_id).await;

    match events_result {
        Ok(events) => {
            let stream = execute_event_stream(events);
            HttpResponse::Ok()
                .content_type("text/event-stream")
                .insert_header(("Cache-Control", "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .insert_header(("Access-Control-Allow-Origin", "*"))
                .insert_header(("X-Accel-Buffering", "no"))
                .streaming(stream)
        }
        Err(resp) => resp,
    }
}

pub async fn run_prompt(
    state: web::Data<ServerState>,
    req: HttpRequest,
    body: web::Json<RunRequest>,
) -> impl Responder {
    let mut validator = RequestValidator::new();
    validator.validate_required_string("prompt", Some(&body.prompt));
    if let Some(ref agent) = body.agent {
        validator.validate_enum(
            "agent",
            agent,
            &[
                "build", "plan", "explore", "review", "refactor", "debug", "general",
            ],
        );
    }
    if let Some(ref model) = body.model {
        validator.validate_optional_string("model", Some(model), 100);
    }
    if let Err(errors) = validator.validate() {
        return errors.to_response();
    }

    let mut session = Session::new();
    session.add_message(Message::user(&body.prompt));
    if let Err(e) = state.storage.save_session(&session).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        );
    }

    if body.stream || accepts_sse(&req) {
        return run_prompt_streaming(state, body, session.id.to_string()).await;
    }

    let events_result = run_prompt_with_agent_execution(state, body, session.id.to_string()).await;

    match events_result {
        Ok(events) => {
            let stream = execute_event_stream(events);
            HttpResponse::Ok()
                .content_type("text/event-stream")
                .insert_header(("Cache-Control", "no-cache"))
                .insert_header(("Connection", "keep-alive"))
                .insert_header(("Access-Control-Allow-Origin", "*"))
                .insert_header(("X-Accel-Buffering", "no"))
                .streaming(stream)
        }
        Err(resp) => resp,
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(run_prompt));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_request_deserialize_basic() {
        let json = r#"{"prompt": "hello", "model": "gpt-4", "agent": "build"}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "hello");
        assert_eq!(req.model, Some("gpt-4".to_string()));
        assert_eq!(req.agent, Some("build".to_string()));
        assert!(!req.stream);
    }

    #[test]
    fn test_run_request_deserialize_with_stream() {
        let json = r#"{"prompt": "hello", "stream": true}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "hello");
        assert!(req.stream);
    }

    #[test]
    fn test_run_request_deserialize_stream_defaults_to_false() {
        let json = r#"{"prompt": "hello"}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert!(!req.stream);
    }

    #[test]
    fn test_stream_param_is_accepted() {
        let json = r#"{"prompt": "hello", "stream": true}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert!(req.stream);
        assert_eq!(req.prompt, "hello");
    }

    #[test]
    fn test_stream_param_defaults_to_false() {
        let json = r#"{"prompt": "hello"}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert!(!req.stream);
    }

    #[test]
    fn test_header_detection_accepts_sse_with_text_event_stream() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "text/event-stream"))
            .to_http_request();
        assert!(accepts_sse(&req));
    }

    #[test]
    fn test_header_detection_rejects_non_sse_accept_header() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "application/json"))
            .to_http_request();
        assert!(!accepts_sse(&req));
    }

    #[test]
    fn test_header_detection_no_accept_header_returns_false() {
        let req = actix_web::test::TestRequest::default().to_http_request();
        assert!(!accepts_sse(&req));
    }

    #[test]
    fn test_header_detection_accepts_sse_in_multi_value_accept() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "application/json, text/event-stream"))
            .to_http_request();
        assert!(accepts_sse(&req));
    }

    #[test]
    fn test_header_detection_rejects_partial_match() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "text/html"))
            .to_http_request();
        assert!(!accepts_sse(&req));
    }

    #[test]
    fn test_agent_type_from_string() {
        assert_eq!(agent_type_from_string("build"), AgentType::Build);
        assert_eq!(agent_type_from_string("BUILD"), AgentType::Build);
        assert_eq!(agent_type_from_string("plan"), AgentType::Plan);
        assert_eq!(agent_type_from_string("PLAN"), AgentType::Plan);
        assert_eq!(agent_type_from_string("explore"), AgentType::Explore);
        assert_eq!(agent_type_from_string("review"), AgentType::Review);
        assert_eq!(agent_type_from_string("refactor"), AgentType::Refactor);
        assert_eq!(agent_type_from_string("debug"), AgentType::Debug);
        assert_eq!(agent_type_from_string("general"), AgentType::General);
        assert_eq!(agent_type_from_string("unknown"), AgentType::General);
    }

    #[test]
    fn test_run_request_deserialization_all_fields() {
        let json = r#"{
            "prompt": "Hello AI",
            "model": "claude-3-opus",
            "agent": "plan",
            "stream": true
        }"#;

        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "Hello AI");
        assert_eq!(req.model, Some("claude-3-opus".to_string()));
        assert_eq!(req.agent, Some("plan".to_string()));
        assert!(req.stream);
    }

    #[test]
    fn test_run_request_deserialization_prompt_only() {
        let json = r#"{"prompt": "Just a prompt"}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "Just a prompt");
        assert!(req.model.is_none());
        assert!(req.agent.is_none());
        assert!(!req.stream);
    }

    #[test]
    fn test_run_request_deserialization_stream_true() {
        let json = r#"{"prompt": "test", "stream": true}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert!(req.stream);
    }

    #[test]
    fn test_run_request_deserialization_stream_false_explicit() {
        let json = r#"{"prompt": "test", "stream": false}"#;
        let req: RunRequest = serde_json::from_str(json).unwrap();
        assert!(!req.stream);
    }

    #[test]
    fn test_api_key_for_provider_openai_from_env() {
        std::env::set_var("OPENAI_API_KEY", "test-openai-key");
        let config = opencode_core::Config::default();
        let key = api_key_for_provider(&config, "openai");
        assert_eq!(key, Some("test-openai-key".to_string()));
        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_api_key_for_provider_anthropic_from_env() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-anthropic-key");
        let config = opencode_core::Config::default();
        let key = api_key_for_provider(&config, "anthropic");
        assert_eq!(key, Some("test-anthropic-key".to_string()));
        std::env::remove_var("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_api_key_for_provider_unknown() {
        let config = opencode_core::Config::default();
        let key = api_key_for_provider(&config, "unknown-provider");
        assert!(key.is_none());
    }

    #[test]
    fn test_ollama_base_url_from_env() {
        std::env::set_var("OLLAMA_BASE_URL", "http://localhost:11434");
        let config = opencode_core::Config::default();
        let url = ollama_base_url(&config);
        assert_eq!(url, Some("http://localhost:11434".to_string()));
        std::env::remove_var("OLLAMA_BASE_URL");
    }

    #[test]
    fn test_ollama_base_url_not_set() {
        std::env::remove_var("OLLAMA_BASE_URL");
        let config = opencode_core::Config::default();
        let url = ollama_base_url(&config);
        assert!(url.is_none());
    }

    #[test]
    fn test_accepts_sse_with_text_event_stream_header() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "text/event-stream"))
            .to_http_request();
        assert!(accepts_sse(&req));
    }

    #[test]
    fn test_accepts_sse_case_insensitive() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "text/event-stream"))
            .to_http_request();
        assert!(accepts_sse(&req));
    }

    #[test]
    fn test_accepts_sse_with_additional_params() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "text/event-stream; charset=utf-8"))
            .to_http_request();
        assert!(accepts_sse(&req));
    }

    #[test]
    fn test_accepts_sse_not_present() {
        let req = actix_web::test::TestRequest::default()
            .insert_header(("Accept", "*/*"))
            .to_http_request();
        assert!(!accepts_sse(&req));
    }

    #[test]
    fn test_agent_type_all_cases() {
        assert_eq!(agent_type_from_string("BUILD"), AgentType::Build);
        assert_eq!(agent_type_from_string("Plan"), AgentType::Plan);
        assert_eq!(agent_type_from_string("EXPLORE"), AgentType::Explore);
        assert_eq!(agent_type_from_string("Review"), AgentType::Review);
        assert_eq!(agent_type_from_string("REFACTOR"), AgentType::Refactor);
        assert_eq!(agent_type_from_string("Debug"), AgentType::Debug);
        assert_eq!(agent_type_from_string("General"), AgentType::General);
        assert_eq!(agent_type_from_string(""), AgentType::General);
    }

    #[test]
    fn test_build_provider_missing_openai_api_key_returns_typed_error() {
        let was_set = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("OPENAI_API_KEY");
        let config = opencode_core::Config::default();
        let result = build_provider("openai", "gpt-4o", &config);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, RouteError::ProviderAuthFailed(_)));
            assert_eq!(err.code(), 3002);
            assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        } else {
            panic!("Expected error");
        }
        if let Some(val) = was_set {
            std::env::set_var("OPENAI_API_KEY", val);
        }
    }

    #[test]
    fn test_build_provider_missing_anthropic_api_key_returns_typed_error() {
        let was_set = std::env::var("ANTHROPIC_API_KEY").ok();
        std::env::remove_var("ANTHROPIC_API_KEY");
        let config = opencode_core::Config::default();
        let result = build_provider("anthropic", "claude-3-opus", &config);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, RouteError::ProviderAuthFailed(_)));
            assert_eq!(err.code(), 3002);
            assert_eq!(err.http_status(), actix_web::http::StatusCode::NOT_FOUND);
        } else {
            panic!("Expected error");
        }
        if let Some(val) = was_set {
            std::env::set_var("ANTHROPIC_API_KEY", val);
        }
    }

    #[test]
    fn test_build_provider_with_api_key_returns_provider() {
        let was_set = std::env::var("OPENAI_API_KEY").ok();
        std::env::set_var("OPENAI_API_KEY", "test-key-123");
        let config = opencode_core::Config::default();
        let result = build_provider("openai", "gpt-4o", &config);
        assert!(result.is_ok());
        match was_set {
            Some(val) => std::env::set_var("OPENAI_API_KEY", val),
            None => std::env::remove_var("OPENAI_API_KEY"),
        }
    }

    #[test]
    fn test_build_provider_error_is_typed_enum_not_string() {
        let was_set = std::env::var("OPENAI_API_KEY").ok();
        std::env::remove_var("OPENAI_API_KEY");
        let config = opencode_core::Config::default();
        let result = build_provider("openai", "gpt-4o", &config);
        assert!(result.is_err());
        if let Err(err) = result {
            let err_type = err.error_type();
            assert_eq!(err_type, "provider_auth_failed");
            assert!(err.to_string().contains("Missing OpenAI API key"));
        } else {
            panic!("Expected error");
        }
        if let Some(val) = was_set {
            std::env::set_var("OPENAI_API_KEY", val);
        }
    }
}
