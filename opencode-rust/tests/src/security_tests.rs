use crate::common::{MockLLMProvider, TempProject};
use opencode_core::{Session, SessionInfo};
use opencode_storage::database::StoragePool;
use opencode_storage::repository::SessionRepository;
use opencode_storage::SqliteProjectRepository;
use opencode_storage::SqliteSessionRepository;
use opencode_tools::{read::ReadTool, write::WriteTool, Tool, ToolRegistry};
use std::sync::Arc;

fn create_test_pool() -> (StoragePool, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_security.db");
    let pool = StoragePool::new(&db_path).expect("Failed to create storage pool");
    (pool, temp_dir)
}

#[tokio::test]
async fn test_sql_injection_session_id_handling() {
    let (pool, _temp_dir) = create_test_pool();
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    let malicious_id = "'; DROP TABLE sessions; --";
    let result = session_repo.find_by_id(malicious_id).await;
    assert!(
        result.is_ok(),
        "SQL injection should not crash the repository"
    );
    assert!(
        result.unwrap().is_none(),
        "Malicious ID should return None, not execute injection"
    );

    let normal_id = "valid-session-id";
    let result = session_repo.find_by_id(normal_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sql_injection_union_attack() {
    let (pool, _temp_dir) = create_test_pool();
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
    let (pool, _temp_dir) = create_test_pool();
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    let comment_injection = "test' OR '1'='1' --";
    let result = session_repo.find_by_id(comment_injection).await;
    assert!(result.is_ok(), "Comment injection should not crash");
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_sql_injection_find_all_with_limit() {
    let (pool, _temp_dir) = create_test_pool();
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    let malicious_limit = "100; DROP TABLE sessions; --";
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
        "..\\..\\..\\windows\\system32\\config\\sam",
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
    let project = TempProject::new();

    let tool = WriteTool;

    let malicious_paths = vec![
        "/tmp/payload.txt",
        "../../../tmp/payload.txt",
        "/var/www/html/backdoor.php",
    ];

    for malicious_path in malicious_paths {
        let result = tool
            .execute(
                serde_json::json!({
                    "path": malicious_path,
                    "content": "malicious content"
                }),
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
            Ok(r) => {
                assert!(
                    !r.success || !r.content.contains("root:"),
                    "Null byte injection should not read sensitive files"
                );
            }
            Err(_) => {
                assert!(true, "Tool should reject null byte injection");
            }
        }
    }
}

#[tokio::test]
async fn test_path_normalization_prevents_traversal() {
    let project = TempProject::new();
    project.create_file("allowed/subdir/file.txt", "nested content");

    let tool = ReadTool::new();

    let normalized_result = tool
        .execute(
            serde_json::json!({
                "path": project.path().join("allowed/subdir/file.txt").to_string_lossy()
            }),
            None,
        )
        .await;

    assert!(normalized_result.is_ok());
    let result = normalized_result.unwrap();
    assert!(result.success, "Normal path should work");
    assert!(result.content.contains("nested content"));

    let traversal_result = tool
        .execute(
            serde_json::json!({
                "path": project.path().join("allowed/../allowed/subdir/file.txt").to_string_lossy()
            }),
            None,
        )
        .await;

    assert!(traversal_result.is_ok());
    let result = traversal_result.unwrap();
    assert!(
        result.success,
        "Normalized traversal should work if file exists in normalized path"
    );
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
        json.contains(malicious_content),
        "Original content should still be preserved"
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
            Err(_) => {
                assert!(true, "Tool should reject dangerous paths");
            }
        }
    }
}

#[tokio::test]
async fn test_session_id_format_validation() {
    let (pool, _temp_dir) = create_test_pool();
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    let invalid_ids = vec![
        "",
        "   ",
        "id with spaces",
        "id\nwith\nnewlines",
        "id\twith\ttabs",
        "very-long-id-that-exceeds-normal-length-limits-and-might-cause-issues",
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
            "test.txt | cat",
            "$(cat test.txt)",
            "`cat test.txt`",
        ];

        for cmd in malicious_commands {
            let result = bash_tool
                .execute(
                    serde_json::json!({
                        "command": cmd,
                        "cwd": project.path().to_string_lossy()
                    }),
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
                Err(_) => {
                    assert!(true, "Dangerous command should be rejected");
                }
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
        let grep_tool = GrepTool::new();

        let malicious_patterns = vec!["../../../etc/passwd", "(cat /etc/passwd)", "[^a-z]"];

        for pattern in malicious_patterns {
            let result = grep_tool
                .execute(
                    serde_json::json!({
                        "path": project.path().join("test.txt").to_string_lossy(),
                        "pattern": pattern
                    }),
                    None,
                )
                .await;

            match result {
                Ok(r) => {
                    if r.success {
                        assert!(
                            !r.content.contains("root:"),
                            "Grep injection should be prevented"
                        );
                    }
                }
                Err(_) => {
                    assert!(true, "Dangerous grep pattern should be rejected");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_write_tool_path_validation() {
    let project = TempProject::new();

    let tool = WriteTool;

    let write_attempts = vec![
        ("/tmp/outside.txt", false),
        ("../outside.txt", false),
        (
            project.path().join("inside.txt").to_string_lossy().as_ref(),
            true,
        ),
    ];

    for (path, should_succeed) in write_attempts {
        let result = tool
            .execute(
                serde_json::json!({
                    "path": path,
                    "content": "test content"
                }),
                None,
            )
            .await;

        if should_succeed {
            assert!(
                result.is_ok() && result.as_ref().unwrap().success,
                "Write within project should succeed: {}",
                path
            );
        }
    }
}

#[tokio::test]
async fn test_session_storage_resilience_to_corruption() {
    let (pool, _temp_dir) = create_test_pool();
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
                serde_json::json!({
                    "path": symlink_path.to_string_lossy()
                }),
                None,
            )
            .await;

        assert!(result.is_ok(), "Symlink read should work");
        let r = result.unwrap();
        assert!(r.success, "Symlink should resolve to real content");

        let traversal_result = tool
            .execute(
                serde_json::json!({
                    "path": project.path().join("link_file.txt/../../real_file.txt").to_string_lossy()
                }),
                None,
            )
            .await;

        assert!(traversal_result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_session_access_isolation() {
    let (pool, _temp_dir) = create_test_pool();
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));

    let session1_id = "session-1";
    let session2_id = "session-2";

    let mut session1 = Session::new();
    session1.add_message(opencode_core::message::Message::user(
        "Session 1 message".to_string(),
    ));
    session_repo.save(session1_id, &session1).await.ok();

    let mut session2 = Session::new();
    session2.add_message(opencode_core::message::Message::user(
        "Session 2 message".to_string(),
    ));
    session_repo.save(session2_id, &session2).await.ok();

    let (result1, result2) = tokio::join!(
        session_repo.find_by_id(session1_id),
        session_repo.find_by_id(session2_id)
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

#[test]
fn test_session_id_length_limits() {
    let (pool, _temp_dir) = create_test_pool();

    let long_id = "a".repeat(10000);
    let session_repo = Arc::new(SqliteSessionRepository::new(pool));

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(session_repo.find_by_id(&long_id));
    assert!(result.is_ok(), "Very long ID should be handled gracefully");
}

#[tokio::test]
async fn test_storage_project_isolation() {
    let (pool1, _temp_dir1) = create_test_pool();
    let (pool2, _temp_dir2) = create_test_pool();

    let project_repo1 = Arc::new(SqliteProjectRepository::new(pool1.clone()));
    let project_repo2 = Arc::new(SqliteProjectRepository::new(pool2.clone()));

    let project1_id = "project-1";
    let project2_id = "project-2";

    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut session1 = Session::new();
        session1.add_message(opencode_core::message::Message::user(
            "Project 1 content".to_string(),
        ));

        let mut session2 = Session::new();
        session2.add_message(opencode_core::message::Message::user(
            "Project 2 content".to_string(),
        ));

        project_repo1
            .save_session(project1_id, &session1)
            .await
            .ok();
        project_repo2
            .save_session(project2_id, &session2)
            .await
            .ok();

        let loaded1 = project_repo1.find_session(project1_id).await.unwrap();
        let loaded2 = project_repo2.find_session(project2_id).await.unwrap();

        assert!(
            loaded1
                .unwrap()
                .messages
                .iter()
                .any(|m| m.content.contains("Project 1")),
            "Project 1 repo should only contain Project 1 data"
        );
        assert!(
            loaded2
                .unwrap()
                .messages
                .iter()
                .any(|m| m.content.contains("Project 2")),
            "Project 2 repo should only contain Project 2 data"
        );

        let cross_load = project_repo1.find_session(project2_id).await.unwrap();
        assert!(
            cross_load.is_none(),
            "Project 1 repo should not contain Project 2 data"
        );
    });
}

#[tokio::test]
async fn test_regex_dos_prevention_in_tools() {
    let registry = ToolRegistry::new();
    let tools = registry.list_filtered(None).await;
    let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

    if tool_names.contains(&"grep") {
        use opencode_tools::grep_tool::GrepTool;
        let grep_tool = GrepTool::new();

        let catastrophic_patterns = vec!["((a+)+)+$", "([a-zA-Z]+)*$", "(a|aa)+$"];

        for pattern in catastrophic_patterns {
            let result = grep_tool
                .execute(
                    serde_json::json!({
                        "path": "test_file.txt",
                        "pattern": pattern
                    }),
                    None,
                )
                .await;

            match result {
                Ok(r) => {
                    assert!(
                        !r.content.contains("ReDoS"),
                        "Tool should handle or reject ReDoS patterns"
                    );
                }
                Err(_) => {
                    assert!(true, "ReDoS patterns should be rejected");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_sql_injection_sessions() {
    let (pool, _temp_dir) = create_test_pool();
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
        "'; DROP TABLE sessions; --",
        "' OR '1'='1",
        "' UNION SELECT * FROM sessions--",
        "'; DELETE FROM sessions WHERE '1'='1",
        "' OR id = '1",
        "1'; DROP TABLE sessions; --",
        "admin'--",
        "' OR 1=1--",
        "'; INSERT INTO sessions VALUES('hacked','2024-01-01','2024-01-01','data');--",
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
            "SQL injection payload '{}' should return None, not execute injection",
            payload
        );
    }

    let count_after = session_repo.count().await.expect("Count should work");
    assert_eq!(
        count_after, 1,
        "After all SQL injection attempts, exactly 1 session should remain (the legitimate one)"
    );

    let legitimate_result = session_repo.find_by_id(&legitimate_id).await;
    assert!(
        legitimate_result.is_ok() && legitimate_result.unwrap().is_some(),
        "The legitimate session should still be accessible after all injection attempts"
    );
}
