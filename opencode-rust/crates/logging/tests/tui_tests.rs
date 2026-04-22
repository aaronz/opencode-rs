#[cfg(feature = "tui")]
use opencode_logging::config::TuiLogPosition;
#[cfg(feature = "tui")]
use opencode_logging::event::{LogEvent, LogLevel};
#[cfg(feature = "tui")]
use opencode_logging::tui::log_panel::LogPanel;

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_creation() {
    let panel = LogPanel::new(TuiLogPosition::Bottom);
    assert!(panel.events.is_empty());
    assert!(panel.auto_scroll);
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_push_event() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    panel.push_event(LogEvent::new(1, LogLevel::Info, "test", "info message"));
    assert_eq!(panel.events.len(), 1);
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_filter_by_level() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    panel.set_level_filter(Some(LogLevel::Error));

    panel.push_event(LogEvent::new(1, LogLevel::Info, "test", "info message"));
    panel.push_event(LogEvent::new(2, LogLevel::Error, "test", "error message"));

    assert_eq!(panel.events.len(), 1);
    assert_eq!(panel.events[0].level, LogLevel::Error);
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_filter_by_target() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    panel.set_target_filter(Some("llm".to_string()));

    panel.push_event(LogEvent::new(1, LogLevel::Info, "llm.openai", "response"));
    panel.push_event(LogEvent::new(2, LogLevel::Info, "tool.read", "read file"));

    assert_eq!(panel.events.len(), 1);
    assert_eq!(panel.events[0].target, "llm.openai");
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_search() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    panel.set_search_query(Some("error".to_string()));

    panel.push_event(LogEvent::new(1, LogLevel::Info, "test", "normal message"));
    panel.push_event(LogEvent::new(2, LogLevel::Error, "test", "error occurred"));

    assert_eq!(panel.events.len(), 1);
    assert!(panel.events[0].message.contains("error"));
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_auto_scroll_toggle() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    assert!(panel.auto_scroll);

    panel.toggle_auto_scroll();
    assert!(!panel.auto_scroll);

    panel.toggle_auto_scroll();
    assert!(panel.auto_scroll);
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_clear() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    panel.push_event(LogEvent::new(1, LogLevel::Info, "test", "message"));
    assert_eq!(panel.events.len(), 1);

    panel.clear();
    assert!(panel.events.is_empty());
}

#[cfg(feature = "tui")]
#[test]
fn test_log_panel_filtered_events() {
    let mut panel = LogPanel::new(TuiLogPosition::Bottom);
    panel.push_event(LogEvent::new(1, LogLevel::Info, "test", "info1"));
    panel.push_event(LogEvent::new(2, LogLevel::Debug, "test", "debug1"));
    panel.push_event(LogEvent::new(3, LogLevel::Error, "test", "error1"));

    let filtered = panel.filtered_events();
    assert_eq!(filtered.len(), 3);
}
