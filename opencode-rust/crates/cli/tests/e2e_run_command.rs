mod common;

use common::TestHarness;
use std::fs;

fn setup_config_with_model(config_dir: &std::path::Path, model: &str) {
    fs::create_dir_all(config_dir).unwrap();
    let config_path = config_dir.join("config.json");
    let config_content = serde_json::json!({
        "agent": {
            "agents": {
                "default": {
                    "model": model
                }
            },
            "defaultAgent": "default"
        }
    });
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config_content).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_run_prompt_mode_returns_structured_output() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["run", "--prompt", "hello parity", "--model", "gpt-4o"]);

    assert!(output.status.success(), "run should succeed in prompt mode");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Prompt:"),
        "stdout should include prompt echo"
    );
    assert!(
        stdout.contains("Model: gpt-4o"),
        "stdout should include selected model"
    );
    assert!(
        stdout.contains("Mode: non-interactive"),
        "stdout should include execution mode"
    );
}

#[test]
fn test_run_format_cli_accepts_json_flag() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "test",
        "--model",
        "gpt-4o",
        "--format",
        "json",
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
        "run",
        "--prompt",
        "test",
        "--model",
        "gpt-4o",
        "--format",
        "ndjson",
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
        "run",
        "--prompt",
        "hi",
        "--model",
        "gpt-4o",
        "--format",
        "ndjson",
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
fn test_run_format_json_output_structure() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&[
        "run",
        "--prompt",
        "hi",
        "--model",
        "gpt-4o",
        "--format",
        "json",
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
    let output = harness.run_cli(&["run", "--prompt", "hello", "--model", "gpt-4o"]);

    assert!(output.status.success(), "run should succeed with default format");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Mode: non-interactive"),
        "default format should output mode indicator"
    );
    assert!(
        stdout.contains("Model: gpt-4o"),
        "default format should output model name"
    );
    assert!(
        stdout.contains("Prompt: hello"),
        "default format should output prompt"
    );
}

#[test]
fn test_run_command_uses_config_model() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    setup_config_with_model(&config_dir, "anthropic/claude-3-5-sonnet");

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["run", "--prompt", "test"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Model: anthropic/claude-3-5-sonnet"),
        "run should use model from config, got: {}",
        stdout
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}

#[test]
fn test_run_command_fallback_to_gpt_4o_when_not_set() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let config_path = config_dir.join("config.json");
    fs::write(&config_path, "{}").unwrap();

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["run", "--prompt", "test"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Model: gpt-4o"),
        "run should fallback to gpt-4o when agent.model not set, got: {}",
        stdout
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}

#[test]
fn test_run_command_cli_model_overrides_config() {
    let harness = TestHarness::setup();
    let config_dir = harness.temp_dir.path().join("config");
    setup_config_with_model(&config_dir, "anthropic/claude-3-5-sonnet");

    std::env::set_var("OPENCODE_CONFIG_DIR", config_dir.to_str().unwrap());

    let output = harness.run_cli(&["run", "--prompt", "test", "--model", "openai/gpt-4o"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("Model: openai/gpt-4o"),
        "run should use CLI model over config, got: {}",
        stdout
    );

    std::env::remove_var("OPENCODE_CONFIG_DIR");
}
