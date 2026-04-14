use opencode_tui::config::{Config, CustomTheme, TuiConfig};
use opencode_tui::theme::{Theme, ThemeManager};

fn make_default_tui_config() -> TuiConfig {
    TuiConfig {
        theme: "dark".to_string(),
        scroll_speed: 10,
        scroll_acceleration: 0.5,
        show_file_tree: true,
        show_skills_panel: false,
        diff_style: "auto".to_string(),
        typewriter_speed: 20,
        max_context_size: 5000,
        keybinds: None,
        custom_themes: Vec::new(),
    }
}

#[test]
fn test_tui_config_theme_default() {
    let tui_config = make_default_tui_config();
    assert_eq!(tui_config.theme, "dark");
}

#[test]
fn test_tui_config_theme_custom_name() {
    let mut tui_config = make_default_tui_config();
    tui_config.theme = "catppuccin".to_string();
    assert_eq!(tui_config.theme, "catppuccin");
}

#[test]
fn test_theme_manager_with_tui_config_theme() {
    let mut theme_manager = ThemeManager::new();

    let tui_config = TuiConfig {
        theme: "tokyonight".to_string(),
        scroll_speed: 10,
        scroll_acceleration: 0.5,
        show_file_tree: true,
        show_skills_panel: false,
        diff_style: "auto".to_string(),
        typewriter_speed: 20,
        max_context_size: 5000,
        keybinds: None,
        custom_themes: Vec::new(),
    };

    theme_manager.set_theme_by_name(&tui_config.theme).unwrap();
    assert_eq!(theme_manager.current().name, "tokyonight");
}

#[test]
fn test_theme_manager_list_themes_includes_tui_config_preset() {
    let theme_manager = ThemeManager::new();
    let themes: Vec<&str> = theme_manager.list_themes();

    assert!(
        themes.contains(&"default"),
        "default theme should be available"
    );
    assert!(themes.contains(&"dark"), "dark theme should be available");
    assert!(themes.contains(&"light"), "light theme should be available");
    assert!(
        themes.contains(&"catppuccin"),
        "catppuccin theme should be available"
    );
    assert!(
        themes.contains(&"tokyonight"),
        "tokyonight theme should be available"
    );
    assert!(themes.contains(&"nord"), "nord theme should be available");
    assert!(
        themes.contains(&"gruvbox"),
        "gruvbox theme should be available"
    );
}

#[test]
fn test_custom_theme_in_tui_config() {
    let custom = CustomTheme {
        name: "my-custom-theme".to_string(),
        background: "#1a1a1a".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#ff6b6b".to_string(),
        secondary: "#4ecdc4".to_string(),
        accent: "#ffe66d".to_string(),
        error: "#ff6b6b".to_string(),
        warning: "#ffe66d".to_string(),
        success: "#4ecdc4".to_string(),
        muted: "#6c7086".to_string(),
        border: "#313244".to_string(),
    };

    assert!(custom.validate().is_ok());

    let tui_config = TuiConfig {
        theme: "my-custom-theme".to_string(),
        scroll_speed: 10,
        scroll_acceleration: 0.5,
        show_file_tree: true,
        show_skills_panel: false,
        diff_style: "auto".to_string(),
        typewriter_speed: 20,
        max_context_size: 5000,
        keybinds: None,
        custom_themes: vec![custom],
    };

    assert_eq!(tui_config.custom_themes.len(), 1);
    assert_eq!(tui_config.custom_themes[0].name, "my-custom-theme");
}

#[test]
fn test_custom_theme_color_validation() {
    let valid_theme = CustomTheme {
        name: "valid".to_string(),
        background: "#000000".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#89b4fa".to_string(),
        secondary: "#cba6f7".to_string(),
        accent: "#f38ba8".to_string(),
        error: "#f38ba8".to_string(),
        warning: "#fab387".to_string(),
        success: "#a6e3a1".to_string(),
        muted: "#6c7086".to_string(),
        border: "#313244".to_string(),
    };
    assert!(valid_theme.validate().is_ok());

    let invalid_theme = CustomTheme {
        name: "invalid".to_string(),
        background: "#gggggg".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#89b4fa".to_string(),
        secondary: "#cba6f7".to_string(),
        accent: "#f38ba8".to_string(),
        error: "#f38ba8".to_string(),
        warning: "#fab387".to_string(),
        success: "#a6e3a1".to_string(),
        muted: "#6c7086".to_string(),
        border: "#313244".to_string(),
    };
    assert!(invalid_theme.validate().is_err());
}

#[test]
fn test_custom_theme_invalid_length() {
    let invalid_theme = CustomTheme {
        name: "invalid".to_string(),
        background: "#000".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#89b4fa".to_string(),
        secondary: "#cba6f7".to_string(),
        accent: "#f38ba8".to_string(),
        error: "#f38ba8".to_string(),
        warning: "#fab387".to_string(),
        success: "#a6e3a1".to_string(),
        muted: "#6c7086".to_string(),
        border: "#313244".to_string(),
    };
    assert!(invalid_theme.validate().is_err());
}

#[test]
fn test_theme_manager_register_custom_theme_from_tui_config() {
    let mut theme_manager = ThemeManager::new();

    let custom = CustomTheme {
        name: "registered-custom".to_string(),
        background: "#282828".to_string(),
        foreground: "#ebdbb2".to_string(),
        primary: "#83a598".to_string(),
        secondary: "#d3869b".to_string(),
        accent: "#fe8019".to_string(),
        error: "#fb4934".to_string(),
        warning: "#fabd2f".to_string(),
        success: "#b8bb26".to_string(),
        muted: "#928374".to_string(),
        border: "#504945".to_string(),
    };

    let errors = theme_manager.load_custom_themes(vec![custom]);
    assert!(errors.is_empty());

    let custom_themes = theme_manager.list_custom_themes();
    assert!(custom_themes.contains(&"registered-custom"));
}

#[test]
fn test_theme_manager_set_custom_theme_by_name() {
    let mut theme_manager = ThemeManager::new();

    let custom = CustomTheme {
        name: "set-custom-test".to_string(),
        background: "#000000".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#0000ff".to_string(),
        secondary: "#ff00ff".to_string(),
        accent: "#ff0000".to_string(),
        error: "#ff0000".to_string(),
        warning: "#ffff00".to_string(),
        success: "#00ff00".to_string(),
        muted: "#888888".to_string(),
        border: "#444444".to_string(),
    };

    theme_manager.load_custom_themes(vec![custom]);
    theme_manager.set_theme_by_name("set-custom-test").unwrap();

    assert_eq!(theme_manager.current().name, "set-custom-test");
    assert_eq!(theme_manager.current().colors.primary, "#0000ff");
}

#[test]
fn test_theme_manager_custom_themes_do_not_conflict_with_presets() {
    let mut theme_manager = ThemeManager::new();

    let custom = CustomTheme {
        name: "default".to_string(),
        background: "#000000".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#0000ff".to_string(),
        secondary: "#ff00ff".to_string(),
        accent: "#ff0000".to_string(),
        error: "#ff0000".to_string(),
        warning: "#ffff00".to_string(),
        success: "#00ff00".to_string(),
        muted: "#888888".to_string(),
        border: "#444444".to_string(),
    };

    let errors = theme_manager.load_custom_themes(vec![custom]);
    assert!(!errors.is_empty());
}

#[test]
fn test_config_load_tui_config_with_theme() {
    let config = Config::default_config();
    let tui = config.tui_config();
    assert_eq!(tui.theme, "dark");
}

#[test]
fn test_config_tui_config_theme_persists_through_merge() {
    let mut config = Config::default_config();
    let original_theme = config.tui_config().theme;

    let new_tui = TuiConfig {
        theme: "nord".to_string(),
        scroll_speed: 5,
        scroll_acceleration: 0.3,
        show_file_tree: false,
        show_skills_panel: true,
        diff_style: "side-by-side".to_string(),
        typewriter_speed: 10,
        max_context_size: 3000,
        keybinds: None,
        custom_themes: Vec::new(),
    };

    config.merge_tui_config(new_tui);

    assert_eq!(config.tui_config().theme, "nord");
    assert_ne!(config.tui_config().theme, original_theme);
}

#[test]
fn test_theme_loading_from_json_string() {
    let json = r##"{
        "name": "test-theme",
        "colors": {
            "background": "#1e1e2e",
            "foreground": "#cdd6f4",
            "primary": "#89b4fa",
            "secondary": "#cba6f7",
            "accent": "#f38ba8",
            "error": "#f38ba8",
            "warning": "#fab387",
            "success": "#a6e3a1",
            "muted": "#6c7086",
            "border": "#313244"
        }
    }"##;

    let theme = Theme::load_from_json(json);
    assert!(theme.is_ok());
    let theme = theme.unwrap();
    assert_eq!(theme.name, "test-theme");
    assert_eq!(theme.colors.background, "#1e1e2e");
    assert_eq!(theme.colors.primary, "#89b4fa");
}

#[test]
fn test_theme_manager_export_custom_themes_roundtrip() {
    let mut theme_manager = ThemeManager::new();

    let custom = CustomTheme {
        name: "roundtrip-test".to_string(),
        background: "#123456".to_string(),
        foreground: "#abcdef".to_string(),
        primary: "#111111".to_string(),
        secondary: "#222222".to_string(),
        accent: "#333333".to_string(),
        error: "#444444".to_string(),
        warning: "#555555".to_string(),
        success: "#666666".to_string(),
        muted: "#777777".to_string(),
        border: "#888888".to_string(),
    };

    theme_manager.load_custom_themes(vec![custom.clone()]);

    let exported = theme_manager.export_custom_themes();
    assert_eq!(exported.len(), 1);
    assert_eq!(exported[0].name, "roundtrip-test");
    assert_eq!(exported[0].background, "#123456");
}

#[test]
fn test_theme_manager_unregister_custom_theme() {
    let mut theme_manager = ThemeManager::new();

    let custom = CustomTheme {
        name: "to-be-removed".to_string(),
        background: "#000000".to_string(),
        foreground: "#ffffff".to_string(),
        primary: "#0000ff".to_string(),
        secondary: "#ff00ff".to_string(),
        accent: "#ff0000".to_string(),
        error: "#ff0000".to_string(),
        warning: "#ffff00".to_string(),
        success: "#00ff00".to_string(),
        muted: "#888888".to_string(),
        border: "#444444".to_string(),
    };

    theme_manager.load_custom_themes(vec![custom]);
    assert!(theme_manager.is_custom_theme("to-be-removed"));

    theme_manager.unregister_custom_theme("to-be-removed");
    assert!(!theme_manager.is_custom_theme("to-be-removed"));
}

#[test]
fn test_all_preset_themes_have_valid_colors() {
    let theme_manager = ThemeManager::new();
    let preset_names = [
        "default",
        "dark",
        "light",
        "catppuccin",
        "tokyonight",
        "nord",
        "gruvbox",
        "solarized-dark",
        "solarized-light",
    ];

    for name in preset_names {
        let theme = theme_manager.get_preset(name);
        assert!(theme.is_some(), "Preset '{}' should exist", name);

        let theme = theme.unwrap();
        assert!(!theme.colors.background.is_empty());
        assert!(!theme.colors.foreground.is_empty());
        assert!(!theme.colors.primary.is_empty());

        assert!(
            theme.colors.background.starts_with('#'),
            "Preset '{}' background should be hex color",
            name
        );
        assert!(
            theme.colors.foreground.starts_with('#'),
            "Preset '{}' foreground should be hex color",
            name
        );
    }
}

#[test]
fn test_theme_manager_load_theme_from_json_validates_colors() {
    let json = r##"{
        "name": "valid-colors",
        "colors": {
            "background": "#000000",
            "foreground": "#ffffff",
            "primary": "#89b4fa",
            "secondary": "#cba6f7",
            "accent": "#f38ba8",
            "error": "#f38ba8",
            "warning": "#fab387",
            "success": "#a6e3a1",
            "muted": "#6c7086",
            "border": "#313244"
        }
    }"##;

    let result = Theme::load_from_json(json);
    assert!(result.is_ok());
}

#[test]
fn test_theme_config_with_name_in_tui_json() {
    let mut config = Config::default_config();

    let tui_config_with_theme = TuiConfig {
        theme: "catppuccin".to_string(),
        scroll_speed: 10,
        scroll_acceleration: 0.5,
        show_file_tree: true,
        show_skills_panel: false,
        diff_style: "auto".to_string(),
        typewriter_speed: 20,
        max_context_size: 5000,
        keybinds: None,
        custom_themes: Vec::new(),
    };

    config.merge_tui_config(tui_config_with_theme);

    let theme_name = config.tui_config().theme;
    assert_eq!(theme_name, "catppuccin");

    let mut theme_manager = ThemeManager::new();
    theme_manager.set_theme_by_name(&theme_name).unwrap();
    assert_eq!(theme_manager.current().name, "catppuccin");
}
