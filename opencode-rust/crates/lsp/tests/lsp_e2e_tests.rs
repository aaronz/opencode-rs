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
}
