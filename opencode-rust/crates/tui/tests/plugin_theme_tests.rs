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
