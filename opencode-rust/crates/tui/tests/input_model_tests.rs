use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opencode_tui::command::CommandRegistry;
use opencode_tui::input::history::InputHistory;
use opencode_tui::input::input_box::{InputBox, InputBoxAction};

#[test]
fn test_multiline_input_shift_enter_inserts_newline() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty()));

    let multiline_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
    let action = input_box.handle_key(multiline_key);

    assert_eq!(action, InputBoxAction::None);
    assert!(
        input_box.input().contains('\n'),
        "Shift+Enter should insert newline"
    );
}

#[test]
fn test_multiline_input_accumulates_multiple_lines() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()));

    let input = input_box.input();
    assert!(input.contains("fi"), "First line should be 'fi'");
    assert!(input.contains("second"), "Second line should be 'second'");
}

#[test]
fn test_regular_enter_submits_input_not_newline() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));

    let action = input_box.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

    match action {
        InputBoxAction::Submit(result) => {
            assert_eq!(result.raw, "test");
        }
        InputBoxAction::None => {
            panic!("Expected Submit action for regular Enter");
        }
    }
}

#[test]
fn test_history_previous_navigates_through_entries() {
    let mut history = InputHistory::new();

    history.push("first command".to_string());
    history.push("second command".to_string());
    history.push("third command".to_string());

    assert_eq!(history.previous(), Some("third command".to_string()));
    assert_eq!(history.previous(), Some("second command".to_string()));
    assert_eq!(history.previous(), Some("first command".to_string()));
}

#[test]
fn test_history_next_navigates_forward() {
    let mut history = InputHistory::new();

    history.push("first command".to_string());
    history.push("second command".to_string());
    history.push("third command".to_string());

    let _ = history.previous();
    let _ = history.previous();
    let _ = history.previous();

    assert_eq!(history.next(), Some("second command".to_string()));
    assert_eq!(history.next(), Some("third command".to_string()));
}

#[test]
fn test_history_next_at_end_returns_empty() {
    let mut history = InputHistory::new();

    history.push("only command".to_string());

    let _ = history.previous();

    assert_eq!(
        history.next(),
        Some(String::new()),
        "Should return empty string at end"
    );
}

#[test]
fn test_history_previous_at_beginning_returns_none() {
    let mut history = InputHistory::new();

    history.push("only command".to_string());

    let first = history.previous();
    let second = history.previous();

    assert_eq!(first, Some("only command".to_string()));
    assert_eq!(second, None, "Should return None at beginning");
}

#[test]
fn test_history_push_clears_navigation_index() {
    let mut history = InputHistory::new();

    history.push("first".to_string());
    history.push("second".to_string());

    let _ = history.previous();
    assert!(history.previous().is_some());

    history.push("third".to_string());

    assert_eq!(history.previous(), Some("third".to_string()));
    assert_eq!(history.previous(), Some("second".to_string()));
}

#[test]
fn test_history_reset_navigation_clears_index() {
    let mut history = InputHistory::new();

    history.push("first".to_string());
    history.push("second".to_string());

    let _ = history.previous();
    history.reset_navigation();

    assert_eq!(
        history.next(),
        None,
        "next() should return None after reset"
    );
}

#[test]
fn test_history_empty_input_not_stored() {
    let mut history = InputHistory::new();

    history.push("".to_string());
    history.push("   ".to_string());
    history.push("valid command".to_string());

    assert_eq!(history.len(), 1);
}

#[test]
fn test_history_duplicate_consecutive_entries_not_stored() {
    let mut history = InputHistory::new();

    history.push("same command".to_string());
    history.push("same command".to_string());
    history.push("different command".to_string());

    assert_eq!(history.len(), 2);
}

#[test]
fn test_history_max_size_enforcement() {
    let mut history = InputHistory::with_max_size(2);

    history.push("first".to_string());
    history.push("second".to_string());
    history.push("third".to_string());

    assert_eq!(history.len(), 2);
    assert_eq!(history.previous(), Some("third".to_string()));
    assert_eq!(history.previous(), Some("second".to_string()));
    assert_eq!(history.previous(), None, "First entry should be evicted");
}

#[test]
fn test_history_search_by_prefix() {
    let mut history = InputHistory::new();

    history.push("cargo build".to_string());
    history.push("cargo test".to_string());
    history.push("cargo check".to_string());
    history.push("ls -la".to_string());

    let results = history.search("cargo ");
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.starts_with("cargo ")));
}

#[test]
fn test_history_search_no_matches() {
    let mut history = InputHistory::new();

    history.push("cargo build".to_string());
    history.push("cargo test".to_string());

    let results = history.search("unknown");
    assert!(results.is_empty());
}

#[test]
fn test_history_clear() {
    let mut history = InputHistory::new();

    history.push("command1".to_string());
    history.push("command2".to_string());

    assert!(!history.is_empty());

    history.clear();

    assert!(history.is_empty());
}

#[test]
fn test_autocomplete_triggers_on_file_reference() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    std::fs::create_dir_all(temp_dir.path().join("src")).expect("create src");
    std::fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").expect("write main");
    std::fs::write(temp_dir.path().join("src/lib.rs"), "").expect("write lib");

    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('@'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));

    let completions = input_box.completions();
    assert!(
        !completions.is_empty(),
        "Should have file completions for @s"
    );
}

#[test]
fn test_autocomplete_triggers_on_slash_command() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty()));

    let completions = input_box.completions();
    assert!(
        !completions.is_empty(),
        "Should have command completions for /co"
    );
    assert!(
        completions.iter().any(|c| c.starts_with("/compact")),
        "Should suggest /compact command"
    );
}

#[test]
fn test_autocomplete_tab_key_applies_completion() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty()));

    let initial_input = input_box.input().to_string();

    input_box.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));

    let new_input = input_box.input();
    assert_ne!(initial_input, new_input, "Tab should apply completion");
}

#[test]
fn test_autocomplete_file_reference_partial_match() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");

    std::fs::create_dir_all(temp_dir.path().join("src/utils")).expect("create src/utils");
    std::fs::write(temp_dir.path().join("src/utils/helper.rs"), "").expect("write helper");
    std::fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").expect("write main");

    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('@'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty()));

    let completions = input_box.completions();
    assert!(
        completions.iter().any(|c| c.contains("main.rs")),
        "Should suggest main.rs for @m"
    );
}

#[test]
fn test_autocomplete_no_duplicates() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty()));

    let completions = input_box.completions();
    let help_count = completions.iter().filter(|c| *c == "/help").count();
    assert_eq!(help_count, 1, "Should not have duplicate /help entries");
}

#[test]
fn test_autocomplete_empty_when_no_match() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));

    let completions = input_box.completions();
    assert!(
        completions.is_empty(),
        "Should have no completions for unknown command"
    );
}

#[test]
fn test_input_box_cursor_position_after_completion() {
    let registry = CommandRegistry::new();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let mut input_box = InputBox::new(temp_dir.path(), &registry);

    input_box.handle_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty()));
    input_box.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty()));

    let input = input_box.input();
    assert_eq!(
        input.len(),
        input.chars().count(),
        "Cursor should be at end after completion"
    );
}
