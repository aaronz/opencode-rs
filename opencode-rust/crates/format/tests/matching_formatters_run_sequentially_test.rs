use std::collections::HashMap;
use tempfile::tempdir;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_format::FormatService;

#[tokio::test]
async fn matching_formatters_run_sequentially() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.ts");
    let order_file = temp.path().join("order.log");
    std::fs::write(&source_file, "const x = 1;\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "a-first".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo first >> \"{}\"", order_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );
    formatters.insert(
        "b-second".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo second >> \"{}\"", order_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );
    formatters.insert(
        "c-third".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo third >> \"{}\"", order_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service
        .init(temp.path(), config)
        .await;

    let result = service.file(&source_file).await;
    assert!(result.is_ok(), "file() should return success");

    let content = std::fs::read_to_string(&order_file).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines, vec!["first", "second", "third"],
        "Formatters should run sequentially in sorted name order, but got: {:?}", lines);
}

#[tokio::test]
async fn matching_formatters_respects_name_sort_order() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.rs");
    let order_file = temp.path().join("order.log");
    std::fs::write(&source_file, "fn main() {}\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "z-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo z >> \"{}\"", order_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["rs".to_string()]),
        },
    );
    formatters.insert(
        "a-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo a >> \"{}\"", order_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["rs".to_string()]),
        },
    );
    formatters.insert(
        "m-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo m >> \"{}\"", order_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["rs".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service
        .init(temp.path(), config)
        .await;

    let result = service.file(&source_file).await;
    assert!(result.is_ok());

    let content = std::fs::read_to_string(&order_file).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines, vec!["a", "m", "z"],
        "Formatters should run in alphabetical order by name, got: {:?}", lines);
}

#[tokio::test]
async fn matching_formatters_run_sequentially_one_at_a_time() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.py");
    let marker_file = temp.path().join("marker.txt");
    std::fs::write(&source_file, "x = 1\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "first-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "touch \"{}\"; sleep 0.05",
                    marker_file.display()
                ),
            ]),
            environment: None,
            extensions: Some(vec!["py".to_string()]),
        },
    );
    formatters.insert(
        "second-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "while [ ! -f \"{}\" ]; do sleep 0.01; done; echo second >> \"{}/order.txt\"",
                    marker_file.display(),
                    temp.path().display()
                ),
            ]),
            environment: None,
            extensions: Some(vec!["py".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service
        .init(temp.path(), config)
        .await;

    let result = service.file(&source_file).await;
    assert!(result.is_ok());

    let order_file = temp.path().join("order.txt");
    let content = std::fs::read_to_string(&order_file).unwrap();
    assert_eq!(content.trim(), "second",
        "Second formatter should wait for marker created by first formatter");
}