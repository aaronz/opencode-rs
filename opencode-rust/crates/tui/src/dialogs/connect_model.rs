use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use opencode_llm::BrowserAuthModelInfo;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
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

        if self.models.is_empty() {
            let empty_msg = Paragraph::new("No models available")
                .style(Style::default().fg(self.theme.muted_color()));
            f.render_widget(empty_msg, inner);
            return;
        }

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

        let mut state = ListState::default();
        state.select(Some(self.selected_index));
        f.render_stateful_widget(List::new(items), inner, &mut state);
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
    use ratatui::{backend::TestBackend, Frame, Terminal};

    fn make_model(id: &str, name: &str) -> BrowserAuthModelInfo {
        BrowserAuthModelInfo {
            id: id.into(),
            name: name.into(),
            variants: vec![],
        }
    }

    #[test]
    fn test_connect_model_dialog_empty_list_enter_closes() {
        let mut dialog = ConnectModelDialog::new(Theme::default(), vec![]);

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }

    #[test]
    fn test_connect_model_dialog_empty_list_up_does_not_panic() {
        let mut dialog = ConnectModelDialog::new(Theme::default(), vec![]);

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::None);
    }

    #[test]
    fn test_connect_model_dialog_single_item_down_stays_at_zero() {
        let mut dialog =
            ConnectModelDialog::new(Theme::default(), vec![make_model("gpt-4o", "GPT-4o")]);

        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }

    #[test]
    fn test_connect_model_dialog_renders_empty_state() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f: &mut Frame| {
                let dialog = ConnectModelDialog::new(Theme::default(), vec![]);
                dialog.draw(f, f.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_border = buffer
            .content
            .iter()
            .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
        assert!(has_border, "Empty dialog should render border");
    }

    #[test]
    fn test_connect_model_dialog_renders_models() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f: &mut Frame| {
                let dialog = ConnectModelDialog::new(
                    Theme::default(),
                    vec![
                        make_model("gpt-4o", "GPT-4o"),
                        make_model("gpt-4o-mini", "GPT-4o Mini"),
                    ],
                );
                dialog.draw(f, f.area());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
        assert!(has_content, "Dialog should render with content");
    }

    #[test]
    fn connect_model_dialog_confirms_selected_model() {
        let mut dialog = ConnectModelDialog::new(
            Theme::default(),
            vec![
                make_model("gpt-5.3-codex", "GPT-5.3 Codex"),
                make_model("gpt-4o", "GPT-4o"),
            ],
        );

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("gpt-5.3-codex".into()));
    }

    #[test]
    fn empty_models_down_does_not_panic() {
        let mut dialog = ConnectModelDialog::new(Theme::default(), vec![]);

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::None);
    }

    #[test]
    fn single_model_up_wraps_to_zero() {
        let mut dialog =
            ConnectModelDialog::new(Theme::default(), vec![make_model("gpt-4o", "GPT-4o")]);

        dialog.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    }

    #[test]
    fn two_models_navigate_down_wraps() {
        let mut dialog = ConnectModelDialog::new(
            Theme::default(),
            vec![
                make_model("gpt-4o", "GPT-4o"),
                make_model("gpt-4o-mini", "GPT-4o Mini"),
            ],
        );

        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("gpt-4o-mini".into()));

        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("gpt-4o".into()));
    }

    #[test]
    fn two_models_navigate_up_wraps_to_last() {
        let mut dialog = ConnectModelDialog::new(
            Theme::default(),
            vec![
                make_model("gpt-4o", "GPT-4o"),
                make_model("gpt-4o-mini", "GPT-4o Mini"),
            ],
        );

        let action = dialog.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::None);
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("gpt-4o-mini".into()));
    }

    #[test]
    fn three_models_traverse_all() {
        let mut dialog = ConnectModelDialog::new(
            Theme::default(),
            vec![
                make_model("gpt-4o", "GPT-4o"),
                make_model("gpt-4o-mini", "GPT-4o Mini"),
                make_model("claude-3.5", "Claude 3.5"),
            ],
        );

        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            DialogAction::Confirm("gpt-4o".into())
        );

        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            DialogAction::None
        );
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            DialogAction::Confirm("gpt-4o-mini".into())
        );

        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            DialogAction::None
        );
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            DialogAction::Confirm("claude-3.5".into())
        );
    }

    #[test]
    fn escape_always_closes() {
        let mut dialog = ConnectModelDialog::new(Theme::default(), vec![]);
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            DialogAction::Close
        );

        let mut dialog =
            ConnectModelDialog::new(Theme::default(), vec![make_model("gpt-4o", "GPT-4o")]);
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            DialogAction::Close
        );
    }

    #[test]
    fn unhandled_keys_return_none() {
        let mut dialog =
            ConnectModelDialog::new(Theme::default(), vec![make_model("gpt-4o", "GPT-4o")]);

        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            DialogAction::None
        );
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            DialogAction::None
        );
        assert_eq!(
            dialog.handle_input(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
            DialogAction::None
        );
    }
}
