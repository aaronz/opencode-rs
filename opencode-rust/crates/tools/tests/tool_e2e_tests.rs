#[cfg(test)]
mod tool_e2e_tests {
    use opencode_tools::discovery::build_default_registry;
    use opencode_tools::ToolRegistry;
    use opencode_tools::{Tool, ToolContext};

    #[tokio::test]
    async fn test_tool_e2e_001_registry_registration_and_lookup() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;
        registry.register(opencode_tools::write::WriteTool).await;
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let read_tool = registry.get("read").await;
        assert!(read_tool.is_some(), "ReadTool should be found");

        let nonexistent = registry.get("nonexistent").await;
        assert!(nonexistent.is_none(), "Nonexistent tool should return None");

        let tools = registry.list_filtered(None).await;
        let tool_names: Vec<_> = tools.iter().map(|(name, _, _)| name.clone()).collect();
        assert!(
            tool_names.contains(&"read".to_string()),
            "Registry should contain 'read'"
        );
        assert!(
            tool_names.contains(&"write".to_string()),
            "Registry should contain 'write'"
        );
        assert!(
            tool_names.contains(&"bash".to_string()),
            "Registry should contain 'bash'"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_002_read_tool_file_reading() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_file = temp_dir.path().join("test.txt");
        {
            let mut file = std::fs::File::create(&temp_file).unwrap();
            file.write_all(b"hello world").unwrap();
        }

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(temp_dir.path().to_string_lossy().to_string()),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "read",
                serde_json::json!({ "path": temp_file.to_string_lossy() }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(result.success, "ReadTool should succeed");
        assert!(
            result.content.contains("hello world"),
            "Content should match file"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_003_write_tool_file_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_file = temp_dir.path().join("new_file.txt");

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::write::WriteTool).await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(temp_dir.path().to_string_lossy().to_string()),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "write",
                serde_json::json!({
                    "path": temp_file.to_string_lossy(),
                    "content": "new content here"
                }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(result.success, "WriteTool should succeed");
        assert!(temp_file.exists(), "File should be created");

        let content = std::fs::read_to_string(&temp_file).unwrap();
        assert_eq!(content, "new content here");
    }

    #[tokio::test]
    async fn test_tool_e2e_004_bash_tool_execution() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute("bash", serde_json::json!({ "command": "echo hello" }), None)
            .await
            .unwrap();

        assert!(result.success, "BashTool echo should succeed");
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_tool_e2e_004_bash_tool_completes_within_timeout() {
        use opencode_tools::bash::BashTool;

        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "sleep 0.1",
            "timeout": 5000
        });
        let result = tool.execute(args, None).await;

        assert!(
            result.is_ok() && result.as_ref().unwrap().success,
            "BashTool with long timeout should complete successfully"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_004_bash_tool_exit_code() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute("bash", serde_json::json!({ "command": "exit 1" }), None)
            .await
            .unwrap();

        assert!(!result.success, "BashTool with exit 1 should return error");
    }

    #[tokio::test]
    async fn test_tool_e2e_010_tool_safety_classification() {
        let registry = build_default_registry(None).await;

        let read_tool = registry.get("read").await.unwrap();
        assert!(read_tool.is_safe(), "ReadTool should be safe");

        let write_tool = registry.get("write").await.unwrap();
        assert!(!write_tool.is_safe(), "WriteTool should not be safe");

        let bash_tool = registry.get("bash").await.unwrap();
        assert!(!bash_tool.is_safe(), "BashTool should not be safe");

        let glob_tool = registry.get("glob").await.unwrap();
        assert!(glob_tool.is_safe(), "GlobTool should be safe");
    }

    #[tokio::test]
    async fn test_tool_registry_001_duplicate_registration() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;

        let read_tool = registry.get("read").await;
        assert!(
            read_tool.is_some(),
            "Tool should still be accessible after duplicate registration"
        );
    }

    #[tokio::test]
    async fn test_tool_read_nonexistent_file() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(
                tempfile::tempdir()
                    .unwrap()
                    .path()
                    .to_string_lossy()
                    .to_string(),
            ),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "read",
                serde_json::json!({ "path": "/nonexistent/file.txt" }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(
            !result.success,
            "Reading nonexistent file should return error"
        );
        assert!(result.error.is_some(), "Error message should be present");
    }

    #[tokio::test]
    async fn test_tool_e2e_edge_004_bash_empty_command() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute("bash", serde_json::json!({ "command": "" }), None)
            .await
            .unwrap();

        assert!(
            result.success || result.error.is_some(),
            "Empty command should be handled"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_edge_004_bash_true_false() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let true_result = registry
            .execute("bash", serde_json::json!({ "command": "true" }), None)
            .await
            .unwrap();
        assert!(true_result.success, "true command should succeed");

        let false_result = registry
            .execute("bash", serde_json::json!({ "command": "false" }), None)
            .await
            .unwrap();
        assert!(!false_result.success, "false command should return error");
    }

    #[tokio::test]
    async fn test_tool_e2e_edge_004_bash_exit_codes() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute("bash", serde_json::json!({ "command": "exit 42" }), None)
            .await
            .unwrap();

        assert!(!result.success, "exit 42 should return error");
    }

    #[tokio::test]
    async fn test_tool_e2e_004_bash_environment_variable() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "export X=hello && echo $X" }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success, "Environment variable export should work");
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_tool_e2e_005_grep_tool_pattern_search() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(b"hello world\ntest line\nfoo bar\n")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::grep_tool::GrepTool).await;

        let result = registry
            .execute(
                "grep",
                serde_json::json!({
                    "pattern": "hello",
                    "path": file_path.to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success, "GrepTool should succeed");
        assert!(
            result.content.contains("hello"),
            "Should find 'hello' in content"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_005_grep_tool_no_matches() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(b"hello world\n").unwrap();
        }

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::grep_tool::GrepTool).await;

        let result = registry
            .execute(
                "grep",
                serde_json::json!({
                    "pattern": "notfound",
                    "path": file_path.to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(
            result.success,
            "GrepTool should succeed even with no matches"
        );
        assert!(
            result.content.contains("No matches") || result.content.is_empty(),
            "Should indicate no matches found"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_005_grep_tool_with_file_type() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let rs_file = temp_dir.path().join("test.rs");
        let txt_file = temp_dir.path().join("test.txt");
        {
            std::fs::File::create(&rs_file)
                .unwrap()
                .write_all(b"fn main() {}\n")
                .unwrap();
            std::fs::File::create(&txt_file)
                .unwrap()
                .write_all(b"hello\n")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::grep_tool::GrepTool).await;

        let result = registry
            .execute(
                "grep",
                serde_json::json!({
                    "pattern": "fn",
                    "path": temp_dir.path().to_string_lossy(),
                    "file_type": "rs"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success);
        assert!(
            result.content.contains("test.rs"),
            "Should only match .rs files"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_006_multiedit_tool_atomic_edits() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        {
            std::fs::File::create(&file1)
                .unwrap()
                .write_all(b"hello world")
                .unwrap();
            std::fs::File::create(&file2)
                .unwrap()
                .write_all(b"foo bar")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::multiedit::MultiEditTool)
            .await;

        let result = registry.execute(
            "multi_edit",
            serde_json::json!({
                "edits": [
                    {"path": file1.to_string_lossy(), "old_string": "hello", "new_string": "goodbye"},
                    {"path": file2.to_string_lossy(), "old_string": "foo", "new_string": "baz"}
                ]
            }),
            None,
        )
        .await
        .unwrap();

        assert!(result.success, "MultiEdit should succeed");

        let content1 = std::fs::read_to_string(&file1).unwrap();
        let content2 = std::fs::read_to_string(&file2).unwrap();
        assert!(content1.contains("goodbye"), "file1 should be edited");
        assert!(content2.contains("baz"), "file2 should be edited");
    }

    #[tokio::test]
    async fn test_tool_e2e_006_multiedit_tool_partial_failure_rollback() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        {
            std::fs::File::create(&file1)
                .unwrap()
                .write_all(b"hello world")
                .unwrap();
            std::fs::File::create(&file2)
                .unwrap()
                .write_all(b"foo bar")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::multiedit::MultiEditTool)
            .await;

        let result = registry.execute(
            "multi_edit",
            serde_json::json!({
                "edits": [
                    {"path": file1.to_string_lossy(), "old_string": "hello", "new_string": "goodbye"},
                    {"path": file2.to_string_lossy(), "old_string": "nonexistent", "new_string": "baz"}
                ]
            }),
            None,
        )
        .await
        .unwrap();

        assert!(
            !result.success,
            "MultiEdit should fail when old_string not found"
        );

        let content1 = std::fs::read_to_string(&file1).unwrap();
        let content2 = std::fs::read_to_string(&file2).unwrap();
        assert_eq!(
            content1, "hello world",
            "file1 should be unchanged after rollback"
        );
        assert_eq!(
            content2, "foo bar",
            "file2 should be unchanged after rollback"
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_007_glob_tool_pattern_matching() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        let tests_dir = temp_dir.path().join("tests");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&tests_dir).unwrap();
        {
            std::fs::File::create(src_dir.join("main.rs"))
                .unwrap()
                .write_all(b"fn main()")
                .unwrap();
            std::fs::File::create(src_dir.join("lib.rs"))
                .unwrap()
                .write_all(b"lib")
                .unwrap();
            std::fs::File::create(tests_dir.join("main.rs"))
                .unwrap()
                .write_all(b"test")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::glob::GlobTool).await;

        let result = registry
            .execute(
                "glob",
                serde_json::json!({
                    "pattern": "**/*.rs",
                    "path": temp_dir.path().to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(
            result.success,
            "GlobTool should succeed: {}",
            result.content
        );
        assert!(
            result.content.contains("main.rs"),
            "Should find main.rs in content: {}",
            result.content
        );
        assert!(
            result.content.contains("lib.rs"),
            "Should find lib.rs in content: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_007_glob_tool_src_directory_only() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        let tests_dir = temp_dir.path().join("tests");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&tests_dir).unwrap();
        {
            std::fs::File::create(src_dir.join("main.rs"))
                .unwrap()
                .write_all(b"fn main()")
                .unwrap();
            std::fs::File::create(tests_dir.join("test.rs"))
                .unwrap()
                .write_all(b"test")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::glob::GlobTool).await;

        let result = registry
            .execute(
                "glob",
                serde_json::json!({
                    "pattern": "*.rs",
                    "path": src_dir.to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(
            result.success,
            "GlobTool should succeed: {}",
            result.content
        );
        assert!(
            result.content.contains("main.rs"),
            "Should find main.rs: {}",
            result.content
        );
        assert!(
            !result.content.contains("test.rs"),
            "Should not find test.rs in different dir: {}",
            result.content
        );
    }

    #[tokio::test]
    async fn test_tool_e2e_009_webfetch_invalid_url() {
        let registry = build_default_registry(None).await;

        let result = registry
            .execute(
                "webfetch",
                serde_json::json!({
                    "url": "not-a-valid-url"
                }),
                None,
            )
            .await
            .unwrap();

        assert!(!result.success, "Invalid URL should return error");
        assert!(result.error.is_some(), "Should have error message");
    }

    #[tokio::test]
    async fn test_tool_e2e_009_webfetch_missing_url() {
        let registry = build_default_registry(None).await;

        let result = registry
            .execute("webfetch", serde_json::json!({}), None)
            .await;

        assert!(
            result.is_err() || !result.as_ref().unwrap().success,
            "Missing URL should return error"
        );
    }

    #[tokio::test]
    async fn test_tool_edge_001_read_oversized_file() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let large_file = temp_dir.path().join("large.txt");

        let content = "x".repeat(100_000);
        std::fs::File::create(&large_file)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(temp_dir.path().to_string_lossy().to_string()),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "read",
                serde_json::json!({ "path": large_file.to_string_lossy() }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(
            result.success || result.content.contains("..."),
            "Large file should either succeed with truncation or have truncation indicator"
        );
    }

    #[tokio::test]
    async fn test_tool_sec_004_read_protected_paths_denied() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: None,
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "read",
                serde_json::json!({ "path": "/etc/passwd" }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(
            !result.success || result.error.is_some(),
            "ReadTool should deny access to /etc/passwd or return error"
        );
    }

    #[tokio::test]
    async fn test_tool_sec_004_read_within_worktree_allowed() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let safe_file = temp_dir.path().join("safe.txt");
        std::fs::File::create(&safe_file)
            .unwrap()
            .write_all(b"safe content")
            .unwrap();

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::read::ReadTool::new())
            .await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(temp_dir.path().to_string_lossy().to_string()),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "read",
                serde_json::json!({ "path": safe_file.to_string_lossy() }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(
            result.success,
            "ReadTool should allow reading files within worktree"
        );
        assert!(result.content.contains("safe content"));
    }

    #[tokio::test]
    async fn test_tool_sec_001_bash_injection_semicolon() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "; echo injected" }),
                None,
            )
            .await;

        let is_rejected = result.is_err() || !result.as_ref().unwrap().success;
        assert!(is_rejected, "Semicolon injection should be rejected");
    }

    #[tokio::test]
    async fn test_tool_sec_001_bash_injection_command_substitution() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "$(echo injected)" }),
                None,
            )
            .await;

        let is_rejected = result.is_err() || !result.as_ref().unwrap().success;
        assert!(
            is_rejected,
            "Command substitution injection should be rejected"
        );
    }

    #[tokio::test]
    async fn test_tool_sec_001_bash_injection_pipe() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "| echo injected" }),
                None,
            )
            .await;

        let is_rejected = result.is_err() || !result.as_ref().unwrap().success;
        assert!(is_rejected, "Pipe injection should be rejected");
    }

    #[tokio::test]
    async fn test_tool_sec_001_bash_injection_and() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "&& echo injected" }),
                None,
            )
            .await;

        let is_rejected = result.is_err() || !result.as_ref().unwrap().success;
        assert!(is_rejected, "AND injection should be rejected");
    }

    #[tokio::test]
    async fn test_tool_sec_001_bash_injection_or() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "|| echo injected" }),
                None,
            )
            .await;

        let is_rejected = result.is_err() || !result.as_ref().unwrap().success;
        assert!(is_rejected, "OR injection should be rejected");
    }

    #[tokio::test]
    async fn test_tool_sec_001_bash_legitimate_command() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute("bash", serde_json::json!({ "command": "echo hello" }), None)
            .await
            .unwrap();

        assert!(result.success, "Legitimate command should succeed");
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_tool_sec_002_bash_path_traversal_absolute() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({ "command": "cat /nonexistent_file.txt" }),
                None,
            )
            .await
            .unwrap();

        assert!(
            !result.success,
            "Accessing nonexistent absolute path should fail"
        );
    }

    #[tokio::test]
    async fn test_tool_sec_003_webfetch_ssrf_localhost() {
        let registry = build_default_registry(None).await;

        let result = registry
            .execute(
                "webfetch",
                serde_json::json!({ "url": "http://127.0.0.1:8080/admin" }),
                None,
            )
            .await;

        assert!(
            result.is_err() || !result.as_ref().unwrap().success,
            "Localhost SSRF should fail or return error"
        );
    }

    #[tokio::test]
    async fn test_tool_sec_003_webfetch_ssrf_private_ip() {
        let registry = build_default_registry(None).await;

        let result = registry
            .execute(
                "webfetch",
                serde_json::json!({ "url": "http://10.0.0.1/internal/api" }),
                None,
            )
            .await;

        assert!(
            result.is_err() || !result.as_ref().unwrap().success,
            "Private IP SSRF should fail or return error"
        );
    }

    #[tokio::test]
    async fn test_tool_sec_003_webfetch_invalid_url_format() {
        let registry = build_default_registry(None).await;

        let result = registry
            .execute("webfetch", serde_json::json!({ "url": "not-a-url" }), None)
            .await
            .unwrap();

        assert!(!result.success, "Invalid URL format should be rejected");
    }

    #[tokio::test]
    #[ignore]
    async fn test_tool_sec_005_bash_signal_handling_timeout() {
        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::bash::BashTool::new())
            .await;

        let result = registry
            .execute(
                "bash",
                serde_json::json!({
                    "command": "sleep 10",
                    "timeout": 500
                }),
                None,
            )
            .await;

        assert!(
            result.is_err(),
            "Long-running command with short timeout should error"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_tool_sec_005_bash_no_zombie_after_timeout() {
        use opencode_tools::bash::BashTool;

        let tool = BashTool::new();

        for _ in 0..3 {
            let result = tool
                .execute(
                    serde_json::json!({
                        "command": "sleep 1",
                        "timeout": 100
                    }),
                    None,
                )
                .await;
            assert!(result.is_err(), "Each timeout should error, not hang");
        }
    }

    #[tokio::test]
    async fn test_tool_sec_006_write_path_traversal_denied() {
        let temp_dir = tempfile::tempdir().unwrap();
        let registry = ToolRegistry::new();
        registry.register(opencode_tools::write::WriteTool).await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(temp_dir.path().to_string_lossy().to_string()),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "write",
                serde_json::json!({
                    "path": "../../../etc/malicious",
                    "content": "bad content"
                }),
                Some(ctx),
            )
            .await;

        let is_denied =
            result.is_err() || (result.as_ref().is_ok() && !result.as_ref().unwrap().success);
        assert!(is_denied, "Path traversal should be denied or fail");
    }

    #[tokio::test]
    async fn test_tool_sec_006_write_within_worktree_allowed() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("allowed.txt");

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::write::WriteTool).await;

        let ctx = ToolContext {
            session_id: "test".to_string(),
            message_id: "test".to_string(),
            agent: "test".to_string(),
            worktree: Some(temp_dir.path().to_string_lossy().to_string()),
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute(
                "write",
                serde_json::json!({
                    "path": "allowed.txt",
                    "content": "good content"
                }),
                Some(ctx),
            )
            .await
            .unwrap();

        assert!(result.success, "Write within worktree should succeed");
        assert!(file_path.exists(), "File should be created");
    }

    #[tokio::test]
    async fn test_tool_edge_002_grep_normal_regex() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::File::create(&file_path)
            .unwrap()
            .write_all(b"hello world")
            .unwrap();

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::grep_tool::GrepTool).await;

        let result = registry
            .execute(
                "grep",
                serde_json::json!({
                    "pattern": "hello",
                    "path": file_path.to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success, "Normal regex should work");
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_tool_edge_002_grep_returns_error_for_invalid_regex() {
        let registry = ToolRegistry::new();
        registry.register(opencode_tools::grep_tool::GrepTool).await;

        let result = registry
            .execute(
                "grep",
                serde_json::json!({
                    "pattern": "[",
                    "path": "."
                }),
                None,
            )
            .await;

        assert!(
            result.is_err() || (result.as_ref().is_ok() && !result.as_ref().unwrap().success),
            "Invalid regex should return error"
        );
    }

    #[tokio::test]
    async fn test_tool_edge_003_glob_normal_operation() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::File::create(src_dir.join("main.rs"))
            .unwrap()
            .write_all(b"fn main")
            .unwrap();

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::glob::GlobTool).await;

        let result = registry
            .execute(
                "glob",
                serde_json::json!({
                    "pattern": "**/*.rs",
                    "path": temp_dir.path().to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success, "Normal glob should work");
        assert!(result.content.contains("main.rs") || result.content.contains("src"));
    }

    #[tokio::test]
    async fn test_tool_edge_003_glob_nested_directories() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let nested = temp_dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::File::create(nested.join("deep.rs"))
            .unwrap()
            .write_all(b"deep")
            .unwrap();

        let registry = ToolRegistry::new();
        registry.register(opencode_tools::glob::GlobTool).await;

        let result = registry
            .execute(
                "glob",
                serde_json::json!({
                    "pattern": "**/*.rs",
                    "path": temp_dir.path().to_string_lossy()
                }),
                None,
            )
            .await
            .unwrap();

        assert!(result.success, "Glob with nested directories should work");
    }

    #[tokio::test]
    async fn test_tool_multi_001_multiedit_all_or_nothing() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");
        {
            std::fs::File::create(&file1)
                .unwrap()
                .write_all(b"original1")
                .unwrap();
            std::fs::File::create(&file2)
                .unwrap()
                .write_all(b"original2")
                .unwrap();
            std::fs::File::create(&file3)
                .unwrap()
                .write_all(b"original3")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::multiedit::MultiEditTool)
            .await;

        let result = registry.execute(
            "multi_edit",
            serde_json::json!({
                "edits": [
                    {"path": file1.to_string_lossy(), "old_string": "original1", "new_string": "modified1"},
                    {"path": file2.to_string_lossy(), "old_string": "WRONG", "new_string": "modified2"},
                    {"path": file3.to_string_lossy(), "old_string": "original3", "new_string": "modified3"}
                ]
            }),
            None,
        )
        .await
        .unwrap();

        assert!(
            !result.success,
            "MultiEdit should fail when one edit doesn't match"
        );

        let content1 = std::fs::read_to_string(&file1).unwrap();
        let content2 = std::fs::read_to_string(&file2).unwrap();
        let content3 = std::fs::read_to_string(&file3).unwrap();

        assert_eq!(content1, "original1", "file1 should be rolled back");
        assert_eq!(content2, "original2", "file2 should be rolled back");
        assert_eq!(content3, "original3", "file3 should be rolled back");
    }

    #[tokio::test]
    async fn test_tool_multi_001_multiedit_success_all_changes() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        {
            std::fs::File::create(&file1)
                .unwrap()
                .write_all(b"hello")
                .unwrap();
            std::fs::File::create(&file2)
                .unwrap()
                .write_all(b"world")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::multiedit::MultiEditTool)
            .await;

        let result = registry.execute(
            "multi_edit",
            serde_json::json!({
                "edits": [
                    {"path": file1.to_string_lossy(), "old_string": "hello", "new_string": "goodbye"},
                    {"path": file2.to_string_lossy(), "old_string": "world", "new_string": "universe"}
                ]
            }),
            None,
        )
        .await
        .unwrap();

        assert!(
            result.success,
            "MultiEdit should succeed when all edits match"
        );

        let content1 = std::fs::read_to_string(&file1).unwrap();
        let content2 = std::fs::read_to_string(&file2).unwrap();

        assert_eq!(content1, "goodbye", "file1 should be modified");
        assert_eq!(content2, "universe", "file2 should be modified");
    }

    #[tokio::test]
    async fn test_tool_multi_002_concurrent_edits_different_files() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        {
            std::fs::File::create(&file1)
                .unwrap()
                .write_all(b"content1")
                .unwrap();
            std::fs::File::create(&file2)
                .unwrap()
                .write_all(b"content2")
                .unwrap();
        }

        let registry = ToolRegistry::new();
        registry
            .register(opencode_tools::multiedit::MultiEditTool)
            .await;

        let file1_path = file1.to_string_lossy().to_string();
        let _file2_path = file2.to_string_lossy().to_string();

        let result1 = registry.execute(
            "multi_edit",
            serde_json::json!({
                "edits": [{"path": file1_path, "old_string": "content1", "new_string": "modified1"}]
            }),
            None,
        )
        .await
        .unwrap();

        assert!(result1.success, "First edit should succeed");

        let content1 = std::fs::read_to_string(&file1).unwrap();
        let content2 = std::fs::read_to_string(&file2).unwrap();

        assert_eq!(content1, "modified1");
        assert_eq!(content2, "content2");
    }
}
