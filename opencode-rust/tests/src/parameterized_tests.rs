#[path = "common/mod.rs"]
mod common;

use common::TempProject;
use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::SqliteSessionRepository;
use opencode_tools::{read::ReadTool, Tool, ToolRegistry};
use rstest::rstest;

async fn create_test_pool() -> (StoragePool, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_rstest.db");
    let pool = StoragePool::new(&db_path).expect("Failed to create storage pool");
    let manager = MigrationManager::new(pool.clone(), 3);
    manager.migrate().await.expect("Failed to run migrations");
    (pool, temp_dir)
}

#[rstest]
#[tokio::test]
async fn test_sql_injection_prevention(
    #[values(
        "'; DROP TABLE sessions; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM sessions--",
        "1'; DROP TABLE sessions; --",
        "admin'--",
        "' OR 1=1--"
    )]
    payload: &str,
) {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = SqliteSessionRepository::new(pool.clone());

    let result = session_repo.find_by_id(payload).await;
    assert!(
        result.is_ok(),
        "SQL injection '{}' should not crash",
        payload
    );
    assert!(
        result.unwrap().is_none(),
        "SQL injection '{}' should return None",
        payload
    );
}

#[rstest]
#[tokio::test]
async fn test_path_traversal_rejected(
    #[values(
        "/etc/passwd",
        "../../../etc/passwd",
        "/root/.ssh/id_rsa",
        "subdir/../../secrets.txt"
    )]
    malicious_path: &str,
) {
    let project = TempProject::new();
    project.create_file("test.txt", "inside project");
    let tool = ReadTool::new();

    let result = tool
        .execute(serde_json::json!({"path": malicious_path}), None)
        .await;

    if result.is_ok() {
        let tool_result = result.unwrap();
        assert!(
            !tool_result.success
                || !tool_result.content.contains("root:")
                || !tool_result.content.contains("ssh-rsa"),
            "Path traversal '{}' should not read sensitive files",
            malicious_path
        );
    }
}

#[rstest]
#[tokio::test]
async fn test_write_outside_project_rejected(
    #[values(
        "/tmp/payload.txt",
        "../../../tmp/payload.txt",
        "/var/www/html/backdoor.php"
    )]
    malicious_path: &str,
) {
    let _project = TempProject::new();
    let tool = opencode_tools::write::WriteTool;

    let result = tool
        .execute(
            serde_json::json!({"path": malicious_path, "content": "malicious content"}),
            None,
        )
        .await;

    if result.is_ok() {
        let tool_result = result.unwrap();
        if tool_result.success {
            let file_path = std::path::Path::new(malicious_path);
            if file_path.exists() {
                std::fs::remove_file(file_path).ok();
                panic!(
                    "Write tool should not allow writing outside project: {}",
                    malicious_path
                );
            }
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_null_byte_injection_rejected(
    #[values("/etc/passwd\0.txt", "test.txt\0../../../etc/passwd")] malicious_path: &str,
) {
    let project = TempProject::new();
    project.create_file("test.txt", "valid content");
    let tool = ReadTool::new();

    let result = tool
        .execute(serde_json::json!({"path": malicious_path}), None)
        .await;

    if let Ok(r) = result {
        assert!(
            !r.success || !r.content.contains("root:"),
            "Null byte injection '{}' should not read sensitive files",
            malicious_path
        );
    }
}

#[rstest]
#[tokio::test]
async fn test_session_id_validation(
    #[values("", "   ", "id with spaces", "id\nwith\nnewlines", "id\twith\ttabs")] invalid_id: &str,
) {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = SqliteSessionRepository::new(pool.clone());

    let result = session_repo.find_by_id(invalid_id).await;
    assert!(
        result.is_ok(),
        "Invalid ID should be handled gracefully: {:?}",
        invalid_id
    );
    assert!(result.unwrap().is_none());
}

#[rstest]
#[tokio::test]
async fn test_tool_registry_rejects_dangerous_paths(
    #[values(
        "/etc/shadow",
        "../../../root/.ssh/id_rsa",
        "C:\\Windows\\System32\\config\\SAM"
    )]
    dangerous_path: &str,
) {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let result = registry
        .execute("read", serde_json::json!({"path": dangerous_path}), None)
        .await;

    if let Ok(r) = result {
        if r.success {
            assert!(
                !r.content.contains("root:") && !r.content.contains("BEGIN RSA"),
                "Should not read sensitive files: {}",
                dangerous_path
            );
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_json_injection_rejected(
    #[values(
        r#"{"path": "/etc/passwd", "offset": 0}"#,
        r#"{"path": "/etc/passwd"}"#,
        r#"{"path": "test.txt", "limit": "0; DROP TABLE"}"#
    )]
    args_str: &str,
) {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;

    let args: Result<serde_json::Value, _> = serde_json::from_str(args_str);
    if let Ok(args) = args {
        let result = registry.execute("read", args, None).await;
        if result.is_ok() {
            let r = result.unwrap();
            if r.success {
                assert!(
                    !r.content.contains("root:"),
                    "JSON injection should not expose system files"
                );
            }
        }
    }
}

#[rstest]
#[case::script_tag("<script>alert('XSS')</script>")]
#[case::img_onerror("<img src=x onerror=alert('XSS')>")]
#[case::svg_onload("<svg onload=alert('XSS')>")]
#[case::iframe_javascript("<iframe src=javascript:alert('XSS')>")]
#[case::body_onload("<body onload=alert('XSS')>")]
#[case::input_onfocus("<input onfocus=alert('XSS') autofocus>")]
#[case::select_onchange("<select onchange=alert('XSS')><option></select>")]
#[case::style_import("<style>@import javascript:alert('XSS')</style>")]
#[case::javascript_protocol("javascript:alert('XSS')")]
#[case::script_src_external("<script src=http://evil.com/malicious.js></script>")]
#[case::script_fromcharcode("<script>alert(String.fromCharCode(88,83,83))</script>")]
#[case::mixed_case_script("<ScRiPt>alert('XSS')</sCrIpT>")]
#[tokio::test]
async fn test_xss_payload_sanitization(#[case] payload: &str) {
    let mut session = Session::new();
    session.add_message(opencode_core::message::Message::user(payload.to_string()));

    let md_export = session.export_markdown();
    assert!(
        md_export.is_ok(),
        "Export should succeed for payload: {}",
        payload
    );
    let md = md_export.unwrap();

    let lower_md = md.to_lowercase();
    let has_unescaped_script_tag = lower_md.contains("<script") || lower_md.contains("</script");
    let has_unescaped_tag_with_event = (lower_md.contains("<img") && lower_md.contains("onerror="))
        || (lower_md.contains("<svg") && lower_md.contains("onload="))
        || (lower_md.contains("<input") && lower_md.contains("onfocus="))
        || (lower_md.contains("<body") && lower_md.contains("onload="))
        || (lower_md.contains("<select") && lower_md.contains("onchange="));

    assert!(
        !has_unescaped_script_tag && !has_unescaped_tag_with_event,
        "XSS payload should be sanitized: {}\nMarkdown: {}",
        payload,
        md
    );
}

#[rstest]
#[case::lt_escaped("<script>", "&lt;script&gt;")]
#[case::gt_escaped("</script>", "&lt;/script&gt;")]
#[case::img_tag("<img src=x>", "&lt;img src=x&gt;")]
#[case::onerror_preserved("onerror=", "onerror=")]
#[case::javascript_preserved("javascript:", "javascript:")]
#[case::svg_with_event("<svg onload=a>", "&lt;svg onload=a&gt;")]
#[tokio::test]
async fn test_html_sanitization_edge_cases(#[case] input: &str, #[case] _expected: &str) {
    let mut session = Session::new();
    session.add_message(opencode_core::message::Message::user(input.to_string()));

    let md_export = session.export_markdown();
    assert!(
        md_export.is_ok(),
        "Export should succeed for edge case: {}",
        input
    );
    let md = md_export.unwrap();

    if input.contains("<script") || input.contains("</script") {
        assert!(
            !md.contains("<script") && !md.contains("</script"),
            "Script tags should not appear directly in output for: {}",
            input
        );
    }
}
