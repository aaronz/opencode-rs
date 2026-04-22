use std::collections::HashMap;
use tempfile::tempdir;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_format::FormatService;

#[tokio::test]
async fn file_placeholder_substituted() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.ts");
    let marker_file = temp.path().join("replaced.txt");
    std::fs::write(&source_file, "const x = 1;\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "test-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "if [ \"$1\" = \"{}\" ]; then touch \"{}\"; fi",
                    source_file.display(),
                    marker_file.display()
                ),
                "sh".to_string(),
                "$FILE".to_string(),
            ]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(result.is_ok(), "file() should return success");

    assert!(
        marker_file.exists(),
        "Marker file should exist, confirming $FILE was correctly substituted with actual file path"
    );
}

#[tokio::test]
async fn file_placeholder_substituted_with_special_chars_in_path() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("my-file.ts");
    let marker_file = temp.path().join("replaced_special.txt");
    std::fs::write(&source_file, "const x = 1;\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "test-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "cp \"$1\" \"{}\"",
                    marker_file.display()
                ),
                "sh".to_string(),
                "$FILE".to_string(),
            ]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(result.is_ok(), "file() should return success");

    assert!(
        marker_file.exists(),
        "Marker file should exist, confirming $FILE works with paths containing special chars"
    );
}

#[tokio::test]
async fn file_placeholder_substituted_multiple_times() {
    let temp = tempdir().unwrap();
    let source_file = temp.path().join("main.ts");
    let output_file = temp.path().join("output.txt");
    std::fs::write(&source_file, "const x = 1;\n").unwrap();

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "test-formatter".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("echo '$FILE' >> \"{}\"", output_file.display()),
            ]),
            environment: None,
            extensions: Some(vec!["ts".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(temp.path(), config).await;

    let result = service.file(&source_file).await;
    assert!(result.is_ok(), "file() should return success");

    let content = std::fs::read_to_string(&output_file).unwrap();
    let actual_path = source_file.to_string_lossy();
    assert_eq!(
        content.trim(),
        actual_path,
        "Output should contain the actual file path, got: {}",
        content
    );
}