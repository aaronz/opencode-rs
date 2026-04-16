use indexmap::IndexMap;
use opencode_plugin::{
    config::PluginConfig, sealed::SealedPlugin, Plugin, PluginDomain, PluginError, PluginManager,
    PluginPermissions,
};
use serde_json::Value;

struct DeterministicTestPlugin {
    name: String,
    priority: i32,
}

impl DeterministicTestPlugin {
    fn new(name: &str, priority: i32) -> Self {
        Self {
            name: name.to_string(),
            priority,
        }
    }

    fn to_config(&self) -> PluginConfig {
        PluginConfig {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            enabled: true,
            priority: self.priority,
            domain: PluginDomain::Runtime,
            options: IndexMap::new(),
            permissions: PluginPermissions::default(),
        }
    }
}

impl SealedPlugin for DeterministicTestPlugin {}

impl Plugin for DeterministicTestPlugin {
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
        "deterministic test plugin"
    }

    fn on_init(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_tool_call(
        &mut self,
        _tool_name: &str,
        _args: &Value,
        _session_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_message(&mut self, _: &str, _: &str) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_session_end(&mut self, _: &str) -> Result<(), PluginError> {
        Ok(())
    }
}

fn create_manager_with_plugins() -> PluginManager {
    let mut manager = PluginManager::new();
    let plugins = vec![
        DeterministicTestPlugin::new("plugin_z", 10),
        DeterministicTestPlugin::new("plugin_a", 1),
        DeterministicTestPlugin::new("plugin_m", 5),
        DeterministicTestPlugin::new("plugin_b", 1),
        DeterministicTestPlugin::new("plugin_y", 10),
        DeterministicTestPlugin::new("plugin_c", 2),
        DeterministicTestPlugin::new("plugin_x", 10),
    ];

    for plugin in plugins {
        let config = plugin.to_config();
        manager
            .register_with_config(Box::new(plugin), config)
            .expect("plugin registration should succeed");
    }

    manager
}

#[test]
fn test_hook_ordering_is_deterministic_across_iterations() {
    let iterations = 100;

    let first_run = create_manager_with_plugins();
    let expected_order = first_run.sorted_plugin_names().clone();

    assert_eq!(
        expected_order,
        vec!["plugin_a", "plugin_b", "plugin_c", "plugin_m", "plugin_x", "plugin_y", "plugin_z"],
        "Expected plugins sorted by priority ascending"
    );

    for i in 0..iterations {
        let manager = create_manager_with_plugins();
        let order = manager.sorted_plugin_names();

        assert_eq!(
            order, expected_order,
            "Iteration {}: hook ordering must be deterministic",
            i
        );
    }
}

#[test]
fn test_hook_ordering_respects_priority_values() {
    let mut manager = PluginManager::new();

    manager
        .register_with_config(
            Box::new(DeterministicTestPlugin::new("low_priority", 100)),
            PluginConfig {
                name: "low_priority".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
                priority: 100,
                domain: PluginDomain::Runtime,
                options: IndexMap::new(),
                permissions: PluginPermissions::default(),
            },
        )
        .unwrap();

    manager
        .register_with_config(
            Box::new(DeterministicTestPlugin::new("high_priority", -50)),
            PluginConfig {
                name: "high_priority".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
                priority: -50,
                domain: PluginDomain::Runtime,
                options: IndexMap::new(),
                permissions: PluginPermissions::default(),
            },
        )
        .unwrap();

    manager
        .register_with_config(
            Box::new(DeterministicTestPlugin::new("default_priority", 0)),
            PluginConfig {
                name: "default_priority".to_string(),
                version: "1.0.0".to_string(),
                enabled: true,
                priority: 0,
                domain: PluginDomain::Runtime,
                options: IndexMap::new(),
                permissions: PluginPermissions::default(),
            },
        )
        .unwrap();

    let order = manager.sorted_plugin_names();
    assert_eq!(
        order,
        vec!["high_priority", "default_priority", "low_priority"],
        "Plugins should be ordered by priority: negative (high) -> zero (default) -> positive (low)"
    );
}

#[test]
fn test_hook_ordering_same_priority_maintains_registration_order() {
    let mut manager = PluginManager::new();

    for (i, name) in ["first", "second", "third", "fourth", "fifth"]
        .iter()
        .enumerate()
    {
        manager
            .register_with_config(
                Box::new(DeterministicTestPlugin::new(name, 0)),
                PluginConfig {
                    name: name.to_string(),
                    version: "1.0.0".to_string(),
                    enabled: true,
                    priority: 0,
                    domain: PluginDomain::Runtime,
                    options: IndexMap::new(),
                    permissions: PluginPermissions::default(),
                },
            )
            .unwrap();
    }

    let order = manager.sorted_plugin_names();
    assert_eq!(
        order,
        vec!["first", "second", "third", "fourth", "fifth"],
        "Plugins with same priority should maintain registration order"
    );
}
