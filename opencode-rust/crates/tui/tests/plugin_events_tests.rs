//! Tests for TUI plugin events API
//!
//! # Semantics
//!
//! - Plugins can subscribe to events via the PluginEventRegistry
//! - Plugins can emit events with data payload
//! - Event handlers receive correct event data
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_events
//! ```

use opencode_tui::plugin::TuiPluginManager;
use opencode_tui::plugin_api::{
    PluginEvent, PluginEventData, PluginEventError, PluginEventRegistry,
};

struct TestEventHandler {
    event_name: String,
    handle_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    last_data: std::sync::Arc<std::sync::Mutex<Option<PluginEventData>>>,
}

impl TestEventHandler {
    fn new(event_name: &str) -> Self {
        Self {
            event_name: event_name.to_string(),
            handle_count: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            last_data: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn handle_count(&self) -> usize {
        self.handle_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn get_last_data(&self) -> Option<PluginEventData> {
        self.last_data.lock().unwrap().clone()
    }
}

impl Clone for TestEventHandler {
    fn clone(&self) -> Self {
        Self {
            event_name: self.event_name.clone(),
            handle_count: std::sync::Arc::clone(&self.handle_count),
            last_data: std::sync::Arc::clone(&self.last_data),
        }
    }
}

impl PluginEvent for TestEventHandler {
    fn event_name(&self) -> &str {
        &self.event_name
    }

    fn handle(&self, data: &PluginEventData) -> Result<(), PluginEventError> {
        self.handle_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        *self.last_data.lock().unwrap() = Some(data.clone());
        Ok(())
    }
}

#[test]
fn test_plugin_event_registry_new() {
    let registry = PluginEventRegistry::new();
    assert!(registry.list_subscriptions().is_empty());
}

#[test]
fn test_plugin_event_registry_subscribe() {
    let registry = PluginEventRegistry::new();
    let handler = TestEventHandler::new("test.event");

    registry.subscribe("test.plugin", handler).unwrap();

    let subscriptions = registry.list_subscriptions();
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].plugin_id, "test.plugin");
    assert_eq!(subscriptions[0].event_name, "test.event");
}

#[test]
fn test_plugin_event_registry_subscribe_multiple_handlers() {
    let registry = PluginEventRegistry::new();
    let handler1 = TestEventHandler::new("multi.event");
    let handler2 = TestEventHandler::new("multi.event");

    registry.subscribe("plugin1", handler1).unwrap();
    registry.subscribe("plugin2", handler2).unwrap();

    let subscriptions = registry.list_subscriptions();
    assert_eq!(subscriptions.len(), 2);
}

#[test]
fn test_plugin_event_registry_unsubscribe_plugin() {
    let registry = PluginEventRegistry::new();
    let handler1 = TestEventHandler::new("event1");
    let handler2 = TestEventHandler::new("event2");

    registry.subscribe("unsub.plugin", handler1).unwrap();
    registry.subscribe("unsub.plugin", handler2).unwrap();

    assert_eq!(registry.list_subscriptions().len(), 2);

    registry.unsubscribe_plugin("unsub.plugin");

    assert!(registry.list_subscriptions().is_empty());
}

#[test]
fn test_plugin_event_registry_list_subscriptions_for_plugin() {
    let registry = PluginEventRegistry::new();
    let handler1 = TestEventHandler::new("evt1");
    let handler2 = TestEventHandler::new("evt2");

    registry.subscribe("target.plugin", handler1).unwrap();
    registry.subscribe("target.plugin", handler2).unwrap();
    registry
        .subscribe("other.plugin", TestEventHandler::new("evt3"))
        .unwrap();

    let subscriptions = registry.list_subscriptions_for_plugin("target.plugin");
    assert_eq!(subscriptions.len(), 2);
}

#[test]
fn test_plugin_event_registry_emit() {
    let registry = PluginEventRegistry::new();
    let handler = TestEventHandler::new("emit.test");
    registry.subscribe("emit.plugin", handler).unwrap();

    let data = PluginEventData::new("emit.test", serde_json::json!({"key": "value"}));
    let results = registry.emit(&data);

    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());
}

#[test]
fn test_plugin_event_registry_emit_no_subscribers() {
    let registry = PluginEventRegistry::new();

    let data = PluginEventData::new("no.subscriber", serde_json::json!({"key": "value"}));
    let results = registry.emit(&data);

    assert!(results.is_empty());
}

#[test]
fn test_plugin_event_registry_emit_multiple_handlers() {
    let registry = PluginEventRegistry::new();
    let handler1 = TestEventHandler::new("shared.event");
    let handler2 = TestEventHandler::new("shared.event");

    registry.subscribe("plugin1", handler1).unwrap();
    registry.subscribe("plugin2", handler2).unwrap();

    let data = PluginEventData::new("shared.event", serde_json::json!({"data": 123}));
    let results = registry.emit(&data);

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[test]
fn test_plugin_event_data_new() {
    let data = PluginEventData::new("test.event", serde_json::json!({"foo": "bar"}));
    assert_eq!(data.event_name, "test.event");
    assert!(data.source_plugin.is_none());
    assert_eq!(data.payload, serde_json::json!({"foo": "bar"}));
}

#[test]
fn test_plugin_event_data_with_source() {
    let data =
        PluginEventData::new("test.event", serde_json::json!({})).with_source("source.plugin");
    assert_eq!(data.event_name, "test.event");
    assert_eq!(data.source_plugin, Some("source.plugin".to_string()));
}

#[test]
fn test_plugin_event_data_serialization() {
    let data = PluginEventData::new(
        "serialize.test",
        serde_json::json!({"nested": {"value": 42}}),
    )
    .with_source("serialize.plugin");

    let json = serde_json::to_string(&data).unwrap();
    let deserialized: PluginEventData = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.event_name, "serialize.test");
    assert_eq!(
        deserialized.source_plugin,
        Some("serialize.plugin".to_string())
    );
    assert_eq!(
        deserialized.payload,
        serde_json::json!({"nested": {"value": 42}})
    );
}

#[test]
fn test_plugin_event_handler_receives_correct_data() {
    let registry = PluginEventRegistry::new();
    let handler = TestEventHandler::new("data.test");
    let handler_clone = handler.clone();
    registry.subscribe("data.plugin", handler).unwrap();

    let data = PluginEventData::new("data.test", serde_json::json!({"received": true}))
        .with_source("emitter.plugin");

    registry.emit(&data);

    let received = handler_clone.get_last_data().unwrap();
    assert_eq!(received.event_name, "data.test");
    assert_eq!(received.source_plugin, Some("emitter.plugin".to_string()));
    assert_eq!(received.payload, serde_json::json!({"received": true}));
}

#[test]
fn test_plugin_event_handler_count() {
    let registry = PluginEventRegistry::new();
    let handler = TestEventHandler::new("count.test");
    let handler_clone = handler.clone();
    registry.subscribe("count.plugin", handler).unwrap();

    for _ in 0..5 {
        let data = PluginEventData::new("count.test", serde_json::json!({}));
        registry.emit(&data);
    }

    assert_eq!(handler_clone.handle_count(), 5);
}

#[test]
fn test_plugin_event_clear() {
    let registry = PluginEventRegistry::new();
    registry
        .subscribe("clear.plugin", TestEventHandler::new("clear1"))
        .unwrap();
    registry
        .subscribe("clear.plugin", TestEventHandler::new("clear2"))
        .unwrap();

    assert!(!registry.list_subscriptions().is_empty());

    registry.clear();

    assert!(registry.list_subscriptions().is_empty());
}

#[test]
fn test_tui_plugin_manager_event_registry_access() {
    let manager = TuiPluginManager::new();
    let registry = manager.event_registry();

    registry
        .subscribe("access.plugin", TestEventHandler::new("access.event"))
        .unwrap();

    assert_eq!(registry.list_subscriptions().len(), 1);
}

#[test]
fn test_tui_plugin_manager_subscribe_to_event() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "event.plugin".to_string(),
            "npm:event.plugin".to_string(),
            "@event/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .subscribe_to_event("event.plugin", TestEventHandler::new("my.event"))
        .unwrap();

    let subscriptions = manager.list_event_subscriptions();
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].event_name, "my.event");
}

#[test]
fn test_tui_plugin_manager_subscribe_nonexistent_plugin() {
    let manager = TuiPluginManager::new();

    let result = manager.subscribe_to_event("nonexistent", TestEventHandler::new("event"));
    assert!(matches!(result, Err(PluginEventError::PluginNotFound(_))));
}

#[test]
fn test_tui_plugin_manager_emit_event() {
    let manager = TuiPluginManager::new();
    let handler = TestEventHandler::new("emit.event");
    let handler_clone = handler.clone();

    manager
        .register_plugin(
            "emit.plugin".to_string(),
            "npm:emit.plugin".to_string(),
            "@emit/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager.subscribe_to_event("emit.plugin", handler).unwrap();

    let data = PluginEventData::new("emit.event", serde_json::json!({"emitted": true}));
    let results = manager.emit_event(data);

    assert!(results.iter().all(|r| r.is_ok()));

    let received = handler_clone.get_last_data().unwrap();
    assert_eq!(received.payload, serde_json::json!({"emitted": true}));
}

#[test]
fn test_tui_plugin_manager_emit_event_no_subscribers() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "no-sub.plugin".to_string(),
            "npm:no-sub.plugin".to_string(),
            "@no-sub/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let data = PluginEventData::new("no-sub.event", serde_json::json!({}));
    let results = manager.emit_event(data);

    assert!(results.is_empty());
}

#[test]
fn test_tui_plugin_manager_unsubscribe_plugin_events() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "unsub.plugin".to_string(),
            "npm:unsub.plugin".to_string(),
            "@unsub/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .subscribe_to_event("unsub.plugin", TestEventHandler::new("evt1"))
        .unwrap();
    manager
        .subscribe_to_event("unsub.plugin", TestEventHandler::new("evt2"))
        .unwrap();

    assert_eq!(manager.list_event_subscriptions().len(), 2);

    manager.unsubscribe_plugin_events("unsub.plugin");

    assert!(manager.list_event_subscriptions().is_empty());
}

#[test]
fn test_tui_plugin_manager_list_subscriptions_for_plugin() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "list.plugin".to_string(),
            "npm:list.plugin".to_string(),
            "@list/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .subscribe_to_event("list.plugin", TestEventHandler::new("evt1"))
        .unwrap();
    manager
        .subscribe_to_event("list.plugin", TestEventHandler::new("evt2"))
        .unwrap();
    manager
        .register_plugin(
            "other.plugin".to_string(),
            "npm:other.plugin".to_string(),
            "@other/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .subscribe_to_event("other.plugin", TestEventHandler::new("evt3"))
        .unwrap();

    let subscriptions = manager.list_event_subscriptions_for_plugin("list.plugin");
    assert_eq!(subscriptions.len(), 2);
}

#[test]
fn test_tui_plugin_manager_clear_clears_events() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "clear.plugin".to_string(),
            "npm:clear.plugin".to_string(),
            "@clear/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .subscribe_to_event("clear.plugin", TestEventHandler::new("clear-event"))
        .unwrap();

    assert!(!manager.list_event_subscriptions().is_empty());

    manager.clear();

    assert!(manager.list_event_subscriptions().is_empty());
}

#[test]
fn test_multiple_plugins_subscribe_same_event() {
    let registry = PluginEventRegistry::new();
    let handler1 = TestEventHandler::new("shared");
    let handler2 = TestEventHandler::new("shared");

    registry.subscribe("plugin1", handler1).unwrap();
    registry.subscribe("plugin2", handler2).unwrap();

    let data = PluginEventData::new("shared", serde_json::json!({"shared": true}));
    let results = registry.emit(&data);

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[test]
fn test_event_data_payload_various_types() {
    let registry = PluginEventRegistry::new();
    let handler = TestEventHandler::new("types");
    let handler_clone = handler.clone();
    registry.subscribe("types.plugin", handler).unwrap();

    let test_cases = vec![
        serde_json::json!({"string": "hello"}),
        serde_json::json!({"number": 42}),
        serde_json::json!({"array": [1, 2, 3]}),
        serde_json::json!({"nested": {"key": "value"}}),
        serde_json::json!(null),
        serde_json::json!([{"obj": true}, {"obj": false}]),
    ];

    for payload in test_cases {
        let data = PluginEventData::new("types", payload.clone());
        registry.emit(&data);

        let received = handler_clone.get_last_data().unwrap();
        assert_eq!(received.payload, payload);
    }
}

#[test]
fn test_subscribed_event_handler_only_receives_matching_events() {
    let registry = PluginEventRegistry::new();
    let handler = TestEventHandler::new("specific");
    let handler_clone = handler.clone();
    registry.subscribe("specific.plugin", handler).unwrap();

    registry.emit(&PluginEventData::new("other.event", serde_json::json!({})));

    assert_eq!(handler_clone.handle_count(), 0);

    registry.emit(&PluginEventData::new("specific", serde_json::json!({})));
    assert_eq!(handler_clone.handle_count(), 1);
}
