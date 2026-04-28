use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub content: String,
    pub is_error: bool,
    pub timestamp: String,
}

impl TerminalLine {
    pub fn new(content: impl Into<String>, is_error: bool) -> Self {
        Self {
            content: content.into(),
            is_error,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        }
    }
}

pub struct TerminalPanel {
    pub lines: VecDeque<TerminalLine>,
    pub current_command: String,
    pub max_lines: usize,
    pub theme: Theme,
    pub scroll_offset: usize,
}

impl TerminalPanel {
    pub fn new(theme: Theme) -> Self {
        Self {
            lines: VecDeque::with_capacity(1000),
            current_command: String::new(),
            max_lines: 1000,
            theme,
            scroll_offset: 0,
        }
    }

    pub fn add_line(&mut self, content: impl Into<String>, is_error: bool) {
        let line = TerminalLine::new(content, is_error);

        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }

        self.lines.push_back(line);
        self.scroll_to_bottom();
    }

    pub fn add_stdout(&mut self, content: impl Into<String>) {
        for line in content.into().lines() {
            self.add_line(line, false);
        }
    }

    pub fn add_stderr(&mut self, content: impl Into<String>) {
        for line in content.into().lines() {
            self.add_line(line, true);
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = (self.scroll_offset + amount).min(self.lines.len().saturating_sub(1));
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Terminal (Ctrl+~ to toggle)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));

        let inner_area = block.inner(area);
        f.render_widget(block, area);

        let visible_height = inner_area.height as usize;
        let total_lines = self.lines.len();

        let start_idx = if total_lines > visible_height {
            total_lines.saturating_sub(visible_height + self.scroll_offset)
        } else {
            0
        };

        let end_idx = (start_idx + visible_height).min(total_lines);

        let lines: Vec<Line> = self
            .lines
            .range(start_idx..end_idx)
            .map(|line| {
                let style = if line.is_error {
                    Style::default().fg(self.theme.error_color())
                } else {
                    Style::default()
                };

                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", line.timestamp),
                        Style::default().fg(self.theme.muted_color()),
                    ),
                    Span::styled(&line.content, style),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines);
        f.render_widget(paragraph, inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_panel_new() {
        let theme = crate::theme::Theme::default();
        let panel = TerminalPanel::new(theme);
        assert!(panel.lines.is_empty());
        assert!(panel.current_command.is_empty());
    }

    #[test]
    fn test_terminal_panel_add_line() {
        let theme = crate::theme::Theme::default();
        let mut panel = TerminalPanel::new(theme);
        panel.add_line("Test output", false);
        assert_eq!(panel.lines.len(), 1);
    }

    #[test]
    fn test_terminal_panel_max_lines() {
        let theme = crate::theme::Theme::default();
        let mut panel = TerminalPanel::new(theme);
        panel.max_lines = 5;

        for i in 0..10 {
            panel.add_line(format!("Line {}", i), false);
        }

        assert_eq!(panel.lines.len(), 5);
    }
}
