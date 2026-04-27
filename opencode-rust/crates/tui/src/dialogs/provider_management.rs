use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub status: ProviderStatus,
    pub api_key_set: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderStatus {
    Connected,
    Disconnected,
    Error,
}

pub struct ProviderManagementDialog {
    providers: Vec<ProviderInfo>,
    selected_index: usize,
    scroll_offset: usize,
    theme: Theme,
}

impl ProviderManagementDialog {
    pub fn new(theme: Theme) -> Self {
        Self {
            providers: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            theme,
        }
    }

    pub fn set_providers(&mut self, providers: Vec<ProviderInfo>) {
        self.providers = providers;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn providers(&self) -> &[ProviderInfo] {
        &self.providers
    }

    fn status_color(&self, status: ProviderStatus) -> Color {
        match status {
            ProviderStatus::Connected => Color::Green,
            ProviderStatus::Disconnected => Color::Yellow,
            ProviderStatus::Error => Color::Red,
        }
    }

    fn status_text(&self, status: ProviderStatus) -> &'static str {
        match status {
            ProviderStatus::Connected => "Connected",
            ProviderStatus::Disconnected => "Disconnected",
            ProviderStatus::Error => "Error",
        }
    }
}

impl sealed::Sealed for ProviderManagementDialog {}

impl Dialog for ProviderManagementDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 18.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Provider Management")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let items: Vec<ListItem> = self
            .providers
            .iter()
            .map(|provider| {
                let status_color = self.status_color(provider.status);
                let status_text = self.status_text(provider.status);
                let key_indicator = if provider.api_key_set { "✓" } else { "✗" };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", provider.name)),
                    Span::styled(
                        format!("[{}]", status_text),
                        Style::default().fg(status_color),
                    ),
                    Span::raw(format!(" API Key: {}", key_indicator)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut state = ratatui::widgets::ListState::default();
        state = state.with_selected(Some(self.selected_index));
        state = state.with_offset(self.scroll_offset);
        f.render_stateful_widget(list, chunks[0], &mut state);

        let help_text = Paragraph::new("↑↓: Navigate | Enter: Configure | Esc: Close")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(help_text, chunks[1]);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        let visible_height = 11usize;

        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                if let Some(provider) = self.providers.get(self.selected_index) {
                    DialogAction::Navigate(format!("provider:{}", provider.id))
                } else {
                    DialogAction::None
                }
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                self.scroll_offset = self
                    .scroll_offset
                    .saturating_sub(if self.selected_index < self.scroll_offset { 1 } else { 0 });
                DialogAction::None
            }
            KeyCode::Down => {
                if self.selected_index < self.providers.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                if self.selected_index >= self.scroll_offset + visible_height {
                    self.scroll_offset = (self.selected_index + 1).saturating_sub(visible_height);
                }
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}
