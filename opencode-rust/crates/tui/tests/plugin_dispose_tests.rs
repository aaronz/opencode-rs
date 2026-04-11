//! Tests for TUI plugin onDispose lifecycle hook
//!
//! # Semantics
//!
//! - onDispose is called when plugin is deactivated
//! - Cleanup runs even on unexpected deactivation
//! - Resources are properly released
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_dispose
//! ```

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use opencode_tui::plugin::{TuiPluginError, TuiPluginManager};
use opencode_tui::plugin_api::{PluginDispose, PluginDisposeError, PluginDisposeRegistry};

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

#[test]
fn test_dispose_called_on_deactivate() {
    let manager = TuiPluginManager::new();
    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called_clone = Arc::clone(&dispose_called);

    manager
        .register_plugin(
            "test.dispose".to_string(),
            "npm:test.dispose".to_string(),
            "@test/dispose@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "test.dispose",
            ClosureDisposer::new(move |plugin_id| {
                assert_eq!(plugin_id, "test.dispose");
                dispose_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("test.dispose").unwrap();
    manager.deactivate("test.dispose").unwrap();

    assert!(dispose_called.load(Ordering::SeqCst));
}

#[test]
fn test_dispose_called_on_set_plugin_enabled_false() {
    let manager = TuiPluginManager::new();
    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called_clone = Arc::clone(&dispose_called);

    manager
        .register_plugin(
            "test.dispose".to_string(),
            "npm:test.dispose".to_string(),
            "@test/dispose@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "test.dispose",
            ClosureDisposer::new(move |plugin_id| {
                assert_eq!(plugin_id, "test.dispose");
                dispose_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("test.dispose").unwrap();
    manager.set_plugin_enabled("test.dispose", false).unwrap();

    assert!(dispose_called.load(Ordering::SeqCst));
}

#[test]
fn test_dispose_not_called_when_not_active() {
    let manager = TuiPluginManager::new();
    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called_clone = Arc::clone(&dispose_called);

    manager
        .register_plugin(
            "test.dispose".to_string(),
            "npm:test.dispose".to_string(),
            "@test/dispose@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "test.dispose",
            ClosureDisposer::new(move |_plugin_id| {
                dispose_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    let result = manager.deactivate("test.dispose");
    assert!(matches!(result, Err(TuiPluginError::NotActive(_))));
    assert!(!dispose_called.load(Ordering::SeqCst));
}

#[test]
fn test_dispose_called_multiple_times_if_deactivated_multiple_times() {
    let manager = TuiPluginManager::new();
    let dispose_count = Arc::new(AtomicUsize::new(0));
    let dispose_count_clone = Arc::clone(&dispose_count);

    manager
        .register_plugin(
            "test.dispose".to_string(),
            "npm:test.dispose".to_string(),
            "@test/dispose@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "test.dispose",
            ClosureDisposer::new(move |_plugin_id| {
                dispose_count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("test.dispose").unwrap();
    manager.deactivate("test.dispose").unwrap();

    manager.activate("test.dispose").unwrap();
    manager.deactivate("test.dispose").unwrap();

    assert_eq!(dispose_count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_dispose_receives_correct_plugin_id() {
    let manager = TuiPluginManager::new();
    let received_plugin_id = Arc::new(std::sync::Mutex::new(None));
    let received_plugin_id_clone = Arc::clone(&received_plugin_id);

    manager
        .register_plugin(
            "specific.plugin.id".to_string(),
            "npm:specific.plugin.id".to_string(),
            "@specific/plugin@2.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "specific.plugin.id",
            ClosureDisposer::new(move |plugin_id: &str| {
                *received_plugin_id_clone.lock().unwrap() = Some(plugin_id.to_string());
            }),
        )
        .unwrap();

    manager.activate("specific.plugin.id").unwrap();
    manager.deactivate("specific.plugin.id").unwrap();

    assert_eq!(
        *received_plugin_id.lock().unwrap(),
        Some("specific.plugin.id".to_string())
    );
}

#[test]
fn test_dispose_can_clean_up_resources() {
    let manager = TuiPluginManager::new();
    let resource_released = Arc::new(AtomicBool::new(false));
    let resource_released_for_drop = Arc::clone(&resource_released);
    let resource_released_for_dispose = Arc::clone(&resource_released);

    struct TestResource {
        released: Arc<AtomicBool>,
    }

    impl Drop for TestResource {
        fn drop(&mut self) {
            self.released.store(true, Ordering::SeqCst);
        }
    }

    manager
        .register_plugin(
            "test.resource".to_string(),
            "npm:test.resource".to_string(),
            "@test/resource@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let _resource = TestResource {
        released: resource_released_for_drop,
    };

    manager
        .register_plugin_dispose(
            "test.resource",
            ClosureDisposer::new(move |_plugin_id| {
                resource_released_for_dispose.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("test.resource").unwrap();
    manager.deactivate("test.resource").unwrap();

    assert!(resource_released.load(Ordering::SeqCst));
}

#[test]
fn test_dispose_error_does_not_prevent_deactivation() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "test.dispose".to_string(),
            "npm:test.dispose".to_string(),
            "@test/dispose@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "test.dispose",
            ClosureDisposer::new(move |_plugin_id| {
                panic!("dispose failed");
            }),
        )
        .unwrap();

    manager.activate("test.dispose").unwrap();

    let result = manager.deactivate("test.dispose");
    assert!(result.is_ok());

    let entry = manager.get_plugin("test.dispose").unwrap();
    assert!(!entry.active);
    assert_eq!(
        entry.state,
        opencode_tui::plugin::PluginLifecycleState::Inactive
    );
}

#[test]
fn test_dispose_registry_works_independently() {
    let registry = PluginDisposeRegistry::new();

    let dispose1_called = Arc::new(AtomicBool::new(false));
    let dispose2_called = Arc::new(AtomicBool::new(false));
    let dispose1_called_clone = Arc::clone(&dispose1_called);
    let dispose2_called_clone = Arc::clone(&dispose2_called);

    registry
        .register_disposer(
            "plugin1",
            ClosureDisposer::new(move |_id| {
                dispose1_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    registry
        .register_disposer(
            "plugin2",
            ClosureDisposer::new(move |_id| {
                dispose2_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    registry.dispose_plugin("plugin1").unwrap();
    assert!(dispose1_called.load(Ordering::SeqCst));
    assert!(!dispose2_called.load(Ordering::SeqCst));

    registry.dispose_plugin("plugin2").unwrap();
    assert!(dispose2_called.load(Ordering::SeqCst));
}

#[test]
fn test_dispose_registry_unregister() {
    let registry = PluginDisposeRegistry::new();

    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called_clone = Arc::clone(&dispose_called);

    registry
        .register_disposer(
            "test.plugin",
            ClosureDisposer::new(move |_id| {
                dispose_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    assert!(registry.has_disposer("test.plugin"));

    registry.unregister_disposer("test.plugin");

    assert!(!registry.has_disposer("test.plugin"));
}

#[test]
fn test_dispose_registry_clear() {
    let registry = PluginDisposeRegistry::new();

    let dispose_called = Arc::new(AtomicBool::new(false));
    let dispose_called1 = Arc::clone(&dispose_called);
    let dispose_called2 = Arc::clone(&dispose_called);

    registry
        .register_disposer(
            "plugin1",
            ClosureDisposer::new(move |_id| {
                dispose_called1.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    registry
        .register_disposer(
            "plugin2",
            ClosureDisposer::new(move |_id| {
                dispose_called2.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    registry.clear();

    assert!(!registry.has_disposer("plugin1"));
    assert!(!registry.has_disposer("plugin2"));
}

#[test]
fn test_dispose_on_reactivation_after_dispose() {
    let manager = TuiPluginManager::new();
    let dispose_count = Arc::new(AtomicUsize::new(0));
    let dispose_count_clone = Arc::clone(&dispose_count);

    manager
        .register_plugin(
            "test.reactivate".to_string(),
            "npm:test.reactivate".to_string(),
            "@test/reactivate@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_dispose(
            "test.reactivate",
            ClosureDisposer::new(move |_plugin_id| {
                dispose_count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("test.reactivate").unwrap();
    manager.deactivate("test.reactivate").unwrap();
    assert_eq!(dispose_count.load(Ordering::SeqCst), 1);

    manager.activate("test.reactivate").unwrap();
    manager.deactivate("test.reactivate").unwrap();
    assert_eq!(dispose_count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_dispose_with_multiple_plugins() {
    let manager = TuiPluginManager::new();
    let dispose1_called = Arc::new(AtomicBool::new(false));
    let dispose2_called = Arc::new(AtomicBool::new(false));
    let dispose3_called = Arc::new(AtomicBool::new(false));

    manager
        .register_plugin(
            "plugin1".to_string(),
            "npm:plugin1".to_string(),
            "@plugin1@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "plugin2".to_string(),
            "npm:plugin2".to_string(),
            "@plugin2@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "plugin3".to_string(),
            "npm:plugin3".to_string(),
            "@plugin3@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let dispose1_called_clone = Arc::clone(&dispose1_called);
    manager
        .register_plugin_dispose(
            "plugin1",
            ClosureDisposer::new(move |_id| {
                dispose1_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    let dispose2_called_clone = Arc::clone(&dispose2_called);
    manager
        .register_plugin_dispose(
            "plugin2",
            ClosureDisposer::new(move |_id| {
                dispose2_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    let dispose3_called_clone = Arc::clone(&dispose3_called);
    manager
        .register_plugin_dispose(
            "plugin3",
            ClosureDisposer::new(move |_id| {
                dispose3_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("plugin1").unwrap();
    manager.activate("plugin2").unwrap();
    manager.activate("plugin3").unwrap();

    manager.deactivate("plugin1").unwrap();
    assert!(dispose1_called.load(Ordering::SeqCst));
    assert!(!dispose2_called.load(Ordering::SeqCst));
    assert!(!dispose3_called.load(Ordering::SeqCst));

    manager.deactivate("plugin2").unwrap();
    manager.deactivate("plugin3").unwrap();

    assert!(dispose1_called.load(Ordering::SeqCst));
    assert!(dispose2_called.load(Ordering::SeqCst));
    assert!(dispose3_called.load(Ordering::SeqCst));
}

#[test]
fn test_register_dispose_nonexistent_plugin_fails() {
    let manager = TuiPluginManager::new();

    let result = manager.register_plugin_dispose("nonexistent", ClosureDisposer::new(|_id| {}));
    assert!(matches!(result, Err(PluginDisposeError::PluginNotFound(_))));
}

#[test]
fn test_clear_calls_dispose_for_all_active_plugins() {
    let manager = TuiPluginManager::new();
    let dispose1_called = Arc::new(AtomicBool::new(false));
    let dispose2_called = Arc::new(AtomicBool::new(false));

    manager
        .register_plugin(
            "clear1".to_string(),
            "npm:clear1".to_string(),
            "@clear1@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "clear2".to_string(),
            "npm:clear2".to_string(),
            "@clear2@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let dispose1_called_clone = Arc::clone(&dispose1_called);
    manager
        .register_plugin_dispose(
            "clear1",
            ClosureDisposer::new(move |_id| {
                dispose1_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    let dispose2_called_clone = Arc::clone(&dispose2_called);
    manager
        .register_plugin_dispose(
            "clear2",
            ClosureDisposer::new(move |_id| {
                dispose2_called_clone.store(true, Ordering::SeqCst);
            }),
        )
        .unwrap();

    manager.activate("clear1").unwrap();
    manager.activate("clear2").unwrap();

    manager.clear();

    assert!(dispose1_called.load(Ordering::SeqCst));
    assert!(dispose2_called.load(Ordering::SeqCst));
}
