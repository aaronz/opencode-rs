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

pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub is_paid: bool,
    pub is_available: bool,
}

pub struct ModelSelectionDialog {
    models: Vec<ModelInfo>,
    selected_index: usize,
    filter: String,
    theme: Theme,
}

impl ModelSelectionDialog {
    pub fn new(theme: Theme) -> Self {
        Self {
            models: Vec::new(),
            selected_index: 0,
            filter: String::new(),
            theme,
        }
    }

    pub fn set_models(&mut self, models: Vec<ModelInfo>) {
        self.models = models;
        self.selected_index = 0;
        self.filter.clear();
    }

    pub fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn filtered_models(&self) -> Vec<&ModelInfo> {
        self.models
            .iter()
            .filter(|m| {
                self.filter.is_empty()
                    || m.name.to_lowercase().contains(&self.filter.to_lowercase())
                    || m.provider
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
            })
            .collect()
    }
}

impl Dialog for ModelSelectionDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 20.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Select Model")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(inner_area);

        let filter_text = if self.filter.is_empty() {
            "Type to filter...".to_string()
        } else {
            format!("Filter: {}", self.filter)
        };
        let filter_widget = Paragraph::new(filter_text)
            .block(Block::default().borders(Borders::ALL).title("Search"));
        f.render_widget(filter_widget, chunks[0]);

        let filtered = self.filtered_models();
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|model| {
                let paid_marker = if model.is_paid { " $" } else { "" };
                let availability = if model.is_available {
                    ""
                } else {
                    " (unavailable)"
                };
                let style = if !model.is_available {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} {}{}", model.name, model.provider, paid_marker),
                        style,
                    ),
                    Span::styled(availability, Style::default().fg(Color::Red)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Models"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));
        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                let filtered = self.filtered_models();
                if let Some(model) = filtered.get(self.selected_index) {
                    DialogAction::Confirm(model.id.clone())
                } else {
                    DialogAction::None
                }
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                let max = self.filtered_models().len().saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
                }
                DialogAction::None
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.selected_index = 0;
                DialogAction::None
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected_index = 0;
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}
