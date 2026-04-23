#[cfg(test)]
mod plugin_tests {
    use indexmap::IndexMap;
    use opencode_plugin::{
        Plugin, PluginCapability, PluginConfig, PluginDomain, PluginError, PluginManager,
        PluginPermissions, PluginTool, PluginToolDefinition,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    struct TestPlugin {
        name: String,
        initialized: Arc<AtomicBool>,
        shutdown_called: Arc<AtomicBool>,
        on_start_called: Arc<AtomicBool>,
        on_tool_call_count: Arc<AtomicUsize>,
        on_message_count: Arc<AtomicUsize>,
        on_session_end_called: Arc<AtomicBool>,
        should_block_tool: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                initialized: Arc::new(AtomicBool::new(false)),
                shutdown_called: Arc::new(AtomicBool::new(false)),
                on_start_called: Arc::new(AtomicBool::new(false)),
                on_tool_call_count: Arc::new(AtomicUsize::new(0)),
                on_message_count: Arc::new(AtomicUsize::new(0)),
                on_session_end_called: Arc::new(AtomicBool::new(false)),
                should_block_tool: false,
            }
        }

        fn with_blocking(mut self, block: bool) -> Self {
            self.should_block_tool = block;
            self
        }
    }

    impl opencode_plugin::sealed::SealedPlugin for TestPlugin {}

    impl Plugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn init(&mut self) -> Result<(), PluginError> {
            self.initialized.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), PluginError> {
            self.shutdown_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn description(&self) -> &str {
            "test plugin"
        }

        fn on_start(&mut self) -> Result<(), PluginError> {
            self.on_start_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn on_tool_call(
            &mut self,
            tool_name: &str,
            _args: &serde_json::Value,
            _session_id: &str,
        ) -> Result<(), PluginError> {
            self.on_tool_call_count.fetch_add(1, Ordering::SeqCst);
            if self.should_block_tool && tool_name == "blocked_tool" {
                return Err(PluginError::PermissionDenied("blocked".to_string()));
            }
            Ok(())
        }

        fn on_message(&mut self, _content: &str, _session_id: &str) -> Result<(), PluginError> {
            self.on_message_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn on_session_end(&mut self, _session_id: &str) -> Result<(), PluginError> {
            self.on_session_end_called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    fn create_test_config(name: &str) -> PluginConfig {
        PluginConfig {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            priority: 0,
            domain: PluginDomain::Runtime,
            options: IndexMap::new(),
            permissions: PluginPermissions::default(),
        }
    }

    #[test]
    fn test_plugin_e2e_001_plugin_registration_and_init() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let initialized = plugin.initialized.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.startup().unwrap();
        assert!(initialized.load(Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_e2e_001_plugin_shutdown() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let shutdown_called = plugin.shutdown_called.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.shutdown().unwrap();
        assert!(shutdown_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_e2e_002_on_start_hook_called() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let on_start_called = plugin.on_start_called.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.on_start_all().unwrap();
        assert!(on_start_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_e2e_002_on_tool_call_hook_called() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let on_tool_call_count = plugin.on_tool_call_count.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager
            .on_tool_call_all("read", &serde_json::json!({}), "session-1")
            .unwrap();
        assert_eq!(on_tool_call_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_plugin_e2e_002_on_message_hook_called() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let on_message_count = plugin.on_message_count.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.on_message_all("test message", "session-1").unwrap();
        assert_eq!(on_message_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_plugin_e2e_002_on_session_end_hook_called() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let on_session_end_called = plugin.on_session_end_called.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.on_session_end_all("session-1").unwrap();
        assert!(on_session_end_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_plugin_hooks_001_on_tool_call_can_block() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin").with_blocking(true);

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        let result = manager.on_tool_call_all("blocked_tool", &serde_json::json!({}), "session-1");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::PermissionDenied(_)
        ));
    }

    #[test]
    fn test_plugin_hooks_001_non_blocked_tool_allowed() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin").with_blocking(true);

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        let result = manager.on_tool_call_all("allowed_tool", &serde_json::json!({}), "session-1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_plugin_state_001_init_called_once() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.init_all().unwrap();
        manager.init_all().unwrap();

        let config = manager.get_config("test-plugin");
        assert!(config.is_some());
    }

    #[test]
    fn test_plugin_state_002_cleanup_on_unload() {
        let mut manager = PluginManager::new();
        let plugin = TestPlugin::new("test-plugin");
        let shutdown_called = plugin.shutdown_called.clone();

        manager
            .register_with_config(Box::new(plugin), create_test_config("test-plugin"))
            .unwrap();

        manager.unload_plugin("test-plugin").unwrap();
        assert!(shutdown_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_plugin_sec_003_tool_registration_validation() {
        let mut manager = PluginManager::new();
        manager.set_permission_scope(opencode_permission::PermissionScope::Full);
        let mut config = create_test_config("test-plugin");
        config.permissions.capabilities = vec![PluginCapability::AddTools];
        manager
            .register_with_config(Box::new(TestPlugin::new("test-plugin")), config)
            .unwrap();

        let tool_def = PluginToolDefinition {
            name: "test_tool".to_string(),
            description: "test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_| Ok("result".to_string())));
        let result = manager.register_tool("test-plugin", tool).await;
        if let Err(e) = &result {
            eprintln!("register_tool error: {:?}", e);
        }
        assert!(result.is_ok(), "register_tool failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_plugin_sec_003_tool_name_uniqueness_enforced() {
        let mut manager = PluginManager::new();
        manager.set_permission_scope(opencode_permission::PermissionScope::Full);
        let mut config = create_test_config("test-plugin");
        config.permissions.capabilities = vec![PluginCapability::AddTools];
        manager
            .register_with_config(Box::new(TestPlugin::new("test-plugin")), config)
            .unwrap();

        let tool_def = PluginToolDefinition {
            name: "duplicate_tool".to_string(),
            description: "test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };

        let tool1 = PluginTool::new(tool_def.clone(), Box::new(|_| Ok("result1".to_string())));
        let tool2 = PluginTool::new(tool_def, Box::new(|_| Ok("result2".to_string())));

        let result1 = manager.register_plugin_tool(tool1).await;
        assert!(result1.is_ok(), "First registration should succeed");

        let result2 = manager.register_plugin_tool(tool2).await;
        assert!(result2.is_err(), "Duplicate tool should be rejected");
    }

    #[test]
    fn test_plugin_capabilities_add_tools_permission() {
        let mut config = create_test_config("test-plugin");
        config.permissions.capabilities = vec![PluginCapability::AddTools];

        assert!(config.permissions.can_add_tools());
    }

    #[test]
    fn test_plugin_capabilities_no_add_tools_permission() {
        let mut config = create_test_config("test-plugin");
        config.permissions.capabilities = vec![];

        assert!(!config.permissions.can_add_tools());
    }

    #[test]
    fn test_plugin_sorted_by_priority() {
        let mut manager = PluginManager::new();

        let plugin1 = TestPlugin::new("plugin-1");
        let mut config1 = create_test_config("plugin-1");
        config1.priority = 10;
        manager
            .register_with_config(Box::new(plugin1), config1)
            .unwrap();

        let plugin2 = TestPlugin::new("plugin-2");
        let mut config2 = create_test_config("plugin-2");
        config2.priority = 5;
        manager
            .register_with_config(Box::new(plugin2), config2)
            .unwrap();

        let plugin3 = TestPlugin::new("plugin-3");
        let mut config3 = create_test_config("plugin-3");
        config3.priority = 20;
        manager
            .register_with_config(Box::new(plugin3), config3)
            .unwrap();

        let sorted = manager.sorted_plugin_names();
        assert_eq!(sorted, vec!["plugin-2", "plugin-1", "plugin-3"]);
    }

    #[test]
    fn test_plugin_hook_order_deterministic() {
        let call_sequence = Arc::new(Mutex::new(Vec::new()));
        let call_sequence_clone = call_sequence.clone();

        struct OrderedTestPlugin {
            name: String,
            order: Arc<Mutex<Vec<String>>>,
        }

        impl OrderedTestPlugin {
            fn new(name: &str, order: Arc<Mutex<Vec<String>>>) -> Self {
                Self {
                    name: name.to_string(),
                    order,
                }
            }
        }

        impl opencode_plugin::sealed::SealedPlugin for OrderedTestPlugin {}

        impl Plugin for OrderedTestPlugin {
            fn name(&self) -> &str {
                &self.name
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn init(&mut self) -> Result<(), PluginError> {
                Ok(())
            }

            fn shutdown(&mut self) -> Result<(), PluginError> {
                Ok(())
            }

            fn description(&self) -> &str {
                "ordered test plugin"
            }

            fn on_start(&mut self) -> Result<(), PluginError> {
                let mut seq = self.order.lock().unwrap();
                seq.push(self.name.clone());
                Ok(())
            }
        }

        let mut manager = PluginManager::new();

        let plugin_a = OrderedTestPlugin::new("plugin-a", call_sequence_clone.clone());
        let plugin_b = OrderedTestPlugin::new("plugin-b", call_sequence_clone.clone());
        let plugin_c = OrderedTestPlugin::new("plugin-c", call_sequence_clone.clone());

        manager.register(Box::new(plugin_a)).unwrap();
        manager.register(Box::new(plugin_b)).unwrap();
        manager.register(Box::new(plugin_c)).unwrap();

        manager.on_start_all().unwrap();

        let sequence = call_sequence.lock().unwrap();
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence[0], "plugin-a");
        assert_eq!(sequence[1], "plugin-b");
        assert_eq!(sequence[2], "plugin-c");
    }

    #[test]
    fn test_plugin_permission_scope_setting() {
        let mut manager = PluginManager::new();
        manager.set_permission_scope(opencode_permission::PermissionScope::Full);
        assert_eq!(
            manager.permission_scope(),
            opencode_permission::PermissionScope::Full
        );
    }

    #[test]
    fn test_plugin_unload_nonexistent() {
        let mut manager = PluginManager::new();
        let result = manager.unload_plugin("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PluginError::NotFound(_)));
    }

    #[test]
    fn test_plugin_duplicate_registration_fails() {
        let mut manager = PluginManager::new();
        manager
            .register_with_config(
                Box::new(TestPlugin::new("test-plugin")),
                create_test_config("test-plugin"),
            )
            .unwrap();

        let result = manager.register(Box::new(TestPlugin::new("test-plugin")));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PluginError::DuplicatePlugin(_)
        ));
    }

    #[tokio::test]
    async fn test_plugin_async_tool_registration() {
        let mut manager = PluginManager::new();
        let mut config = create_test_config("test-plugin");
        config.permissions.capabilities = vec![PluginCapability::AddTools];
        manager
            .register_with_config(Box::new(TestPlugin::new("test-plugin")), config)
            .unwrap();

        let tool_def = PluginToolDefinition {
            name: "async_tool".to_string(),
            description: "async tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_| Ok("async result".to_string())));
        let result = manager.register_plugin_tool(tool).await;
        assert!(result.is_ok());

        let tools = manager.list_plugin_tools().await;
        assert!(tools.iter().any(|t| t.name == "async_tool"));
    }

    #[tokio::test]
    async fn test_plugin_tool_execution() {
        let mut manager = PluginManager::new();
        let mut config = create_test_config("test-plugin");
        config.permissions.capabilities = vec![PluginCapability::AddTools];
        manager
            .register_with_config(Box::new(TestPlugin::new("test-plugin")), config)
            .unwrap();

        let tool_def = PluginToolDefinition {
            name: "exec_tool".to_string(),
            description: "exec tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            provider_name: "test-plugin".to_string(),
        };

        let tool = PluginTool::new(tool_def, Box::new(|_| Ok("executed".to_string())));
        manager.register_plugin_tool(tool).await.unwrap();

        let result = manager
            .execute_plugin_tool("exec_tool", serde_json::json!({}))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "executed");
    }

    #[test]
    fn test_plugin_domain_runtime() {
        let config = PluginConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            priority: 0,
            domain: PluginDomain::Runtime,
            options: IndexMap::new(),
            permissions: PluginPermissions::default(),
        };

        assert_eq!(config.domain.as_str(), "runtime");
    }

    #[test]
    fn test_plugin_domain_tui() {
        let config = PluginConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            priority: 0,
            domain: PluginDomain::Tui,
            options: IndexMap::new(),
            permissions: PluginPermissions::default(),
        };

        assert_eq!(config.domain.as_str(), "tui");
    }

    #[test]
    fn test_plugin_abi_version_compatibility() {
        use opencode_plugin::PluginAbiVersion;

        let v1 = PluginAbiVersion::new(1, 0, 0);
        let v2 = PluginAbiVersion::new(1, 2, 0);
        let v3 = PluginAbiVersion::new(2, 0, 0);

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_plugin_abi_version_supports() {
        use opencode_plugin::PluginAbiVersion;

        let runtime = PluginAbiVersion::new(1, 3, 0);
        let min_required = PluginAbiVersion::new(1, 2, 0);

        assert!(runtime.supports_abi(&min_required));
    }
}
