//! Tests for TUI plugin state API
//!
//! # Semantics
//!
//! - Plugins can store persistent state
//! - State persists across TUI restarts
//! - State is isolated between plugins
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_state
//! ```

use opencode_tui::plugin::TuiPluginManager;
use opencode_tui::plugin_api::{PluginStateError, PluginStateRegistry};
use tempfile::TempDir;

fn create_temp_state_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

#[test]
fn test_plugin_state_save_and_load() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let state = serde_json::json!({
        "count": 42,
        "name": "test_plugin",
        "enabled": true
    });

    registry.save_state("test.plugin", state.clone()).unwrap();
    let loaded = registry.load_state("test.plugin").unwrap().unwrap();

    assert_eq!(loaded, state);
}

#[test]
fn test_plugin_state_load_nonexistent() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let result = registry.load_state("nonexistent.plugin").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_plugin_state_delete() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let state = serde_json::json!({"key": "value"});
    registry.save_state("delete.me", state).unwrap();
    assert!(registry.has_state("delete.me"));

    registry.delete_state("delete.me").unwrap();
    assert!(!registry.has_state("delete.me"));
}

#[test]
fn test_plugin_state_isolation_between_plugins() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let state_a = serde_json::json!({"plugin": "a", "value": 1});
    let state_b = serde_json::json!({"plugin": "b", "value": 2});

    registry.save_state("plugin.a", state_a).unwrap();
    registry.save_state("plugin.b", state_b).unwrap();

    let loaded_a = registry.load_state("plugin.a").unwrap().unwrap();
    let loaded_b = registry.load_state("plugin.b").unwrap().unwrap();

    assert_eq!(loaded_a, serde_json::json!({"plugin": "a", "value": 1}));
    assert_eq!(loaded_b, serde_json::json!({"plugin": "b", "value": 2}));

    registry.delete_state("plugin.a").unwrap();
    assert!(!registry.has_state("plugin.a"));
    assert!(registry.has_state("plugin.b"));
}

#[test]
fn test_plugin_state_persists_after_restart() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();

    let state = serde_json::json!({
        "session": "abc123",
        "preferences": {"theme": "dark", "font_size": 14}
    });

    {
        let registry = PluginStateRegistry::new(state_dir.clone());
        registry
            .save_state("persist.plugin", state.clone())
            .unwrap();
    }

    {
        let registry = PluginStateRegistry::new(state_dir);
        let loaded = registry.load_state("persist.plugin").unwrap().unwrap();
        assert_eq!(loaded, state);
    }
}

#[test]
fn test_plugin_state_clear_all() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    registry
        .save_state("plugin.1", serde_json::json!({"n": 1}))
        .unwrap();
    registry
        .save_state("plugin.2", serde_json::json!({"n": 2}))
        .unwrap();
    registry
        .save_state("plugin.3", serde_json::json!({"n": 3}))
        .unwrap();

    assert!(registry.has_state("plugin.1"));
    assert!(registry.has_state("plugin.2"));
    assert!(registry.has_state("plugin.3"));

    registry.clear_all_states().unwrap();

    assert!(!registry.has_state("plugin.1"));
    assert!(!registry.has_state("plugin.2"));
    assert!(!registry.has_state("plugin.3"));
}

#[test]
fn test_plugin_state_save_requires_registered_plugin() {
    let manager = TuiPluginManager::new();
    let state = serde_json::json!({"key": "value"});

    let result = manager.save_plugin_state("unregistered.plugin", state);
    assert!(matches!(result, Err(PluginStateError::PluginNotFound(_))));
}

#[test]
fn test_plugin_state_load_requires_registered_plugin() {
    let manager = TuiPluginManager::new();

    let result = manager.load_plugin_state("unregistered.plugin");
    assert!(matches!(result, Err(PluginStateError::PluginNotFound(_))));
}

#[test]
fn test_plugin_state_delete_requires_registered_plugin() {
    let manager = TuiPluginManager::new();

    let result = manager.delete_plugin_state("unregistered.plugin");
    assert!(matches!(result, Err(PluginStateError::PluginNotFound(_))));
}

#[test]
fn test_plugin_state_manager_lifecycle() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "state.test".to_string(),
            "npm:state.test".to_string(),
            "@state/test@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let state = serde_json::json!({
        "last_action": "initialized",
        "count": 0
    });
    manager
        .save_plugin_state("state.test", state.clone())
        .unwrap();

    manager.activate("state.test").unwrap();

    let loaded = manager.load_plugin_state("state.test").unwrap().unwrap();
    assert_eq!(loaded, state);

    let updated_state = serde_json::json!({
        "last_action": "activated",
        "count": 1
    });
    manager
        .save_plugin_state("state.test", updated_state.clone())
        .unwrap();

    manager.deactivate("state.test").unwrap();

    let loaded_after_deactivate = manager.load_plugin_state("state.test").unwrap().unwrap();
    assert_eq!(loaded_after_deactivate, updated_state);
}

#[test]
fn test_plugin_state_manager_isolation() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "plugin.a".to_string(),
            "npm:plugin.a".to_string(),
            "@plugin/a@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin(
            "plugin.b".to_string(),
            "npm:plugin.b".to_string(),
            "@plugin/b@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let state_a = serde_json::json!({"from": "plugin_a", "value": 100});
    let state_b = serde_json::json!({"from": "plugin_b", "value": 200});

    manager.save_plugin_state("plugin.a", state_a).unwrap();
    manager.save_plugin_state("plugin.b", state_b).unwrap();

    let loaded_a = manager.load_plugin_state("plugin.a").unwrap().unwrap();
    let loaded_b = manager.load_plugin_state("plugin.b").unwrap().unwrap();

    assert_eq!(
        loaded_a,
        serde_json::json!({"from": "plugin_a", "value": 100})
    );
    assert_eq!(
        loaded_b,
        serde_json::json!({"from": "plugin_b", "value": 200})
    );

    manager.delete_plugin_state("plugin.a").unwrap();

    assert!(manager.load_plugin_state("plugin.a").unwrap().is_none());
    assert!(manager.load_plugin_state("plugin.b").unwrap().is_some());
}

#[test]
fn test_plugin_state_nested_json() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let nested_state = serde_json::json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep"
                }
            },
            "array": [1, 2, 3, {"nested": true}]
        },
        "string": "text",
        "number": 42,
        "bool": false,
        "null": null
    });

    registry
        .save_state("nested.plugin", nested_state.clone())
        .unwrap();
    let loaded = registry.load_state("nested.plugin").unwrap().unwrap();

    assert_eq!(loaded, nested_state);
    assert_eq!(loaded["level1"]["level2"]["level3"]["value"], "deep");
    assert_eq!(loaded["level1"]["array"][3]["nested"], true);
}

#[test]
fn test_plugin_state_empty_object() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let empty_state = serde_json::json!({});

    registry
        .save_state("empty.plugin", empty_state.clone())
        .unwrap();
    let loaded = registry.load_state("empty.plugin").unwrap().unwrap();

    assert_eq!(loaded, empty_state);
}

#[test]
fn test_plugin_state_array() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = PluginStateRegistry::new(state_dir);

    let array_state = serde_json::json!([1, "two", true, null, {"key": "value"}]);

    registry
        .save_state("array.plugin", array_state.clone())
        .unwrap();
    let loaded = registry.load_state("array.plugin").unwrap().unwrap();

    assert_eq!(loaded, array_state);
}

#[test]
fn test_plugin_state_via_registry() {
    let temp_dir = create_temp_state_dir();
    let state_dir = temp_dir.path().to_path_buf();
    let registry = std::sync::Arc::new(PluginStateRegistry::new(state_dir));

    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "registry.test".to_string(),
            "npm:registry.test".to_string(),
            "@registry/test@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let state = serde_json::json!({"via": "registry", "value": 123});
    registry.save_state("registry.test", state.clone()).unwrap();

    let loaded = registry.load_state("registry.test").unwrap().unwrap();
    assert_eq!(loaded, state);
}

#[test]
fn test_plugin_state_clear_includes_states() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "clear.me".to_string(),
            "npm:clear.me".to_string(),
            "@clear/me@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .save_plugin_state("clear.me", serde_json::json!({"will": "be_cleared"}))
        .unwrap();

    assert!(manager.load_plugin_state("clear.me").unwrap().is_some());

    manager.clear();

    assert!(manager.load_plugin_state("clear.me").is_err());
}
