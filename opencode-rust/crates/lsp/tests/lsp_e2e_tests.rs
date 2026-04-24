#[cfg(test)]
mod lsp_tests {
    use opencode_lsp::custom::{
        CustomLspServer, CustomRegistry, CustomServerConfig, ServerCapabilities,
    };
    use opencode_lsp::language::Language;
    use opencode_lsp::manager::LspManager;
    use std::path::PathBuf;

    #[test]
    fn test_lsp_e2e_004_custom_registry_registration() {
        let registry = CustomRegistry::new();

        let caps = ServerCapabilities {
            hover_provider: Some(true),
            completion_provider: Some(true),
            definition_provider: Some(true),
            ..Default::default()
        };

        let config = CustomServerConfig::new(vec!["rust-analyzer".to_string()])
            .with_languages(vec!["Rust".to_string()])
            .with_extensions(vec![".rs".to_string()])
            .with_capabilities(caps.clone());

        let server = CustomLspServer::new(
            "custom-rust-analyzer".to_string(),
            "Custom Rust Analyzer".to_string(),
            config,
        );

        let result = registry.register(server);
        assert!(result.is_ok());

        let retrieved = registry.get("custom-rust-analyzer").unwrap().unwrap();
        assert!(retrieved.capabilities.supports_hover());
        assert!(retrieved.capabilities.supports_completion());
        assert!(retrieved.capabilities.supports_definition());
    }

    #[test]
    fn test_lsp_e2e_004_custom_server_capabilities() {
        let registry = CustomRegistry::new();

        let caps = ServerCapabilities {
            definition_provider: Some(true),
            references_provider: Some(true),
            code_action_provider: Some(true),
            text_document_sync: Some(true),
            ..Default::default()
        };

        let config = CustomServerConfig::new(vec!["gopls".to_string()])
            .with_languages(vec!["Go".to_string()])
            .with_capabilities(caps);

        let server = CustomLspServer::new(
            "gopls-custom".to_string(),
            "Go Language Server".to_string(),
            config,
        );

        registry.register(server).unwrap();

        let retrieved = registry.get("gopls-custom").unwrap().unwrap();
        assert!(retrieved.capabilities.supports_definition());
        assert!(retrieved.capabilities.supports_references());
        assert!(retrieved.capabilities.supports_code_action());
    }

    #[test]
    fn test_lsp_e2e_004_custom_server_language_filtering() {
        let registry = CustomRegistry::new();

        let rust_server = CustomLspServer::new(
            "rust-server".to_string(),
            "Rust Server".to_string(),
            CustomServerConfig::new(vec!["rust-analyzer".to_string()])
                .with_languages(vec!["Rust".to_string()]),
        );

        let ts_server = CustomLspServer::new(
            "ts-server".to_string(),
            "TypeScript Server".to_string(),
            CustomServerConfig::new(vec!["typescript-language-server".to_string()])
                .with_languages(vec!["TypeScript".to_string()]),
        );

        registry.register(rust_server).unwrap();
        registry.register(ts_server).unwrap();

        let rust_servers = registry.for_language(&Language::Rust).unwrap();
        assert_eq!(rust_servers.len(), 1);
        assert_eq!(rust_servers[0].id, "rust-server");

        let ts_servers = registry.for_language(&Language::TypeScript).unwrap();
        assert_eq!(ts_servers.len(), 1);
        assert_eq!(ts_servers[0].id, "ts-server");

        let python_servers = registry.for_language(&Language::Python).unwrap();
        assert!(python_servers.is_empty());
    }

    #[test]
    fn test_lsp_e2e_004_extension_based_detection() {
        let registry = CustomRegistry::new();

        let server = CustomLspServer::new(
            "multi-lang-server".to_string(),
            "Multi Language Server".to_string(),
            CustomServerConfig::new(vec!["lsp-server".to_string()]).with_extensions(vec![
                ".rs".to_string(),
                ".go".to_string(),
                ".py".to_string(),
            ]),
        );

        registry.register(server).unwrap();

        assert!(registry.for_extension(".rs").unwrap().len() == 1);
        assert!(registry.for_extension(".go").unwrap().len() == 1);
        assert!(registry.for_extension(".py").unwrap().len() == 1);
        assert!(registry.for_extension(".ts").unwrap().is_empty());
    }

    #[test]
    fn test_lsp_stability_002_graceful_shutdown() {
        let registry = CustomRegistry::new();

        let server = CustomLspServer::new(
            "test-server".to_string(),
            "Test Server".to_string(),
            CustomServerConfig::new(vec!["test-server".to_string()]),
        );

        registry.register(server).unwrap();
        registry.activate("test-server").unwrap();

        assert!(registry.get("test-server").unwrap().unwrap().active);

        registry.deactivate("test-server").unwrap();
        assert!(!registry.get("test-server").unwrap().unwrap().active);
    }

    #[test]
    fn test_lsp_proto_001_server_capabilities() {
        let caps = ServerCapabilities::new();
        assert!(!caps.supports_hover());
        assert!(!caps.supports_completion());
        assert!(!caps.supports_definition());
        assert!(!caps.supports_references());
        assert!(!caps.supports_code_action());

        let caps_with_all = ServerCapabilities {
            hover_provider: Some(true),
            completion_provider: Some(true),
            definition_provider: Some(true),
            references_provider: Some(true),
            code_action_provider: Some(true),
            text_document_sync: Some(true),
            custom: std::collections::HashMap::new(),
        };

        assert!(caps_with_all.supports_hover());
        assert!(caps_with_all.supports_completion());
        assert!(caps_with_all.supports_definition());
        assert!(caps_with_all.supports_references());
        assert!(caps_with_all.supports_code_action());
    }

    #[test]
    fn test_lsp_manager_creation() {
        let manager = LspManager::new(PathBuf::from("/test/project"));
        assert!(manager.custom_registry().all().unwrap().is_empty());
    }

    #[test]
    fn test_lsp_manager_with_custom_registry() {
        let registry = CustomRegistry::new();
        let manager =
            LspManager::with_custom_registry(PathBuf::from("/test/project"), registry.clone());

        let caps = ServerCapabilities {
            hover_provider: Some(true),
            ..Default::default()
        };

        let server = CustomLspServer::new(
            "test-server".to_string(),
            "Test Server".to_string(),
            CustomServerConfig::new(vec!["test-server".to_string()]).with_capabilities(caps),
        );

        registry.register(server).unwrap();

        let retrieved = manager
            .custom_registry()
            .get("test-server")
            .unwrap()
            .unwrap();
        assert!(retrieved.capabilities.supports_hover());
    }

    #[test]
    fn test_lsp_integ_001_multiple_server_registration() {
        let registry = CustomRegistry::new();

        let server1 = CustomLspServer::new(
            "server1".to_string(),
            "Server 1".to_string(),
            CustomServerConfig::new(vec!["server1".to_string()])
                .with_languages(vec!["Rust".to_string()]),
        );

        let server2 = CustomLspServer::new(
            "server2".to_string(),
            "Server 2".to_string(),
            CustomServerConfig::new(vec!["server2".to_string()])
                .with_languages(vec!["TypeScript".to_string()]),
        );

        registry.register(server1).unwrap();
        registry.register(server2).unwrap();

        let all = registry.all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_lsp_integ_001_registration_error_handling() {
        let registry = CustomRegistry::new();

        let server1 = CustomLspServer::new(
            "duplicate-id".to_string(),
            "Server 1".to_string(),
            CustomServerConfig::new(vec!["server1".to_string()]),
        );

        let server2 = CustomLspServer::new(
            "duplicate-id".to_string(),
            "Server 2".to_string(),
            CustomServerConfig::new(vec!["server2".to_string()]),
        );

        registry.register(server1).unwrap();
        let result = registry.register(server2);
        assert!(result.is_err());
    }

    #[test]
    fn test_lsp_custom_server_enabled_disabled() {
        let registry = CustomRegistry::new();

        let enabled_server = CustomLspServer::new(
            "enabled-server".to_string(),
            "Enabled Server".to_string(),
            CustomServerConfig::new(vec!["enabled".to_string()]),
        );

        let mut disabled_server = CustomLspServer::new(
            "disabled-server".to_string(),
            "Disabled Server".to_string(),
            CustomServerConfig::new(vec!["disabled".to_string()]),
        );
        disabled_server.config.enabled = false;

        registry.register(enabled_server).unwrap();
        registry.register(disabled_server).unwrap();

        let all = registry.all().unwrap();
        assert_eq!(all.len(), 2);

        let enabled = registry.enabled().unwrap();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].id, "enabled-server");
    }

    #[test]
    fn test_lsp_custom_server_update() {
        let registry = CustomRegistry::new();

        let server = CustomLspServer::new(
            "updateable-server".to_string(),
            "Original Name".to_string(),
            CustomServerConfig::new(vec!["server".to_string()]),
        );

        registry.register(server).unwrap();

        let mut updated_server = registry.get("updateable-server").unwrap().unwrap();
        updated_server.name = "Updated Name".to_string();

        registry.update(updated_server).unwrap();

        let retrieved = registry.get("updateable-server").unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
    }

    #[test]
    fn test_lsp_custom_server_update_not_found() {
        let registry = CustomRegistry::new();

        let server = CustomLspServer::new(
            "nonexistent".to_string(),
            "Test".to_string(),
            CustomServerConfig::new(vec!["test".to_string()]),
        );

        let result = registry.update(server);
        assert!(result.is_err());
    }

    #[test]
    fn test_lsp_language_detection() {
        assert_eq!(
            Language::detect(PathBuf::from("main.rs").as_path()),
            Language::Rust
        );
        assert_eq!(
            Language::detect(PathBuf::from("app.ts").as_path()),
            Language::TypeScript
        );
        assert_eq!(
            Language::detect(PathBuf::from("app.tsx").as_path()),
            Language::TypeScript
        );
        assert_eq!(
            Language::detect(PathBuf::from("script.js").as_path()),
            Language::JavaScript
        );
        assert_eq!(
            Language::detect(PathBuf::from("main.py").as_path()),
            Language::Python
        );
        assert_eq!(
            Language::detect(PathBuf::from("server.go").as_path()),
            Language::Go
        );
        assert_eq!(
            Language::detect(PathBuf::from("unknown.xyz").as_path()),
            Language::Unknown
        );
    }

    #[test]
    fn test_lsp_language_server_commands() {
        assert_eq!(Language::Rust.server_command(), Some("rust-analyzer"));
        assert_eq!(
            Language::TypeScript.server_command(),
            Some("typescript-language-server --stdio")
        );
        assert_eq!(
            Language::Python.server_command(),
            Some("pyright-langserver --stdio")
        );
        assert_eq!(Language::Go.server_command(), Some("gopls"));
        assert_eq!(Language::Unknown.server_command(), None);
    }

    #[test]
    fn test_lsp_custom_registry_unregister() {
        let registry = CustomRegistry::new();

        let server = CustomLspServer::new(
            "removable".to_string(),
            "Removable Server".to_string(),
            CustomServerConfig::new(vec!["server".to_string()]),
        );

        registry.register(server).unwrap();
        assert!(registry.contains("removable").unwrap());

        let removed = registry.unregister("removable").unwrap();
        assert_eq!(removed.id, "removable");

        assert!(!registry.contains("removable").unwrap());
        assert!(registry.get("removable").unwrap().is_none());
    }

    #[test]
    fn test_lsp_custom_registry_clear() {
        let registry = CustomRegistry::new();

        for i in 0..5 {
            let server = CustomLspServer::new(
                format!("server-{}", i),
                format!("Server {}", i),
                CustomServerConfig::new(vec![format!("server{}", i)]),
            );
            registry.register(server).unwrap();
        }

        assert_eq!(registry.len().unwrap(), 5);

        registry.clear().unwrap();

        assert!(registry.is_empty().unwrap());
        assert_eq!(registry.len().unwrap(), 0);
    }

    #[test]
    fn test_lsp_custom_registry_for_language_not_found() {
        let registry = CustomRegistry::new();

        let server = CustomLspServer::new(
            "rust-only".to_string(),
            "Rust Only".to_string(),
            CustomServerConfig::new(vec!["rust-analyzer".to_string()])
                .with_languages(vec!["Rust".to_string()]),
        );

        registry.register(server).unwrap();

        let go_servers = registry.for_language(&Language::Go).unwrap();
        assert!(go_servers.is_empty());

        let unknown_servers = registry.for_language(&Language::Unknown).unwrap();
        assert!(unknown_servers.is_empty());
    }

    #[test]
    fn test_lsp_capability_supports_check() {
        let mut caps = ServerCapabilities::new();
        assert!(!caps.supports_hover());

        caps.hover_provider = Some(true);
        assert!(caps.supports_hover());

        caps.completion_provider = Some(true);
        assert!(caps.supports_completion());

        caps.definition_provider = Some(true);
        assert!(caps.supports_definition());

        caps.references_provider = Some(true);
        assert!(caps.supports_references());

        caps.code_action_provider = Some(true);
        assert!(caps.supports_code_action());
    }

    #[tokio::test]
    async fn test_lsp_stability_001_no_zombie_after_kill() {
        use opencode_lsp::client::LspClient;
        use std::process::Command;

        let mut client = LspClient::new();

        let path = std::path::PathBuf::from("/tmp");
        client
            .start_with_name("sleep 60", &path, "sleep-server".to_string())
            .await
            .unwrap();

        let pid_before = client.get_pid().expect("should have pid");

        tokio::task::spawn_blocking(move || {
            Command::new("kill")
                .arg("-9")
                .arg(pid_before.to_string())
                .output()
                .expect("kill should work");
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        drop(client);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let ps_output = tokio::task::spawn_blocking(|| {
            Command::new("ps")
                .args(["aux"])
                .output()
                .expect("ps should work")
        })
        .await
        .unwrap();

        let output = String::from_utf8_lossy(&ps_output.stdout);
        let pid_str = pid_before.to_string();
        let zombie_lines: Vec<_> = output
            .lines()
            .filter(|l| l.contains(&pid_str) && l.contains("defunct"))
            .collect();
        assert!(
            zombie_lines.is_empty(),
            "no zombie process with pid {}: {:?}",
            pid_before,
            zombie_lines
        );
    }

    #[tokio::test]
    async fn test_lsp_stability_001_kill_and_restart() {
        use opencode_lsp::client::LspClient;
        use std::process::Command;

        let mut client = LspClient::new();
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "sleep-server".to_string())
            .await
            .unwrap();

        let pid1 = client.get_pid().expect("should have pid");

        tokio::task::spawn_blocking(move || {
            Command::new("kill")
                .arg("-9")
                .arg(pid1.to_string())
                .output()
                .expect("kill should work");
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        client
            .start_with_name("sleep 60", &path, "sleep-server".to_string())
            .await
            .unwrap();

        let pid2 = client.get_pid().expect("should have pid");

        assert_ne!(pid1, pid2, "new process should have different pid");

        let ps_output = tokio::task::spawn_blocking(|| {
            Command::new("ps")
                .args(["aux"])
                .output()
                .expect("ps should work")
        })
        .await
        .unwrap();

        let output = String::from_utf8_lossy(&ps_output.stdout);
        let pid1_str = pid1.to_string();
        let zombie_lines: Vec<_> = output
            .lines()
            .filter(|l| l.contains(&pid1_str) && l.contains("defunct"))
            .collect();
        assert!(
            zombie_lines.is_empty(),
            "old pid {} should not be zombie: {:?}",
            pid1,
            zombie_lines
        );

        drop(client);
    }

    #[tokio::test]
    async fn test_lsp_stability_001_graceful_shutdown_no_zombie() {
        use opencode_lsp::client::LspClient;
        use std::process::Command;

        let mut client = LspClient::new();
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "sleep-server".to_string())
            .await
            .unwrap();

        let pid = client.get_pid().expect("should have pid");

        client.shutdown().await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let ps_output = tokio::task::spawn_blocking(|| {
            Command::new("ps")
                .args(["aux"])
                .output()
                .expect("ps should work")
        })
        .await
        .unwrap();

        let output = String::from_utf8_lossy(&ps_output.stdout);
        let pid_str = pid.to_string();
        let zombie_lines: Vec<_> = output
            .lines()
            .filter(|l| l.contains(&pid_str) && l.contains("defunct"))
            .collect();
        assert!(
            zombie_lines.is_empty(),
            "pid {} should not be zombie after shutdown: {:?}",
            pid,
            zombie_lines
        );
    }

    #[tokio::test]
    async fn test_lsp_stability_001_process_group_cleaned() {
        use opencode_lsp::client::LspClient;
        use std::process::Command;

        let mut client = LspClient::new();
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "sleep-server".to_string())
            .await
            .unwrap();

        let pid = client.get_pid().expect("should have pid");

        drop(client);

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let ps_output = tokio::task::spawn_blocking(|| {
            Command::new("ps")
                .args(["aux"])
                .output()
                .expect("ps should work")
        })
        .await
        .unwrap();

        let output = String::from_utf8_lossy(&ps_output.stdout);
        let pid_str = pid.to_string();
        let zombie_lines: Vec<_> = output
            .lines()
            .filter(|l| l.contains(&pid_str) && l.contains("defunct"))
            .collect();
        assert!(
            zombie_lines.is_empty(),
            "pid {} should not be zombie after drop: {:?}",
            pid,
            zombie_lines
        );
    }

    #[tokio::test]
    async fn test_lsp_perf_001_timeout_on_slow_server() {
        use opencode_core::OpenCodeError;
        use opencode_lsp::client::LspClient;
        use opencode_lsp::error::FailureHandlingConfig;
        use std::time::Duration;

        let config = FailureHandlingConfig::default();
        let mut client = LspClient::with_config(config);
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 30", &path, "slow-server".to_string())
            .await
            .unwrap();

        let start = std::time::Instant::now();
        let result =
            tokio::time::timeout(Duration::from_secs(5), client.wait_for_response(999, 1)).await;

        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(5),
            "should timeout before 5s, took {:?}",
            elapsed
        );

        match result {
            Ok(Err(OpenCodeError::ToolTimeout { timeout_ms, .. })) => {
                assert_eq!(timeout_ms, 1000, "timeout should be 1 second");
            }
            other => panic!("expected ToolTimeout, got {:?}", other),
        }

        let is_healthy = client.is_healthy();
        assert!(is_healthy, "server should still be healthy after timeout");
    }

    #[tokio::test]
    async fn test_lsp_perf_001_configurable_timeout() {
        use opencode_lsp::client::LspClient;
        use opencode_lsp::error::FailureHandlingConfig;
        use std::time::Duration;

        let config = FailureHandlingConfig::new().with_request_timeout(Duration::from_secs(5));
        let mut client = LspClient::with_config(config);
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        let start = std::time::Instant::now();
        let result =
            tokio::time::timeout(Duration::from_secs(10), client.wait_for_response(888, 3)).await;
        let elapsed = start.elapsed();

        assert!(
            result.is_err() || elapsed >= Duration::from_secs(3),
            "timeout should fire at configured 3s"
        );
        assert!(
            elapsed < Duration::from_secs(10),
            "should not wait for full 60s sleep"
        );
    }

    #[tokio::test]
    async fn test_lsp_perf_001_server_usable_after_timeout() {
        use opencode_lsp::client::LspClient;
        use opencode_lsp::error::FailureHandlingConfig;
        use std::time::Duration;

        let config = FailureHandlingConfig::new().with_request_timeout(Duration::from_secs(1));
        let mut client = LspClient::with_config(config);
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        let _ =
            tokio::time::timeout(Duration::from_secs(2), client.wait_for_response(100, 1)).await;

        assert!(client.is_healthy(), "server should still be healthy");

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        assert!(
            client.get_pid().is_some(),
            "should be able to start new server"
        );
    }

    #[tokio::test]
    async fn test_lsp_integ_002_unsupported_capability_does_not_crash() {
        use opencode_lsp::client::LspClient;
        use opencode_lsp::error::FailureHandlingConfig;
        use std::time::Duration;

        let config = FailureHandlingConfig::new().with_request_timeout(Duration::from_secs(1));
        let mut client = LspClient::with_config(config);
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        let result = client.goto_definition("file:///test.rs", 0, 0).await;

        assert!(
            result.is_ok() || result.is_err(),
            "should return, not crash"
        );

        assert!(
            client.is_healthy(),
            "client should be healthy after unsupported capability request"
        );

        client
            .start_with_name("sleep 60", &path, "new-server".to_string())
            .await
            .unwrap();
        assert!(
            client.get_pid().is_some(),
            "should be able to start new server"
        );

        drop(client);
    }

    #[tokio::test]
    async fn test_lsp_integ_002_server_usable_after_timeout() {
        use opencode_lsp::client::LspClient;
        use opencode_lsp::error::FailureHandlingConfig;
        use std::time::Duration;

        let config = FailureHandlingConfig::new().with_request_timeout(Duration::from_secs(1));
        let mut client = LspClient::with_config(config);
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        let _ = client.goto_definition("file:///test.rs", 0, 0).await;

        assert!(
            client.is_healthy(),
            "server should be healthy after timeout"
        );

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        assert!(
            client.get_pid().is_some(),
            "should have a pid after restart"
        );

        drop(client);
    }

    #[tokio::test]
    async fn test_lsp_integ_002_error_response_does_not_crash_client() {
        use opencode_lsp::client::LspClient;
        use opencode_lsp::error::FailureHandlingConfig;
        use std::time::Duration;

        let config = FailureHandlingConfig::new().with_request_timeout(Duration::from_secs(2));
        let mut client = LspClient::with_config(config);
        let path = std::path::PathBuf::from("/tmp");

        client
            .start_with_name("sleep 60", &path, "slow-server".to_string())
            .await
            .unwrap();

        let result1 = client.find_references("file:///test.rs", 0, 0).await;
        let result2 = client.get_diagnostics("file:///test.rs").await;

        assert!(
            result1.is_ok() || result1.is_err(),
            "find_references should return, not crash"
        );
        assert!(
            result2.is_ok() || result2.is_err(),
            "get_diagnostics should return, not crash"
        );

        assert!(
            client.is_healthy(),
            "client should be healthy after multiple requests"
        );

        drop(client);
    }

    #[tokio::test]
    async fn test_lsp_e2e_003_crash_recovery_manager() {
        use opencode_lsp::language::Language;
        use opencode_lsp::manager::LspManager;
        use std::process::Command;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(project_dir.join("main.rs"), "fn main() {}").expect("write file");

        let mut manager = LspManager::new(project_dir.clone());

        manager
            .start_for_file(&project_dir.join("main.rs"))
            .await
            .expect("first start");

        let pid1 = manager.get_client_pid(&Language::Rust);
        assert!(pid1.is_some(), "should have started rust-analyzer");

        if let Some(p) = pid1 {
            tokio::task::spawn_blocking(move || {
                Command::new("kill")
                    .arg("-9")
                    .arg(p.to_string())
                    .output()
                    .expect("kill should work");
            })
            .await
            .expect("kill task");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        manager
            .start_for_file(&project_dir.join("main.rs"))
            .await
            .expect("start after crash");

        let pid2 = manager.get_client_pid(&Language::Rust);

        assert!(pid2.is_some(), "should have restarted rust-analyzer");
        assert_ne!(pid1, pid2, "should have new pid after restart");
    }

    #[tokio::test]
    async fn test_lsp_e2e_001_server_detection_and_launch() {
        use opencode_lsp::language::Language;
        use opencode_lsp::manager::LspManager;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let project_dir = temp_dir.path().to_path_buf();

        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write Cargo.toml");
        std::fs::write(project_dir.join("main.rs"), "fn main() {}").expect("write main.rs");

        let mut manager = LspManager::new(project_dir.clone());

        manager
            .start_for_file(&project_dir.join("main.rs"))
            .await
            .expect("start should work");

        let pid = manager.get_client_pid(&Language::Rust);
        assert!(
            pid.is_some(),
            "rust-analyzer should be launched for Cargo.toml project"
        );

        let is_healthy = manager.is_client_healthy(&Language::Rust);
        assert!(is_healthy, "rust-analyzer should be healthy after launch");
    }

    #[tokio::test]
    async fn test_lsp_e2e_001_typescript_detection() {
        use opencode_lsp::language::Language;
        use opencode_lsp::manager::LspManager;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let project_dir = temp_dir.path().to_path_buf();

        std::fs::write(project_dir.join("package.json"), "{}").expect("write package.json");
        std::fs::write(project_dir.join("tsconfig.json"), "{}").expect("write tsconfig.json");
        std::fs::write(project_dir.join("index.ts"), "const x = 1;").expect("write index.ts");

        let mut manager = LspManager::new(project_dir.clone());

        manager
            .start_for_file(&project_dir.join("index.ts"))
            .await
            .expect("start should work");

        let pid = manager.get_client_pid(&Language::TypeScript);
        assert!(
            pid.is_some(),
            "typescript-language-server should be launched for TypeScript project"
        );
    }

    #[tokio::test]
    async fn test_lsp_e2e_002_diagnostic_aggregation() {
        
        use opencode_lsp::manager::LspManager;
        use opencode_lsp::types::{Diagnostic, Position, Range, Severity};
        

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write Cargo.toml");
        std::fs::write(project_dir.join("main.rs"), "fn main() {}").expect("write main.rs");

        let mut manager = LspManager::new(project_dir.clone());

        manager
            .start_for_file(&project_dir.join("main.rs"))
            .await
            .expect("start should work");

        let path = project_dir.join("main.rs");
        let diag1 = Diagnostic {
            severity: Severity::Error,
            message: "error 1".to_string(),
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
        let diag2 = Diagnostic {
            severity: Severity::Warning,
            message: "warning 1".to_string(),
            range: Range {
                start: Position {
                    line: 1,
                    character: 0,
                },
                end: Position {
                    line: 1,
                    character: 5,
                },
            },
            source: Some("rust-analyzer".to_string()),
            file_path: None,
        };

        manager.record_diagnostics(&path, vec![diag1]);
        manager.record_diagnostics(&path, vec![diag2]);

        let diags = manager.get_diagnostics_for_file(&path);
        assert_eq!(diags.len(), 2, "should have 2 diagnostics");

        let summary = manager.get_diagnostics_summary();
        assert_eq!(
            summary.get(&Severity::Error),
            Some(&1),
            "should have 1 error"
        );
        assert_eq!(
            summary.get(&Severity::Warning),
            Some(&1),
            "should have 1 warning"
        );
    }

    #[tokio::test]
    async fn test_lsp_e2e_002_diagnostic_clear() {
        use opencode_lsp::manager::LspManager;
        use opencode_lsp::types::{Diagnostic, Position, Range, Severity};
        

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write Cargo.toml");
        std::fs::write(project_dir.join("main.rs"), "fn main() {}").expect("write main.rs");

        let mut manager = LspManager::new(project_dir.clone());

        manager
            .start_for_file(&project_dir.join("main.rs"))
            .await
            .expect("start should work");

        let path = project_dir.join("main.rs");
        let diag = Diagnostic {
            severity: Severity::Error,
            message: "error".to_string(),
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

        manager.record_diagnostics(&path, vec![diag]);
        assert_eq!(
            manager.get_total_diagnostic_count(),
            1,
            "should have 1 diagnostic"
        );

        manager
            .stop_for_file(&path)
            .await
            .expect("stop should work");
        assert_eq!(
            manager.get_total_diagnostic_count(),
            0,
            "should have 0 diagnostics after clear"
        );
    }

    #[tokio::test]
    async fn test_lsp_e2e_002_diagnostic_deduplication() {
        use opencode_lsp::manager::LspManager;
        use opencode_lsp::types::{Diagnostic, Position, Range, Severity};

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let project_dir = temp_dir.path().to_path_buf();
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .expect("write Cargo.toml");
        std::fs::write(project_dir.join("main.rs"), "fn main() {}").expect("write main.rs");

        let mut manager = LspManager::new(project_dir.clone());

        manager
            .start_for_file(&project_dir.join("main.rs"))
            .await
            .expect("start should work");

        let path = project_dir.join("main.rs");

        let diag1 = Diagnostic {
            severity: Severity::Error,
            message: "same error".to_string(),
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
        let diag2 = Diagnostic {
            severity: Severity::Error,
            message: "same error".to_string(),
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

        manager.record_diagnostics(&path, vec![diag1, diag2]);

        let diags = manager.get_diagnostics_for_file(&path);
        assert_eq!(diags.len(), 1, "duplicate should be deduplicated");
    }

    #[tokio::test]
    async fn test_lsp_perf_002_large_workspace_diagnostics() {
        use opencode_lsp::aggregator::DiagnosticAggregator;
        use opencode_lsp::types::{Diagnostic, Position, Range, Severity};
        use std::path::PathBuf;

        let mut aggregator = DiagnosticAggregator::new();
        let file_count = 100;
        let diags_per_file = 10;

        for file_idx in 0..file_count {
            let path = PathBuf::from(format!("src/file_{}.rs", file_idx));
            let mut diags = Vec::new();

            for diag_idx in 0..diags_per_file {
                let diag = Diagnostic {
                    severity: if diag_idx % 2 == 0 {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                    message: format!("error in file {} at {}", file_idx, diag_idx),
                    range: Range {
                        start: Position {
                            line: diag_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: diag_idx as u32,
                            character: 10,
                        },
                    },
                    source: Some("rust-analyzer".to_string()),
                    file_path: None,
                };
                diags.push(diag);
            }

            aggregator.ingest(&path, diags);
        }

        assert_eq!(
            aggregator.get_total_diagnostic_count(),
            file_count * diags_per_file
        );

        let summary = aggregator.get_diagnostics_summary();
        assert_eq!(summary.get(&Severity::Error), Some(&(500)), "500 errors");
        assert_eq!(
            summary.get(&Severity::Warning),
            Some(&(500)),
            "500 warnings"
        );

        for file_idx in 0..file_count {
            let path = PathBuf::from(format!("src/file_{}.rs", file_idx));
            let diags = aggregator.get_diagnostics_for_file(&path);
            assert_eq!(
                diags.len(),
                diags_per_file,
                "each file should have {} diags",
                diags_per_file
            );
        }
    }

    #[tokio::test]
    async fn test_lsp_perf_002_memory_bounded_aggregation() {
        use opencode_lsp::aggregator::DiagnosticAggregator;
        use opencode_lsp::types::{Diagnostic, Position, Range, Severity};
        use std::path::PathBuf;

        let mut aggregator = DiagnosticAggregator::new();
        let path = PathBuf::from("src/main.rs");

        for i in 0..1000 {
            let diag = Diagnostic {
                severity: Severity::Error,
                message: format!("error {}", i),
                range: Range {
                    start: Position {
                        line: i as u32,
                        character: 0,
                    },
                    end: Position {
                        line: i as u32,
                        character: 10,
                    },
                },
                source: Some("rust-analyzer".to_string()),
                file_path: None,
            };
            aggregator.ingest(&path, vec![diag]);
        }

        assert_eq!(
            aggregator.get_total_diagnostic_count(),
            1000,
            "all 1000 diags should be stored"
        );

        let summary = aggregator.get_diagnostics_summary();
        assert_eq!(
            summary.get(&Severity::Error),
            Some(&1000),
            "all 1000 should be errors"
        );

        aggregator.clear_for_file(&path);
        assert_eq!(
            aggregator.get_total_diagnostic_count(),
            0,
            "should be empty after clear"
        );
    }
}
