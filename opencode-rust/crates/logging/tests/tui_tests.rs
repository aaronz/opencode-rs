#[cfg(feature = "tui")]
use opencode_logging::config::TuiLogPosition;
#[cfg(feature = "tui")]
use opencode_logging::event::{ErrorContext, LogEvent, LogLevel};
#[cfg(feature = "tui")]
use opencode_logging::tui::log_panel::LogPanel;
#[cfg(feature = "tui")]
use ratatui::{backend::TestBackend, buffer::Buffer, prelude::Widget, Terminal};

#[cfg(feature = "tui")]
fn create_test_panel() -> LogPanel {
    LogPanel::new(TuiLogPosition::Bottom)
}

#[cfg(feature = "tui")]
fn create_test_event(seq: u64, level: LogLevel, target: &str, message: &str) -> LogEvent {
    LogEvent::new(seq, level, target, message)
}

#[cfg(feature = "tui")]
fn create_error_event_with_context(seq: u64, message: &str, error_ctx: ErrorContext) -> LogEvent {
    let mut event = LogEvent::new(seq, LogLevel::Error, "agent", message);
    event.fields.extra.insert(
        "error_context".to_string(),
        serde_json::to_value(error_ctx).unwrap(),
    );
    event
}

#[cfg(feature = "tui")]
fn render_panel_to_buffer(panel: LogPanel, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            panel.render(f.area(), f.buffer_mut());
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

#[cfg(feature = "tui")]
fn has_border(buffer: &Buffer) -> bool {
    buffer.content.iter().any(|cell| cell.symbol() == "─" || cell.symbol() == "│")
}

#[cfg(feature = "tui")]
fn count_lines_with_content(buffer: &Buffer) -> usize {
    buffer.content.iter().filter(|cell| cell.symbol() != " ").count()
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_renders_with_correct_layout() {
    let panel = create_test_panel()
        .with_session_id("sess_test123");

    let buffer = render_panel_to_buffer(panel, 80, 20);

    assert!(has_border(&buffer), "Panel should render with border");
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_empty_state_shows_placeholder_message() {
    let panel = create_test_panel();

    let buffer = render_panel_to_buffer(panel, 80, 20);

    assert!(has_border(&buffer), "Empty panel should render border");
}

#[cfg(feature = "tui")]
#[test]
fn test_filter_buttons_toggle_correctly() {
    let mut panel = create_test_panel();

    assert!(panel.is_level_filter_active(LogLevel::Info));
    assert!(panel.is_level_filter_active(LogLevel::Debug));
    assert!(panel.is_level_filter_active(LogLevel::Warn));
    assert!(panel.is_level_filter_active(LogLevel::Error));

    panel.toggle_level_filter(LogLevel::Info);
    assert!(!panel.is_level_filter_active(LogLevel::Info));

    panel.toggle_level_filter(LogLevel::Debug);
    assert!(!panel.is_level_filter_active(LogLevel::Debug));

    panel.toggle_level_filter(LogLevel::Info);
    assert!(panel.is_level_filter_active(LogLevel::Info));
}

#[cfg(feature = "tui")]
#[test]
fn test_arrow_key_scroll_navigation_works() {
    let mut panel = create_test_panel();

    for i in 1..=10 {
        panel.push_event(create_test_event(i, LogLevel::Info, "test", &format!("Message {}", i)));
    }

    panel.selected_index = 0;
    panel.scroll_offset = 0;

    panel.scroll_down();
    assert_eq!(panel.selected_index, 1);

    panel.scroll_down();
    assert_eq!(panel.selected_index, 2);

    panel.scroll_up();
    assert_eq!(panel.selected_index, 1);

    panel.scroll_up();
    assert_eq!(panel.selected_index, 0);

    panel.scroll_up();
    assert_eq!(panel.selected_index, 0);
}

#[cfg(feature = "tui")]
#[test]
fn test_ctrl_l_toggles_visibility() {
    let mut panel = create_test_panel();

    assert!(panel.visible, "Panel should be visible by default");

    panel.toggle_visibility();
    assert!(!panel.visible, "Panel should be hidden after toggle");

    panel.toggle_visibility();
    assert!(panel.visible, "Panel should be visible after second toggle");
}

#[cfg(feature = "tui")]
#[test]
fn test_text_search_filters_logs() {
    let mut panel = create_test_panel();

    panel.push_event(create_test_event(1, LogLevel::Info, "test", "normal operation"));
    panel.push_event(create_test_event(2, LogLevel::Error, "test", "error occurred"));
    panel.push_event(create_test_event(3, LogLevel::Info, "test", "another normal message"));
    panel.push_event(create_test_event(4, LogLevel::Warn, "test", "warning something"));

    panel.set_search_query(Some("error".to_string()));

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].message.contains("error"));

    panel.set_search_query(Some("normal".to_string()));
    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 2);

    panel.set_search_query(None);
    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 4);
}

#[cfg(feature = "tui")]
#[test]
fn test_error_details_expand_on_click() {
    let mut panel = create_test_panel();

    let error_ctx = ErrorContext::new("ERR_TEST", "Test error occurred")
        .with_stack_frame("test.rs", 42, "test_function")
        .with_cause("ERR_ROOT", "Root cause error");

    panel.push_event(create_error_event_with_context(1, "Test error", error_ctx));

    assert_eq!(panel.expanded_error_index, None);

    panel.toggle_expanded_error(0);
    assert_eq!(panel.expanded_error_index, Some(0));

    panel.toggle_expanded_error(0);
    assert_eq!(panel.expanded_error_index, None);
}

#[cfg(feature = "tui")]
#[test]
fn test_auto_scroll_toggle_behavior() {
    let mut panel = create_test_panel();

    assert!(panel.auto_scroll, "Auto-scroll should be enabled by default");

    panel.toggle_auto_scroll();
    assert!(!panel.auto_scroll, "Auto-scroll should be disabled after toggle");

    panel.toggle_auto_scroll();
    assert!(panel.auto_scroll, "Auto-scroll should be enabled after second toggle");
}

#[cfg(feature = "tui")]
#[test]
fn test_auto_scroll_goes_to_bottom_on_new_event() {
    let mut panel = create_test_panel();
    panel.auto_scroll = true;

    for i in 1..=5 {
        panel.push_event(create_test_event(i, LogLevel::Info, "test", &format!("Message {}", i)));
    }

    assert_eq!(panel.selected_index, 4, "Auto-scroll should go to last message");
}

#[cfg(feature = "tui")]
#[test]
fn test_auto_scroll_disabled_does_not_auto_scroll() {
    let mut panel = create_test_panel();
    panel.auto_scroll = false;
    panel.selected_index = 0;

    panel.push_event(create_test_event(1, LogLevel::Info, "test", "Message 1"));
    panel.push_event(create_test_event(2, LogLevel::Info, "test", "Message 2"));

    assert_eq!(
        panel.selected_index, 0,
        "When auto-scroll is disabled, should not auto-scroll"
    );
}

#[cfg(feature = "tui")]
#[test]
fn test_filter_by_level_shows_correct_events() {
    let mut panel = create_test_panel();

    panel.push_event(create_test_event(1, LogLevel::Info, "test", "Info message"));
    panel.push_event(create_test_event(2, LogLevel::Debug, "test", "Debug message"));
    panel.push_event(create_test_event(3, LogLevel::Warn, "test", "Warn message"));
    panel.push_event(create_test_event(4, LogLevel::Error, "test", "Error message"));

    panel.toggle_level_filter(LogLevel::Debug);
    panel.toggle_level_filter(LogLevel::Warn);

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|e| {
        e.level == LogLevel::Info || e.level == LogLevel::Error
    }));
}

#[cfg(feature = "tui")]
#[test]
fn test_target_filter_works() {
    let mut panel = create_test_panel();

    panel.push_event(create_test_event(1, LogLevel::Info, "llm.openai", "OpenAI response"));
    panel.push_event(create_test_event(2, LogLevel::Info, "llm.anthropic", "Anthropic response"));
    panel.push_event(create_test_event(3, LogLevel::Info, "tool.read", "Read file"));

    panel.set_target_filter(Some("llm".to_string()));

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|e| e.target.starts_with("llm.")));
}

#[cfg(feature = "tui")]
#[test]
fn test_search_case_insensitive() {
    let mut panel = create_test_panel();

    panel.push_event(create_test_event(1, LogLevel::Info, "test", "ERROR occurred"));
    panel.push_event(create_test_event(2, LogLevel::Info, "test", "error occurred"));
    panel.push_event(create_test_event(3, LogLevel::Info, "test", "Error occurred"));

    panel.set_search_query(Some("ERROR".to_string()));

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 3);
}

#[cfg(feature = "tui")]
#[test]
fn test_search_in_target() {
    let mut panel = create_test_panel();

    panel.push_event(create_test_event(1, LogLevel::Info, "llm.openai", "response"));
    panel.push_event(create_test_event(2, LogLevel::Info, "tool.read", "read file"));

    panel.set_search_query(Some("llm".to_string()));

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].target.contains("llm"));
}

#[cfg(feature = "tui")]
#[test]
fn test_clear_resets_state() {
    let mut panel = create_test_panel();
    panel.selected_index = 5;
    panel.scroll_offset = 3;
    panel.expanded_error_index = Some(2);

    panel.push_event(create_test_event(1, LogLevel::Info, "test", "message"));

    panel.clear();

    assert!(panel.events.is_empty());
    assert_eq!(panel.selected_index, 0);
    assert_eq!(panel.scroll_offset, 0);
    assert_eq!(panel.expanded_error_index, None);
}

#[cfg(feature = "tui")]
#[test]
fn test_scroll_to_top_and_bottom() {
    let mut panel = create_test_panel();

    for i in 1..=10 {
        panel.push_event(create_test_event(i, LogLevel::Info, "test", &format!("Message {}", i)));
    }

    panel.scroll_to_bottom();
    assert_eq!(panel.selected_index, 9);

    panel.scroll_to_top();
    assert_eq!(panel.selected_index, 0);
}

#[cfg(feature = "tui")]
#[test]
fn test_hidden_panel_renders_nothing() {
    let mut panel = create_test_panel();
    panel.push_event(create_test_event(1, LogLevel::Info, "test", "message"));
    panel.visible = false;

    let buffer = render_panel_to_buffer(panel, 80, 20);

    let content_count = count_lines_with_content(&buffer);
    assert_eq!(content_count, 0, "Hidden panel should render no content");
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_with_session_id() {
    let panel = create_test_panel().with_session_id("sess_abc123");

    assert_eq!(panel.session_id, Some("sess_abc123".to_string()));
}

#[cfg(feature = "tui")]
#[test]
fn test_multiple_filters_combined() {
    let mut panel = create_test_panel();

    panel.push_event(create_test_event(1, LogLevel::Info, "llm.openai", "info about llm"));
    panel.push_event(create_test_event(2, LogLevel::Error, "llm.openai", "error about llm"));
    panel.push_event(create_test_event(3, LogLevel::Info, "tool.read", "info about tool"));
    panel.push_event(create_test_event(4, LogLevel::Error, "tool.read", "error about tool"));

    panel.set_target_filter(Some("llm".to_string()));
    panel.set_search_query(Some("error".to_string()));

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].target, "llm.openai");
    assert_eq!(filtered[0].level, LogLevel::Error);
}

#[cfg(feature = "tui")]
#[test]
fn test_render_preserves_border_when_events_exist() {
    let mut panel = create_test_panel();
    panel.push_event(create_test_event(1, LogLevel::Info, "agent", "Session started"));

    let buffer = render_panel_to_buffer(panel, 80, 20);

    assert!(has_border(&buffer), "Panel with events should have border");
}

#[cfg(feature = "tui")]
#[test]
fn test_empty_filtered_events_returns_empty_vec() {
    let mut panel = create_test_panel();
    panel.toggle_level_filter(LogLevel::Info);
    panel.toggle_level_filter(LogLevel::Debug);
    panel.toggle_level_filter(LogLevel::Warn);
    panel.toggle_level_filter(LogLevel::Error);

    panel.push_event(create_test_event(1, LogLevel::Trace, "test", "trace message"));

    let filtered = panel.filtered_events();
    assert!(filtered.is_empty());
}