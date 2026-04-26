#[tokio::test]
#[ignore = "requires real Minimax key and network access"]
async fn minimax_cn_real_validation_smoke_test() {
    if std::env::var("OPENCODE_RS_RUN_REAL_PROVIDER_TESTS")
        .ok()
        .as_deref()
        != Some("1")
    {
        eprintln!("Skipping real provider test. Set OPENCODE_RS_RUN_REAL_PROVIDER_TESTS=1.");
        return;
    }

    let api_key = std::env::var("OPENCODE_RS_TEST_MINIMAX_CN_API_KEY")
        .expect("missing OPENCODE_RS_TEST_MINIMAX_CN_API_KEY");

    let redacted_key = if api_key.len() > 8 {
        format!("****{}", &api_key[api_key.len() - 8..])
    } else {
        "****".to_string()
    };
    eprintln!("Testing Minimax CN validation with key: {}", redacted_key);

    use opencode_tui::app::validate_api_key_and_fetch_models;

    let result = validate_api_key_and_fetch_models("minimax-cn", &api_key).await;

    match result {
        Ok(models) => {
            eprintln!("Validation succeeded! Received {} models:", models.len());
            for model in &models {
                eprintln!("  - {} ({})", model.name, model.id);
            }
            assert!(!models.is_empty(), "Should return at least one model");
        }
        Err(e) => {
            eprintln!("Validation failed with error: {}", e);
            eprintln!("Error type: {:?}", e.error_type);
            if let Some(status) = e.status_code {
                eprintln!("HTTP Status: {}", status);
            }
            panic!("Minimax CN validation failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires real Minimax key and network access"]
async fn minimax_cn_real_chat_completion_smoke_test() {
    if std::env::var("OPENCODE_RS_RUN_REAL_PROVIDER_TESTS")
        .ok()
        .as_deref()
        != Some("1")
    {
        eprintln!("Skipping real provider test. Set OPENCODE_RS_RUN_REAL_PROVIDER_TESTS=1.");
        return;
    }

    let api_key = std::env::var("OPENCODE_RS_TEST_MINIMAX_CN_API_KEY")
        .expect("missing OPENCODE_RS_TEST_MINIMAX_CN_API_KEY");

    let redacted_key = if api_key.len() > 8 {
        format!("****{}", &api_key[api_key.len() - 8..])
    } else {
        "****".to_string()
    };
    eprintln!(
        "Testing Minimax CN chat completion with key: {}",
        redacted_key
    );

    use opencode_llm::minimax::MiniMaxProvider;
    use opencode_llm::provider::Provider;

    let provider = MiniMaxProvider::with_base_url(
        api_key,
        "MiniMax-M2.7".to_string(),
        "https://api.minimaxi.com".to_string(),
    );

    let result = provider
        .complete("Say 'Hello, World!' in exactly those words.", None)
        .await;

    match result {
        Ok(response) => {
            eprintln!("Chat completion succeeded!");
            eprintln!("Response: {}", response);
            assert!(
                response.contains("Hello") && response.contains("World"),
                "Response should contain 'Hello' and 'World'"
            );
        }
        Err(e) => {
            eprintln!("Chat completion failed: {}", e);
            panic!("Minimax CN chat completion failed: {}", e);
        }
    }
}
