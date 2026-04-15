use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use opencode_llm::BrowserAuthModelInfo;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

pub struct ConnectModelDialog {
    selected_index: usize,
    models: Vec<BrowserAuthModelInfo>,
    theme: Theme,
}

impl ConnectModelDialog {
    pub fn new(theme: Theme, models: Vec<BrowserAuthModelInfo>) -> Self {
        Self {
            selected_index: 0,
            models,
            theme,
        }
    }
}

impl sealed::Sealed for ConnectModelDialog {}

impl Dialog for ConnectModelDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let width = 70.min(area.width.saturating_sub(4));
        let height = 14.min(area.height.saturating_sub(4));
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, dialog_area);
        let block = Block::default()
            .title("Select Model")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);
        let items: Vec<ListItem> = self
            .models
            .iter()
            .enumerate()
            .map(|(index, model)| {
                let style = if index == self.selected_index {
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.foreground_color())
                };
                ListItem::new(Line::from(vec![
                    Span::styled(model.name.clone(), style),
                    Span::raw(format!(" ({})", model.id)),
                ]))
            })
            .collect();

        f.render_widget(List::new(items), inner);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Up => {
                if self.selected_index == 0 {
                    self.selected_index = self.models.len().saturating_sub(1);
                } else {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % self.models.len().max(1);
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.models.is_empty() {
                    DialogAction::Close
                } else {
                    DialogAction::Confirm(self.models[self.selected_index].id.clone())
                }
            }
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn connect_model_dialog_confirms_selected_model() {
        let mut dialog = ConnectModelDialog::new(
            Theme::default(),
            vec![
                BrowserAuthModelInfo {
                    id: "gpt-5.3-codex".into(),
                    name: "GPT-5.3 Codex".into(),
                },
                BrowserAuthModelInfo {
                    id: "gpt-4o".into(),
                    name: "GPT-4o".into(),
                },
            ],
        );

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("gpt-5.3-codex".into()));
    }
}
