//! Tests for TUI plugin theme API
//!
//! # Semantics
//!
//! - Plugins can register custom themes via the PluginThemeRegistry
//! - Theme changes apply correctly when plugins register/unregister themes
//! - Theme API provides builder pattern for easy theme creation
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_theme
//! ```

use opencode_tui::plugin::TuiPluginManager;
use opencode_tui::plugin_api::{
    PluginTheme, PluginThemeError, PluginThemeRegistry, RegisteredTheme, ThemeColors,
};

#[test]
fn test_plugin_theme_registry_new() {
    let registry = PluginThemeRegistry::new();
    assert!(registry.list_themes().is_empty());
}

#[test]
fn test_register_theme() {
    let registry = PluginThemeRegistry::new();
    let theme = PluginTheme::new("my-theme")
        .background("#000000")
        .foreground("#ffffff")
        .primary("#89b4fa");

    registry.register_theme("test.plugin", theme).unwrap();

    let themes = registry.list_themes();
    assert_eq!(themes.len(), 1);
    assert_eq!(themes[0].name, "my-theme");
    assert_eq!(themes[0].plugin_id, "test.plugin");
}

#[test]
fn test_register_duplicate_theme() {
    let registry = PluginThemeRegistry::new();
    let theme1 = PluginTheme::new("duplicate-theme");
    let theme2 = PluginTheme::new("duplicate-theme");

    registry.register_theme("test.plugin", theme1).unwrap();

    let result = registry.register_theme("test.plugin", theme2);
    assert!(matches!(
        result,
        Err(PluginThemeError::ThemeAlreadyRegistered(_))
    ));
}

#[test]
fn test_unregister_plugin_themes() {
    let registry = PluginThemeRegistry::new();
    let theme1 = PluginTheme::new("theme1");
    let theme2 = PluginTheme::new("theme2");

    registry.register_theme("test.plugin", theme1).unwrap();
    registry.register_theme("test.plugin", theme2).unwrap();

    assert_eq!(registry.list_themes().len(), 2);

    registry.unregister_plugin_themes("test.plugin");
    assert!(registry.list_themes().is_empty());
}

#[test]
fn test_get_theme() {
    let registry = PluginThemeRegistry::new();
    let theme = PluginTheme::new("get-theme")
        .background("#1a1a2e")
        .foreground("#eaeaea")
        .primary("#ff6b6b");

    registry.register_theme("get.plugin", theme).unwrap();

    let found = registry.get_theme("get.plugin", "get-theme");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "get-theme");
}

#[test]
fn test_get_theme_not_found() {
    let registry = PluginThemeRegistry::new();
    let found = registry.get_theme("nonexistent.plugin", "nonexistent");
    assert!(found.is_none());
}

#[test]
fn test_list_themes_for_plugin() {
    let registry = PluginThemeRegistry::new();
    let theme1 = PluginTheme::new("theme1");
    let theme2 = PluginTheme::new("theme2");

    registry.register_theme("multi.plugin", theme1).unwrap();
    registry.register_theme("multi.plugin", theme2).unwrap();
    registry
        .register_theme("other.plugin", PluginTheme::new("other-theme"))
        .unwrap();

    let plugin_themes = registry.list_themes_for_plugin("multi.plugin");
    assert_eq!(plugin_themes.len(), 2);
}

#[test]
fn test_multiple_plugins_same_theme_name() {
    let registry = PluginThemeRegistry::new();
    let theme1 = PluginTheme::new("shared-theme");
    let theme2 = PluginTheme::new("shared-theme");

    registry.register_theme("plugin1", theme1).unwrap();

    let result = registry.register_theme("plugin2", theme2);
    assert!(result.is_ok());
    assert_eq!(registry.list_themes().len(), 2);
}

#[test]
fn test_get_all_themes() {
    let registry = PluginThemeRegistry::new();
    let theme1 = PluginTheme::new("theme1").primary("#ff0000");
    let theme2 = PluginTheme::new("theme2").primary("#00ff00");

    registry.register_theme("p1", theme1).unwrap();
    registry.register_theme("p2", theme2).unwrap();

    let all = registry.get_all_themes();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_clear_themes() {
    let registry = PluginThemeRegistry::new();
    registry
        .register_theme("clear.plugin", PluginTheme::new("clear-theme"))
        .unwrap();

    registry.clear();
    assert!(registry.list_themes().is_empty());
}

#[test]
fn test_plugin_theme_builder_pattern() {
    let theme = PluginTheme::new("builder-test")
        .background("#282828")
        .foreground("#ebdbb2")
        .primary("#83a598")
        .secondary("#d3869b")
        .accent("#fe8019")
        .error("#fb4934")
        .warning("#fabd2f")
        .success("#b8bb26")
        .muted("#928374")
        .border("#504945");

    assert_eq!(theme.name, "builder-test");
    assert_eq!(theme.colors.background, "#282828");
    assert_eq!(theme.colors.foreground, "#ebdbb2");
    assert_eq!(theme.colors.primary, "#83a598");
    assert_eq!(theme.colors.secondary, "#d3869b");
    assert_eq!(theme.colors.accent, "#fe8019");
    assert_eq!(theme.colors.error, "#fb4934");
    assert_eq!(theme.colors.warning, "#fabd2f");
    assert_eq!(theme.colors.success, "#b8bb26");
    assert_eq!(theme.colors.muted, "#928374");
    assert_eq!(theme.colors.border, "#504945");
}

#[test]
fn test_theme_colors_default() {
    let colors = ThemeColors::default();
    assert_eq!(colors.background, "#1e1e2e");
    assert_eq!(colors.foreground, "#cdd6f4");
    assert_eq!(colors.primary, "#89b4fa");
}

#[test]
fn test_theme_with_colors() {
    let colors = ThemeColors {
        background: "#000000".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#ff0000".to_string(),
        secondary: "#00ff00".to_string(),
        accent: "#0000ff".to_string(),
        error: "#ffff00".to_string(),
        warning: "#00ffff".to_string(),
        success: "#ff00ff".to_string(),
        muted: "#888888".to_string(),
        border: "#222222".to_string(),
    };

    let theme = PluginTheme::new("custom").with_colors(colors.clone());

    assert_eq!(theme.colors.background, "#000000");
    assert_eq!(theme.colors.primary, "#ff0000");
}

#[test]
fn test_tui_plugin_manager_register_theme() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "theme.plugin".to_string(),
            "npm:theme.plugin".to_string(),
            "@theme/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme = PluginTheme::new("plugin-theme")
        .background("#0d1117")
        .foreground("#c9d1d9")
        .primary("#58a6ff");

    manager
        .register_plugin_theme("theme.plugin", theme)
        .unwrap();

    let themes = manager.list_plugin_themes();
    assert_eq!(themes.len(), 1);
    assert_eq!(themes[0].name, "plugin-theme");
}

#[test]
fn test_tui_plugin_manager_register_theme_nonexistent_plugin() {
    let manager = TuiPluginManager::new();

    let theme = PluginTheme::new("orphan-theme");
    let result = manager.register_plugin_theme("nonexistent", theme);

    assert!(matches!(result, Err(PluginThemeError::PluginNotFound(_))));
}

#[test]
fn test_tui_plugin_manager_unregister_themes() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "unreg.plugin".to_string(),
            "npm:unreg.plugin".to_string(),
            "@unreg/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_theme("unreg.plugin", PluginTheme::new("theme1"))
        .unwrap();
    manager
        .register_plugin_theme("unreg.plugin", PluginTheme::new("theme2"))
        .unwrap();

    assert_eq!(manager.list_plugin_themes().len(), 2);

    manager.unregister_plugin_themes("unreg.plugin");
    assert!(manager.list_plugin_themes().is_empty());
}

#[test]
fn test_tui_plugin_manager_list_themes_for_plugin() {
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
        .register_plugin_theme("list.plugin", PluginTheme::new("theme1"))
        .unwrap();
    manager
        .register_plugin_theme("list.plugin", PluginTheme::new("theme2"))
        .unwrap();

    let themes = manager.list_themes_for_plugin("list.plugin");
    assert_eq!(themes.len(), 2);
}

#[test]
fn test_tui_plugin_manager_get_plugin_theme() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "get.plugin".to_string(),
            "npm:get.plugin".to_string(),
            "@get/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme = PluginTheme::new("specific-theme").primary("#abc123");
    manager.register_plugin_theme("get.plugin", theme).unwrap();

    let found = manager.get_plugin_theme("get.plugin", "specific-theme");
    assert!(found.is_some());
    assert_eq!(found.unwrap().colors.primary, "#abc123");
}

#[test]
fn test_tui_plugin_manager_get_all_plugin_themes() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "all.plugin".to_string(),
            "npm:all.plugin".to_string(),
            "@all/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    manager
        .register_plugin_theme("all.plugin", PluginTheme::new("theme1"))
        .unwrap();
    manager
        .register_plugin_theme("all.plugin", PluginTheme::new("theme2"))
        .unwrap();

    let all_themes = manager.get_all_plugin_themes();
    assert_eq!(all_themes.len(), 2);
}

#[test]
fn test_clear_clears_themes() {
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
        .register_plugin_theme("clear.plugin", PluginTheme::new("theme-to-clear"))
        .unwrap();

    manager.clear();

    assert!(manager.list_plugin_themes().is_empty());
    assert!(manager.list_plugins().is_empty());
}

#[test]
fn test_plugin_theme_serialization() {
    let theme = PluginTheme::new("serial-test")
        .background("#000000")
        .foreground("#ffffff");

    let json = serde_json::to_string(&theme).unwrap();
    let deserialized: PluginTheme = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "serial-test");
    assert_eq!(deserialized.colors.background, "#000000");
    assert_eq!(deserialized.colors.foreground, "#ffffff");
}

#[test]
fn test_registered_theme_serialization() {
    let registered = RegisteredTheme {
        plugin_id: "test.plugin".to_string(),
        name: "reg-theme".to_string(),
    };

    let json = serde_json::to_string(&registered).unwrap();
    let deserialized: RegisteredTheme = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.plugin_id, "test.plugin");
    assert_eq!(deserialized.name, "reg-theme");
}

#[test]
fn test_theme_changes_apply_correctly() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "apply.plugin".to_string(),
            "npm:apply.plugin".to_string(),
            "@apply/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme1 = PluginTheme::new("changing-theme")
        .background("#111111")
        .primary("#aaaaaa");
    manager
        .register_plugin_theme("apply.plugin", theme1)
        .unwrap();

    let found1 = manager.get_plugin_theme("apply.plugin", "changing-theme");
    assert_eq!(found1.unwrap().colors.background, "#111111");

    let theme2 = PluginTheme::new("changing-theme")
        .background("#222222")
        .primary("#bbbbbb");

    manager
        .register_plugin_theme("apply.plugin", theme2)
        .unwrap_err();

    let all_themes = manager.get_all_plugin_themes();
    assert_eq!(all_themes.len(), 1);
    assert_eq!(
        manager
            .get_plugin_theme("apply.plugin", "changing-theme")
            .unwrap()
            .colors
            .background,
        "#111111"
    );
}

#[test]
fn test_different_plugins_can_have_same_theme_name() {
    let registry = PluginThemeRegistry::new();

    let theme1 = PluginTheme::new("unique-name").primary("#ff0000");
    let theme2 = PluginTheme::new("unique-name").primary("#00ff00");

    registry.register_theme("plugin-a", theme1).unwrap();
    registry.register_theme("plugin-b", theme2).unwrap();

    let all = registry.get_all_themes();
    assert_eq!(all.len(), 2);

    let from_a = registry.get_theme("plugin-a", "unique-name").unwrap();
    let from_b = registry.get_theme("plugin-b", "unique-name").unwrap();

    assert_eq!(from_a.colors.primary, "#ff0000");
    assert_eq!(from_b.colors.primary, "#00ff00");
}

#[test]
fn test_theme_auto_sync_when_plugin_installed() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "autosync.plugin".to_string(),
            "npm:autosync.plugin".to_string(),
            "@autosync/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme = PluginTheme::new("autosync-theme")
        .background("#0d1117")
        .foreground("#c9d1d9")
        .primary("#58a6ff");

    manager
        .register_plugin_theme("autosync.plugin", theme)
        .unwrap();

    let all_themes = manager.get_all_plugin_themes();
    assert_eq!(all_themes.len(), 1);
    assert_eq!(all_themes[0].name, "autosync-theme");
    assert_eq!(all_themes[0].colors.background, "#0d1117");
}

#[test]
fn test_theme_auto_sync_multiple_plugins_installed() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "plugin-a".to_string(),
            "npm:plugin-a".to_string(),
            "@plugin-a@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "plugin-b".to_string(),
            "npm:plugin-b".to_string(),
            "@plugin-b@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme_a = PluginTheme::new("theme-a").primary("#ff0000");
    let theme_b = PluginTheme::new("theme-b").primary("#00ff00");

    manager.register_plugin_theme("plugin-a", theme_a).unwrap();
    manager.register_plugin_theme("plugin-b", theme_b).unwrap();

    let all_themes = manager.get_all_plugin_themes();
    assert_eq!(all_themes.len(), 2);

    let from_a = manager.get_plugin_theme("plugin-a", "theme-a").unwrap();
    let from_b = manager.get_plugin_theme("plugin-b", "theme-b").unwrap();

    assert_eq!(from_a.colors.primary, "#ff0000");
    assert_eq!(from_b.colors.primary, "#00ff00");
}

#[test]
fn test_theme_applies_immediately_after_install() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "immediate.plugin".to_string(),
            "npm:immediate.plugin".to_string(),
            "@immediate/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme = PluginTheme::new("immediate-theme")
        .background("#1a1b26")
        .foreground("#c0caf5")
        .primary("#7aa2f7");

    manager
        .register_plugin_theme("immediate.plugin", theme)
        .unwrap();

    let registered_themes = manager.list_plugin_themes();
    assert_eq!(registered_themes.len(), 1);
    assert_eq!(registered_themes[0].name, "immediate-theme");
    assert_eq!(registered_themes[0].plugin_id, "immediate.plugin");

    let fetched_theme = manager.get_plugin_theme("immediate.plugin", "immediate-theme");
    assert!(fetched_theme.is_some());
    let fetched = fetched_theme.unwrap();
    assert_eq!(fetched.colors.background, "#1a1b26");
    assert_eq!(fetched.colors.foreground, "#c0caf5");
    assert_eq!(fetched.colors.primary, "#7aa2f7");
}

#[test]
fn test_existing_themes_not_affected_by_new_plugin_theme() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "existing.plugin".to_string(),
            "npm:existing.plugin".to_string(),
            "@existing/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let existing_theme = PluginTheme::new("existing-theme")
        .background("#111111")
        .primary("#aaaaaa");
    manager
        .register_plugin_theme("existing.plugin", existing_theme)
        .unwrap();

    let original_theme = manager.get_plugin_theme("existing.plugin", "existing-theme");
    assert!(original_theme.is_some());
    assert_eq!(original_theme.unwrap().colors.background, "#111111");

    manager
        .register_plugin(
            "new.plugin".to_string(),
            "npm:new.plugin".to_string(),
            "@new/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let new_theme = PluginTheme::new("new-theme")
        .background("#222222")
        .primary("#bbbbbb");
    manager
        .register_plugin_theme("new.plugin", new_theme)
        .unwrap();

    let existing_still_there = manager.get_plugin_theme("existing.plugin", "existing-theme");
    assert!(existing_still_there.is_some());
    assert_eq!(existing_still_there.unwrap().colors.background, "#111111");

    let new_is_there = manager.get_plugin_theme("new.plugin", "new-theme");
    assert!(new_is_there.is_some());
    assert_eq!(new_is_there.unwrap().colors.background, "#222222");

    let all_themes = manager.get_all_plugin_themes();
    assert_eq!(all_themes.len(), 2);
}

#[test]
fn test_plugin_theme_unregistration_does_not_affect_other_plugins() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "keep.plugin".to_string(),
            "npm:keep.plugin".to_string(),
            "@keep/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();
    manager
        .register_plugin(
            "remove.plugin".to_string(),
            "npm:remove.plugin".to_string(),
            "@remove/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme_keep = PluginTheme::new("keep-theme").primary("#111111");
    let theme_remove = PluginTheme::new("remove-theme").primary("#222222");

    manager
        .register_plugin_theme("keep.plugin", theme_keep)
        .unwrap();
    manager
        .register_plugin_theme("remove.plugin", theme_remove)
        .unwrap();

    assert_eq!(manager.get_all_plugin_themes().len(), 2);

    manager.unregister_plugin_themes("remove.plugin");

    assert!(manager
        .get_plugin_theme("remove.plugin", "remove-theme")
        .is_none());

    let kept = manager.get_plugin_theme("keep.plugin", "keep-theme");
    assert!(kept.is_some());
    assert_eq!(kept.unwrap().colors.primary, "#111111");

    assert_eq!(manager.get_all_plugin_themes().len(), 1);
}

#[test]
fn test_theme_auto_sync_with_multiple_themes_per_plugin() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "multi.theme.plugin".to_string(),
            "npm:multi.theme.plugin".to_string(),
            "@multi/theme/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme1 = PluginTheme::new("dark").background("#000000");
    let theme2 = PluginTheme::new("light").background("#ffffff");

    manager
        .register_plugin_theme("multi.theme.plugin", theme1)
        .unwrap();
    manager
        .register_plugin_theme("multi.theme.plugin", theme2)
        .unwrap();

    let themes_for_plugin = manager.list_themes_for_plugin("multi.theme.plugin");
    assert_eq!(themes_for_plugin.len(), 2);

    let all_themes = manager.get_all_plugin_themes();
    assert_eq!(all_themes.len(), 2);

    assert_eq!(
        manager
            .get_plugin_theme("multi.theme.plugin", "dark")
            .unwrap()
            .colors
            .background,
        "#000000"
    );
    assert_eq!(
        manager
            .get_plugin_theme("multi.theme.plugin", "light")
            .unwrap()
            .colors
            .background,
        "#ffffff"
    );
}

#[test]
fn test_theme_auto_sync_idempotent_registration() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "idempotent.plugin".to_string(),
            "npm:idempotent.plugin".to_string(),
            "@idempotent/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme1 = PluginTheme::new("idempotent-theme").primary("#111111");
    let theme2 = PluginTheme::new("idempotent-theme").primary("#222222");

    manager
        .register_plugin_theme("idempotent.plugin", theme1)
        .unwrap();

    let result = manager.register_plugin_theme("idempotent.plugin", theme2);
    assert!(result.is_err());

    let themes = manager.get_all_plugin_themes();
    assert_eq!(themes.len(), 1);
    assert_eq!(
        manager
            .get_plugin_theme("idempotent.plugin", "idempotent-theme")
            .unwrap()
            .colors
            .primary,
        "#111111"
    );
}

#[test]
fn test_theme_manager_not_affected_by_plugin_themes() {
    use opencode_tui::theme::ThemeManager;

    let mut theme_manager = ThemeManager::new();
    theme_manager.set_theme_by_name("catppuccin").unwrap();

    let initial_theme = theme_manager.current().name.clone();
    assert_eq!(initial_theme, "catppuccin");

    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "theme.plugin".to_string(),
            "npm:theme.plugin".to_string(),
            "@theme/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let plugin_theme = PluginTheme::new("plugin-custom").background("#999999");
    manager
        .register_plugin_theme("theme.plugin", plugin_theme)
        .unwrap();

    assert_eq!(
        manager.get_all_plugin_themes().len(),
        1,
        "Plugin should have its theme"
    );

    assert_eq!(
        theme_manager.current().name,
        initial_theme,
        "ThemeManager should not be affected by plugin themes"
    );

    let next_theme = theme_manager.set_theme_by_name("tokyonight");
    assert!(next_theme.is_ok());
    assert_eq!(theme_manager.current().name, "tokyonight");

    assert_eq!(
        manager.get_all_plugin_themes().len(),
        1,
        "Plugin themes should remain unchanged"
    );
}

#[test]
fn test_plugin_activate_does_not_change_theme() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "activate.test.plugin".to_string(),
            "npm:activate.test.plugin".to_string(),
            "@activate/test/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let theme = PluginTheme::new("activate-test-theme")
        .background("#123456")
        .primary("#654321");

    manager
        .register_plugin_theme("activate.test.plugin", theme)
        .unwrap();

    let theme_before = manager.get_plugin_theme("activate.test.plugin", "activate-test-theme");
    assert!(theme_before.is_some());
    assert_eq!(theme_before.unwrap().colors.background, "#123456");

    manager.activate("activate.test.plugin").unwrap();
    assert!(manager.is_plugin_active("activate.test.plugin"));

    let theme_after = manager.get_plugin_theme("activate.test.plugin", "activate-test-theme");
    assert!(theme_after.is_some());
    assert_eq!(
        theme_after.unwrap().colors.background,
        "#123456",
        "Theme should remain unchanged after plugin activation"
    );

    manager.deactivate("activate.test.plugin").unwrap();

    let theme_still_same = manager.get_plugin_theme("activate.test.plugin", "activate-test-theme");
    assert!(theme_still_same.is_some());
    assert_eq!(
        theme_still_same.unwrap().colors.background,
        "#123456",
        "Theme should remain unchanged after plugin deactivation"
    );
}
