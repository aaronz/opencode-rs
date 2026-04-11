use opencode_lsp::aggregator::DiagnosticAggregator;
use opencode_lsp::builtin::{BuiltInRegistry, BundledConfig, DetectionResult, PathIndicator};
use opencode_lsp::client::LspClient;
use opencode_lsp::error::{
    CrashCause, FailureHandlingConfig, LspError, ProtocolViolationType, UnhealthyReason,
};
use opencode_lsp::language::Language;
use opencode_lsp::manager::LspManager;
use opencode_lsp::types::{Diagnostic, Position, Range, Severity};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn integration_test_builtin_server_detection_rust() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    assert!(
        detected.iter().any(|s| s.id == "rust-analyzer"),
        "rust-analyzer should be detected for Rust projects"
    );
}

#[tokio::test]
async fn integration_test_builtin_server_detection_typescript() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    std::fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    assert!(
        detected
            .iter()
            .any(|s| s.id == "typescript-language-server"),
        "typescript-language-server should be detected for TypeScript projects"
    );
}

#[tokio::test]
async fn integration_test_builtin_server_detection_go() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module test").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    assert!(
        detected.iter().any(|s| s.id == "gopls"),
        "gopls should be detected for Go projects"
    );
}

#[tokio::test]
async fn integration_test_builtin_server_detection_python() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("pyproject.toml"), "[project]").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    assert!(
        detected.iter().any(|s| s.id == "pylsp"),
        "pylsp should be detected for Python projects"
    );
}

#[tokio::test]
async fn integration_test_builtin_server_detection_javascript() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    assert!(
        detected
            .iter()
            .any(|s| s.id == "javascript-language-server"),
        "javascript-language-server should be detected for JavaScript projects"
    );
}

#[tokio::test]
async fn integration_test_detection_with_details() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let registry = BuiltInRegistry::new();
    let results = registry.detect_with_details(dir.path());

    assert!(!results.is_empty(), "Should detect at least one server");
    assert_eq!(results[0].server, "Rust Analyzer");
    assert_eq!(results[0].language, "Rust");
}

#[tokio::test]
async fn integration_test_path_indicator_file_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module test").unwrap();

    let indicator = PathIndicator::FileExists("go.mod".to_string());
    assert!(indicator.matches(dir.path()), "FileExists indicator should match");

    let indicator_not_exists = PathIndicator::FileExists("nonexistent".to_string());
    assert!(
        !indicator_not_exists.matches(dir.path()),
        "FileExists indicator should not match nonexistent file"
    );
}

#[tokio::test]
async fn integration_test_path_indicator_dir_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();

    let indicator = PathIndicator::DirExists("src".to_string());
    assert!(indicator.matches(dir.path()), "DirExists indicator should match");

    let indicator_not_exists = PathIndicator::DirExists("nonexistent".to_string());
    assert!(
        !indicator_not_exists.matches(dir.path()),
        "DirExists indicator should not match nonexistent dir"
    );
}

#[tokio::test]
async fn integration_test_path_indicator_file_contains() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module test\ngo 1.21").unwrap();

    let indicator = PathIndicator::FileContains {
        path: "go.mod".to_string(),
        content: "go 1.21".to_string(),
    };
    assert!(
        indicator.matches(dir.path()),
        "FileContains indicator should match when content exists"
    );

    let indicator_wrong_content = PathIndicator::FileContains {
        path: "go.mod".to_string(),
        content: "nonexistent content".to_string(),
    };
    assert!(
        !indicator_wrong_content.matches(dir.path()),
        "FileContains indicator should not match when content missing"
    );
}

#[tokio::test]
async fn integration_test_builtin_server_all_match_any_false_requires_all() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    let ts_server = detected
        .iter()
        .find(|s| s.id == "typescript-language-server");
    assert!(
        ts_server.is_none(),
        "TypeScript server should not be detected without tsconfig.json when match_any is false"
    );
}

#[tokio::test]
async fn integration_test_builtin_server_python_match_any_true() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("requirements.txt"), "flask").unwrap();

    let registry = BuiltInRegistry::new();
    let detected = registry.detect_for_root(dir.path());

    assert!(
        detected.iter().any(|s| s.id == "pylsp"),
        "pylsp should be detected with only requirements.txt (match_any=true)"
    );
}

#[tokio::test]
async fn integration_test_bundled_config_custom_paths() {
    let mut config = BundledConfig::default();
    config
        .custom_paths
        .insert("rust-analyzer".to_string(), "/custom/rust-analyzer".to_string());

    let cmd = config.get_command("rust-analyzer", "rust-analyzer");
    assert_eq!(cmd, "/custom/rust-analyzer", "Custom path should be used when set");

    let cmd_default = config.get_command("unknown-server", "default-command");
    assert_eq!(
        cmd_default, "default-command",
        "Default command should be used when no custom path set"
    );
}

#[tokio::test]
async fn integration_test_bundled_config_excluded_servers() {
    let mut config = BundledConfig::default();
    config
        .excluded_servers
        .push("rust-analyzer".to_string());

    assert!(
        config.excluded_servers.contains(&"rust-analyzer".to_string()),
        "rust-analyzer should be in excluded list"
    );
}

#[tokio::test]
async fn integration_test_lsp_client_new_not_running() {
    let client = LspClient::new();
    assert!(
        !client.is_healthy(),
        "New client should not be healthy since it hasn't been started"
    );
    assert_eq!(
        client.get_consecutive_error_count(),
        0,
        "New client should have zero consecutive errors"
    );
}

#[tokio::test]
async fn integration_test_lsp_client_with_custom_config() {
    let config = FailureHandlingConfig::new()
        .with_max_consecutive_errors(10)
        .with_auto_restart(false);
    let client = LspClient::with_config(config);

    assert!(
        !client.is_healthy(),
        "Client with custom config should still not be healthy before start"
    );
    assert_eq!(client.get_config().max_consecutive_errors, 10);
    assert!(!client.get_config().auto_restart);
}

#[tokio::test]
async fn integration_test_lsp_manager_unknown_file_noop() {
    let root = std::env::current_dir().unwrap();
    let mut manager = LspManager::new(root);

    manager
        .start_for_file(PathBuf::from("README.unknown_ext").as_path())
        .await
        .unwrap();

    assert_eq!(
        manager.get_total_diagnostic_count(),
        0,
        "Unknown file extension should result in no diagnostics"
    );
}

#[tokio::test]
async fn integration_test_diagnostic_aggregator_empty() {
    let aggregator = DiagnosticAggregator::new();
    assert_eq!(
        aggregator.get_total_diagnostic_count(),
        0,
        "New aggregator should have zero diagnostics"
    );
}

#[tokio::test]
async fn integration_test_diagnostic_aggregator_ingest_and_retrieve() {
    use std::path::PathBuf;

    let mut aggregator = DiagnosticAggregator::new();
    let path = PathBuf::from("test.rs");

    let diagnostic = Diagnostic {
        severity: Severity::Error,
        message: "test error".to_string(),
        range: Range {
            start: Position {
                line: 1,
                character: 1,
            },
            end: Position {
                line: 1,
                character: 4,
            },
        },
        source: Some("rust-analyzer".to_string()),
        file_path: None,
    };

    aggregator.ingest(&path, vec![diagnostic]);

    assert_eq!(
        aggregator.get_total_diagnostic_count(),
        1,
        "Aggregator should have one diagnostic after ingest"
    );

    let diags = aggregator.get_diagnostics_for_file(&path);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].message, "test error");
}

#[tokio::test]
async fn integration_test_diagnostic_aggregator_clear() {
    use std::path::PathBuf;

    let mut aggregator = DiagnosticAggregator::new();
    let path = PathBuf::from("test.rs");

    let diagnostic = Diagnostic {
        severity: Severity::Warning,
        message: "test warning".to_string(),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 5,
            },
        },
        source: Some("rust-analyzer".to_string()),
        file_path: None,
    };

    aggregator.ingest(&path, vec![diagnostic]);
    assert_eq!(aggregator.get_total_diagnostic_count(), 1);

    aggregator.clear_for_file(&path);
    assert_eq!(
        aggregator.get_total_diagnostic_count(),
        0,
        "Aggregator should have zero diagnostics after clear"
    );
}

#[tokio::test]
async fn integration_test_diagnostic_aggregator_summary() {
    use std::path::PathBuf;

    let mut aggregator = DiagnosticAggregator::new();
    let path = PathBuf::from("test.rs");

    let error_diag = Diagnostic {
        severity: Severity::Error,
        message: "error".to_string(),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        },
        source: Some("test".to_string()),
        file_path: None,
    };

    let warning_diag = Diagnostic {
        severity: Severity::Warning,
        message: "warning".to_string(),
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        },
        source: Some("test".to_string()),
        file_path: None,
    };

    aggregator.ingest(&path, vec![error_diag, warning_diag]);

    let summary = aggregator.get_diagnostics_summary();
    assert_eq!(summary.get(&Severity::Error), Some(&1));
    assert_eq!(summary.get(&Severity::Warning), Some(&1));
}

#[tokio::test]
async fn integration_test_failure_handling_crash_detection() {
    let cause = CrashCause::ProcessExited { code: 1 };
    let err = LspError::ServerCrash {
        server_name: "rust-analyzer".to_string(),
        cause,
    };

    assert_eq!(err.code(), 4100);
    assert!(err.to_string().contains("rust-analyzer"));
    assert!(err.to_string().contains("crashed"));
}

#[tokio::test]
async fn integration_test_failure_handling_request_timeout() {
    let err = LspError::RequestTimeout {
        method: "textDocument/definition".to_string(),
        timeout_ms: 5000,
    };

    assert_eq!(err.code(), 4101);
    assert_eq!(err.http_status(), 504);
    assert!(err.to_string().contains("textDocument/definition"));
}

#[tokio::test]
async fn integration_test_failure_handling_protocol_violation() {
    let err = LspError::ProtocolViolation {
        violation: ProtocolViolationType::InvalidJson,
        detail: "missing jsonrpc field".to_string(),
    };

    assert_eq!(err.code(), 4102);
    assert_eq!(err.http_status(), 422);
    assert!(err.to_string().contains("protocol violation"));
}

#[tokio::test]
async fn integration_test_failure_handling_server_unhealthy() {
    let err = LspError::ServerUnhealthy {
        server_name: "rust-analyzer".to_string(),
        reason: UnhealthyReason::NotResponding,
    };

    assert_eq!(err.code(), 4103);
    assert_eq!(err.http_status(), 503);
}

#[tokio::test]
async fn integration_test_failure_handling_invalid_message() {
    let err = LspError::InvalidMessage("truncated JSON".to_string());
    assert_eq!(err.code(), 4104);
    assert_eq!(err.http_status(), 400);
}

#[tokio::test]
async fn integration_test_failure_handling_capability_not_supported() {
    let err = LspError::CapabilityNotSupported {
        capability: "textDocument/hover".to_string(),
        server: "pylsp".to_string(),
    };

    assert_eq!(err.code(), 4105);
    assert_eq!(err.http_status(), 501);
}

#[tokio::test]
async fn integration_test_failure_handling_config_default() {
    let config = FailureHandlingConfig::default();

    assert_eq!(config.default_request_timeout_ms, 30_000);
    assert_eq!(config.max_consecutive_errors, 5);
    assert_eq!(config.health_check_interval_ms, 5_000);
    assert!(config.auto_restart);
}

#[tokio::test]
async fn integration_test_failure_handling_config_builder() {
    let config = FailureHandlingConfig::new()
        .with_request_timeout(std::time::Duration::from_secs(60))
        .with_max_consecutive_errors(10)
        .with_auto_restart(false);

    assert_eq!(config.default_request_timeout_ms, 60_000);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(!config.auto_restart);
}

#[tokio::test]
async fn integration_test_lsp_error_to_opencode_error() {
    let err = LspError::RequestTimeout {
        method: "textDocument/hover".to_string(),
        timeout_ms: 3000,
    };
    let oc_err = err.into_opencode_error();

    assert!(matches!(
        oc_err,
        opencode_core::OpenCodeError::ToolTimeout { .. }
    ));
}

#[tokio::test]
async fn integration_test_crash_causes_all_types() {
    let causes = vec![
        (CrashCause::ProcessExited { code: 0 }, "exited with code 0"),
        (CrashCause::Killed, "killed"),
        (
            CrashCause::Panic {
                message: "assertion failed".to_string(),
            },
            "assertion failed",
        ),
        (CrashCause::BrokenPipe, "broken pipe"),
        (
            CrashCause::ConnectionRefused,
            "connection refused",
        ),
        (
            CrashCause::Unknown("test".to_string()),
            "unknown",
        ),
    ];

    for (cause, expected_substring) in causes {
        let msg = cause.to_string();
        assert!(
            msg.contains(expected_substring),
            "Expected '{}' to contain '{}'",
            msg,
            expected_substring
        );
    }
}

#[tokio::test]
async fn integration_test_unhealthy_reasons_all_types() {
    assert_eq!(
        format!("{:?}", UnhealthyReason::NotResponding),
        "NotResponding"
    );
    assert_eq!(
        format!("{:?}", UnhealthyReason::ErrorThresholdExceeded),
        "ErrorThresholdExceeded"
    );
    assert_eq!(
        format!("{:?}", UnhealthyReason::ZombieProcess),
        "ZombieProcess"
    );
}

#[tokio::test]
async fn integration_test_language_detection_rust() {
    assert_eq!(
        Language::detect(&PathBuf::from("main.rs")),
        Language::Rust
    );
    assert_eq!(
        Language::detect(&PathBuf::from("lib.rs")),
        Language::Rust
    );
}

#[tokio::test]
async fn integration_test_language_detection_typescript() {
    assert_eq!(
        Language::detect(&PathBuf::from("app.ts")),
        Language::TypeScript
    );
    assert_eq!(
        Language::detect(&PathBuf::from("app.tsx")),
        Language::TypeScript
    );
}

#[tokio::test]
async fn integration_test_language_detection_javascript() {
    assert_eq!(
        Language::detect(&PathBuf::from("app.js")),
        Language::JavaScript
    );
    assert_eq!(
        Language::detect(&PathBuf::from("app.jsx")),
        Language::JavaScript
    );
}

#[tokio::test]
async fn integration_test_language_detection_python() {
    assert_eq!(
        Language::detect(&PathBuf::from("main.py")),
        Language::Python
    );
}

#[tokio::test]
async fn integration_test_language_detection_go() {
    assert_eq!(
        Language::detect(&PathBuf::from("main.go")),
        Language::Go
    );
}

#[tokio::test]
async fn integration_test_language_detection_unknown() {
    assert_eq!(
        Language::detect(&PathBuf::from("README.txt")),
        Language::Unknown
    );
    assert_eq!(
        Language::detect(&PathBuf::from("data.json")),
        Language::Unknown
    );
}

#[tokio::test]
async fn integration_test_language_detect_from_root() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let languages = Language::detect_from_root(dir.path());
    assert!(
        languages.contains(&Language::Rust),
        "Should detect Rust from Cargo.toml"
    );
}

#[tokio::test]
async fn integration_test_language_server_command() {
    assert_eq!(Language::Rust.server_command(), Some("rust-analyzer"));
    assert_eq!(
        Language::TypeScript.server_command(),
        Some("typescript-language-server --stdio")
    );
    assert_eq!(
        Language::JavaScript.server_command(),
        Some("typescript-language-server --stdio")
    );
    assert_eq!(Language::Python.server_command(), Some("pyright-langserver --stdio"));
    assert_eq!(Language::Go.server_command(), Some("gopls"));
    assert_eq!(Language::Unknown.server_command(), None);
}

#[tokio::test]
async fn integration_test_detection_result_serialization() {
    let result = DetectionResult {
        server: "Rust Analyzer".to_string(),
        is_available: true,
        language: "Rust".to_string(),
        command: "rust-analyzer".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap();
    let deserialized: DetectionResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.server, result.server);
    assert_eq!(deserialized.is_available, result.is_available);
    assert_eq!(deserialized.language, result.language);
    assert_eq!(deserialized.command, result.command);
}

#[tokio::test]
async fn integration_test_diagnostic_severity_conversion() {
    use opencode_lsp::types::Severity;

    assert_eq!(Severity::from(1), Severity::Error);
    assert_eq!(Severity::from(2), Severity::Warning);
    assert_eq!(Severity::from(3), Severity::Information);
    assert_eq!(Severity::from(4), Severity::Hint);
    assert_eq!(Severity::from(999), Severity::Warning);
}

#[tokio::test]
async fn integration_test_protocol_violation_type_display() {
    assert_eq!(
        format!("{}", ProtocolViolationType::InvalidJson),
        "InvalidJson"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::MissingField),
        "MissingField"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::InvalidMessageType),
        "InvalidMessageType"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::ResponseIdMismatch),
        "ResponseIdMismatch"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::MissingContentLength),
        "MissingContentLength"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::InvalidContentLength),
        "InvalidContentLength"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::InvalidBatch),
        "InvalidBatch"
    );
    assert_eq!(
        format!("{}", ProtocolViolationType::UnexpectedResponse),
        "UnexpectedResponse"
    );
}
