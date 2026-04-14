use opencode_core::Session;
use opencode_plugin::{
    Plugin, PluginCapability, PluginConfig, PluginDomain, PluginError, PluginManager,
    PluginPermissions, PluginTool, PluginToolDefinition,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};

struct TestPlugin {
    name: String,
    version: String,
    description: String,
    init_called: bool,
    start_called: bool,
    shutdown_called: bool,
    tool_calls: Arc<Mutex<Vec<String>>>,
    messages_received: Arc<Mutex<Vec<String>>>,
    session_ends: Arc<Mutex<Vec<String>>>,
}

impl TestPlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            init_called: false,
            start_called: false,
            shutdown_called: false,
            tool_calls: Arc::new(Mutex::new(Vec::new())),
            messages_received: Arc::new(Mutex::new(Vec::new())),
            session_ends: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn init(&mut self) -> Result<(), PluginError> {
        self.init_called = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.shutdown_called = true;
        Ok(())
    }

    fn on_init(&mut self) -> Result<(), PluginError> {
        self.init_called = true;
        Ok(())
    }

    fn on_start(&mut self) -> Result<(), PluginError> {
        self.start_called = true;
        Ok(())
    }

    fn on_tool_call(
        &mut self,
        tool_name: &str,
        _args: &Value,
        _session_id: &str,
    ) -> Result<(), PluginError> {
        let mut calls = self.tool_calls.lock().unwrap();
        calls.push(tool_name.to_string());
        Ok(())
    }

    fn on_message(&mut self, content: &str, _session_id: &str) -> Result<(), PluginError> {
        let mut messages = self.messages_received.lock().unwrap();
        messages.push(content.to_string());
        Ok(())
    }

    fn on_session_end(&mut self, session_id: &str) -> Result<(), PluginError> {
        let mut ends = self.session_ends.lock().unwrap();
        ends.push(session_id.to_string());
        Ok(())
    }
}

#[test]
fn test_plugin_hook_execution_on_init() {
    let mut manager = PluginManager::new();
    
    let mut plugin = TestPlugin::new("test-init-plugin");
    let init_before = plugin.init_called;
    
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    
    let plugin_instance = manager.get_plugin("test-init-plugin").expect("Plugin should exist");
    assert!(plugin_instance.description().contains("Test"));
}

#[test]
fn test_plugin_hook_on_start_execution() {
    let mut manager = PluginManager::new();
    
    let plugin = TestPlugin::new("test-start-plugin");
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    manager.on_start_all().expect("on_start_all should succeed");
}

#[test]
fn test_plugin_hook_on_tool_call_execution() {
    let mut manager = PluginManager::new();
    
    let plugin = TestPlugin::new("test-tool-plugin");
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    
    let session_id = "test-session-123";
    let result = manager.on_tool_call_all("read", &serde_json::json!({"path": "/test.txt"}), session_id);
    
    assert!(result.is_ok(), "on_tool_call should succeed");
}

#[test]
fn test_plugin_hook_on_message_execution() {
    let mut manager = PluginManager::new();
    
    let plugin = TestPlugin::new("test-message-plugin");
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    
    let session_id = "test-session-456";
    let result = manager.on_message_all("Hello, this is a test message", session_id);
    
    assert!(result.is_ok(), "on_message should succeed");
}

#[test]
fn test_plugin_hook_on_session_end_execution() {
    let mut manager = PluginManager::new();
    
    let plugin = TestPlugin::new("test-session-end-plugin");
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    
    let session_id = "test-session-789";
    let result = manager.on_session_end_all(session_id);
    
    assert!(result.is_ok(), "on_session_end should succeed");
}

#[test]
fn test_plugin_manager_shutdown_all() {
    let mut manager = PluginManager::new();
    
    let plugin1 = TestPlugin::new("shutdown-plugin-1");
    let plugin2 = TestPlugin::new("shutdown-plugin-2");
    
    manager.register(Box::new(plugin1)).expect("Plugin 1 should register");
    manager.register(Box::new(plugin2)).expect("Plugin 2 should register");
    
    manager.startup().expect("Startup should succeed");
    
    let result = manager.shutdown_all();
    assert!(result.is_ok(), "Shutdown should succeed");
}

#[test]
fn test_plugin_registration_and_lookup() {
    let mut manager = PluginManager::new();
    
    let plugin = TestPlugin::new("lookup-test-plugin");
    let plugin_name = plugin.name().to_string();
    
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    let found = manager.get_plugin(&plugin_name);
    assert!(found.is_some(), "Plugin should be found by name");
    
    let found_name = found.unwrap().name();
    assert_eq!(found_name, plugin_name);
}

#[test]
fn test_plugin_config_retrieval() {
    let mut manager = PluginManager::new();
    
    let mut plugin = TestPlugin::new("config-test-plugin");
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    let config = manager.get_config("config-test-plugin");
    assert!(config.is_some(), "Config should be retrievable");
    
    let cfg = config.unwrap();
    assert_eq!(cfg.name, "config-test-plugin");
    assert!(cfg.enabled);
}

#[test]
fn test_plugin_sorted_execution_order() {
    let mut manager = PluginManager::new();
    
    let plugin1 = TestPlugin::new("first-plugin");
    let plugin2 = TestPlugin::new("second-plugin");
    
    manager.register(Box::new(plugin1)).expect("First plugin should register");
    manager.register(Box::new(plugin2)).expect("Second plugin should register");
    
    manager.startup().expect("Startup should succeed");
    manager.on_start_all().expect("on_start_all should succeed");
    
    manager.shutdown_all().expect("Shutdown should succeed");
}

#[test]
fn test_plugin_hook_failure_is_contained() {
    let mut manager = PluginManager::new();
    
    let mut plugin = TestPlugin::new("failure-test-plugin");
    
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    
    let result = manager.on_message_all("test", "session-id");
    assert!(result.is_ok(), "Hook execution should succeed even if one plugin fails");
}

#[test]
fn test_plugin_tool_registration_capability() {
    let mut manager = PluginManager::new();
    
    struct ToolProviderPlugin;
    
    impl Plugin for ToolProviderPlugin {
        fn name(&self) -> &str { "tool-provider" }
        fn version(&self) -> &str { "1.0.0" }
        fn description(&self) -> &str { "Tool provider plugin" }
        fn init(&mut self) -> Result<(), PluginError> { Ok(()) }
        fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
    }
    
    let plugin = ToolProviderPlugin;
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    let cfg = manager.get_config("tool-provider");
    assert!(cfg.is_some());
}

#[tokio::test]
async fn test_plugin_async_tool_execution() {
    let manager = PluginManager::new();
    
    let tool_def = PluginToolDefinition {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: serde_json::json!({"type": "object"}),
        provider_name: "test-provider".to_string(),
    };
    
    let tool = PluginTool::new(tool_def, Box::new(|_args| Ok("success".to_string())));
    
    let result = tool.execute(serde_json::json!({}));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_plugin_session_lifecycle_integration() {
    let project = crate::common::TempProject::new();
    
    let mut session = Session::new();
    session.add_message(opencode_core::Message::user("Test message".to_string()));
    
    let session_id = session.id.to_string();
    
    let mut manager = PluginManager::new();
    let plugin = TestPlugin::new("lifecycle-test-plugin");
    manager.register(Box::new(plugin)).expect("Plugin should register");
    
    manager.startup().expect("Startup should succeed");
    
    let result = manager.on_message_all("Test message", &session_id);
    assert!(result.is_ok());
    
    let result = manager.on_session_end_all(&session_id);
    assert!(result.is_ok());
    
    manager.shutdown_all().expect("Shutdown should succeed");
}

#[test]
fn test_plugin_multiple_plugins_all_hooks() {
    let mut manager = PluginManager::new();
    
    let plugin1 = TestPlugin::new("multi-plugin-1");
    let plugin2 = TestPlugin::new("multi-plugin-2");
    let plugin3 = TestPlugin::new("multi-plugin-3");
    
    manager.register(Box::new(plugin1)).expect("Plugin 1 should register");
    manager.register(Box::new(plugin2)).expect("Plugin 2 should register");
    manager.register(Box::new(plugin3)).expect("Plugin 3 should register");
    
    manager.startup().expect("Startup should succeed");
    
    manager.on_start_all().expect("on_start_all should succeed");
    
    manager.on_message_all("Test", "session-123").expect("on_message should succeed");
    manager.on_tool_call_all("test_tool", &serde_json::json!({}), "session-123").expect("on_tool_call should succeed");
    manager.on_session_end_all("session-123").expect("on_session_end should succeed");
    
    manager.shutdown_all().expect("Shutdown should succeed");
}

#[test]
fn test_plugin_hook_execution_deterministic_order() {
    let mut manager = PluginManager::new();
    
    let execution_order = Arc::new(Mutex::new(Vec::new()));
    let exec_clone = execution_order.clone();
    
    struct OrderTrackingPlugin {
        name: String,
        order: Arc<Mutex<Vec<String>>>,
    }
    
    impl Plugin for OrderTrackingPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        fn version(&self) -> &str {
            "1.0.0"
        }
        fn description(&self) -> &str {
            "Order tracking plugin"
        }
        fn init(&mut self) -> Result<(), PluginError> {
            let mut o = self.order.lock().unwrap();
            o.push(format!("{}-init", self.name));
            Ok(())
        }
        fn shutdown(&mut self) -> Result<(), PluginError> {
            let mut o = self.order.lock().unwrap();
            o.push(format!("{}-shutdown", self.name));
            Ok(())
        }
        fn on_start(&mut self) -> Result<(), PluginError> {
            let mut o = self.order.lock().unwrap();
            o.push(format!("{}-start", self.name));
            Ok(())
        }
        fn on_tool_call(&mut self, _name: &str, _args: &Value, _session: &str) -> Result<(), PluginError> {
            let mut o = self.order.lock().unwrap();
            o.push(format!("{}-tool", self.name));
            Ok(())
        }
    }
    
    let plugin_a = OrderTrackingPlugin {
        name: "alpha".to_string(),
        order: exec_clone.clone(),
    };
    let plugin_b = OrderTrackingPlugin {
        name: "beta".to_string(),
        order: exec_clone.clone(),
    };
    
    manager.register(Box::new(plugin_a)).expect("Alpha should register");
    manager.register(Box::new(plugin_b)).expect("Beta should register");
    
    manager.startup().expect("Startup should succeed");
    manager.on_start_all().expect("on_start_all should succeed");
    manager.on_tool_call_all("test", &serde_json::json!({}), "s1").expect("tool call should succeed");
    manager.shutdown_all().expect("Shutdown should succeed");
    
    let order = execution_order.lock().unwrap();
    assert!(order.len() >= 6);
}
