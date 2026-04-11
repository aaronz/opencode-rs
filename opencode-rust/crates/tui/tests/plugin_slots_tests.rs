//! Tests for TUI plugin slots system
//!
//! # Semantics
//!
//! - Slots can be registered dynamically
//! - Slots render content correctly
//! - Slot lifecycle is managed properly
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- slots
//! ```

use opencode_tui::plugin::TuiPluginManager;
use opencode_tui::plugin_api::{PluginSlot, PluginSlotError, SlotContext, SlotRenderResult};

struct TestSlot {
    id: String,
    slot_name: String,
    should_succeed: bool,
}

impl TestSlot {
    fn new(id: &str, slot_name: &str, should_succeed: bool) -> Self {
        Self {
            id: id.to_string(),
            slot_name: slot_name.to_string(),
            should_succeed,
        }
    }
}

impl PluginSlot for TestSlot {
    fn id(&self) -> &str {
        &self.id
    }

    fn slot_name(&self) -> &str {
        &self.slot_name
    }

    fn render(&self, _ctx: &SlotContext) -> SlotRenderResult {
        if self.should_succeed {
            SlotRenderResult::success(format!("Rendered {} in {}", self.id, self.slot_name))
        } else {
            SlotRenderResult::error("Render failed")
        }
    }
}

#[test]
fn test_slots_registry_new() {
    let manager = TuiPluginManager::new();
    assert!(manager.list_plugin_slots().is_empty());
}

#[test]
fn test_slots_register() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "slot.plugin".to_string(),
            "npm:slot.plugin".to_string(),
            "@slot/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("test-slot", "home_logo", true);
    manager.register_plugin_slot("slot.plugin", slot).unwrap();

    let slots = manager.list_plugin_slots();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].id, "test-slot");
    assert_eq!(slots[0].plugin_id, "slot.plugin");
    assert_eq!(slots[0].slot_name, "home_logo");
}

#[test]
fn test_slots_register_for_nonexistent_plugin() {
    let manager = TuiPluginManager::new();
    let slot = TestSlot::new("test-slot", "home_logo", true);

    let result = manager.register_plugin_slot("nonexistent", slot);
    assert!(matches!(result, Err(PluginSlotError::PluginNotFound(_))));
}

#[test]
fn test_slots_register_duplicate() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "dup.plugin".to_string(),
            "npm:dup.plugin".to_string(),
            "@dup/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot1 = TestSlot::new("duplicate-slot", "home_logo", true);
    let slot2 = TestSlot::new("duplicate-slot", "home_logo", true);

    manager.register_plugin_slot("dup.plugin", slot1).unwrap();

    let result = manager.register_plugin_slot("dup.plugin", slot2);
    assert!(matches!(
        result,
        Err(PluginSlotError::SlotAlreadyRegistered(_))
    ));
}

#[test]
fn test_unregister_plugin_slots() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "unreg.plugin".to_string(),
            "npm:unreg.plugin".to_string(),
            "@unreg/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot1 = TestSlot::new("slot1", "home_logo", true);
    let slot2 = TestSlot::new("slot2", "sidebar_title", true);

    manager.register_plugin_slot("unreg.plugin", slot1).unwrap();
    manager.register_plugin_slot("unreg.plugin", slot2).unwrap();

    assert_eq!(manager.list_plugin_slots().len(), 2);

    manager.unregister_plugin_slots("unreg.plugin");
    assert!(manager.list_plugin_slots().is_empty());
}

#[test]
fn test_slots_unregister_single() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "single.plugin".to_string(),
            "npm:single.plugin".to_string(),
            "@single/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("remove-me", "home_logo", true);
    manager.register_plugin_slot("single.plugin", slot).unwrap();

    assert_eq!(manager.list_plugin_slots().len(), 1);

    manager
        .unregister_plugin_slot("single.plugin", "remove-me")
        .unwrap();
    assert!(manager.list_plugin_slots().is_empty());
}

#[test]
fn test_slots_unregister_nonexistent() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let result = manager.unregister_plugin_slot("test.plugin", "nonexistent");
    assert!(matches!(result, Err(PluginSlotError::SlotNotFound(_))));
}

#[test]
fn test_list_slots_for_plugin() {
    let manager = TuiPluginManager::new();
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

    let slot1 = TestSlot::new("slot1", "home_logo", true);
    let slot2 = TestSlot::new("slot2", "sidebar_title", true);

    manager.register_plugin_slot("plugin1", slot1).unwrap();
    manager.register_plugin_slot("plugin2", slot2).unwrap();

    let plugin1_slots = manager.list_slots_for_plugin("plugin1");
    let plugin2_slots = manager.list_slots_for_plugin("plugin2");

    assert_eq!(plugin1_slots.len(), 1);
    assert_eq!(plugin2_slots.len(), 1);
    assert_eq!(plugin1_slots[0].id, "slot1");
    assert_eq!(plugin2_slots[0].id, "slot2");
}

#[test]
fn test_list_slots_by_name() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot1 = TestSlot::new("slot1", "home_logo", true);
    let slot2 = TestSlot::new("slot2", "home_logo", true);
    let slot3 = TestSlot::new("slot3", "sidebar_title", true);

    manager.register_plugin_slot("test.plugin", slot1).unwrap();
    manager.register_plugin_slot("test.plugin", slot2).unwrap();
    manager.register_plugin_slot("test.plugin", slot3).unwrap();

    let home_logo_slots = manager.list_slots_by_name("home_logo");
    let sidebar_slots = manager.list_slots_by_name("sidebar_title");

    assert_eq!(home_logo_slots.len(), 2);
    assert_eq!(sidebar_slots.len(), 1);
}

#[test]
fn test_slots_render() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "render.plugin".to_string(),
            "npm:render.plugin".to_string(),
            "@render/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("render-slot", "home_logo", true);
    manager.register_plugin_slot("render.plugin", slot).unwrap();

    let ctx = SlotContext::new("render.plugin", "home_logo");
    let result = manager.render_plugin_slot("render.plugin", "render-slot", &ctx);

    assert!(result.is_ok());
    let render_result = result.unwrap();
    assert!(render_result.success);
    assert_eq!(
        render_result.content,
        Some("Rendered render-slot in home_logo".to_string())
    );
}

#[test]
fn test_slots_render_with_context() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "ctx.plugin".to_string(),
            "npm:ctx.plugin".to_string(),
            "@ctx/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("ctx-slot", "session_prompt", true);
    manager.register_plugin_slot("ctx.plugin", slot).unwrap();

    let ctx = SlotContext::new("ctx.plugin", "session_prompt")
        .with_session_id("session-123")
        .with_visible(true);

    let result = manager.render_plugin_slot("ctx.plugin", "ctx-slot", &ctx);
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[test]
fn test_slots_render_nonexistent() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let ctx = SlotContext::new("test.plugin", "home_logo");
    let result = manager.render_plugin_slot("test.plugin", "nonexistent", &ctx);
    assert!(matches!(result, Err(PluginSlotError::SlotNotFound(_))));
}

#[test]
fn test_slots_update() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "update.plugin".to_string(),
            "npm:update.plugin".to_string(),
            "@update/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot_v1 = TestSlot::new("update-slot", "home_logo", true);
    manager
        .register_plugin_slot("update.plugin", slot_v1)
        .unwrap();

    let slot_v2 = TestSlot::new("update-slot", "home_logo", true);
    manager
        .update_plugin_slot("update.plugin", slot_v2)
        .unwrap();

    let ctx = SlotContext::new("update.plugin", "home_logo");
    let result = manager.render_plugin_slot("update.plugin", "update-slot", &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_slots_update_nonexistent() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "test.plugin".to_string(),
            "npm:test.plugin".to_string(),
            "@test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("nonexistent", "home_logo", true);
    let result = manager.update_plugin_slot("test.plugin", slot);
    assert!(matches!(result, Err(PluginSlotError::SlotNotFound(_))));
}

#[test]
fn test_slots_cleared_with_manager() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "clear.plugin".to_string(),
            "npm:clear.plugin".to_string(),
            "@clear/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("clear-slot", "home_logo", true);
    manager.register_plugin_slot("clear.plugin", slot).unwrap();

    assert_eq!(manager.list_plugin_slots().len(), 1);

    manager.clear();
    assert!(manager.list_plugin_slots().is_empty());
}

#[test]
fn test_slots_multiple_plugins_same_name() {
    let manager = TuiPluginManager::new();
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

    let slot1 = TestSlot::new("shared-slot", "home_logo", true);
    let slot2 = TestSlot::new("shared-slot", "home_logo", true);

    manager.register_plugin_slot("plugin1", slot1).unwrap();
    manager.register_plugin_slot("plugin2", slot2).unwrap();

    assert_eq!(manager.list_plugin_slots().len(), 2);
}

#[test]
fn test_slots_lifecycle_persists_through_activation() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "lifecycle.plugin".to_string(),
            "npm:lifecycle.plugin".to_string(),
            "@lifecycle/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("lifecycle-slot", "home_logo", true);
    manager
        .register_plugin_slot("lifecycle.plugin", slot)
        .unwrap();

    assert_eq!(manager.list_plugin_slots().len(), 1);

    manager.activate("lifecycle.plugin").unwrap();
    assert_eq!(manager.list_plugin_slots().len(), 1);

    manager.deactivate("lifecycle.plugin").unwrap();
    assert_eq!(manager.list_plugin_slots().len(), 1);
}

#[test]
fn test_slots_render_fails_gracefully() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "fail.plugin".to_string(),
            "npm:fail.plugin".to_string(),
            "@fail/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("fail-slot", "home_logo", false);
    manager.register_plugin_slot("fail.plugin", slot).unwrap();

    let ctx = SlotContext::new("fail.plugin", "home_logo");
    let result = manager.render_plugin_slot("fail.plugin", "fail-slot", &ctx);

    assert!(result.is_ok());
    let render_result = result.unwrap();
    assert!(!render_result.success);
    assert_eq!(render_result.error, Some("Render failed".to_string()));
}

#[test]
fn test_slots_unregister_after_plugin_deactivation() {
    let manager = TuiPluginManager::new();
    manager
        .register_plugin(
            "deact.plugin".to_string(),
            "npm:deact.plugin".to_string(),
            "@deact/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let slot = TestSlot::new("deact-slot", "home_logo", true);
    manager.register_plugin_slot("deact.plugin", slot).unwrap();

    manager.activate("deact.plugin").unwrap();
    manager.deactivate("deact.plugin").unwrap();

    manager
        .unregister_plugin_slot("deact.plugin", "deact-slot")
        .unwrap();
    assert!(manager.list_plugin_slots().is_empty());
}
