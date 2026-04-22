use std::collections::HashMap;
use std::path::Path;

use opencode_config::{FormatterConfig, FormatterEntry};
use opencode_format::FormatService;

#[tokio::test]
async fn status_empty_when_disabled() {
    let service = FormatService::new();

    let config = FormatterConfig::Disabled(true);
    let _ = service.init(Path::new("/tmp/test-project"), config).await;

    let statuses = service.status(Path::new("/tmp/test-project")).await;

    assert!(
        statuses.is_empty(),
        "status() should return empty list when formatter: false"
    );
}

#[tokio::test]
async fn status_includes_gofmt_when_all_enabled() {
    let service = FormatService::new();

    let config = FormatterConfig::Disabled(false);
    let _ = service.init(Path::new("/tmp/test-project"), config).await;

    let statuses = service.status(Path::new("/tmp/test-project")).await;

    let gofmt_status = statuses.iter().find(|s| s.name == "gofmt");
    assert!(
        gofmt_status.is_some(),
        "status() should include gofmt when formatter: true"
    );

    let gofmt = gofmt_status.unwrap();
    assert_eq!(gofmt.extensions, vec![".go"]);
}

#[tokio::test]
async fn status_returns_formatters_when_enabled() {
    use opencode_format::FormatService;

    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "gofmt".to_string(),
        FormatterEntry {
            disabled: Some(false),
            command: Some(vec!["gofmt".to_string()]),
            environment: None,
            extensions: Some(vec![".go".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(Path::new("/tmp/test-project"), config).await;

    let statuses = service.status(Path::new("/tmp/test-project")).await;

    assert!(
        !statuses.is_empty(),
        "status() should return formatters when configured"
    );
}

#[tokio::test]
async fn status_excludes_disabled_formatter() {
    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "gofmt".to_string(),
        FormatterEntry {
            disabled: Some(true),
            command: Some(vec!["gofmt".to_string()]),
            environment: None,
            extensions: Some(vec![".go".to_string()]),
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(Path::new("/tmp/test-project"), config).await;

    let statuses = service.status(Path::new("/tmp/test-project")).await;

    let gofmt_status = statuses.iter().find(|s| s.name == "gofmt");
    assert!(
        gofmt_status.is_none(),
        "status() should exclude formatters marked as disabled"
    );
}

#[tokio::test]
async fn disabling_ruff_removes_uv() {
    let service = FormatService::new();

    let mut formatters = HashMap::new();
    formatters.insert(
        "ruff".to_string(),
        FormatterEntry {
            disabled: Some(true),
            command: None,
            environment: None,
            extensions: None,
        },
    );

    let config = FormatterConfig::Formatters(formatters);
    let _ = service.init(Path::new("/tmp/test-project"), config).await;

    let statuses = service.status(Path::new("/tmp/test-project")).await;

    let uv_status = statuses.iter().find(|s| s.name == "uvformat");
    assert!(
        uv_status.map(|s| !s.enabled).unwrap_or(false),
        "uvformat should be disabled when ruff is disabled"
    );

    let ruff_status = statuses.iter().find(|s| s.name == "ruff");
    assert!(
        ruff_status.is_none(),
        "ruff should be excluded when explicitly disabled"
    );
}
