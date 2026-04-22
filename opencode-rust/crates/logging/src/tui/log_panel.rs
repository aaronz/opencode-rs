#[cfg(feature = "tui")]
use ratatui::{Frame, Terminal, backend::Backend};

#[cfg(feature = "tui")]
use crate::event::{LogEvent, LogLevel};
#[cfg(feature = "tui")]
use crate::config::TuiLogPosition;

#[cfg(feature = "tui")]
pub struct LogPanel {
    pub events: Vec<LogEvent>,
    pub level_filter: Option<LogLevel>,
    pub target_filter: Option<String>,
    pub search_query: Option<String>,
    pub auto_scroll: bool,
    pub position: TuiLogPosition,
}

#[cfg(feature = "tui")]
impl LogPanel {
    pub fn new(position: TuiLogPosition) -> Self {
        Self {
            events: Vec::new(),
            level_filter: None,
            target_filter: None,
            search_query: None,
            auto_scroll: true,
            position,
        }
    }

    pub fn push_event(&mut self, event: LogEvent) {
        if let Some(ref level) = self.level_filter {
            if event.level != *level {
                return;
            }
        }

        if let Some(ref target) = self.target_filter {
            if !event.target.contains(target) {
                return;
            }
        }

        if let Some(ref query) = self.search_query {
            if !event.message.contains(query) && !event.target.contains(query) {
                return;
            }
        }

        self.events.push(event);
    }

    pub fn set_level_filter(&mut self, level: Option<LogLevel>) {
        self.level_filter = level;
    }

    pub fn set_target_filter(&mut self, target: Option<String>) {
        self.target_filter = target;
    }

    pub fn set_search_query(&mut self, query: Option<String>) {
        self.search_query = query;
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn filtered_events(&self) -> Vec<&LogEvent> {
        self.events.iter().collect()
    }
}

#[cfg(feature = "tui")]
impl Default for LogPanel {
    fn default() -> Self {
        Self::new(TuiLogPosition::Bottom)
    }
}

#[cfg(test)]
#[cfg(feature = "tui")]
mod tests {
    use super::*;

    #[test]
    fn test_log_panel_creation() {
        let panel = LogPanel::new(TuiLogPosition::Bottom);
        assert!(panel.events.is_empty());
        assert!(panel.auto_scroll);
    }

    #[test]
    #[cfg(feature = "tui")]
    fn test_log_panel_filter_by_level() {
        let mut panel = LogPanel::new(TuiLogPosition::Bottom);
        panel.set_level_filter(Some(LogLevel::Error));

        panel.push_event(LogEvent::new(1, LogLevel::Info, "test", "info message"));
        panel.push_event(LogEvent::new(2, LogLevel::Error, "test", "error message"));

        assert_eq!(panel.events.len(), 1);
        assert_eq!(panel.events[0].level, LogLevel::Error);
    }
}