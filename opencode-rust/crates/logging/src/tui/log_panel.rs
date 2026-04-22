#[cfg(feature = "tui")]
use {
    ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
    },
    std::collections::HashSet,
};

#[cfg(feature = "tui")]
use crate::config::TuiLogPosition;
#[cfg(feature = "tui")]
use crate::event::{ErrorContext, LogEvent, LogLevel};

#[cfg(feature = "tui")]
const LEVEL_COLORS: &[(LogLevel, Color)] = &[
    (LogLevel::Trace, Color::DarkGray),
    (LogLevel::Debug, Color::Blue),
    (LogLevel::Info, Color::Green),
    (LogLevel::Warn, Color::Yellow),
    (LogLevel::Error, Color::Red),
];

#[cfg(feature = "tui")]
pub struct LogPanel {
    pub events: Vec<LogEvent>,
    pub session_id: Option<String>,
    pub level_filter: Option<LogLevel>,
    pub target_filter: Option<String>,
    pub search_query: Option<String>,
    pub auto_scroll: bool,
    pub position: TuiLogPosition,
    pub visible: bool,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub expanded_error_index: Option<usize>,
    pub active_level_filters: HashSet<LogLevel>,
    pub component_filter_active: bool,
    pub component_filter_value: String,
}

#[cfg(feature = "tui")]
impl LogPanel {
    pub fn new(position: TuiLogPosition) -> Self {
        Self {
            events: Vec::new(),
            session_id: None,
            level_filter: None,
            target_filter: None,
            search_query: None,
            auto_scroll: true,
            position,
            visible: true,
            selected_index: 0,
            scroll_offset: 0,
            expanded_error_index: None,
            active_level_filters: HashSet::from([
                LogLevel::Info,
                LogLevel::Debug,
                LogLevel::Warn,
                LogLevel::Error,
            ]),
            component_filter_active: false,
            component_filter_value: String::new(),
        }
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn push_event(&mut self, event: LogEvent) {
        self.events.push(event);
        if self.auto_scroll && !self.events.is_empty() {
            self.selected_index = self.events.len() - 1;
            self.scroll_to_bottom();
        }
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

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.expanded_error_index = None;
    }

    pub fn scroll_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn scroll_down(&mut self) {
        let filtered = self.filtered_events();
        if self.selected_index < filtered.len().saturating_sub(1) {
            self.selected_index += 1;
        }
        let visible_height = 10;
        if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        let filtered = self.filtered_events();
        if !filtered.is_empty() {
            self.selected_index = filtered.len() - 1;
            let visible_height = 10;
            self.scroll_offset = self
                .selected_index
                .saturating_sub(visible_height)
                .saturating_sub(1);
        }
    }

    pub fn toggle_level_filter(&mut self, level: LogLevel) {
        if self.active_level_filters.contains(&level) {
            self.active_level_filters.remove(&level);
        } else {
            self.active_level_filters.insert(level);
        }
    }

    pub fn is_level_filter_active(&self, level: LogLevel) -> bool {
        self.active_level_filters.contains(&level)
    }

    pub fn toggle_expanded_error(&mut self, index: usize) {
        if self.expanded_error_index == Some(index) {
            self.expanded_error_index = None;
        } else {
            self.expanded_error_index = Some(index);
        }
    }

    pub fn filtered_events(&self) -> Vec<&LogEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Some(ref level) = self.level_filter {
                    if e.level != *level {
                        return false;
                    }
                } else if !self.active_level_filters.contains(&e.level) {
                    return false;
                }
                if let Some(ref target) = self.target_filter {
                    if !e.target.contains(target) {
                        return false;
                    }
                }
                if let Some(ref query) = self.search_query {
                    let q = query.to_lowercase();
                    if !e.message.to_lowercase().contains(&q)
                        && !e.target.to_lowercase().contains(&q)
                    {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    fn visible_count(&self) -> usize {
        10
    }

    fn get_level_color(level: LogLevel) -> Color {
        LEVEL_COLORS
            .iter()
            .find(|(l, _)| *l == level)
            .map(|(_, c)| *c)
            .unwrap_or(Color::White)
    }

    fn get_level_str(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warn => "WARN ",
            LogLevel::Error => "ERROR",
        }
    }



    fn extract_error_context(&self, event: &LogEvent) -> Option<ErrorContext> {
        event
            .fields
            .extra
            .get("error_context")
            .and_then(|v| serde_json::from_value::<ErrorContext>(v.clone()).ok())
    }
}

#[cfg(feature = "tui")]
impl Default for LogPanel {
    fn default() -> Self {
        Self::new(TuiLogPosition::Bottom)
    }
}

#[cfg(feature = "tui")]
impl Widget for LogPanel {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.visible {
            return;
        }

        let [header_area, list_area] = Layout::new(
            Direction::Vertical,
            [Constraint::Length(1), Constraint::Min(1)],
        )
        .areas(area);

        self.render_header_stateful(header_area, buf);
        self.render_list_stateful(list_area, buf);
    }
}

#[cfg(feature = "tui")]
impl LogPanel {
    fn render_header_stateful(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let widget = self.render_header_widget();
        widget.render(area, buf);
    }

    fn render_header_widget(&self) -> impl Widget {
        let levels = [LogLevel::Info, LogLevel::Debug, LogLevel::Warn, LogLevel::Error];

        let session_text = if let Some(ref sid) = self.session_id {
            format!("Session: {}", sid)
        } else {
            "Session: -".to_string()
        };

        let count = self.filtered_events().len();
        let count_text = format!("Logs ({})", count);

        let auto_scroll_str = if self.auto_scroll {
            "[AutoScroll ON ]"
        } else {
            "[AutoScroll OFF]"
        };
        let auto_scroll_style = if self.auto_scroll {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let clear_style = Style::default().fg(Color::Yellow);

        let level_spans: Vec<Span> = levels
            .iter()
            .map(|&level| {
                let active = self.active_level_filters.contains(&level);
                let style = if active {
                    Style::default()
                        .fg(Self::get_level_color(level))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Span::styled(
                    format!("[{:?} ", Self::get_level_str(level).trim()),
                    style,
                )
            })
            .collect();

        let all_content: Vec<Span> = vec![
            vec![Span::raw(session_text.to_string()), Span::raw("  ")],
            vec![Span::raw(count_text.to_string()), Span::raw("  ")],
            level_spans,
            vec![Span::raw("  ")],
            vec![Span::styled(auto_scroll_str, auto_scroll_style)],
            vec![Span::raw("  ")],
            vec![Span::styled("[Clear]", clear_style)],
        ]
        .into_iter()
        .flatten()
        .collect();

        Paragraph::new(Line::from(all_content))
    }

    fn render_list_stateful(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let filtered = self.filtered_events();

        if filtered.is_empty() {
            let empty_msg = Paragraph::new("No logs yet")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().title("Logs").borders(Borders::ALL));
            empty_msg.render(area, buf);
            return;
        }

        let visible_count = self.visible_count();
        let start = self.scroll_offset.min(filtered.len().saturating_sub(1));
        let end = (start + visible_count).min(filtered.len());

        let items: Vec<ListItem> = filtered[start..end]
            .iter()
            .enumerate()
            .map(|(i, event)| {
                let global_idx = start + i;
                let is_selected = global_idx == self.selected_index;
                let is_expanded = self.expanded_error_index == Some(global_idx)
                    && event.level == LogLevel::Error;

                let level_str = Self::get_level_str(event.level);
                let level_color = Self::get_level_color(event.level);
                let time_str = event.timestamp.format("%H:%M:%S").to_string();

                let mut spans = vec![
                    Span::raw(format!("{} ", time_str)),
                    Span::styled(level_str, Style::default().fg(level_color)),
                    Span::raw(format!(" {}  {}", event.target, event.message)),
                ];

                if event.level == LogLevel::Error {
                    spans.push(Span::styled(
                        " [click]",
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                if is_expanded {
                    if let Some(ref error_ctx) = self.extract_error_context(event) {
                        spans.push(Span::raw("\n"));
                        for frame in &error_ctx.stack {
                            spans.push(Span::raw(format!(
                                "  at {}:{} in {}\n",
                                frame.file, frame.line, frame.function
                            )));
                        }
                        if !error_ctx.cause_chain.is_empty() {
                            spans.push(Span::raw("Caused by: "));
                            for cause in &error_ctx.cause_chain {
                                spans.push(Span::raw(format!(
                                    "{}: {}; ",
                                    cause.code, cause.message
                                )));
                            }
                            spans.push(Span::raw("\n"));
                        }
                    }
                }

                let line = Line::from(spans);
                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("Logs").borders(Borders::ALL));

        list.render(area, buf);
    }
}

#[cfg(test)]
#[cfg(feature = "tui")]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn create_test_panel() -> LogPanel {
        LogPanel::new(TuiLogPosition::Bottom)
    }

    fn create_test_event(seq: u64, level: LogLevel, target: &str, message: &str) -> LogEvent {
        LogEvent::new(seq, level, target, message)
    }

    #[test]
    fn test_log_panel_creation() {
        let panel = create_test_panel();
        assert!(panel.events.is_empty());
        assert!(panel.auto_scroll);
        assert!(panel.visible);
    }

    #[test]
    fn test_log_panel_filter_by_level() {
        let mut panel = create_test_panel();
        panel.set_level_filter(Some(LogLevel::Error));

        panel.push_event(create_test_event(1, LogLevel::Info, "test", "info message"));
        panel.push_event(create_test_event(2, LogLevel::Error, "test", "error message"));

        assert_eq!(panel.events.len(), 2);
        assert_eq!(panel.filtered_events().len(), 1);
    }

    #[test]
    fn test_toggle_level_filter() {
        let mut panel = create_test_panel();

        assert!(panel.is_level_filter_active(LogLevel::Info));
        assert!(panel.is_level_filter_active(LogLevel::Debug));

        panel.toggle_level_filter(LogLevel::Info);
        assert!(!panel.is_level_filter_active(LogLevel::Info));
        assert!(panel.is_level_filter_active(LogLevel::Debug));

        panel.toggle_level_filter(LogLevel::Info);
        assert!(panel.is_level_filter_active(LogLevel::Info));
    }

    #[test]
    fn test_toggle_auto_scroll() {
        let mut panel = create_test_panel();
        assert!(panel.auto_scroll);

        panel.toggle_auto_scroll();
        assert!(!panel.auto_scroll);

        panel.toggle_auto_scroll();
        assert!(panel.auto_scroll);
    }

    #[test]
    fn test_toggle_visibility() {
        let mut panel = create_test_panel();
        assert!(panel.visible);

        panel.toggle_visibility();
        assert!(!panel.visible);

        panel.toggle_visibility();
        assert!(panel.visible);
    }

    #[test]
    fn test_scroll_up() {
        let mut panel = create_test_panel();
        panel.push_event(create_test_event(1, LogLevel::Info, "test", "msg1"));
        panel.push_event(create_test_event(2, LogLevel::Info, "test", "msg2"));
        panel.push_event(create_test_event(3, LogLevel::Info, "test", "msg3"));
        panel.selected_index = 2;

        panel.scroll_up();
        assert_eq!(panel.selected_index, 1);

        panel.scroll_up();
        assert_eq!(panel.selected_index, 0);

        panel.scroll_up();
        assert_eq!(panel.selected_index, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut panel = create_test_panel();
        panel.push_event(create_test_event(1, LogLevel::Info, "test", "msg1"));
        panel.push_event(create_test_event(2, LogLevel::Info, "test", "msg2"));
        panel.push_event(create_test_event(3, LogLevel::Info, "test", "msg3"));
        panel.selected_index = 0;

        panel.scroll_down();
        assert_eq!(panel.selected_index, 1);

        panel.scroll_down();
        assert_eq!(panel.selected_index, 2);

        panel.scroll_down();
        assert_eq!(panel.selected_index, 2);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut panel = create_test_panel();
        for i in 1..=5 {
            panel.push_event(create_test_event(i, LogLevel::Info, "test", &format!("msg{}", i)));
        }

        panel.scroll_to_bottom();
        assert_eq!(panel.selected_index, 4);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut panel = create_test_panel();
        for i in 1..=5 {
            panel.push_event(create_test_event(i, LogLevel::Info, "test", &format!("msg{}", i)));
        }
        panel.selected_index = 4;

        panel.scroll_to_top();
        assert_eq!(panel.selected_index, 0);
    }

    #[test]
    fn test_search_filters_logs() {
        let mut panel = create_test_panel();
        panel.set_search_query(Some("error".to_string()));

        panel.push_event(create_test_event(1, LogLevel::Info, "test", "normal message"));
        panel.push_event(create_test_event(2, LogLevel::Error, "test", "error occurred"));

        assert_eq!(panel.filtered_events().len(), 1);
        assert!(panel.filtered_events()[0].message.contains("error"));
    }

    #[test]
    fn test_target_filter() {
        let mut panel = create_test_panel();
        panel.set_target_filter(Some("llm".to_string()));

        panel.push_event(create_test_event(1, LogLevel::Info, "llm.openai", "response"));
        panel.push_event(create_test_event(2, LogLevel::Info, "tool.read", "read file"));

        assert_eq!(panel.filtered_events().len(), 1);
        assert_eq!(panel.filtered_events()[0].target, "llm.openai");
    }

    #[test]
    fn test_toggle_expanded_error() {
        let mut panel = create_test_panel();
        panel.push_event(create_test_event(1, LogLevel::Error, "test", "error"));
        assert_eq!(panel.expanded_error_index, None);

        panel.toggle_expanded_error(0);
        assert_eq!(panel.expanded_error_index, Some(0));

        panel.toggle_expanded_error(0);
        assert_eq!(panel.expanded_error_index, None);
    }

    #[test]
    fn test_render_with_events() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut panel = create_test_panel();
        panel.push_event(create_test_event(1, LogLevel::Info, "agent", "Session started"));
        panel.push_event(create_test_event(2, LogLevel::Debug, "tool.read", "Read 100 lines"));

        terminal
            .draw(|f| {
                panel.render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_border = buffer.content.iter().any(|cell| cell.symbol() == "─");
        assert!(has_border, "Panel should render with border");
    }

    #[test]
    fn test_render_empty_state() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let panel = create_test_panel();

        terminal
            .draw(|f| {
                panel.render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_border = buffer.content.iter().any(|cell| cell.symbol() == "─");
        assert!(has_border, "Empty panel should render with border");
    }

    #[test]
    fn test_panel_respects_visibility() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut panel = create_test_panel();
        panel.push_event(create_test_event(1, LogLevel::Info, "test", "message"));
        panel.visible = false;

        terminal
            .draw(|f| {
                panel.render(f.area(), f.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(!has_content, "Hidden panel should not render content");
    }

    #[test]
    fn test_log_panel_with_session_id() {
        let panel = create_test_panel().with_session_id("sess_abc123");
        assert_eq!(panel.session_id, Some("sess_abc123".to_string()));
    }

    #[test]
    fn test_active_level_filters_default() {
        let panel = create_test_panel();
        assert!(panel.active_level_filters.contains(&LogLevel::Info));
        assert!(panel.active_level_filters.contains(&LogLevel::Debug));
        assert!(panel.active_level_filters.contains(&LogLevel::Warn));
        assert!(panel.active_level_filters.contains(&LogLevel::Error));
        assert!(!panel.active_level_filters.contains(&LogLevel::Trace));
    }

    #[test]
    fn test_multiple_level_filters_active() {
        let mut panel = create_test_panel();

        panel.toggle_level_filter(LogLevel::Debug);
        panel.toggle_level_filter(LogLevel::Warn);

        assert!(panel.active_level_filters.contains(&LogLevel::Info));
        assert!(!panel.active_level_filters.contains(&LogLevel::Debug));
        assert!(!panel.active_level_filters.contains(&LogLevel::Warn));
        assert!(panel.active_level_filters.contains(&LogLevel::Error));
    }

    #[test]
    fn test_log_panel_clear() {
        let mut panel = create_test_panel();
        panel.push_event(create_test_event(1, LogLevel::Info, "test", "msg1"));
        panel.push_event(create_test_event(2, LogLevel::Info, "test", "msg2"));
        assert_eq!(panel.events.len(), 2);

        panel.clear();
        assert!(panel.events.is_empty());
        assert_eq!(panel.selected_index, 0);
        assert_eq!(panel.scroll_offset, 0);
        assert_eq!(panel.expanded_error_index, None);
    }
}