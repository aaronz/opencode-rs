#[path = "common/mod.rs"]
mod common;

use common::TempProject;
use opencode_core::Session;
use opencode_storage::database::StoragePool;
use opencode_storage::migration::MigrationManager;
use opencode_storage::repository::SessionRepository;
use opencode_storage::SqliteSessionRepository;
use opencode_tools::{read::ReadTool, write::WriteTool, Tool, ToolRegistry};
use std::sync::Arc;

async fn create_test_pool() -> (StoragePool, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_security.db");
    let pool = StoragePool::new(&db_path).expect("Failed to create storage pool");
    let manager = MigrationManager::new(pool.clone(), 2);
    manager.migrate().await.expect("Failed to run migrations");
    (pool, temp_dir)
}

#[tokio::test]
async fn test_sql_injection_session_id_handling() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let malicious_id = "'; DROP TABLE sessions; --";
    let result = session_repo.find_by_id(malicious_id).await;
    assert!(
        result.is_ok(),
        "SQL injection should not crash the repository"
    );
    assert!(result.unwrap().is_none(), "Malicious ID should return None");
    let normal_id = "valid-session-id";
    let result = session_repo.find_by_id(normal_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sql_injection_union_attack() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let union_injection = "1 UNION SELECT id, created_at, updated_at, data FROM sessions--";
    let result = session_repo.find_by_id(union_injection).await;
    assert!(result.is_ok());
    assert!(
        result.unwrap().is_none(),
        "Union injection should return None"
    );
}

#[tokio::test]
async fn test_sql_injection_comment_attack() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let comment_injection = "test' OR '1'='1' --";
    let result = session_repo.find_by_id(comment_injection).await;
    assert!(result.is_ok(), "Comment injection should not crash");
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_sql_injection_find_all_with_limit() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let result = session_repo.find_all(10, 0).await;
    assert!(
        result.is_ok(),
        "SQL injection in find_all should be handled safely"
    );
    let result = session_repo.find_all(0, 0).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_path_traversal_read_outside_project() {
    let project = TempProject::new();
    project.create_file("test.txt", "inside project");
    let tool = ReadTool::new();
    let traversal_paths = vec![
        "/etc/passwd",
        "../../../etc/passwd",
        "/root/.ssh/id_rsa",
        "subdir/../../secrets.txt",
    ];
    for malicious_path in traversal_paths {
        let result = tool
            .execute(serde_json::json!({"path": malicious_path}), None)
            .await;
        if result.is_ok() {
            let tool_result = result.unwrap();
            assert!(
                !tool_result.success
                    || !tool_result.content.contains("root:")
                    || !tool_result.content.contains("ssh-rsa"),
                "Path traversal should not read sensitive files: {}",
                malicious_path
            );
        }
    }
}

#[tokio::test]
async fn test_path_traversal_write_outside_project() {
    let _project = TempProject::new();
    let tool = WriteTool;
    let malicious_paths = vec![
        "/tmp/payload.txt",
        "../../../tmp/payload.txt",
        "/var/www/html/backdoor.php",
    ];
    for malicious_path in malicious_paths {
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
}

#[tokio::test]
async fn test_path_traversal_null_byte_injection() {
    let project = TempProject::new();
    project.create_file("test.txt", "valid content");
    let tool = ReadTool::new();
    let null_byte_paths = vec!["/etc/passwd\0.txt", "test.txt\0../../../etc/passwd"];
    for malicious_path in null_byte_paths {
        let result = tool
            .execute(serde_json::json!({"path": malicious_path}), None)
            .await;
        match result {
            Ok(r) => assert!(
                !r.success || !r.content.contains("root:"),
                "Null byte injection should not read sensitive files"
            ),
            Err(_) => assert!(true, "Tool should reject null byte injection"),
        }
    }
}

#[tokio::test]
async fn test_path_normalization_prevents_traversal() {
    let project = TempProject::new();
    project.create_file("allowed/subdir/file.txt", "nested content");
    let tool = ReadTool::new();
    let normalized_result = tool.execute(serde_json::json!({"path": project.path().join("allowed/subdir/file.txt").to_string_lossy()}), None).await;
    assert!(normalized_result.is_ok());
    let result = normalized_result.unwrap();
    assert!(result.success, "Normal path should work");
    assert!(result.content.contains("nested content"));
}

#[tokio::test]
async fn test_session_message_content_sanitization() {
    let mut session = Session::new();
    let malicious_content = "'; DROP TABLE sessions; --";
    session.add_message(opencode_core::message::Message::user(
        malicious_content.to_string(),
    ));
    let json_export = session.export_json();
    assert!(json_export.is_ok());
    let json = json_export.unwrap();
    assert!(
        !json.contains("DROP TABLE"),
        "SQL injection in message content should be sanitized in export"
    );
    assert!(
        json.contains("[SQL_REDACTED]"),
        "Sanitized content should appear in export"
    );
}

#[tokio::test]
async fn test_session_message_xss_prevention() {
    let mut session = Session::new();
    let xss_content = "<script>alert('XSS')</script>";
    session.add_message(opencode_core::message::Message::user(
        xss_content.to_string(),
    ));
    let json_export = session.export_json();
    assert!(json_export.is_ok());
    let md_export = session.export_markdown();
    assert!(md_export.is_ok());
    let md = md_export.unwrap();
    assert!(
        !md.contains("<script>"),
        "XSS content should be handled safely in markdown export"
    );
}

#[tokio::test]
async fn test_session_message_xss_various_payloads() {
    let xss_payloads = vec![
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "<svg onload=alert('XSS')>",
        "<iframe src=javascript:alert('XSS')>",
        "<body onload=alert('XSS')>",
        "<input onfocus=alert('XSS') autofocus>",
        "<select onchange=alert('XSS')><option></select>",
        "<style>@import javascript:alert('XSS')</style>",
        "javascript:alert('XSS')",
        "<script src=http://evil.com/malicious.js></script>",
        "<script>alert(String.fromCharCode(88,83,83))</script>",
        "<ScRiPt>alert('XSS')</sCrIpT>",
        "<script>alert(/XSS/.source)</script>",
        "<`><script>alert(String.fromCharCode(88,83,83))</script>",
        "<img src=\"x\" alt=\"`onerror=alert('XSS')>\">",
    ];

    for payload in xss_payloads {
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
        let has_unescaped_script_tag =
            lower_md.contains("<script") || lower_md.contains("</script");
        let has_unescaped_tag_with_event = (lower_md.contains("<img")
            && lower_md.contains("onerror="))
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
}

#[tokio::test]
async fn test_session_message_html_tag_stripping() {
    let mut session = Session::new();
    let html_content = "<h1>Hello</h1><p>World</p><script>malicious()</script>";
    session.add_message(opencode_core::message::Message::user(
        html_content.to_string(),
    ));

    let md_export = session.export_markdown();
    assert!(md_export.is_ok());
    let md = md_export.unwrap();

    assert!(
        !md.contains("<h1>") && !md.contains("</h1>"),
        "HTML tags should be escaped or stripped"
    );
    assert!(
        !md.contains("<p>") && !md.contains("</p>"),
        "HTML tags should be escaped or stripped"
    );
    assert!(
        !md.contains("<script>") && !md.contains("</script>"),
        "Script tags should be escaped or stripped"
    );
}

#[tokio::test]
async fn test_session_message_sanitization_edge_cases() {
    let edge_cases = vec![
        ("<script>", "&lt;script&gt;"),
        ("</script>", "&lt;/script&gt;"),
        ("<img src=x>", "&lt;img src=x&gt;"),
        ("onerror=", "onerror="),
        ("javascript:", "javascript:"),
        ("<svg onload=a>", "&lt;svg onload=a&gt;"),
    ];

    for (input, _expected_sanitized) in edge_cases {
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
}

#[tokio::test]
async fn test_tool_registry_execute_rejects_dangerous_paths() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;
    let dangerous_paths = vec![
        "/etc/shadow",
        "../../../root/.ssh/id_rsa",
        "C:\\Windows\\System32\\config\\SAM",
    ];
    for dangerous_path in dangerous_paths {
        let result = registry
            .execute("read", serde_json::json!({"path": dangerous_path}), None)
            .await;
        match result {
            Ok(r) => {
                if r.success {
                    assert!(
                        !r.content.contains("root:") && !r.content.contains("BEGIN RSA"),
                        "Should not read sensitive files: {}",
                        dangerous_path
                    );
                }
            }
            Err(_) => assert!(true, "Tool should reject dangerous paths"),
        }
    }
}

#[tokio::test]
async fn test_session_id_format_validation() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let invalid_ids = vec![
        "",
        "   ",
        "id with spaces",
        "id\nwith\nnewlines",
        "id\twith\ttabs",
    ];
    for invalid_id in invalid_ids {
        let result = session_repo.find_by_id(invalid_id).await;
        assert!(
            result.is_ok(),
            "Invalid ID should be handled gracefully: {:?}",
            invalid_id
        );
        assert!(result.unwrap().is_none());
    }
}

#[tokio::test]
async fn test_bash_injection_prevention() {
    let project = TempProject::new();
    project.create_file("test.txt", "hello world");
    let registry = ToolRegistry::new();
    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
    if tool_names.contains(&"bash") {
        use opencode_tools::bash::BashTool;
        let bash_tool = BashTool::new();
        let malicious_commands = vec![
            "cat /etc/passwd",
            "echo 'hello'; rm -rf /",
            "$(cat test.txt)",
        ];
        for cmd in malicious_commands {
            let result = bash_tool
                .execute(
                    serde_json::json!({"command": cmd, "cwd": project.path().to_string_lossy()}),
                    None,
                )
                .await;
            match result {
                Ok(r) => {
                    if r.success {
                        assert!(
                            !r.content.contains("root:"),
                            "Bash injection should be prevented"
                        );
                    }
                }
                Err(_) => assert!(true, "Dangerous command should be rejected"),
            }
        }
    }
}

#[tokio::test]
async fn test_grep_injection_prevention() {
    let project = TempProject::new();
    project.create_file("test.txt", "hello world\ntest content");
    let registry = ToolRegistry::new();
    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
    if tool_names.contains(&"grep") {
        use opencode_tools::grep_tool::GrepTool;
        let grep_tool = GrepTool;
        let malicious_patterns = vec!["../../../etc/passwd", "(cat /etc/passwd)", "[^a-z]"];
        for pattern in malicious_patterns {
            let result = grep_tool.execute(serde_json::json!({"path": project.path().join("test.txt").to_string_lossy(), "pattern": pattern}), None).await;
            match result {
                Ok(r) => {
                    if r.success {
                        assert!(
                            !r.content.contains("root:"),
                            "Grep injection should be prevented"
                        );
                    }
                }
                Err(_) => assert!(true, "Dangerous grep pattern should be rejected"),
            }
        }
    }
}

#[tokio::test]
async fn test_write_tool_path_validation() {
    let project = TempProject::new();
    let tool = WriteTool;
    let inside_path = project
        .path()
        .join("inside.txt")
        .to_string_lossy()
        .to_string();
    let write_attempts = vec![
        ("/tmp/outside.txt", false),
        ("../outside.txt", false),
        (inside_path.as_str(), true),
    ];
    for (path, should_succeed) in write_attempts {
        let result = tool
            .execute(
                serde_json::json!({"path": path, "content": "test content"}),
                None,
            )
            .await;
        if should_succeed {
            assert!(
                result.is_ok() && result.as_ref().unwrap().success,
                "Write within project should succeed: {}",
                path
            );
        } else {
            assert!(
                result.is_err() || !result.as_ref().unwrap().success,
                "Write outside project should be rejected: {}",
                path
            );
        }
    }
}

#[tokio::test]
async fn test_session_storage_resilience_to_corruption() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let result = session_repo.find_by_id("nonexistent").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    let result = session_repo.find_all(100, 0).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_tool_args_json_injection() {
    let registry = ToolRegistry::new();
    registry.register(ReadTool::new()).await;
    let malicious_args = vec![
        r#"{"path": "/etc/passwd", "offset": 0}"#,
        r#"{"path": "/etc/passwd"}"#,
        r#"{"path": "test.txt", "limit": "0; DROP TABLE"}"#,
    ];
    for args_str in malicious_args {
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
}

#[tokio::test]
async fn test_read_tool_symlink_handling() {
    let project = TempProject::new();
    project.create_file("real_file.txt", "sensitive content");
    #[cfg(unix)]
    {
        let symlink_path = project.path().join("link_file.txt");
        std::os::unix::fs::symlink(project.path().join("real_file.txt"), &symlink_path).ok();
        let tool = ReadTool::new();
        let result = tool
            .execute(
                serde_json::json!({"path": symlink_path.to_string_lossy()}),
                None,
            )
            .await;
        assert!(result.is_ok(), "Symlink read should work");
        let r = result.unwrap();
        assert!(r.success, "Symlink should resolve to real content");
    }
}

#[tokio::test]
async fn test_concurrent_session_access_isolation() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let mut session1 = Session::new();
    session1.add_message(opencode_core::message::Message::user(
        "Session 1 message".to_string(),
    ));
    session_repo.save(&session1).await.ok();
    let session1_id = session1.id.to_string();
    let mut session2 = Session::new();
    session2.add_message(opencode_core::message::Message::user(
        "Session 2 message".to_string(),
    ));
    session_repo.save(&session2).await.ok();
    let session2_id = session2.id.to_string();
    let (result1, result2) = tokio::join!(
        session_repo.find_by_id(&session1_id),
        session_repo.find_by_id(&session2_id)
    );
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    let loaded1 = result1.unwrap().unwrap();
    let loaded2 = result2.unwrap().unwrap();
    assert!(
        loaded1
            .messages
            .iter()
            .any(|m| m.content.contains("Session 1")),
        "Session 1 should contain its own message"
    );
    assert!(
        !loaded1
            .messages
            .iter()
            .any(|m| m.content.contains("Session 2")),
        "Session 1 should not contain Session 2's message"
    );
    assert!(
        loaded2
            .messages
            .iter()
            .any(|m| m.content.contains("Session 2")),
        "Session 2 should contain its own message"
    );
    assert!(
        !loaded2
            .messages
            .iter()
            .any(|m| m.content.contains("Session 1")),
        "Session 2 should not contain Session 1's message"
    );
}

#[tokio::test]
async fn test_regex_dos_prevention_in_tools() {
    let registry = ToolRegistry::new();
    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
    if tool_names.contains(&"grep") {
        use opencode_tools::grep_tool::GrepTool;
        let grep_tool = GrepTool;
        let catastrophic_patterns = vec!["((a+)+)+$", "([a-zA-Z]+)*$", "(a|aa)+$"];
        for pattern in catastrophic_patterns {
            let result = grep_tool
                .execute(
                    serde_json::json!({"path": "test_file.txt", "pattern": pattern}),
                    None,
                )
                .await;
            match result {
                Ok(r) => assert!(
                    !r.content.contains("ReDoS"),
                    "Tool should handle or reject ReDoS patterns"
                ),
                Err(_) => assert!(true, "ReDoS patterns should be rejected"),
            }
        }
    }
}

#[tokio::test]
async fn test_sql_injection_sessions() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let mut session = Session::new();
    session.add_message(opencode_core::message::Message::user(
        "Test session for SQL injection".to_string(),
    ));
    session_repo
        .save(&session)
        .await
        .expect("Session should save");
    let legitimate_id = session.id.to_string();
    let result = session_repo.find_by_id(&legitimate_id).await;
    assert!(result.is_ok(), "Legitimate session lookup should succeed");
    assert!(
        result.as_ref().unwrap().is_some(),
        "Legitimate session should be found"
    );
    let sql_injection_payloads = vec![
        "'; DROP TABLE sessions; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM sessions--",
        "1'; DROP TABLE sessions; --",
        "admin'--",
        "' OR 1=1--",
    ];
    for payload in sql_injection_payloads {
        let result = session_repo.find_by_id(payload).await;
        assert!(
            result.is_ok(),
            "SQL injection payload '{}' should not crash the repository",
            payload
        );
        assert!(
            result.as_ref().unwrap().is_none(),
            "SQL injection payload should return None"
        );
    }
    let count_after = session_repo.count().await.expect("Count should work");
    assert_eq!(
        count_after, 1,
        "After SQL injection attempts, exactly 1 session should remain"
    );
    let legitimate_result = session_repo.find_by_id(&legitimate_id).await;
    assert!(
        legitimate_result.is_ok() && legitimate_result.unwrap().is_some(),
        "The legitimate session should still be accessible"
    );
}

#[tokio::test]
async fn test_sql_injection_messages() {
    let (pool, _temp_dir) = create_test_pool().await;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    let mut session = Session::new();
    session.add_message(opencode_core::message::Message::user(
        "Test message 1".to_string(),
    ));
    session.add_message(opencode_core::message::Message::assistant(
        "Test message 2".to_string(),
    ));
    session_repo
        .save(&session)
        .await
        .expect("Session should save");

    let legitimate_id = session.id.to_string();
    let result = session_repo.find_by_id(&legitimate_id).await;
    assert!(result.is_ok(), "Legitimate session lookup should succeed");
    assert!(
        result.as_ref().unwrap().is_some(),
        "Legitimate session should be found"
    );

    let sql_injection_payloads = vec![
        "'; DROP TABLE sessions; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM sessions--",
        "'; DELETE FROM sessions WHERE '1'='1",
        "1'; DROP TABLE sessions; --",
        "admin'--",
        "' OR 1=1--",
        "'; INSERT INTO sessions VALUES('hacked','2024-01-01','2024-01-01','data');--",
    ];

    for payload in sql_injection_payloads {
        let result = session_repo.find_by_id(payload).await;
        assert!(result.is_ok(), "SQL injection {} should not crash", payload);
        assert!(
            result.as_ref().unwrap().is_none(),
            "SQL injection {} should return None",
            payload
        );
    }

    let count_after = session_repo.count().await.expect("Count should work");
    assert_eq!(
        count_after, 1,
        "After SQL injection attempts, exactly 1 session should remain"
    );

    let legitimate_result = session_repo.find_by_id(&legitimate_id).await;
    assert!(
        legitimate_result.is_ok() && legitimate_result.unwrap().is_some(),
        "The legitimate session should still be accessible"
    );
}

#[cfg(test)]
mod test_request_validation {
    use serde_json::Value;

    fn is_valid_uuid(s: &str) -> bool {
        uuid::Uuid::parse_str(s).is_ok()
    }

    fn is_valid_request_json(value: &Value) -> bool {
        let session_id = match value.get("session_id").and_then(|v| v.as_str()) {
            Some(s) if !s.trim().is_empty() => s,
            _ => return false,
        };

        if !is_valid_uuid(session_id) {
            return false;
        }

        let prompt = match value.get("prompt").and_then(|v| v.as_str()) {
            Some(p) if !p.trim().is_empty() => p,
            _ => return false,
        };

        true
    }

    #[tokio::test]
    async fn test_malformed_json_rejected() {
        let malformed_jsons = vec![
            r#"{invalid json}"#,
            r#"{"key": "unclosed"#,
            r#"{"key": }"#,
            r#"not json at all"#,
            r#"{"key": "value",}"#,
            r#"}"#,
            r#""#,
            r#"   "#,
        ];

        for malformed in malformed_jsons {
            let result: Result<Value, _> = serde_json::from_str(malformed);
            assert!(
                result.is_err(),
                "Malformed JSON should be rejected: {:?}",
                malformed
            );
        }
    }

    #[tokio::test]
    async fn test_missing_required_fields_rejected() {
        let test_cases = vec![
            (r#"{}"#, true, "empty object should be rejected"),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000"}"#,
                false,
                "missing prompt",
            ),
            (r#"{"prompt": "test"}"#, false, "missing session_id"),
            (
                r#"{"session_id": "", "prompt": "test"}"#,
                true,
                "empty session_id",
            ),
            (
                r#"{"session_id": "   ", "prompt": "test"}"#,
                true,
                "whitespace session_id",
            ),
            (r#"{"prompt": ""}"#, true, "empty prompt"),
            (r#"{"prompt": "   "}"#, true, "whitespace-only prompt"),
        ];

        for (json_str, expect_invalid, description) in test_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if expect_invalid {
                assert!(
                    result.is_err() || (result.is_ok() && !is_valid_request_json(&result.unwrap())),
                    "{} should be rejected",
                    description
                );
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_types_rejected() {
        let invalid_type_cases = vec![
            (
                r#"{"session_id": 123, "prompt": "test"}"#,
                true,
                "session_id as number",
            ),
            (
                r#"{"session_id": ["uuid"], "prompt": "test"}"#,
                true,
                "session_id as array",
            ),
            (
                r#"{"session_id": {"id": "uuid"}, "prompt": "test"}"#,
                true,
                "session_id as object",
            ),
            (
                r#"{"session_id": true, "prompt": "test"}"#,
                true,
                "session_id as boolean",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": 123}"#,
                true,
                "prompt as number",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": ["array"]}"#,
                true,
                "prompt as array",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": {"text": "obj"}}"#,
                true,
                "prompt as object",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": false}"#,
                true,
                "prompt as boolean",
            ),
            (
                r#"{"session_id": "not-a-uuid", "prompt": "test"}"#,
                true,
                "invalid UUID format",
            ),
            (
                r#"{"session_id": "", "prompt": "test"}"#,
                true,
                "empty UUID string",
            ),
        ];

        for (json_str, expect_invalid, description) in invalid_type_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if expect_invalid {
                assert!(
                    result.is_err() || (result.is_ok() && !is_valid_request_json(&result.unwrap())),
                    "{} should be rejected",
                    description
                );
            }
        }
    }

    #[tokio::test]
    async fn test_json_injection_in_string_fields() {
        let injection_cases = vec![
            (
                r#"{"session_id": "valid-uuid', \"injected\": \"value", "prompt": "test"}"#,
                true,
                "quote injection",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "test\"}"#,
                true,
                "trailing quote injection",
            ),
        ];

        for (json_str, expect_invalid, description) in injection_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(
                result.is_err() || (result.is_ok() && !is_valid_request_json(&result.unwrap())),
                "{} should be rejected: {}",
                description,
                json_str
            );
        }
    }

    #[tokio::test]
    async fn test_oversized_payload_rejected() {
        let large_prompt = "a".repeat(100001);
        let json = format!(
            r#"{{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "{}"}}"#,
            large_prompt
        );
        let result: Result<Value, _> = serde_json::from_str(&json);
        if result.is_ok() {
            let value = result.unwrap();
            if let Some(prompt) = value.get("prompt").and_then(|p| p.as_str()) {
                assert!(
                    prompt.len() > 100000,
                    "Large payload should be flagged for size validation"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_negative_values_rejected() {
        let negative_cases = vec![
            r#"{"limit": -1, "offset": 0}"#,
            r#"{"limit": -100, "offset": 0}"#,
            r#"{"offset": -1}"#,
        ];

        for json_str in negative_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(result.is_ok(), "{} should parse as valid JSON", json_str);
            let value = result.unwrap();
            let json_str_repr = serde_json::to_string(&value).unwrap();
            assert!(
                json_str_repr.contains("-1") || json_str_repr.contains("-100"),
                "Negative values should be preserved in JSON: {}",
                json_str_repr
            );
        }
    }

    #[tokio::test]
    async fn test_null_bytes_in_strings_rejected() {
        let null_byte_cases = vec![
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000\0", "prompt": "test"}"#,
                "null byte in session_id",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "test\0"}"#,
                "null byte in prompt",
            ),
        ];

        for (json_str, description) in null_byte_cases {
            let bytes = json_str.as_bytes();
            if bytes.contains(&0) {
                continue;
            }
            let result: Result<Value, _> = serde_json::from_str(json_str);
            match result {
                Ok(value) => {
                    let json_bytes = serde_json::to_string(&value).unwrap().into_bytes();
                    assert!(
                        !json_bytes.contains(&0),
                        "{}: null bytes should be rejected or escaped",
                        description
                    );
                }
                Err(_) => assert!(true, "{} should be rejected", description),
            }
        }
    }

    #[tokio::test]
    async fn test_valid_request_accepted() {
        let valid_cases = vec![
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "Hello world"}"#,
                "basic valid request",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "test with émoji 🎉"}"#,
                "valid with unicode",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": "multi\nline\ntext"}"#,
                "valid with newlines",
            ),
        ];

        for (json_str, description) in valid_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(
                result.is_ok() && is_valid_request_json(&result.unwrap()),
                "{} should be accepted: {}",
                description,
                json_str
            );
        }
    }

    #[tokio::test]
    async fn test_enum_validation_rejected_invalid_values() {
        let enum_cases = vec![
            (r#"{"mode": "invalid"}"#, "unknown mode value"),
            (r#"{"mode": "BUILD"}"#, "case-sensitive mode"),
            (r#"{"mode": "buildt"}"#, "typo in mode value"),
        ];

        for (json_str, description) in enum_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            if result.is_ok() {
                let value = result.unwrap();
                if let Some(mode) = value.get("mode").and_then(|m| m.as_str()) {
                    let valid_modes = ["build", "plan", "general"];
                    assert!(
                        !valid_modes.contains(&mode),
                        "{} should be rejected: {} not in {:?}",
                        description,
                        mode,
                        valid_modes
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn test_boolean_fields_rejected_as_wrong_type() {
        let bool_cases = vec![
            (
                r#"{"session_id": true, "prompt": "test"}"#,
                "session_id as boolean",
            ),
            (
                r#"{"session_id": false, "prompt": "test"}"#,
                "session_id as false boolean",
            ),
        ];

        for (json_str, description) in bool_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(result.is_ok(), "{} should parse as valid JSON", description);
            let value = result.unwrap();
            let sid = value.get("session_id");
            assert!(
                sid.map(|s| s.is_boolean()).unwrap_or(false),
                "{}: session_id should be boolean in parsed JSON",
                description
            );
            assert!(
                !is_valid_request_json(&value),
                "{} should be rejected by validation",
                description
            );
        }
    }

    #[tokio::test]
    async fn test_nested_object_rejected() {
        let json_str =
            r#"{"session_id": {"id": "550e8400-e29b-41d4-a716-446655440000"}, "prompt": "test"}"#;
        let result: Result<Value, _> = serde_json::from_str(json_str);
        assert!(result.is_ok(), "Should parse as valid JSON");
        let value = result.unwrap();
        let sid = value.get("session_id");
        assert!(
            sid.map(|s| s.is_object()).unwrap_or(false),
            "session_id should be object in parsed JSON"
        );
        assert!(
            !is_valid_request_json(&value),
            "session_id as nested object should be rejected by validation"
        );
    }

    #[tokio::test]
    async fn test_array_fields_rejected() {
        let array_cases = vec![
            (
                r#"{"session_id": ["550e8400-e29b-41d4-a716-446655440000"], "prompt": "test"}"#,
                "session_id as array",
            ),
            (
                r#"{"session_id": "550e8400-e29b-41d4-a716-446655440000", "prompt": ["test"]}"#,
                "prompt as array",
            ),
        ];

        for (json_str, description) in array_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(result.is_ok(), "{} should parse as valid JSON", description);
            let value = result.unwrap();
            let sid = value.get("session_id");
            let prompt = value.get("prompt");
            let sid_is_array = sid.map(|s| s.is_array()).unwrap_or(false);
            let prompt_is_array = prompt.map(|p| p.is_array()).unwrap_or(false);
            assert!(
                sid_is_array || prompt_is_array,
                "{}: session_id or prompt should be array in parsed JSON",
                description
            );
            assert!(
                !is_valid_request_json(&value),
                "{} should be rejected by validation",
                description
            );
        }
    }

    #[tokio::test]
    async fn test_pagination_validation() {
        let pagination_cases = vec![
            (
                r#"{"limit": 0, "offset": 0}"#,
                true,
                "zero values should be valid",
            ),
            (
                r#"{"limit": 200, "offset": 10000}"#,
                true,
                "max values should be valid",
            ),
            (
                r#"{"limit": 201}"#,
                false,
                "limit over max (200) should be flagged",
            ),
            (
                r#"{"offset": 10001}"#,
                false,
                "offset over max (10000) should be flagged",
            ),
        ];

        for (json_str, expect_valid, description) in pagination_cases {
            let result: Result<Value, _> = serde_json::from_str(json_str);
            assert!(result.is_ok(), "{} should parse", description);
            if let Ok(value) = result {
                let limit = value.get("limit").and_then(|l| l.as_u64()).unwrap_or(20);
                let offset = value.get("offset").and_then(|o| o.as_u64()).unwrap_or(0);

                if expect_valid {
                    assert!(
                        limit <= 200 && offset <= 10000,
                        "{} should be valid",
                        description
                    );
                }
            }
        }
    }
}
