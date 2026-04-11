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

#[test]
fn test_activation_state_transition_observable() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "observe.plugin".to_string(),
            "npm:observe.plugin".to_string(),
            "@observe/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let before = manager.get_plugin("observe.plugin").unwrap();
    assert_eq!(before.state, PluginLifecycleState::Registered);

    manager.activate("observe.plugin").unwrap();

    let after = manager.get_plugin("observe.plugin").unwrap();
    assert_eq!(after.state, PluginLifecycleState::Active);
    assert!(after.active);
}

#[test]
fn test_deactivation_state_transition_observable() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "deobserve.plugin".to_string(),
            "npm:deobserve.plugin".to_string(),
            "@deobserve/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("deobserve.plugin").unwrap();

    let active = manager.get_plugin("deobserve.plugin").unwrap();
    assert_eq!(active.state, PluginLifecycleState::Active);

    manager.deactivate("deobserve.plugin").unwrap();

    let deactivated = manager.get_plugin("deobserve.plugin").unwrap();
    assert_eq!(deactivated.state, PluginLifecycleState::Inactive);
    assert!(!deactivated.active);
}

#[test]
fn test_multiple_plugins_activate_deactivate_order() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "first.plugin".to_string(),
            "npm:first.plugin".to_string(),
            "@first/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "second.plugin".to_string(),
            "npm:second.plugin".to_string(),
            "@second/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "third.plugin".to_string(),
            "npm:third.plugin".to_string(),
            "@third/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("first.plugin").unwrap();
    manager.activate("second.plugin").unwrap();
    manager.activate("third.plugin").unwrap();

    assert!(manager.is_plugin_active("first.plugin"));
    assert!(manager.is_plugin_active("second.plugin"));
    assert!(manager.is_plugin_active("third.plugin"));

    manager.deactivate("second.plugin").unwrap();

    assert!(manager.is_plugin_active("first.plugin"));
    assert!(!manager.is_plugin_active("second.plugin"));
    assert!(manager.is_plugin_active("third.plugin"));

    manager.deactivate("first.plugin").unwrap();
    manager.deactivate("third.plugin").unwrap();

    assert!(!manager.is_plugin_active("first.plugin"));
    assert!(!manager.is_plugin_active("second.plugin"));
    assert!(!manager.is_plugin_active("third.plugin"));
}

#[test]
fn test_dispose_hook_called_during_deactivation() {
    use opencode_tui::plugin_api::PluginDispose;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let manager = TuiPluginManager::new();
    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called_clone = Arc::clone(&dispose_called);

    struct ClosureDisposer<F: Fn(&str) + Send + Sync + 'static> {
        f: F,
    }
    impl<F: Fn(&str) + Send + Sync + 'static> ClosureDisposer<F> {
        fn new(f: F) -> Self {
            Self { f }
        }
    }
    impl<F: Fn(&str) + Send + Sync + 'static> PluginDispose for ClosureDisposer<F> {
        fn on_dispose(&self, plugin_id: &str) {
            (self.f)(plugin_id);
        }
    }

    manager
        .register_plugin(
            "hook.plugin".to_string(),
            "npm:hook.plugin".to_string(),
            "@hook/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "hook.plugin",
            ClosureDisposer::new(move |plugin_id| {
                assert_eq!(plugin_id, "hook.plugin");
                dispose_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("hook.plugin").unwrap();
    assert!(!dispose_called.load(Ordering::SeqCst));

    manager.deactivate("hook.plugin").unwrap();
    assert!(dispose_called.load(Ordering::SeqCst));
}

#[test]
fn test_dispose_not_called_on_activate() {
    use opencode_tui::plugin_api::PluginDispose;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let manager = TuiPluginManager::new();
    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called_clone = Arc::clone(&dispose_called);

    struct ClosureDisposer<F: Fn(&str) + Send + Sync + 'static> {
        f: F,
    }
    impl<F: Fn(&str) + Send + Sync + 'static> ClosureDisposer<F> {
        fn new(f: F) -> Self {
            Self { f }
        }
    }
    impl<F: Fn(&str) + Send + Sync + 'static> PluginDispose for ClosureDisposer<F> {
        fn on_dispose(&self, plugin_id: &str) {
            (self.f)(plugin_id);
        }
    }

    manager
        .register_plugin(
            "noactivate.plugin".to_string(),
            "npm:noactivate.plugin".to_string(),
            "@noactivate/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "noactivate.plugin",
            ClosureDisposer::new(move |_plugin_id| {
                dispose_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("noactivate.plugin").unwrap();
    assert!(!dispose_called.load(Ordering::SeqCst));
}

#[test]
fn test_all_registries_persist_across_activation() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "persist.plugin".to_string(),
            "npm:persist.plugin".to_string(),
            "@persist/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    use opencode_tui::plugin_api::{CommandContext, CommandResult, PluginCommand};
    struct TestCmd;
    impl PluginCommand for TestCmd {
        fn name(&self) -> &str {
            "test"
        }
        fn description(&self) -> &str {
            "test"
        }
        fn aliases(&self) -> Vec<String> {
            vec![]
        }
        fn execute(&self, _ctx: &CommandContext) -> CommandResult {
            CommandResult::success("ok")
        }
    }
    manager
        .register_plugin_command("persist.plugin", TestCmd)
        .unwrap();

    use opencode_tui::plugin_api::PluginTheme;
    manager
        .register_plugin_theme("persist.plugin", PluginTheme::new("test-theme"))
        .unwrap();

    manager.activate("persist.plugin").unwrap();

    assert_eq!(manager.list_plugin_commands().len(), 1);
    assert_eq!(manager.list_plugin_themes().len(), 1);

    manager.deactivate("persist.plugin").unwrap();

    assert_eq!(manager.list_plugin_commands().len(), 1);
    assert_eq!(manager.list_plugin_themes().len(), 1);
}

#[test]
fn test_lifecycle_state_reflects_exactly_one_active() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "single.plugin1".to_string(),
            "npm:single.plugin1".to_string(),
            "@single/plugin1@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "single.plugin2".to_string(),
            "npm:single.plugin2".to_string(),
            "@single/plugin2@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("single.plugin1").unwrap();
    let entry1 = manager.get_plugin("single.plugin1").unwrap();
    assert_eq!(entry1.state, PluginLifecycleState::Active);
    assert!(entry1.active);

    manager.activate("single.plugin2").unwrap();
    let entry2 = manager.get_plugin("single.plugin2").unwrap();
    assert_eq!(entry2.state, PluginLifecycleState::Active);
    assert!(entry2.active);

    let entry1_after = manager.get_plugin("single.plugin1").unwrap();
    assert!(entry1_after.active);
    assert_eq!(entry1_after.state, PluginLifecycleState::Active);
}

#[test]
fn test_plugin_state_reflects_enabled_after_toggle() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "toggle2.plugin".to_string(),
            "npm:toggle2.plugin".to_string(),
            "@toggle2/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("toggle2.plugin").unwrap();
    let entry = manager.get_plugin("toggle2.plugin").unwrap();
    assert!(entry.enabled);
    assert!(entry.active);

    manager.set_plugin_enabled("toggle2.plugin", false).unwrap();
    let entry_after = manager.get_plugin("toggle2.plugin").unwrap();
    assert!(!entry_after.enabled);
    assert!(!entry_after.active);
    assert_eq!(entry_after.state, PluginLifecycleState::Inactive);
}

#[test]
fn test_cannot_double_activate() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "double.plugin".to_string(),
            "npm:double.plugin".to_string(),
            "@double/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("double.plugin").unwrap();
    let result = manager.activate("double.plugin");
    assert!(matches!(result, Err(TuiPluginError::AlreadyActive(_))));
}

#[test]
fn test_cannot_double_deactivate() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "double2.plugin".to_string(),
            "npm:double2.plugin".to_string(),
            "@double2/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.activate("double2.plugin").unwrap();
    manager.deactivate("double2.plugin").unwrap();
    let result = manager.deactivate("double2.plugin");
    assert!(matches!(result, Err(TuiPluginError::NotActive(_))));
}

#[test]
fn test_set_plugin_enabled_while_not_active() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "enable.plugin".to_string(),
            "npm:enable.plugin".to_string(),
            "@enable/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.set_plugin_enabled("enable.plugin", false).unwrap();
    let entry = manager.get_plugin("enable.plugin").unwrap();
    assert!(!entry.enabled);
    assert!(!entry.active);
    assert_eq!(entry.state, PluginLifecycleState::Registered);
}

#[test]
fn test_master_switch_blocks_all_activations() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "master1.plugin".to_string(),
            "npm:master1.plugin".to_string(),
            "@master1/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "master2.plugin".to_string(),
            "npm:master2.plugin".to_string(),
            "@master2/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.set_master_enabled(false);

    let result1 = manager.activate("master1.plugin");
    let result2 = manager.activate("master2.plugin");

    assert!(matches!(result1, Err(TuiPluginError::MasterSwitchDisabled)));
    assert!(matches!(result2, Err(TuiPluginError::MasterSwitchDisabled)));

    manager.set_master_enabled(true);
    manager.activate("master1.plugin").unwrap();
    assert!(manager.is_plugin_active("master1.plugin"));
}
