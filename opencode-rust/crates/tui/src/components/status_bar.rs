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
    mcp_cost_usd: f64,
    connection_status: ConnectionStatus,
    activity_message: Option<String>,
}

impl StatusPopover {
    pub fn new(popover_type: StatusPopoverType, theme: Theme) -> Self {
        Self {
            popover_type,
            theme,
            token_count: 0,
            context_usage: (0, 128000),
            total_cost_usd: 0.0,
            mcp_cost_usd: 0.0,
            connection_status: ConnectionStatus::Connected,
            activity_message: None,
        }
    }

    pub fn with_data(
        mut self,
        token_count: usize,
        context_usage: (usize, usize),
        total_cost_usd: f64,
        mcp_cost_usd: f64,
    ) -> Self {
        self.token_count = token_count;
        self.context_usage = context_usage;
        self.total_cost_usd = total_cost_usd;
        self.mcp_cost_usd = mcp_cost_usd;
        self
    }

    pub fn with_connection_data(
        mut self,
        connection_status: ConnectionStatus,
        activity_message: Option<String>,
    ) -> Self {
        self.connection_status = connection_status;
        self.activity_message = activity_message;
        self
    }

    fn connection_status_label(&self) -> &'static str {
        match self.connection_status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Error => "Error",
        }
    }

    fn connection_text_lines(&self) -> Vec<String> {
        vec![
            String::new(),
            format!("Status: {}", self.connection_status_label()),
            format!(
                "Activity: {}",
                self.activity_message.as_deref().unwrap_or("Idle")
            ),
            "Logs: written to file sink".to_string(),
        ]
    }

    pub fn connection_lines_for_testing(&self) -> Vec<String> {
        self.connection_text_lines()
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

        let status_color = match self.connection_status {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Disconnected => Color::Yellow,
            ConnectionStatus::Error => Color::Red,
        };

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(self.connection_status_label(), Style::default().fg(status_color)),
            ]),
            Line::from(vec![
                Span::styled("Activity: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.activity_message.as_deref().unwrap_or("Idle")),
            ]),
            Line::from(vec![
                Span::styled("Logs: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("written to file sink"),
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
                Span::raw(format!(
                    "${:.3} (LLM) + ${:.3} (MCP)",
                    self.total_cost_usd, self.mcp_cost_usd
                )),
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
    pub mcp_cost_usd: f64,
    pub budget_limit_usd: Option<f64>,
    pub active_popover: Option<StatusPopoverType>,
    pub git_branch: Option<String>,
    pub activity_message: Option<String>,
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
            mcp_cost_usd: 0.0,
            budget_limit_usd: None,
            active_popover: None,
            git_branch: Self::detect_git_branch(),
            activity_message: None,
            theme,
        }
    }

    fn status_suffix_text(&self) -> String {
        let token_text = format!("Tokens: {}", self.token_count);
        let total_cost = self.total_cost_usd + self.mcp_cost_usd;
        let cost_text = format!("Cost: ${:.3}", total_cost);
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

        let branch_text = self
            .git_branch
            .as_ref()
            .map(|b| format!("  {} ", b))
            .unwrap_or_default();

        let activity_text = self
            .activity_message
            .as_ref()
            .map(|message| format!(" | {}", message))
            .unwrap_or_default();

        if budget_warn.is_empty() {
            format!(
                "  {} | {} | {}{} |{} ",
                token_text, cost_text, context_text, activity_text, branch_text
            )
        } else {
            format!(
                "  {} | {} | {}{} | ⚠ {} |{} ",
                token_text, cost_text, context_text, activity_text, budget_warn, branch_text
            )
        }
    }

    fn status_text(&self) -> String {
        format!(" ●{}", self.status_suffix_text())
    }

    pub fn status_text_for_testing(&self) -> String {
        self.status_text()
    }

    fn detect_git_branch() -> Option<String> {
        std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn update_usage(
        &mut self,
        tokens: usize,
        context_used: usize,
        context_total: usize,
        total_cost_usd: f64,
        mcp_cost_usd: f64,
        budget_limit_usd: Option<f64>,
    ) {
        self.token_count = tokens;
        self.context_usage = (context_used, context_total);
        self.total_cost_usd = total_cost_usd;
        self.mcp_cost_usd = mcp_cost_usd;
        self.budget_limit_usd = budget_limit_usd;
    }

    pub fn toggle_popover(&mut self, popover_type: StatusPopoverType) {
        if self.active_popover.as_ref() == Some(&popover_type) {
            self.active_popover = None;
        } else {
            self.active_popover = Some(popover_type);
        }
    }

    pub fn close_popover(&mut self) {
        self.active_popover = None;
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let connection_color = match self.connection_status {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Disconnected => Color::Yellow,
            ConnectionStatus::Error => Color::Red,
        };

        let budget_warn = self.budget_limit_usd.and_then(|limit| {
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
        });

        let status_suffix = self.status_suffix_text();

        let status_color = if budget_warn.is_none() {
            self.theme.muted_color()
        } else {
            Color::Yellow
        };

        let paragraph = Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled("●", Style::default().fg(connection_color)),
            Span::styled(status_suffix, Style::default().fg(status_color)),
        ]))
            .alignment(Alignment::Right)
            .style(Style::default());
        f.render_widget(paragraph, area);

        if let Some(ref popover_type) = self.active_popover {
            let popover = StatusPopover::new(popover_type.clone(), self.theme.clone())
                .with_data(
                    self.token_count,
                    self.context_usage,
                    self.total_cost_usd,
                    self.mcp_cost_usd,
                )
                .with_connection_data(self.connection_status, self.activity_message.clone());
            popover.draw(f, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

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

    #[test]
    fn test_status_bar_includes_activity_message() {
        let theme = crate::theme::Theme::default();
        let mut bar = StatusBar::new(theme);
        bar.activity_message = Some("⏳ Validating OpenAI".to_string());

        assert!(
            bar.status_text_for_testing().contains("⏳ Validating OpenAI"),
            "status bar text should include the current activity message"
        );
    }

    #[test]
    fn test_connection_popover_shows_truthful_activity_details() {
        let theme = crate::theme::Theme::default();
        let popover = StatusPopover::new(StatusPopoverType::Connection, theme)
            .with_connection_data(
                ConnectionStatus::Disconnected,
                Some("🌐 Browser auth: Google".to_string()),
            );

        let lines = popover.connection_lines_for_testing();
        let rendered = lines.join("\n");

        assert!(rendered.contains("Status: Disconnected"));
        assert!(rendered.contains("Activity: 🌐 Browser auth: Google"));
    }

    #[test]
    fn test_connection_popover_shows_idle_when_no_activity_exists() {
        let theme = crate::theme::Theme::default();
        let popover = StatusPopover::new(StatusPopoverType::Connection, theme)
            .with_connection_data(ConnectionStatus::Connected, None);

        let lines = popover.connection_lines_for_testing();
        let rendered = lines.join("\n");

        assert!(rendered.contains("Status: Connected"));
        assert!(rendered.contains("Activity: Idle"));
    }

    #[test]
    fn test_status_bar_renders_disconnected_indicator_in_yellow() {
        let theme = crate::theme::Theme::default();
        let mut bar = StatusBar::new(theme);
        bar.connection_status = ConnectionStatus::Disconnected;

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| bar.draw(f, ratatui::layout::Rect::new(0, 0, 80, 1)))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let indicator = buffer
            .content
            .iter()
            .find(|cell| cell.symbol() == "●")
            .expect("status bar should render a connection indicator");

        assert_eq!(indicator.fg, Color::Yellow);
    }

    #[test]
    fn test_status_bar_renders_error_indicator_in_red() {
        let theme = crate::theme::Theme::default();
        let mut bar = StatusBar::new(theme);
        bar.connection_status = ConnectionStatus::Error;

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| bar.draw(f, ratatui::layout::Rect::new(0, 0, 80, 1)))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let indicator = buffer
            .content
            .iter()
            .find(|cell| cell.symbol() == "●")
            .expect("status bar should render a connection indicator");

        assert_eq!(indicator.fg, Color::Red);
    }

    #[test]
    fn test_status_bar_renders_connected_indicator_in_green() {
        let theme = crate::theme::Theme::default();
        let mut bar = StatusBar::new(theme);
        bar.connection_status = ConnectionStatus::Connected;

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| bar.draw(f, ratatui::layout::Rect::new(0, 0, 80, 1)))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let indicator = buffer
            .content
            .iter()
            .find(|cell| cell.symbol() == "●")
            .expect("status bar should render a connection indicator");

        assert_eq!(indicator.fg, Color::Green);
    }
}
