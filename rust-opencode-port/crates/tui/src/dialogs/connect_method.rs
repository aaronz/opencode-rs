use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

pub struct ConnectMethodDialog {
    selected_index: usize,
    methods: Vec<(String, String)>,
    theme: Theme,
}

impl ConnectMethodDialog {
    pub fn new(theme: Theme, provider_id: String) -> Self {
        let methods = if provider_id == "openai" {
            vec![
                ("browser".to_string(), "Browser auth".to_string()),
                ("api_key".to_string(), "API key".to_string()),
            ]
        } else {
            Vec::new()
        };

        Self {
            selected_index: 0,
            methods,
            theme,
        }
    }
}

impl Dialog for ConnectMethodDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let width = 60.min(area.width.saturating_sub(4));
        let height = 10.min(area.height.saturating_sub(4));
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, dialog_area);
        let block = Block::default()
            .title("Select Auth Method")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);
        let items: Vec<ListItem> = self
            .methods
            .iter()
            .enumerate()
            .map(|(index, (_, name))| {
                let style = if index == self.selected_index {
                    Style::default()
                        .fg(self.theme.primary_color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.foreground_color())
                };
                ListItem::new(Line::from(Span::styled(name.clone(), style)))
            })
            .collect();

        f.render_widget(List::new(items), inner);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Up => {
                if self.selected_index == 0 {
                    self.selected_index = self.methods.len().saturating_sub(1);
                } else {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % self.methods.len().max(1);
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.methods.is_empty() {
                    DialogAction::Close
                } else {
                    DialogAction::Confirm(self.methods[self.selected_index].0.clone())
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
    fn connect_method_dialog_confirms_browser_auth_selection() {
        let mut dialog = ConnectMethodDialog::new(Theme::default(), "openai".into());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("browser".into()));
    }
}
