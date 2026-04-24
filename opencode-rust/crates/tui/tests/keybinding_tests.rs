use opencode_tui::keybinding::{
    DefaultKeybindings, Key, KeyCode, KeyModifiers, KeybindingAction, KeybindingConfig,
    KeybindingRegistry,
};

#[test]
fn keybinding_default_command_palette_ctrl_p() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::CommandPalette).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('p'));
}

#[test]
fn keybinding_default_timeline_ctrl_t() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::Timeline).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('t'));
}

#[test]
fn keybinding_default_new_session_ctrl_n() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::NewSession).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('n'));
}

#[test]
fn keybinding_default_cancel_escape() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::Cancel).unwrap();
    assert!(key.modifiers.is_empty());
    assert_eq!(key.code, KeyCode::Esc);
}

#[test]
fn keybinding_default_navigate_arrows() {
    let defaults = DefaultKeybindings::get();

    let up = defaults.get(&KeybindingAction::NavigateUp).unwrap();
    assert_eq!(up.code, KeyCode::Up);

    let down = defaults.get(&KeybindingAction::NavigateDown).unwrap();
    assert_eq!(down.code, KeyCode::Down);
}

#[test]
fn keybinding_default_submit_enter() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::Submit).unwrap();
    assert_eq!(key.code, KeyCode::Enter);
}

#[test]
fn keybinding_default_toggle_files_ctrl_shift_f() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::ToggleFiles).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    assert_eq!(key.code, KeyCode::Char('f'));
}

#[test]
fn keybinding_custom_config_overrides_default() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+Shift+x".to_string());
    let registry = KeybindingRegistry::new(config);

    let key = registry.get_key(&KeybindingAction::CommandPalette).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    assert_eq!(key.code, KeyCode::Char('X'));
}

#[test]
fn keybinding_custom_empty_string_uses_default() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("".to_string());
    let registry = KeybindingRegistry::new(config);

    let key = registry.get_key(&KeybindingAction::CommandPalette).unwrap();
    assert_eq!(key.code, KeyCode::Char('p'));
}

#[test]
fn keybinding_registry_detects_conflict() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+p".to_string());
    config.timeline = Some("Ctrl+p".to_string());

    let registry = KeybindingRegistry::new(config);
    assert!(registry.has_conflicts());
}

#[test]
fn keybinding_registry_no_conflict_when_empty() {
    let config = KeybindingConfig::empty();
    let registry = KeybindingRegistry::new(config);
    assert!(!registry.has_conflicts());
}

#[test]
fn keybinding_registry_get_action_by_key() {
    let config = KeybindingConfig::empty();
    let registry = KeybindingRegistry::new(config);

    let key = Key {
        modifiers: KeyModifiers::CONTROL,
        code: KeyCode::Char('p'),
    };
    assert_eq!(
        registry.get_action(&key),
        Some(KeybindingAction::CommandPalette)
    );
}

#[test]
fn keybinding_registry_get_action_with_custom_binding() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+Shift+m".to_string());
    let registry = KeybindingRegistry::new(config);

    let key = Key {
        modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        code: KeyCode::Char('M'),
    };
    assert_eq!(
        registry.get_action(&key),
        Some(KeybindingAction::CommandPalette)
    );
}

#[test]
fn keybinding_registry_get_all_bindings() {
    let config = KeybindingConfig::empty();
    let registry = KeybindingRegistry::new(config);
    let bindings = registry.get_all_bindings();

    assert!(!bindings.is_empty());
    assert!(bindings.len() > 10);

    let names: Vec<_> = bindings.iter().map(|b| b.action.to_string()).collect();
    assert!(names.contains(&"command_palette".to_string()));
    assert!(names.contains(&"timeline".to_string()));
}

#[test]
fn keybinding_conflicts_returns_descriptive_message() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+x".to_string());
    config.timeline = Some("Ctrl+x".to_string());

    let conflicts = config.detect_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert!(conflicts[0].contains("Ctrl+x"));
    assert!(conflicts[0].contains("conflict"));
}

#[test]
fn keybinding_multiple_conflicts_detected() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+a".to_string());
    config.timeline = Some("Ctrl+a".to_string());
    config.new_session = Some("Ctrl+a".to_string());

    let conflicts = config.detect_conflicts();
    assert!(!conflicts.is_empty());
}

#[test]
fn keybinding_no_conflict_different_keys() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+p".to_string());
    config.timeline = Some("Ctrl+t".to_string());
    config.new_session = Some("Ctrl+n".to_string());

    let conflicts = config.detect_conflicts();
    assert!(conflicts.is_empty());
}

#[test]
fn keybinding_key_parsing_ctrl_plus_char() {
    let key = Key::parse("Ctrl+p").unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('p'));
}

#[test]
fn keybinding_key_parsing_alt_ctrl() {
    let key = Key::parse("Alt+Ctrl+x").unwrap();
    assert!(key.modifiers.contains(KeyModifiers::ALT));
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('x'));
}

#[test]
fn keybinding_key_parsing_function_keys() {
    assert_eq!(Key::parse("F1").unwrap().code, KeyCode::F(1));
    assert_eq!(Key::parse("F12").unwrap().code, KeyCode::F(12));
}

#[test]
fn keybinding_key_parsing_arrow_keys() {
    assert_eq!(Key::parse("Up").unwrap().code, KeyCode::Up);
    assert_eq!(Key::parse("Down").unwrap().code, KeyCode::Down);
    assert_eq!(Key::parse("Left").unwrap().code, KeyCode::Left);
    assert_eq!(Key::parse("Right").unwrap().code, KeyCode::Right);
}

#[test]
fn keybinding_key_parsing_navigation_keys() {
    assert_eq!(Key::parse("Home").unwrap().code, KeyCode::Home);
    assert_eq!(Key::parse("End").unwrap().code, KeyCode::End);
    assert_eq!(Key::parse("PageUp").unwrap().code, KeyCode::PageUp);
    assert_eq!(Key::parse("PageDown").unwrap().code, KeyCode::PageDown);
}

#[test]
fn keybinding_key_parsing_special_keys() {
    assert_eq!(Key::parse("Esc").unwrap().code, KeyCode::Esc);
    assert_eq!(Key::parse("Enter").unwrap().code, KeyCode::Enter);
    assert_eq!(Key::parse("Tab").unwrap().code, KeyCode::Tab);
    assert_eq!(Key::parse("Backspace").unwrap().code, KeyCode::Backspace);
}

#[test]
fn keybinding_key_parsing_case_insensitive() {
    assert!(Key::parse("CTRL+P").is_some());
    assert!(Key::parse("ctrl+p").is_some());
    assert!(Key::parse("CtRl+P").is_some());
}

#[test]
fn keybinding_key_parsing_invalid() {
    assert!(Key::parse("").is_none());
    assert!(Key::parse("NotAKey").is_none());
    assert!(Key::parse("Ctrl+").is_none());
}

#[test]
fn keybinding_config_has_custom_keybindings() {
    let mut config = KeybindingConfig::empty();
    assert!(!config.has_custom_keybindings());

    config.command_palette = Some("Ctrl+b".to_string());
    assert!(config.has_custom_keybindings());
}

#[test]
fn keybinding_config_all_empty_when_none_set() {
    let config = KeybindingConfig::empty();
    assert!(!config.has_custom_keybindings());
    assert!(config.detect_conflicts().is_empty());
}

#[test]
fn keybinding_registry_with_custom_config_applies_all() {
    let mut config = KeybindingConfig::empty();
    config.command_palette = Some("Ctrl+Shift+a".to_string());
    config.timeline = Some("Ctrl+Shift+b".to_string());
    config.new_session = Some("Ctrl+Shift+c".to_string());

    let registry = KeybindingRegistry::new(config);
    assert!(!registry.has_conflicts());

    let cmd_key = registry.get_key(&KeybindingAction::CommandPalette).unwrap();
    assert_eq!(cmd_key.code, KeyCode::Char('A'));

    let tl_key = registry.get_key(&KeybindingAction::Timeline).unwrap();
    assert_eq!(tl_key.code, KeyCode::Char('B'));

    let ns_key = registry.get_key(&KeybindingAction::NewSession).unwrap();
    assert_eq!(ns_key.code, KeyCode::Char('C'));
}

#[test]
fn keybinding_unused_action_returns_none() {
    let config = KeybindingConfig::empty();
    let registry = KeybindingRegistry::new(config);

    let key = Key {
        modifiers: KeyModifiers::CONTROL,
        code: KeyCode::Char('z'),
    };
    assert_eq!(registry.get_action(&key), None);
}

#[test]
fn keybinding_modifiers_empty_by_default() {
    let modifiers = KeyModifiers::empty();
    assert!(modifiers.is_empty());
    assert!(!modifiers.contains(KeyModifiers::CONTROL));
}

#[test]
fn keybinding_modifiers_bitwise_operations() {
    let combined = KeyModifiers::CONTROL | KeyModifiers::ALT;
    assert!(combined.contains(KeyModifiers::CONTROL));
    assert!(combined.contains(KeyModifiers::ALT));
    assert!(!combined.contains(KeyModifiers::SHIFT));
}

#[test]
fn keybinding_action_display() {
    assert_eq!(
        KeybindingAction::CommandPalette.to_string(),
        "command_palette"
    );
    assert_eq!(KeybindingAction::Timeline.to_string(), "timeline");
    assert_eq!(KeybindingAction::NewSession.to_string(), "new_session");
    assert_eq!(
        KeybindingAction::Custom("test".to_string()).to_string(),
        "custom:test"
    );
}

#[test]
fn keybinding_key_display() {
    let key = Key {
        modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT,
        code: KeyCode::Char('c'),
    };
    assert_eq!(key.to_string(), "Ctrl+Alt+c");
}

#[test]
fn keybinding_key_display_no_modifiers() {
    let key = Key {
        modifiers: KeyModifiers::empty(),
        code: KeyCode::Up,
    };
    assert_eq!(key.to_string(), "Up");
}

#[test]
fn keybinding_key_display_shift_letter() {
    let key = Key {
        modifiers: KeyModifiers::SHIFT,
        code: KeyCode::Char('A'),
    };
    assert_eq!(key.to_string(), "Shift+A");
}

#[test]
fn keybinding_default_scroll_keys() {
    let defaults = DefaultKeybindings::get();

    let page_up = defaults.get(&KeybindingAction::PageUp).unwrap();
    assert_eq!(page_up.code, KeyCode::PageUp);

    let page_down = defaults.get(&KeybindingAction::PageDown).unwrap();
    assert_eq!(page_down.code, KeyCode::PageDown);

    let scroll_up = defaults.get(&KeybindingAction::ScrollUp).unwrap();
    assert_eq!(scroll_up.code, KeyCode::PageUp);

    let scroll_down = defaults.get(&KeybindingAction::ScrollDown).unwrap();
    assert_eq!(scroll_down.code, KeyCode::PageDown);
}

#[test]
fn keybinding_default_interrupt_ctrl_c() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::Interrupt).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('c'));
}

#[test]
fn keybinding_default_settings_ctrl_comma() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::Settings).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char(','));
}

#[test]
fn keybinding_default_search_ctrl_slash() {
    let defaults = DefaultKeybindings::get();
    let key = defaults.get(&KeybindingAction::Search).unwrap();
    assert!(key.modifiers.contains(KeyModifiers::CONTROL));
    assert_eq!(key.code, KeyCode::Char('/'));
}
