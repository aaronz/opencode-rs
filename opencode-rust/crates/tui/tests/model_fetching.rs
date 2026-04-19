use opencode_llm::BrowserAuthModelInfo;
use opencode_tui::app::{parse_anthropic_models, parse_lm_studio_models, parse_openai_models};

#[test]
fn test_parse_openai_models_success() {
    let json = r#"{
        "data": [
            {"id": "gpt-4o", "object": "model"},
            {"id": "gpt-4o-mini", "object": "model"},
            {"id": "gpt-4-turbo", "object": "model"}
        ]
    }"#;

    let models = parse_openai_models(json);

    assert_eq!(models.len(), 3);
    assert_eq!(models[0].id, "gpt-4o");
    assert_eq!(models[1].id, "gpt-4o-mini");
    assert_eq!(models[2].id, "gpt-4-turbo");
}

#[test]
fn test_parse_openai_models_empty_list() {
    let json = r#"{"data": []}"#;

    let models = parse_openai_models(json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_openai_models_invalid_json_returns_empty() {
    let invalid_json = r#"{"not": "valid"}"#;

    let models = parse_openai_models(invalid_json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_openai_models_malformed_json_returns_empty() {
    let malformed_json = r#"{"data": [{"id":}]"#;

    let models = parse_openai_models(malformed_json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_openai_models_id_and_name_are_same() {
    let json = r#"{
        "data": [
            {"id": "gpt-4o", "object": "model"}
        ]
    }"#;

    let models = parse_openai_models(json);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, models[0].name);
}

#[test]
fn test_parse_anthropic_models_success() {
    let json = r#"{
        "models": [
            {"id": "claude-sonnet-4-20250514", "object": "model"},
            {"id": "claude-haiku-3", "object": "model"},
            {"id": "claude-opus-4-20250514", "object": "model"}
        ]
    }"#;

    let models = parse_anthropic_models(json);

    assert_eq!(models.len(), 3);
    assert_eq!(models[0].id, "claude-sonnet-4-20250514");
    assert_eq!(models[1].id, "claude-haiku-3");
    assert_eq!(models[2].id, "claude-opus-4-20250514");
}

#[test]
fn test_parse_anthropic_models_empty_list() {
    let json = r#"{"models": []}"#;

    let models = parse_anthropic_models(json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_anthropic_models_invalid_json_returns_empty() {
    let invalid_json = r#"{"not": "valid"}"#;

    let models = parse_anthropic_models(invalid_json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_anthropic_models_malformed_json_returns_empty() {
    let malformed_json = r#"{"models": [{"id":}]"#;

    let models = parse_anthropic_models(malformed_json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_anthropic_models_id_and_name_are_same() {
    let json = r#"{
        "models": [
            {"id": "claude-sonnet-4-20250514", "object": "model"}
        ]
    }"#;

    let models = parse_anthropic_models(json);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, models[0].name);
}

#[test]
fn test_parse_lm_studio_models_success() {
    let json = r#"{
        "models": [
            {"name": "llama3", "model": "llama3.gguf"},
            {"name": "codellama", "model": "codellama.gguf"}
        ]
    }"#;

    let models = parse_lm_studio_models(json);

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "llama3");
    assert_eq!(models[1].id, "codellama");
}

#[test]
fn test_parse_lm_studio_models_empty_list() {
    let json = r#"{"models": []}"#;

    let models = parse_lm_studio_models(json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_lm_studio_models_invalid_json_returns_empty() {
    let invalid_json = r#"{"not": "valid"}"#;

    let models = parse_lm_studio_models(invalid_json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_lm_studio_models_malformed_json_returns_empty() {
    let malformed_json = r#"{"models": [{"name":}]"#;

    let models = parse_lm_studio_models(malformed_json);

    assert!(models.is_empty());
}

#[test]
fn test_parse_lm_studio_models_id_and_name_are_same() {
    let json = r#"{
        "models": [
            {"name": "llama3", "model": "llama3.gguf"}
        ]
    }"#;

    let models = parse_lm_studio_models(json);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, models[0].name);
}

#[test]
fn test_verify_models_fetched_from_openai_endpoint() {
    let openai_response = r#"{
        "data": [
            {"id": "gpt-4o", "object": "model"},
            {"id": "gpt-4o-mini", "object": "model"},
            {"id": "gpt-4-turbo", "object": "model"},
            {"id": "gpt-3.5-turbo", "object": "model"}
        ]
    }"#;

    let models = parse_openai_models(openai_response);

    assert_eq!(models.len(), 4);
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(ids.contains(&"gpt-4o"));
    assert!(ids.contains(&"gpt-4o-mini"));
    assert!(ids.contains(&"gpt-4-turbo"));
    assert!(ids.contains(&"gpt-3.5-turbo"));
}

#[test]
fn test_verify_models_fetched_from_anthropic_endpoint() {
    let anthropic_response = r#"{
        "models": [
            {"id": "claude-sonnet-4-20250514", "object": "model"},
            {"id": "claude-haiku-3", "object": "model"},
            {"id": "claude-opus-4-20250514", "object": "model"}
        ]
    }"#;

    let models = parse_anthropic_models(anthropic_response);

    assert_eq!(models.len(), 3);
    let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(ids.contains(&"claude-sonnet-4-20250514"));
    assert!(ids.contains(&"claude-haiku-3"));
    assert!(ids.contains(&"claude-opus-4-20250514"));
}

#[test]
fn test_verify_models_filtered_correctly_per_provider() {
    let openai_response = r#"{
        "data": [
            {"id": "gpt-4o", "object": "model"},
            {"id": "gpt-4o-mini", "object": "model"},
            {"id": "dall-e-3", "object": "model"},
            {"id": "whisper-1", "object": "model"}
        ]
    }"#;

    let models = parse_openai_models(openai_response);

    assert_eq!(models.len(), 4);

    let all_ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(all_ids.contains(&"gpt-4o"));
    assert!(all_ids.contains(&"gpt-4o-mini"));
    assert!(all_ids.contains(&"dall-e-3"));
    assert!(all_ids.contains(&"whisper-1"));
}

#[test]
fn test_verify_empty_model_list_handled_gracefully_openai() {
    let empty_response = r#"{"data": []}"#;

    let models = parse_openai_models(empty_response);

    assert!(models.is_empty());
}

#[test]
fn test_verify_empty_model_list_handled_gracefully_anthropic() {
    let empty_response = r#"{"models": []}"#;

    let models = parse_anthropic_models(empty_response);

    assert!(models.is_empty());
}

#[test]
fn test_verify_empty_model_list_handled_gracefully_lm_studio() {
    let empty_response = r#"{"models": []}"#;

    let models = parse_lm_studio_models(empty_response);

    assert!(models.is_empty());
}

#[test]
fn test_verify_invalid_response_returns_empty_vector_openai() {
    let invalid_response = "this is not json at all";

    let models = parse_openai_models(invalid_response);

    assert!(models.is_empty());
}

#[test]
fn test_verify_invalid_response_returns_empty_vector_anthropic() {
    let invalid_response = "this is not json at all";

    let models = parse_anthropic_models(invalid_response);

    assert!(models.is_empty());
}

#[test]
fn test_verify_invalid_response_returns_empty_vector_lm_studio() {
    let invalid_response = "this is not json at all";

    let models = parse_lm_studio_models(invalid_response);

    assert!(models.is_empty());
}

#[test]
fn test_browser_auth_model_info_structure() {
    let model = BrowserAuthModelInfo {
        id: "test-model".to_string(),
        name: "Test Model".to_string(),
        variants: vec![],
    };

    assert_eq!(model.id, "test-model");
    assert_eq!(model.name, "Test Model");
    assert!(model.variants.is_empty());
}

#[test]
fn test_openai_model_response_with_additional_fields_ignored() {
    let json = r#"{
        "data": [
            {
                "id": "gpt-4o",
                "object": "model",
                "created": 1712361441,
                "owned_by": "system",
                "permission": [],
                "root": "gpt-4o"
            }
        ]
    }"#;

    let models = parse_openai_models(json);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "gpt-4o");
}

#[test]
fn test_anthropic_model_response_with_additional_fields_ignored() {
    let json = r#"{
        "models": [
            {
                "id": "claude-sonnet-4-20250514",
                "object": "model",
                "display_name": "Claude Sonnet 4",
                "type": "text",
                "version": "20250514"
            }
        ]
    }"#;

    let models = parse_anthropic_models(json);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "claude-sonnet-4-20250514");
}

#[test]
fn test_lm_studio_model_response_with_additional_fields_ignored() {
    let json = r#"{
        "models": [
            {
                "name": "llama3",
                "model": "llama3.gguf",
                "size": 5368709120,
                "quantization": "Q4_K_M"
            }
        ]
    }"#;

    let models = parse_lm_studio_models(json);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "llama3");
}

#[test]
fn test_openai_models_parsing_is_idempotent() {
    let json = r#"{
        "data": [
            {"id": "gpt-4o", "object": "model"}
        ]
    }"#;

    let first_parse = parse_openai_models(json);
    let second_parse = parse_openai_models(json);

    assert_eq!(first_parse.len(), second_parse.len());
    assert_eq!(first_parse[0].id, second_parse[0].id);
}

#[test]
fn test_anthropic_models_parsing_is_idempotent() {
    let json = r#"{
        "models": [
            {"id": "claude-sonnet-4-20250514", "object": "model"}
        ]
    }"#;

    let first_parse = parse_anthropic_models(json);
    let second_parse = parse_anthropic_models(json);

    assert_eq!(first_parse.len(), second_parse.len());
    assert_eq!(first_parse[0].id, second_parse[0].id);
}

#[test]
fn test_lm_studio_models_parsing_is_idempotent() {
    let json = r#"{
        "models": [
            {"name": "llama3", "model": "llama3.gguf"}
        ]
    }"#;

    let first_parse = parse_lm_studio_models(json);
    let second_parse = parse_lm_studio_models(json);

    assert_eq!(first_parse.len(), second_parse.len());
    assert_eq!(first_parse[0].id, second_parse[0].id);
}
