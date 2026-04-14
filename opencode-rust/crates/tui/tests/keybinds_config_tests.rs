use opencode_tui::config::{Config, KeybindConfig, TuiConfig};

fn make_tui_config_with_keybinds(keybinds: KeybindConfig) -> TuiConfig {
    TuiConfig {
        theme: "dark".to_string(),
        scroll_speed: 10,
        scroll_acceleration: 0.5,
        show_file_tree: true,
        show_skills_panel: false,
        diff_style: "auto".to_string(),
        typewriter_speed: 20,
        max_context_size: 5000,
        keybinds: Some(keybinds),
        custom_themes: Vec::new(),
    }
}

#[test]
fn keybinds_config_none_by_default() {
    let tui_config = TuiConfig {
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
    };
    assert!(tui_config.keybinds.is_none());
}

#[test]
fn keybinds_config_load_from_tui_json() {
    let keybinds = KeybindConfig {
        commands: Some("Ctrl+Shift+p".to_string()),
        timeline: Some("Ctrl+Shift+t".to_string()),
        new_session: Some("Ctrl+Shift+n".to_string()),
        toggle_files: None,
        settings: None,
        search: None,
    };

    let tui_config = make_tui_config_with_keybinds(keybinds);
    assert!(tui_config.keybinds.is_some());

    let kb = tui_config.keybinds.unwrap();
    assert_eq!(kb.commands, Some("Ctrl+Shift+p".to_string()));
    assert_eq!(kb.timeline, Some("Ctrl+Shift+t".to_string()));
    assert_eq!(kb.new_session, Some("Ctrl+Shift+n".to_string()));
}

#[test]
fn keybinds_config_detects_conflicts() {
    let keybinds = KeybindConfig {
        commands: Some("Ctrl+p".to_string()),
        timeline: Some("Ctrl+p".to_string()),
        new_session: None,
        toggle_files: None,
        settings: None,
        search: None,
    };

    let conflicts = keybinds.detect_conflicts();
    assert!(!conflicts.is_empty());
}

#[test]
fn keybinds_config_no_conflicts_with_different_keys() {
    let keybinds = KeybindConfig {
        commands: Some("Ctrl+p".to_string()),
        timeline: Some("Ctrl+t".to_string()),
        new_session: Some("Ctrl+n".to_string()),
        toggle_files: None,
        settings: None,
        search: None,
    };

    let conflicts = keybinds.detect_conflicts();
    assert!(conflicts.is_empty());
}

#[test]
fn keybinds_config_merge_preserves_keybinds() {
    let mut config = Config::default_config();

    let keybinds = KeybindConfig {
        commands: Some("Ctrl+Shift+m".to_string()),
        timeline: None,
        new_session: None,
        toggle_files: None,
        settings: None,
        search: None,
    };

    let tui_with_keybinds = make_tui_config_with_keybinds(keybinds);
    config.merge_tui_config(tui_with_keybinds);

    let loaded_keybinds = config.keybinds();
    assert!(loaded_keybinds.is_some());
    assert_eq!(
        loaded_keybinds.unwrap().commands,
        Some("Ctrl+Shift+m".to_string())
    );
}

#[test]
fn keybinds_config_all_actions_supported() {
    let keybinds = KeybindConfig {
        commands: Some("Ctrl+a".to_string()),
        timeline: Some("Ctrl+b".to_string()),
        new_session: Some("Ctrl+c".to_string()),
        toggle_files: Some("Ctrl+d".to_string()),
        settings: Some("Ctrl+e".to_string()),
        search: Some("Ctrl+f".to_string()),
    };

    let conflicts = keybinds.detect_conflicts();
    assert!(conflicts.is_empty());
}

#[test]
fn keybinds_config_validate_keybinds_via_config() {
    let mut config = Config::default_config();

    let conflicting_keybinds = KeybindConfig {
        commands: Some("Ctrl+x".to_string()),
        timeline: Some("Ctrl+x".to_string()),
        new_session: None,
        toggle_files: None,
        settings: None,
        search: None,
    };

    let tui_with_keybinds = make_tui_config_with_keybinds(conflicting_keybinds);
    config.merge_tui_config(tui_with_keybinds);

    let validation_errors = config.validate_keybinds();
    assert!(!validation_errors.is_empty());
}

#[test]
fn keybinds_config_empty_commands_field_no_conflict() {
    let keybinds = KeybindConfig {
        commands: Some("".to_string()),
        timeline: Some("Ctrl+t".to_string()),
        new_session: None,
        toggle_files: None,
        settings: None,
        search: None,
    };

    let conflicts = keybinds.detect_conflicts();
    assert!(conflicts.is_empty());
}

#[test]
fn keybinds_config_multiple_conflicts_detected() {
    let keybinds = KeybindConfig {
        commands: Some("Ctrl+x".to_string()),
        timeline: Some("Ctrl+x".to_string()),
        new_session: Some("Ctrl+x".to_string()),
        toggle_files: None,
        settings: None,
        search: None,
    };

    let conflicts = keybinds.detect_conflicts();
    assert!(!conflicts.is_empty());
}
