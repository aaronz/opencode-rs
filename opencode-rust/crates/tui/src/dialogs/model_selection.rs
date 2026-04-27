use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use opencode_llm::ModelVariant;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Model selection result containing model ID and optional variant
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSelectionResult {
    pub model_id: String,
    pub variant: Option<ModelVariant>,
}

pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub is_paid: bool,
    pub is_available: bool,
    pub variants: Vec<ModelVariant>,
}

pub struct ModelSelectionDialog {
    models: Vec<ModelInfo>,
    selected_index: usize,
    scroll_offset: usize,
    selected_variant_index: usize,
    in_variant_selection: bool,
    filter: String,
    theme: Theme,
}

impl ModelSelectionDialog {
    pub fn new(theme: Theme) -> Self {
        Self {
            models: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            selected_variant_index: 0,
            in_variant_selection: false,
            filter: String::new(),
            theme,
        }
    }

    pub fn set_models(&mut self, models: Vec<ModelInfo>) {
        self.models = models;
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.selected_variant_index = 0;
        self.in_variant_selection = false;
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

    fn selected_model(&self) -> Option<&ModelInfo> {
        let filtered = self.filtered_models();
        filtered.get(self.selected_index).copied()
    }
}

impl sealed::Sealed for ModelSelectionDialog {}

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

        let filtered = self.filtered_models();

        let filter_text = if self.filter.is_empty() {
            "Type to filter...".to_string()
        } else {
            format!("Filter: {}", self.filter)
        };
        let filter_widget = Paragraph::new(filter_text)
            .block(Block::default().borders(Borders::ALL).title("Search"));
        f.render_widget(filter_widget, chunks[0]);

        if filtered.is_empty() {
            let empty_msg = Paragraph::new("No models match filter")
                .style(Style::default().fg(self.theme.muted_color()));
            f.render_widget(empty_msg, chunks[1]);
            return;
        }

        let items: Vec<ListItem> = filtered
            .iter()
            .map(|model| {
                let paid_marker = if model.is_paid { " $" } else { "" };
                let availability = if model.is_available {
                    ""
                } else {
                    " (unavailable)"
                };
                let variant_hint = if model.variants.is_empty() {
                    String::new()
                } else if model.variants.len() == 1 {
                    " [+variant]".to_string()
                } else {
                    format!(" [{} variants]", model.variants.len())
                };
                let style = if !model.is_available {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!(
                            "{}{}{}{}",
                            model.name, model.provider, paid_marker, variant_hint
                        ),
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
        state = state.with_selected(Some(
            self.selected_index.min(filtered.len().saturating_sub(1)),
        ));
        state = state.with_offset(self.scroll_offset);
        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        let visible_height = 13usize;

        match key.code {
            KeyCode::Esc => {
                if self.in_variant_selection {
                    self.in_variant_selection = false;
                    DialogAction::None
                } else {
                    DialogAction::Close
                }
            }
            KeyCode::Enter => {
                let filtered = self.filtered_models();
                if filtered.is_empty() {
                    DialogAction::Close
                } else if let Some(model) = filtered.get(self.selected_index) {
                    if self.in_variant_selection && !model.variants.is_empty() {
                        let variant = model.variants.get(self.selected_variant_index);
                        DialogAction::ConfirmModelWithVariant {
                            model_id: model.id.clone(),
                            variant_name: variant.map(|v| v.name.clone()),
                        }
                    } else if model.variants.len() == 1 {
                        DialogAction::ConfirmModelWithVariant {
                            model_id: model.id.clone(),
                            variant_name: Some(model.variants[0].name.clone()),
                        }
                    } else if model.variants.len() > 1 {
                        self.in_variant_selection = true;
                        self.selected_variant_index = 0;
                        DialogAction::None
                    } else {
                        DialogAction::Confirm(model.id.clone())
                    }
                } else {
                    DialogAction::Close
                }
            }
            KeyCode::Tab => {
                if let Some(model) = self.selected_model() {
                    if !model.variants.is_empty() {
                        self.in_variant_selection = !self.in_variant_selection;
                        if self.in_variant_selection {
                            self.selected_variant_index = 0;
                        }
                    }
                }
                DialogAction::None
            }
            KeyCode::Up => {
                if self.in_variant_selection {
                    if self.selected_variant_index > 0 {
                        self.selected_variant_index -= 1;
                    }
                } else if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.scroll_offset = self
                        .scroll_offset
                        .saturating_sub(if self.selected_index < self.scroll_offset { 1 } else { 0 });
                }
                DialogAction::None
            }
            KeyCode::Down => {
                if self.in_variant_selection {
                    if let Some(model) = self.selected_model() {
                        let max = model.variants.len().saturating_sub(1);
                        if self.selected_variant_index < max {
                            self.selected_variant_index += 1;
                        }
                    }
                } else {
                    let max = self.filtered_models().len().saturating_sub(1);
                    if self.selected_index < max {
                        self.selected_index += 1;
                        if self.selected_index >= self.scroll_offset + visible_height {
                            self.scroll_offset = (self.selected_index + 1).saturating_sub(visible_height);
                        }
                    }
                }
                DialogAction::None
            }
            KeyCode::Char(c) => {
                if !self.in_variant_selection {
                    self.filter.push(c);
                    self.selected_index = 0;
                    self.scroll_offset = 0;
                }
                DialogAction::None
            }
            KeyCode::Backspace => {
                if !self.in_variant_selection {
                    self.filter.pop();
                    self.selected_index = 0;
                    self.scroll_offset = 0;
                }
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_model(id: &str, name: &str, provider: &str) -> ModelInfo {
        ModelInfo {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
            is_paid: true,
            is_available: true,
            variants: vec![],
        }
    }

    #[test]
    fn model_selection_dialog_confirms_selected_model() {
        let mut dialog = ModelSelectionDialog::new(Theme::default());
        dialog.models = vec![make_model("gpt-4o", "GPT-4o", "OpenAI")];

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("gpt-4o".into()));
    }

    #[test]
    fn model_selection_filter_reduces_list() {
        let theme = Theme::default();
        let mut dialog = ModelSelectionDialog::new(theme);
        dialog.models = vec![
            make_model("gpt-4o", "GPT-4o", "OpenAI"),
            make_model("claude", "Claude 3.5", "Anthropic"),
        ];

        dialog.handle_input(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));

        let filtered = dialog.filtered_models();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "claude");
    }

    #[test]
    fn model_selection_empty_filter_closes_dialog() {
        let theme = Theme::default();
        let mut dialog = ModelSelectionDialog::new(theme);
        dialog.models = vec![make_model("gpt-4o", "GPT-4o", "OpenAI")];

        dialog.handle_input(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            DialogAction::Close
        );
    }

    #[test]
    fn model_selection_navigation_wraps() {
        let theme = Theme::default();
        let mut dialog = ModelSelectionDialog::new(theme);
        dialog.models = vec![
            make_model("gpt-4o", "GPT-4o", "OpenAI"),
            make_model("claude", "Claude 3.5", "Anthropic"),
        ];

        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("claude".into()));
    }

    #[test]
    fn model_selection_escape_always_closes() {
        let theme = Theme::default();
        let mut dialog = ModelSelectionDialog::new(theme);
        dialog.models = vec![];

        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            DialogAction::Close
        );
    }

    #[test]
    fn model_selection_backspace_resets_filter() {
        let theme = Theme::default();
        let mut dialog = ModelSelectionDialog::new(theme);
        dialog.models = vec![
            make_model("gpt-4o", "GPT-4o", "OpenAI"),
            make_model("claude", "Claude 3.5", "Anthropic"),
        ];

        dialog.handle_input(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        assert_eq!(dialog.filtered_models().len(), 1);

        dialog.handle_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(dialog.filtered_models().len(), 2);
    }
}
