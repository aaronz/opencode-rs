use std::collections::HashMap;
use tempfile::tempdir;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_format::FormatService;

#[tokio::test]
async fn failed_formatter_does_not_panic() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.js");
    std::fs::write(&source_file, "const x = 1;\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "failing_formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["sh".to_string(), "-c".to_string(), "exit 1".to_string()]),
            environment: None,
            extensions: Some(vec!["js".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(
        result.is_ok(),
        "file() should return Ok even when formatter fails with exit code 1"
    );
}

#[tokio::test]
async fn failed_formatter_returns_error_not_panic_on_spawn_failure() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.py");
    std::fs::write(&source_file, "x = 1\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "nonexistent_formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["nonexistent_binary_12345".to_string()]),
            environment: None,
            extensions: Some(vec!["py".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(
        result.is_ok(),
        "file() should return Ok even when formatter binary does not exist"
    );
}

#[tokio::test]
async fn system_remains_stable_after_formatter_failure() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("test.ts");
    std::fs::write(&source_file, "let y = 2;\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "broken_formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["sh".to_string(), "-c".to_string(), "exit 127".to_string()]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let first_result = service.file(&source_file).await;
    assert!(first_result.is_ok(), "First call should succeed despite formatter failure");

    let second_result = service.file(&source_file).await;
    assert!(
        second_result.is_ok(),
        "Second call should also succeed - system should remain stable"
    );

    let status = service.status(temp.path()).await;
    assert!(
        !status.is_empty(),
        "Status should still work after formatter failures"
    );
}

#[tokio::test]
async fn multiple_failed_formatters_do_not_panic() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("multi.js");
    std::fs::write(&source_file, "console.log('test');\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "fail_first".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["sh".to_string(), "-c".to_string(), "exit 1".to_string()]),
            environment: None,
            extensions: Some(vec!["js".to_string()]),
        },
    );
    formatters.insert(
        "fail_second".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["sh".to_string(), "-c".to_string(), "exit 2".to_string()]),
            environment: None,
            extensions: Some(vec!["js".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(
        result.is_ok(),
        "file() should return Ok even with multiple failing formatters"
    );
}

#[tokio::test]
async fn formatter_timeout_does_not_panic() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("slow.rs");
    std::fs::write(&source_file, "fn main() {}\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "slow_formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                "sleep 60".to_string(),
            ]),
            environment: None,
            extensions: Some(vec!["rs".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(
        result.is_ok(),
        "file() should return Ok when formatter times out (default 10s timeout)"
    );
}