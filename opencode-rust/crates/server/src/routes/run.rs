use crate::routes::error::json_error;
use crate::routes::validation::RequestValidator;
use crate::streaming::StreamMessage;
use crate::ServerState;
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures::stream::{self, Stream};
use opencode_core::{Message, Session};
use opencode_llm::{AnthropicProvider, ChatMessage, OllamaProvider, OpenAiProvider, Provider};
use serde::Deserialize;
use tokio::sync::mpsc;

#[derive(Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub agent: Option<String>,
    #[serde(default)]
    pub stream: bool,
}

fn agent_system_prompt(agent: &str) -> &'static str {
    match agent.to_ascii_lowercase().as_str() {
        "build" => "You are OpenCode's BUILD agent. Implement user requests with concise, actionable output.",
        "plan" => "You are OpenCode's PLAN agent. Produce an explicit and practical execution plan.",
        "explore" => "You are OpenCode's EXPLORE agent. Investigate and summarize findings with evidence.",
        "review" => "You are OpenCode's REVIEW agent. Analyze quality, risks, and actionable improvements.",
        "refactor" => "You are OpenCode's REFACTOR agent. Improve structure while preserving behavior.",
        "debug" => "You are OpenCode's DEBUG agent. Diagnose root causes and propose precise fixes.",
        _ => "You are OpenCode's GENERAL agent. Respond helpfully and clearly.",
    }
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
) -> Result<Box<dyn Provider + Send + Sync>, String> {
    match provider_id {
        "anthropic" => {
            let key = api_key_for_provider(config, "anthropic")
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| "Missing Anthropic API key".to_string())?;
            Ok(Box::new(AnthropicProvider::new(key, model.to_string())))
        }
        "ollama" => Ok(Box::new(OllamaProvider::new(
            model.to_string(),
            ollama_base_url(config),
        ))),
        _ => {
            let key = api_key_for_provider(config, "openai")
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| "Missing OpenAI API key".to_string())?;
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

async fn run_prompt_streaming(
    state: web::Data<ServerState>,
    req: web::Json<RunRequest>,
    session_id: String,
) -> HttpResponse {
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

    let selected_model = req
        .model
        .clone()
        .or_else(|| config.model.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());
    let (provider_id, model_name) =
        resolve_model_and_provider(&state, Some(selected_model.clone()));
    let provider = match build_provider(&provider_id, &model_name, &config) {
        Ok(provider) => provider,
        Err(message) => {
            return json_error(StatusCode::BAD_REQUEST, "provider_init_error", message);
        }
    };

    let selected_agent = req.agent.clone().unwrap_or_else(|| "general".to_string());
    let prompt = format!(
        "{}\n\nUser: {}",
        agent_system_prompt(&selected_agent),
        req.prompt
    );

    let (tx, rx) = mpsc::channel::<Result<web::Bytes, std::convert::Infallible>>(128);

    let tx_for_callback = tx.clone();
    let tx_for_error = tx.clone();
    let session_id_for_callback = session_id.clone();
    let session_id_for_error = session_id.clone();
    let session_id_for_completion = session_id.clone();

    actix_rt::spawn(async move {
        let result = provider
            .complete_streaming(
                &prompt,
                Box::new(move |chunk| {
                    let msg = StreamMessage::Message {
                        session_id: session_id_for_callback.clone(),
                        content: chunk,
                        role: "assistant".to_string(),
                    };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let sse_data = format!("data: {}\n\n", json);
                        let _ = tx_for_callback.try_send(Ok(web::Bytes::from(sse_data)));
                    }
                }),
            )
            .await;

        if let Err(e) = result {
            let msg = StreamMessage::Error {
                session_id: Some(session_id_for_error.clone()),
                error: "streaming_error".to_string(),
                code: "STREAMING_ERROR".to_string(),
                message: e.to_string(),
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                let sse_data = format!("data: {}\n\n", json);
                let _ = tx_for_error.try_send(Ok(web::Bytes::from(sse_data)));
            }
        }

        let completion_msg = StreamMessage::Message {
            session_id: session_id_for_completion,
            content: "[DONE]".to_string(),
            role: "assistant".to_string(),
        };
        if let Ok(json) = serde_json::to_string(&completion_msg) {
            let sse_data = format!("data: {}\n\n", json);
            let _ = tx_for_error.try_send(Ok(web::Bytes::from(sse_data)));
        }
    });

    let stream = stream::unfold(rx, |mut rx| async move {
        rx.recv()
            .await
            .map(|Ok(bytes)| (Ok::<_, std::convert::Infallible>(bytes), rx))
    });

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(stream))
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

    let selected_model = body
        .model
        .clone()
        .or_else(|| config.model.clone())
        .unwrap_or_else(|| "gpt-4o".to_string());
    let (provider_id, model_name) =
        resolve_model_and_provider(&state, Some(selected_model.clone()));
    let provider = match build_provider(&provider_id, &model_name, &config) {
        Ok(provider) => provider,
        Err(message) => {
            return json_error(StatusCode::BAD_REQUEST, "provider_init_error", message);
        }
    };

    let selected_agent = body.agent.clone().unwrap_or_else(|| "general".to_string());
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: agent_system_prompt(&selected_agent).to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: body.prompt.clone(),
        },
    ];

    let response = match provider.chat(&messages).await {
        Ok(response) => response,
        Err(e) => {
            return json_error(
                StatusCode::BAD_GATEWAY,
                "agent_execution_error",
                e.to_string(),
            );
        }
    };

    session.add_message(Message::assistant(response.content.clone()));
    if let Err(e) = state.storage.save_session(&session).await {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "storage_error",
            e.to_string(),
        );
    }

    HttpResponse::Ok().json(serde_json::json!({
        "session_id": session.id.to_string(),
        "status": "completed",
        "agent": selected_agent,
        "provider": provider_id,
        "model": selected_model,
        "response": response.content,
    }))
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
    fn test_agent_system_prompt() {
        assert!(agent_system_prompt("build").contains("BUILD"));
        assert!(agent_system_prompt("plan").contains("PLAN"));
        assert!(agent_system_prompt("explore").contains("EXPLORE"));
        assert!(agent_system_prompt("review").contains("REVIEW"));
        assert!(agent_system_prompt("refactor").contains("REFACTOR"));
        assert!(agent_system_prompt("debug").contains("DEBUG"));
        assert!(agent_system_prompt("general").contains("GENERAL"));
        assert!(agent_system_prompt("unknown").contains("GENERAL"));
    }
}
