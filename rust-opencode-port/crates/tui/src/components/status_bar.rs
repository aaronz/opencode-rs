use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum StatusPopoverType {
    Connection,
    Tokens,
    Context,
}

pub struct StatusPopover {
    popover_type: StatusPopoverType,
    theme: Theme,
}

impl StatusPopover {
    pub fn new(popover_type: StatusPopoverType, theme: Theme) -> Self {
        Self {
            popover_type,
            theme,
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let (width, height) = match self.popover_type {
            StatusPopoverType::Connection => (40, 8),
            StatusPopoverType::Tokens => (35, 6),
            StatusPopoverType::Context => (40, 7),
        };

        let x = area.x.saturating_sub(width / 2);
        let y = area.y.saturating_sub(height + 1);
        let popover_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, popover_area);

        match self.popover_type {
            StatusPopoverType::Connection => self.draw_connection(f, popover_area),
            StatusPopoverType::Tokens => self.draw_tokens(f, popover_area),
            StatusPopoverType::Context => self.draw_context(f, popover_area),
        }
    }

    fn draw_connection(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Connection Status")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("OpenAI: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("Connected", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Anthropic: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("Connected", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Ollama: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("Disconnected", Style::default().fg(Color::Yellow)),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    fn draw_tokens(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Token Usage")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Current: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("1,234 tokens"),
            ]),
            Line::from(vec![
                Span::styled("Limit: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("128,000 tokens"),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    fn draw_context(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Context Usage")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Window: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("12,345 / 128,000"),
            ]),
            Line::from(vec![
                Span::styled("Compacted: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("3 messages"),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }
}

pub struct StatusBar {
    pub connection_status: ConnectionStatus,
    pub token_count: usize,
    pub context_usage: (usize, usize),
    pub active_popover: Option<StatusPopoverType>,
    theme: Theme,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

impl StatusBar {
    pub fn new(theme: Theme) -> Self {
        Self {
            connection_status: ConnectionStatus::Connected,
            token_count: 1234,
            context_usage: (12345, 128000),
            active_popover: None,
            theme,
        }
    }

    pub fn toggle_popover(&mut self, popover_type: StatusPopoverType) {
        if self.active_popover.as_ref() == Some(&&popover_type) {
            self.active_popover = None;
        } else {
            self.active_popover = Some(popover_type);
        }
    }

    pub fn close_popover(&mut self) {
        self.active_popover = None;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let _connection_color = match self.connection_status {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Disconnected => Color::Yellow,
            ConnectionStatus::Error => Color::Red,
        };

        let connection_indicator = "●";
        let token_text = format!("{} tokens", self.token_count);
        let context_text = format!("{}/{} context", self.context_usage.0, self.context_usage.1);

        let status_text = format!(
            " {}  {}  {} ",
            connection_indicator, token_text, context_text
        );

        let paragraph = Paragraph::new(status_text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(self.theme.muted_color()));
        f.render_widget(paragraph, area);

        if let Some(ref popover_type) = self.active_popover {
            let popover = StatusPopover::new(popover_type.clone(), self.theme.clone());
            popover.draw(f, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_new() {
        let theme = crate::theme::Theme::default();
        let bar = StatusBar::new(theme);
        assert_eq!(bar.connection_status, ConnectionStatus::Connected);
        assert!(bar.active_popover.is_none());
    }

    #[test]
    fn test_status_bar_toggle_popover() {
        let theme = crate::theme::Theme::default();
        let mut bar = StatusBar::new(theme);

        bar.toggle_popover(StatusPopoverType::Connection);
        assert!(matches!(
            bar.active_popover,
            Some(StatusPopoverType::Connection)
        ));

        bar.toggle_popover(StatusPopoverType::Connection);
        assert!(bar.active_popover.is_none());
    }
}
