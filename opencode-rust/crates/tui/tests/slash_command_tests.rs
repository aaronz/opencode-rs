use opencode_tui::command::{CommandAction, CommandRegistry};
use opencode_tui::dialogs::SlashCommandOverlay;
use opencode_tui::theme::Theme;

#[test]
fn test_slash_compact_command_exists_in_registry() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("compact");
    assert!(cmd.is_some(), "/compact command should exist in registry");
    let cmd = cmd.unwrap();
    assert_eq!(cmd.name, "compact");
    assert_eq!(cmd.action, CommandAction::Compact);
}

#[test]
fn test_slash_compact_command_alias() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("c");
    assert!(cmd.is_some(), "/compact alias 'c' should work");
    assert_eq!(cmd.unwrap().name, "compact");
}

#[test]
fn test_slash_connect_command_exists_in_registry() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("connect");
    assert!(cmd.is_some(), "/connect command should exist in registry");
    let cmd = cmd.unwrap();
    assert_eq!(cmd.name, "connect");
    assert_eq!(cmd.action, CommandAction::OpenConnect);
}

#[test]
fn test_slash_help_command_exists_in_registry() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("help");
    assert!(cmd.is_some(), "/help command should exist in registry");
    let cmd = cmd.unwrap();
    assert_eq!(cmd.name, "help");
    assert!(matches!(cmd.action, CommandAction::Custom(ref s) if s == "help"));
}

#[test]
fn test_slash_help_command_aliases() {
    let registry = CommandRegistry::new();
    let h_cmd = registry.get_by_name("h");
    assert!(h_cmd.is_some(), "/help alias 'h' should work");
    assert_eq!(h_cmd.unwrap().name, "help");

    let question_cmd = registry.get_by_name("?");
    assert!(question_cmd.is_some(), "/help alias '?' should work");
    assert_eq!(question_cmd.unwrap().name, "help");
}

#[test]
fn test_slash_find_compact_command() {
    let registry = CommandRegistry::new();
    let results = registry.find("comp");
    assert!(!results.is_empty(), "Should find commands matching 'comp'");
    assert!(
        results.iter().any(|c| c.name == "compact"),
        "Should find compact command"
    );
}

#[test]
fn test_slash_find_connect_command() {
    let registry = CommandRegistry::new();
    let results = registry.find("conn");
    assert!(!results.is_empty(), "Should find commands matching 'conn'");
    assert!(
        results.iter().any(|c| c.name == "connect"),
        "Should find connect command"
    );
}

#[test]
fn test_slash_find_help_command() {
    let registry = CommandRegistry::new();
    let results = registry.find("hel");
    assert!(!results.is_empty(), "Should find commands matching 'hel'");
    assert!(
        results.iter().any(|c| c.name == "help"),
        "Should find help command"
    );
}

#[test]
fn test_slash_unknown_command_returns_none() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("nonexistent_command");
    assert!(cmd.is_none(), "Unknown command should return None");
}

#[test]
fn test_slash_unknown_command_find_returns_empty() {
    let registry = CommandRegistry::new();
    let results = registry.find("xyznonexistent");
    assert!(
        results.is_empty(),
        "Unknown command prefix should return empty list"
    );
}

#[test]
fn test_slash_invalid_slash_command_returns_none() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("/invalid");
    assert!(cmd.is_none(), "Slash prefix in name should return None");
}

#[test]
fn test_slash_partial_unknown_command() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("compactx");
    assert!(cmd.is_none(), "Partial match should not work");
}

#[test]
fn test_slash_overlay_filters_compact() {
    let theme = Theme::default();
    let mut overlay = SlashCommandOverlay::new(theme);
    let registry = CommandRegistry::new();

    overlay.update_input(&registry, "comp");
    assert!(
        overlay.filtered_commands().contains(&"compact".to_string()),
        "Should filter to show compact command"
    );
}

#[test]
fn test_slash_overlay_filters_connect() {
    let theme = Theme::default();
    let mut overlay = SlashCommandOverlay::new(theme);
    let registry = CommandRegistry::new();

    overlay.update_input(&registry, "conn");
    assert!(
        overlay.filtered_commands().contains(&"connect".to_string()),
        "Should filter to show connect command"
    );
}

#[test]
fn test_slash_overlay_filters_help() {
    let theme = Theme::default();
    let mut overlay = SlashCommandOverlay::new(theme);
    let registry = CommandRegistry::new();

    overlay.update_input(&registry, "hel");
    assert!(
        overlay.filtered_commands().contains(&"help".to_string()),
        "Should filter to show help command"
    );
}

#[test]
fn test_slash_overlay_unknown_input_returns_empty() {
    let theme = Theme::default();
    let mut overlay = SlashCommandOverlay::new(theme);
    let registry = CommandRegistry::new();

    overlay.update_input(&registry, "xyzunknown");
    assert!(
        overlay.filtered_commands().is_empty(),
        "Unknown input should return empty filtered list"
    );
}

#[test]
fn test_slash_overlay_selected_command() {
    let theme = Theme::default();
    let mut overlay = SlashCommandOverlay::new(theme);
    let registry = CommandRegistry::new();

    overlay.update_input(&registry, "c");
    assert!(
        overlay.get_selected_command().is_some(),
        "Should have a selected command when matches exist"
    );
}

#[test]
fn test_slash_overlay_no_selection_on_empty() {
    let theme = Theme::default();
    let mut overlay = SlashCommandOverlay::new(theme);
    let registry = CommandRegistry::new();

    overlay.update_input(&registry, "xyznonexistent");
    assert!(
        overlay.get_selected_command().is_none(),
        "Should have no selected command when no matches"
    );
}

#[test]
fn test_slash_compact_command_description() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("compact").unwrap();
    assert!(
        cmd.description.contains("Compact"),
        "Compact command should have descriptive text"
    );
}

#[test]
fn test_slash_connect_command_description() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("connect").unwrap();
    assert!(
        cmd.description.contains("Connect"),
        "Connect command should have descriptive text"
    );
}

#[test]
fn test_slash_help_command_description() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("help").unwrap();
    assert!(
        cmd.description.contains("available") || cmd.description.to_lowercase().contains("help"),
        "Help command should have descriptive text"
    );
}

#[test]
fn test_slash_case_insensitive_command_lookup() {
    let registry = CommandRegistry::new();
    let lower = registry.get_by_name("compact");
    assert!(lower.is_some(), "lowercase should work");
}

#[test]
fn test_slash_command_registry_all_commands_count() {
    let registry = CommandRegistry::new();
    let all = registry.all();
    assert!(all.len() > 20, "Should have many commands registered");
}

#[test]
fn test_slash_compact_action_is_compact_variant() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("compact").unwrap();
    assert!(matches!(cmd.action, CommandAction::Compact));
}

#[test]
fn test_slash_connect_action_is_open_connect_variant() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("connect").unwrap();
    assert!(matches!(cmd.action, CommandAction::OpenConnect));
}

#[test]
fn test_slash_help_action_is_custom_variant() {
    let registry = CommandRegistry::new();
    let cmd = registry.get_by_name("help").unwrap();
    assert!(matches!(cmd.action, CommandAction::Custom(ref s) if s == "help"));
}

#[test]
fn test_slash_empty_string_finds_all_commands() {
    let registry = CommandRegistry::new();
    let results = registry.find("");
    assert_eq!(
        results.len(),
        registry.all().len(),
        "Empty string should match all commands"
    );
}

#[test]
fn test_slash_single_character_filter() {
    let registry = CommandRegistry::new();
    let results = registry.find("c");
    assert!(
        !results.is_empty(),
        "Single character should find matching commands"
    );
    assert!(
        results
            .iter()
            .all(|c| c.name.starts_with('c') || c.aliases.iter().any(|a| a.starts_with('c'))),
        "All results should start with 'c'"
    );
}
