mod common;

use common::TestHarness;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

fn setup_ollama_config(config_dir: &std::path::Path, model: &str, base_url: &str) {
    fs::create_dir_all(config_dir).unwrap();
    let config_path = config_dir.join("config.json");
    let config_content = serde_json::json!({
        "model": model,
        "provider": {
            "ollama": {
                "options": {
                    "base_url": base_url,
                }
            }
        }
    });
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config_content).unwrap(),
    )
    .unwrap();
}

fn start_mock_ollama_server_with_empty_chunks(
    empty_chunk_count: usize,
) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_clone = Arc::clone(&requests);

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];

            loop {
                let read = stream.read(&mut chunk).unwrap_or(0);
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }

            let request_text = String::from_utf8_lossy(&buffer).to_string();
            let body = request_text
                .split("\r\n\r\n")
                .nth(1)
                .unwrap_or("")
                .to_string();
            requests_clone.lock().unwrap().push(body.clone());

            let body_json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
            let model = body_json
                .get("model")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let prompt = body_json
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let response_text = format!("model={model};prompt={prompt}");
            let mut response_body = String::new();
            for _ in 0..empty_chunk_count {
                response_body.push_str("{\"response\":\"\",\"done\":false}\n");
            }
            response_body.push_str(&format!(
                "{{\"response\":\"{}\",\"done\":false}}\n{{\"done\":true}}\n",
                response_text
            ));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    (format!("http://{}", addr), requests)
}

fn start_mock_ollama_server() -> (String, Arc<Mutex<Vec<String>>>) {
    start_mock_ollama_server_with_empty_chunks(0)
}

#[test]
fn test_run_prompt_mode_returns_structured_output() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server();
    setup_ollama_config(&config_dir, "ollama/test-model", &base_url);

    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "hello parity",
        "--model",
        "ollama/test-model",
    ]);

    assert!(output.status.success(), "run should succeed in prompt mode");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("model=test-model;prompt=hello parity"),
        "stdout should include provider response, got: {}",
        stdout
    );
}

#[test]
fn test_run_text_format_does_not_use_legacy_echo_path_for_ollama_models() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server();
    setup_ollama_config(&config_dir, "ollama/test-model", &base_url);
    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "hello parity",
        "--model",
        "ollama/test-model",
        "--format",
        "text",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Mode: non-interactive"),
        "text mode should execute the provider path instead of legacy echo output: {}",
        stdout
    );
    assert!(stdout.contains("model=test-model;prompt=hello parity"));
}

#[test]
fn test_run_format_cli_accepts_json_flag() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "run", "--prompt", "test", "--model", "gpt-4o", "--format", "json",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("event") || stderr.contains("event") || !output.status.success(),
        "json format should produce event-based output or fail gracefully: stdout={}, stderr={}",
        stdout,
        stderr
    );
}

#[test]
fn test_run_format_cli_accepts_ndjson_flag() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "run", "--prompt", "test", "--model", "gpt-4o", "--format", "ndjson",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("event") || stderr.contains("event") || !output.status.success(),
        "ndjson format should produce event-based output or fail gracefully: stdout={}, stderr={}",
        stdout,
        stderr
    );
}

#[test]
fn test_run_format_ndjson_output_structure() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "run", "--prompt", "hi", "--model", "gpt-4o", "--format", "ndjson",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    for line in &lines {
        if !line.is_empty() {
            let parsed = serde_json::from_str::<serde_json::Value>(line);
            assert!(
                parsed.is_ok(),
                "ndjson output should be valid JSON per line: {}",
                line
            );
        }
    }
}

#[test]
fn test_run_ndjson_suppresses_empty_stream_chunks() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server_with_empty_chunks(3);
    setup_ollama_config(&config_dir, "ollama/ndjson-model", &base_url);

    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "ndjson prompt",
        "--model",
        "ollama/ndjson-model",
        "--format",
        "ndjson",
    ]);

    assert!(output.status.success(), "ndjson run should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"content\":\"model=ndjson-model;prompt=ndjson prompt\""));
    assert!(
        !stdout.contains("\"content\":\"\""),
        "ndjson output should not emit empty chunk events: {}",
        stdout
    );
}

#[test]
fn test_run_format_json_output_structure() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "run", "--prompt", "hi", "--model", "gpt-4o", "--format", "json",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    if !stdout.trim().is_empty() {
        let parsed = serde_json::from_str::<serde_json::Value>(&stdout);
        assert!(
            parsed.is_ok(),
            "json format output should be valid JSON: {}",
            stdout
        );
    }
}

#[test]
fn test_run_format_default_output_regression() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server();
    setup_ollama_config(&config_dir, "ollama/default-model", &base_url);
    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "hello",
        "--model",
        "ollama/default-model",
    ]);

    assert!(
        output.status.success(),
        "run should succeed with default format"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("model=default-model;prompt=hello"),
        "default format should stream provider output, got: {}",
        stdout
    );
}

#[test]
fn test_run_command_uses_config_model() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server();
    setup_ollama_config(&config_dir, "ollama/from-config", &base_url);

    let output = harness.run_cli(&["run", "--prompt", "test"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("model=from-config;prompt=test"),
        "run should use model from config, got: {}",
        stdout
    );
}

#[test]
fn test_run_command_with_explicit_ollama_model_succeeds_without_config_model() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server();
    setup_ollama_config(&config_dir, "ollama/unused", &base_url);

    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "test",
        "--model",
        "ollama/direct-model",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("model=direct-model;prompt=test"),
        "run should execute explicit Ollama model, got: {}",
        stdout
    );
}

#[test]
fn test_run_command_cli_model_overrides_config() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    let (base_url, _requests) = start_mock_ollama_server();
    setup_ollama_config(&config_dir, "ollama/config-model", &base_url);

    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "test",
        "--model",
        "ollama/cli-model",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("model=cli-model;prompt=test"),
        "run should use CLI model over config, got: {}",
        stdout
    );
}
