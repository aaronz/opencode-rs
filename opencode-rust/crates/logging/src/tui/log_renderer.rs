#[cfg(feature = "tui")]
use ratatui::style::{Color, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Line, Span};
#[cfg(feature = "tui")]
use ratatui::Frame;

#[cfg(feature = "tui")]
use crate::event::LogLevel;
#[cfg(feature = "tui")]
use crate::tui::log_panel::LogPanel;

#[cfg(feature = "tui")]
pub struct LogRenderer;

#[cfg(feature = "tui")]
impl LogRenderer {
    pub fn render_panel(frame: &mut Frame<'_>, panel: &LogPanel, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, List, ListItem};

        if !panel.visible {
            return;
        }

        let title = format!("Logs ({})", panel.filtered_events().len());

        let block = Block::default().title(title.as_str()).borders(Borders::ALL);

        let items: Vec<ListItem> = panel
            .filtered_events()
            .iter()
            .enumerate()
            .map(|(idx, e)| {
                let is_selected = idx == panel.selected_index;
                let level_str = match e.level {
                    LogLevel::Trace => "TRACE",
                    LogLevel::Debug => "DEBUG",
                    LogLevel::Info => "INFO ",
                    LogLevel::Warn => "WARN ",
                    LogLevel::Error => "ERROR",
                };

                let level_color = match e.level {
                    LogLevel::Trace => Color::DarkGray,
                    LogLevel::Debug => Color::Blue,
                    LogLevel::Info => Color::Green,
                    LogLevel::Warn => Color::Yellow,
                    LogLevel::Error => Color::Red,
                };

                let time_str = e.timestamp.format("%H:%M:%S").to_string();
                let line = Line::from(vec![
                    Span::raw(format!("{} ", time_str)),
                    Span::styled(level_str, Style::default().fg(level_color)),
                    Span::raw(format!(" {}  {}", e.target, e.message)),
                ]);

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list = List::new(items).block(block);

        frame.render_widget(list, area);
    }

    #[cfg(feature = "tui")]
    pub fn render_empty_state(frame: &mut Frame<'_>, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};

        let block = Block::default().title("Logs (0)").borders(Borders::ALL);

        let para = Paragraph::new("No logs yet")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);

        frame.render_widget(para, area);
    }
}

#[cfg(test)]
#[cfg(feature = "tui")]
mod tests {
    #[test]
    fn test_renderer_imports() {
        assert!(true);
    }
}
