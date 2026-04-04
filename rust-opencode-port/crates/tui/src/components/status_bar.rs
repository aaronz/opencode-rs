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
    token_count: usize,
    context_usage: (usize, usize),
    total_cost_usd: f64,
}

impl StatusPopover {
    pub fn new(popover_type: StatusPopoverType, theme: Theme) -> Self {
        Self {
            popover_type,
            theme,
            token_count: 0,
            context_usage: (0, 128000),
            total_cost_usd: 0.0,
        }
    }

    pub fn with_data(
        mut self,
        token_count: usize,
        context_usage: (usize, usize),
        total_cost_usd: f64,
    ) -> Self {
        self.token_count = token_count;
        self.context_usage = context_usage;
        self.total_cost_usd = total_cost_usd;
        self
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

        let usage_pct = if self.context_usage.1 > 0 {
            (self.context_usage.0 as f64 / self.context_usage.1 as f64) * 100.0
        } else {
            0.0
        };

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", self.token_count)),
            ]),
            Line::from(vec![
                Span::styled("Context: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!(
                    "{} / {} ({:.0}%)",
                    self.context_usage.0, self.context_usage.1, usage_pct
                )),
            ]),
            Line::from(vec![
                Span::styled("Est. Cost: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("${:.3}", self.total_cost_usd)),
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
    pub total_cost_usd: f64,
    pub budget_limit_usd: Option<f64>,
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
            token_count: 0,
            context_usage: (0, 128000),
            total_cost_usd: 0.0,
            budget_limit_usd: None,
            active_popover: None,
            theme,
        }
    }

    pub fn update_usage(
        &mut self,
        tokens: usize,
        context_used: usize,
        context_total: usize,
        total_cost_usd: f64,
        budget_limit_usd: Option<f64>,
    ) {
        self.token_count = tokens;
        self.context_usage = (context_used, context_total);
        self.total_cost_usd = total_cost_usd;
        self.budget_limit_usd = budget_limit_usd;
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
        let token_text = format!("Tokens: {}", self.token_count);
        let cost_text = format!("Cost: ${:.3}", self.total_cost_usd);
        let context_text = format!("Ctx: {}/{}", self.context_usage.0, self.context_usage.1);

        let budget_warn = self
            .budget_limit_usd
            .and_then(|limit| {
                if limit > 0.0 {
                    let usage = self.total_cost_usd / limit;
                    if usage >= 0.8 {
                        Some(format!("Budget {:.0}%", usage * 100.0))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let status_text = if budget_warn.is_empty() {
            format!(
                " {}  {} | {} | {} ",
                connection_indicator, token_text, cost_text, context_text
            )
        } else {
            format!(
                " {}  {} | {} | {} | ⚠ {} ",
                connection_indicator, token_text, cost_text, context_text, budget_warn
            )
        };

        let status_color = if budget_warn.is_empty() {
            self.theme.muted_color()
        } else {
            Color::Yellow
        };

        let paragraph = Paragraph::new(status_text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(status_color));
        f.render_widget(paragraph, area);

        if let Some(ref popover_type) = self.active_popover {
            let popover = StatusPopover::new(popover_type.clone(), self.theme.clone()).with_data(
                self.token_count,
                self.context_usage,
                self.total_cost_usd,
            );
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
