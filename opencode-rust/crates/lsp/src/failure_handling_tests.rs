use crate::client::LspClient;
use crate::error::{
    CrashCause, FailureHandlingConfig, LspError, ProtocolViolationType, UnhealthyReason,
};
use std::time::Duration;

#[tokio::test]
async fn test_server_crash_detection() {
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
async fn test_request_timeout_error() {
    let err = LspError::RequestTimeout {
        method: "textDocument/definition".to_string(),
        timeout_ms: 5000,
    };

    assert_eq!(err.code(), 4101);
    assert_eq!(err.http_status(), 504);
    assert!(err.to_string().contains("textDocument/definition"));
    assert!(err.to_string().contains("5000"));
}

#[tokio::test]
async fn test_protocol_violation_error() {
    let err = LspError::ProtocolViolation {
        violation: ProtocolViolationType::InvalidJson,
        detail: "missing jsonrpc field".to_string(),
    };

    assert_eq!(err.code(), 4102);
    assert_eq!(err.http_status(), 422);
    assert!(err.to_string().contains("protocol violation"));
}

#[tokio::test]
async fn test_protocol_violation_types() {
    let violations = vec![
        (ProtocolViolationType::InvalidJson, "InvalidJson"),
        (ProtocolViolationType::MissingField, "MissingField"),
        (ProtocolViolationType::InvalidMessageType, "InvalidMessageType"),
        (
            ProtocolViolationType::ResponseIdMismatch,
            "ResponseIdMismatch",
        ),
        (
            ProtocolViolationType::MissingContentLength,
            "MissingContentLength",
        ),
        (
            ProtocolViolationType::InvalidContentLength,
            "InvalidContentLength",
        ),
        (ProtocolViolationType::InvalidBatch, "InvalidBatch"),
        (
            ProtocolViolationType::UnexpectedResponse,
            "UnexpectedResponse",
        ),
    ];

    for (violation, name) in violations {
        assert_eq!(format!("{:?}", violation), name);
    }
}

#[tokio::test]
async fn test_server_unhealthy_error() {
    let err = LspError::ServerUnhealthy {
        server_name: "rust-analyzer".to_string(),
        reason: UnhealthyReason::NotResponding,
    };

    assert_eq!(err.code(), 4103);
    assert_eq!(err.http_status(), 503);
    assert!(err.to_string().contains("unhealthy"));
}

#[tokio::test]
async fn test_invalid_message_error() {
    let err = LspError::InvalidMessage("truncated JSON".to_string());
    assert_eq!(err.code(), 4104);
    assert_eq!(err.http_status(), 400);
}

#[tokio::test]
async fn test_capability_not_supported_error() {
    let err = LspError::CapabilityNotSupported {
        capability: "textDocument/hover".to_string(),
        server: "pylsp".to_string(),
    };

    assert_eq!(err.code(), 4105);
    assert_eq!(err.http_status(), 501);
    assert!(err.to_string().contains("textDocument/hover"));
    assert!(err.to_string().contains("pylsp"));
}

#[tokio::test]
async fn test_crash_causes() {
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
async fn test_lsp_error_to_opencode_error() {
    let err = LspError::RequestTimeout {
        method: "textDocument/hover".to_string(),
        timeout_ms: 3000,
    };
    let oc_err = err.into_opencode_error();

    assert!(matches!(oc_err, opencode_core::OpenCodeError::ToolTimeout { .. }));
}

#[tokio::test]
async fn test_lsp_error_convert_from() {
    let lsp_err = LspError::ServerCrash {
        server_name: "gopls".to_string(),
        cause: CrashCause::Killed,
    };
    let oc_err: opencode_core::OpenCodeError = lsp_err.into();
    assert!(matches!(oc_err, opencode_core::OpenCodeError::ServiceUnavailable(_)));
}

#[tokio::test]
async fn test_failure_handling_config_default() {
    let config = FailureHandlingConfig::default();

    assert_eq!(config.default_request_timeout_ms, 30_000);
    assert_eq!(config.max_consecutive_errors, 5);
    assert_eq!(config.health_check_interval_ms, 5_000);
    assert!(config.auto_restart);
}

#[tokio::test]
async fn test_failure_handling_config_builder() {
    let config = FailureHandlingConfig::new()
        .with_request_timeout(Duration::from_secs(60))
        .with_max_consecutive_errors(10)
        .with_auto_restart(false);

    assert_eq!(config.default_request_timeout_ms, 60_000);
    assert_eq!(config.max_consecutive_errors, 10);
    assert!(!config.auto_restart);
}

#[tokio::test]
async fn test_failure_handling_config_builder_all() {
    let config = FailureHandlingConfig::new()
        .with_request_timeout(Duration::from_secs(120))
        .with_max_consecutive_errors(20)
        .with_auto_restart(true);

    assert_eq!(config.default_request_timeout_ms, 120_000);
    assert_eq!(config.max_consecutive_errors, 20);
    assert!(config.auto_restart);
}

#[tokio::test]
async fn test_unhealthy_reasons() {
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
async fn test_lsp_client_healthy_initial_state() {
    let client = LspClient::new();
    assert!(client.is_healthy());
    assert_eq!(client.get_consecutive_error_count(), 0);
}

#[tokio::test]
async fn test_lsp_client_with_custom_config() {
    let config = FailureHandlingConfig::new()
        .with_max_consecutive_errors(3)
        .with_auto_restart(false);
    let client = LspClient::with_config(config);

    assert!(client.is_healthy());
    assert_eq!(client.get_config().max_consecutive_errors, 3);
    assert!(!client.get_config().auto_restart);
}

#[tokio::test]
async fn test_lsp_client_server_name() {
    let client = LspClient::new();
    assert_eq!(client.get_server_name(), "");
}

#[tokio::test]
async fn test_protocol_violation_all_types_convert() {
    let violations = vec![
        ProtocolViolationType::InvalidJson,
        ProtocolViolationType::MissingField,
        ProtocolViolationType::InvalidMessageType,
        ProtocolViolationType::ResponseIdMismatch,
        ProtocolViolationType::MissingContentLength,
        ProtocolViolationType::InvalidContentLength,
        ProtocolViolationType::InvalidBatch,
        ProtocolViolationType::UnexpectedResponse,
    ];

    for violation in violations {
        let err = LspError::ProtocolViolation {
            violation,
            detail: "test".to_string(),
        };
        let oc_err: opencode_core::OpenCodeError = err.into();
        assert!(matches!(oc_err, opencode_core::OpenCodeError::Tool(_)));
    }
}

#[tokio::test]
async fn test_all_lsp_error_codes() {
    let errors = vec![
        (
            LspError::ServerCrash {
                server_name: "test".to_string(),
                cause: CrashCause::Killed,
            },
            4100u16,
        ),
        (
            LspError::RequestTimeout {
                method: "test".to_string(),
                timeout_ms: 1000,
            },
            4101,
        ),
        (
            LspError::ProtocolViolation {
                violation: ProtocolViolationType::InvalidJson,
                detail: "test".to_string(),
            },
            4102,
        ),
        (
            LspError::ServerUnhealthy {
                server_name: "test".to_string(),
                reason: UnhealthyReason::NotResponding,
            },
            4103,
        ),
        (
            LspError::InvalidMessage("test".to_string()),
            4104,
        ),
        (
            LspError::CapabilityNotSupported {
                capability: "test".to_string(),
                server: "test".to_string(),
            },
            4105,
        ),
    ];

    for (err, expected_code) in errors {
        assert_eq!(err.code(), expected_code);
    }
}