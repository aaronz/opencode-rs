#[cfg(feature = "tui")]
use ratatui::{Frame, Terminal, backend::Backend, widgets::Widget};
#[cfg(feature = "tui")]
use ratatui::style::{Color, Style};
#[cfg(feature = "tui")]
use ratatui::text::{Text, Span, Line};

#[cfg(feature = "tui")]
use crate::event::{LogEvent, LogLevel};
#[cfg(feature = "tui")]
use crate::tui::log_panel::LogPanel;

#[cfg(feature = "tui")]
pub struct LogRenderer;

#[cfg(feature = "tui")]
impl LogRenderer {
    pub fn render_panel(frame: &mut Frame<'_>, panel: &LogPanel, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};

        let title = format!("Logs ({})", panel.events.len());

        let block = Block::default()
            .title(title.as_str())
            .borders(Borders::ALL);

        let items: Vec<ListItem> = panel
            .filtered_events()
            .iter()
            .map(|e| {
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

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(block);

        frame.render_widget(list, area);
    }

    #[cfg(feature = "tui")]
    pub fn render_empty_state(frame: &mut Frame<'_>, area: ratatui::layout::Rect) {
        use ratatui::widgets::{Block, Borders, Paragraph};

        let block = Block::default()
            .title("Logs (0)")
            .borders(Borders::ALL);

        let para = Paragraph::new("No logs yet")
            .block(block);

        frame.render_widget(para, area);
    }
}

#[cfg(test)]
#[cfg(feature = "tui")]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_imports() {
        // Just verify the module compiles with tui feature
        assert!(true);
    }
}