//! Tests for TUI plugin lifecycle (activate/deactivate)
//!
//! # Semantics
//!
//! - Plugins can be activated at runtime
//! - Plugins can be deactivated at runtime
//! - State is properly managed during activation/deactivation
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_lifecycle
//! ```

use opencode_tui::plugin::{PluginLifecycleState, TuiPluginError, TuiPluginManager};

#[test]
fn test_plugin_lifecycle_activate_deactivate() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let entry = manager.get_plugin("test.plugin").unwrap();
    assert_eq!(entry.state, PluginLifecycleState::Registered);
    assert!(!entry.active);

    manager.activate("test.plugin").unwrap();
    let entry = manager.get_plugin("test.plugin").unwrap();
    assert_eq!(entry.state, PluginLifecycleState::Active);
    assert!(entry.active);

    manager.deactivate("test.plugin").unwrap();
    let entry = manager.get_plugin("test.plugin").unwrap();
    assert_eq!(entry.state, PluginLifecycleState::Inactive);
    assert!(!entry.active);
}

#[test]
fn test_plugin_activation_requires_master_enabled() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.set_master_enabled(false);

    let result = manager.activate("test.plugin");
    assert!(matches!(result, Err(TuiPluginError::MasterSwitchDisabled)));

    manager.set_master_enabled(true);
    manager.activate("test.plugin").unwrap();
    assert!(manager.is_plugin_active("test.plugin"));
}

#[test]
fn test_state_transitions_during_activation() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "state.test".to_string(),
            "npm:state.test".to_string(),
            "@state/test@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let initial = manager.get_plugin("state.test").unwrap();
    assert_eq!(initial.state, PluginLifecycleState::Registered);

    manager.activate("state.test").unwrap();

    let activated = manager.get_plugin("state.test").unwrap();
    assert_eq!(activated.state, PluginLifecycleState::Active);
    assert!(activated.active);

    manager.deactivate("state.test").unwrap();

    let deactivated = manager.get_plugin("state.test").unwrap();
    assert_eq!(deactivated.state, PluginLifecycleState::Inactive);
    assert!(!deactivated.active);
}

#[test]
fn test_cannot_activate_disabled_plugin() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "disabled.plugin".to_string(),
            "npm:disabled.plugin".to_string(),
            "@disabled/plugin@1.0.0".to_string(),
            false,
        )
        .unwrap();

    let result = manager.activate("disabled.plugin");
    assert!(matches!(result, Err(TuiPluginError::PluginDisabled(_))));
}

#[test]
fn test_disabling_active_plugin_deactivates_it() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "toggle.plugin".to_string(),
            "npm:toggle.plugin".to_string(),
            "@toggle/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("toggle.plugin").unwrap();
    assert!(manager.is_plugin_active("toggle.plugin"));

    manager.set_plugin_enabled("toggle.plugin", false).unwrap();

    let entry = manager.get_plugin("toggle.plugin").unwrap();
    assert!(!entry.enabled);
    assert!(!entry.active);
    assert_eq!(entry.state, PluginLifecycleState::Inactive);
}

#[test]
fn test_list_plugins_shows_correct_state() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "active.plugin".to_string(),
            "npm:active.plugin".to_string(),
            "@active/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin(
            "inactive.plugin".to_string(),
            "npm:inactive.plugin".to_string(),
            "@inactive/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("active.plugin").unwrap();

    let plugins = manager.list_plugins();
    assert_eq!(plugins.len(), 2);

    let active = plugins.iter().find(|p| p.id == "active.plugin").unwrap();
    assert!(active.active);
    assert_eq!(active.state, PluginLifecycleState::Active);

    let inactive = plugins.iter().find(|p| p.id == "inactive.plugin").unwrap();
    assert!(!inactive.active);
    assert_eq!(inactive.state, PluginLifecycleState::Registered);
}

#[test]
fn test_plugin_entry_contains_required_fields() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "fields.plugin".to_string(),
            "npm:fields.plugin".to_string(),
            "@fields/plugin@2.0.0".to_string(),
            true,
        )
        .unwrap();

    let entry = manager.get_plugin("fields.plugin").unwrap();
    assert_eq!(entry.id, "fields.plugin");
    assert_eq!(entry.source, "npm:fields.plugin");
    assert_eq!(entry.spec, "@fields/plugin@2.0.0");
    assert!(entry.enabled);
    assert!(!entry.active);
}

#[test]
fn test_reactivating_deactivated_plugin() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "reactive.plugin".to_string(),
            "npm:reactive.plugin".to_string(),
            "@reactive/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("reactive.plugin").unwrap();
    manager.deactivate("reactive.plugin").unwrap();
    manager.activate("reactive.plugin").unwrap();

    let entry = manager.get_plugin("reactive.plugin").unwrap();
    assert!(entry.active);
    assert_eq!(entry.state, PluginLifecycleState::Active);
}

#[test]
fn test_unregister_only_inactive_plugins() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "unregister.plugin".to_string(),
            "npm:unregister.plugin".to_string(),
            "@unregister/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("unregister.plugin").unwrap();
    let result = manager.unregister_plugin("unregister.plugin");
    assert!(matches!(result, Err(TuiPluginError::PluginActive(_))));

    manager.deactivate("unregister.plugin").unwrap();
    manager.unregister_plugin("unregister.plugin").unwrap();

    assert!(manager.get_plugin("unregister.plugin").is_none());
}

#[test]
fn test_clear_removes_all_plugins() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "clear.plugin1".to_string(),
            "npm:clear.plugin1".to_string(),
            "@clear/plugin1@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin(
            "clear.plugin2".to_string(),
            "npm:clear.plugin2".to_string(),
            "@clear/plugin2@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("clear.plugin1").unwrap();

    manager.clear();

    assert!(manager.list_plugins().is_empty());
}

#[test]
fn test_activate_nonexistent_plugin() {
    let manager = TuiPluginManager::new();
    let result = manager.activate("nonexistent");
    assert!(matches!(result, Err(TuiPluginError::NotFound(_))));
}

#[test]
fn test_deactivate_nonexistent_plugin() {
    let manager = TuiPluginManager::new();
    let result = manager.deactivate("nonexistent");
    assert!(matches!(result, Err(TuiPluginError::NotFound(_))));
}
