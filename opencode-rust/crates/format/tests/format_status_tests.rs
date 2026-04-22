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