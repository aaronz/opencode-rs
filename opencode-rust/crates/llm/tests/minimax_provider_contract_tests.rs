use serde_json::json;
use std::time::Duration;
use wiremock::{Mock, MockServer, ResponseTemplate};

use opencode_llm::minimax::MiniMaxProvider;
use opencode_llm::provider::Provider;

fn create_success_response() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "MiniMax-M2.7",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello!"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    }))
}

fn create_404_response() -> ResponseTemplate {
    ResponseTemplate::new(404).set_body_json(json!({
        "error": {
            "message": "Not found",
            "type": "invalid_request_error",
            "code": "model_not_found"
        }
    }))
}

fn create_401_response() -> ResponseTemplate {
    ResponseTemplate::new(401).set_body_json(json!({
        "error": {
            "message": "Invalid API key",
            "type": "authentication_error",
            "code": "invalid_api_key"
        }
    }))
}

#[tokio::test]
async fn minimax_cn_validate_key_uses_correct_endpoint() {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/chat/completions"))
        .respond_with(create_success_response())
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MiniMaxProvider::with_base_url(
        "test-api-key".to_string(),
        "MiniMax-M2.7".to_string(),
        mock_server.uri(),
    );

    let result = provider.complete("Hello", None).await;

    if let Err(e) = &result {
        eprintln!("Error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "Minimax CN should use POST /v1/chat/completions endpoint"
    );
}

#[tokio::test]
async fn minimax_cn_404_returns_error_with_diagnostics() {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/chat/completions"))
        .respond_with(create_404_response())
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MiniMaxProvider::with_base_url(
        "test-api-key".to_string(),
        "MiniMax-M2.7".to_string(),
        mock_server.uri(),
    );

    let result = provider.complete("Hello", None).await;
    assert!(result.is_err(), "Should return error on 404");

    let error_msg = result.unwrap_err().to_string();
    eprintln!("404 Error message: {}", error_msg);

    assert!(
        error_msg.contains("404") || error_msg.contains("Not found"),
        "Error should contain 404 or Not found: {}",
        error_msg
    );
}

#[tokio::test]
async fn minimax_cn_401_returns_auth_error() {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/chat/completions"))
        .respond_with(create_401_response())
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MiniMaxProvider::with_base_url(
        "invalid-key".to_string(),
        "MiniMax-M2.7".to_string(),
        mock_server.uri(),
    );

    let result = provider.complete("Hello", None).await;
    assert!(result.is_err(), "Should return error on 401");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.to_lowercase().contains("invalid")
            || error_msg.to_lowercase().contains("authentication")
            || error_msg.contains("401"),
        "Error should mention invalid key or auth: {}",
        error_msg
    );
}

#[tokio::test]
async fn minimax_cn_timeout_is_handled() {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({
                    "id": "chatcmpl-123",
                    "object": "chat.completion",
                    "created": 1677652288,
                    "model": "MiniMax-M2.7",
                    "choices": [{
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": "Hello!"
                        },
                        "finish_reason": "stop"
                    }],
                    "usage": {
                        "prompt_tokens": 10,
                        "completion_tokens": 20,
                        "total_tokens": 30
                    }
                }))
                .set_delay(Duration::from_secs(30)),
        )
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MiniMaxProvider::with_base_url(
        "test-api-key".to_string(),
        "MiniMax-M2.7".to_string(),
        mock_server.uri(),
    );

    let result =
        tokio::time::timeout(Duration::from_secs(2), provider.complete("Hello", None)).await;

    assert!(
        result.is_err() || result.unwrap().is_err(),
        "Should timeout or return error"
    );
}

#[tokio::test]
async fn minimax_cn_success_response_parsing() {
    let mock_server = MockServer::start().await;

    Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "MiniMax-M2.7",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello, how can I help you?"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            }
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = MiniMaxProvider::with_base_url(
        "test-api-key".to_string(),
        "MiniMax-M2.7".to_string(),
        mock_server.uri(),
    );

    let result = provider.complete("Hello", None).await;
    assert!(result.is_ok(), "Should parse success response correctly");

    let response = result.unwrap();
    assert!(
        response.contains("Hello") || response.contains("help"),
        "Should extract content from response: {}",
        response
    );
}

#[test]
fn minimax_cn_base_url_configuration() {
    let provider = MiniMaxProvider::with_base_url(
        "test-key".to_string(),
        "MiniMax-M2.7".to_string(),
        "https://api.minimaxi.com".to_string(),
    );

    let _chat_url = "https://api.minimaxi.com/v1/chat/completions";
    assert_eq!(
        provider.provider_name(),
        "minimax",
        "Provider name should be minimax"
    );
}

#[test]
fn minimax_cn_get_models_returns_expected_models() {
    let provider = MiniMaxProvider::new("test-key".to_string(), "MiniMax-M2.7".to_string());

    let models = provider.get_models();
    assert!(!models.is_empty(), "Should return some models");
    assert!(
        models.iter().any(|m| m.id.contains("MiniMax")),
        "Should include MiniMax models"
    );
}
