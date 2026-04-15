use crate::dialogs::sealed;
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

pub struct ConnectProviderDialog {
    selected_index: usize,
    providers: Vec<(String, String)>,
    theme: Theme,
}

impl ConnectProviderDialog {
    pub fn new(theme: Theme) -> Self {
        Self {
            selected_index: 0,
            providers: vec![
                ("openai".to_string(), "OpenAI".to_string()),
                ("anthropic".to_string(), "Anthropic".to_string()),
                ("ollama".to_string(), "Ollama".to_string()),
            ],
            theme,
        }
    }
}

impl sealed::Sealed for ConnectProviderDialog {}

impl Dialog for ConnectProviderDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let width = 60.min(area.width.saturating_sub(4));
        let height = 12.min(area.height.saturating_sub(4));
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, dialog_area);
        let block = Block::default()
            .title("Connect Provider")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner = block.inner(dialog_area);
        let items: Vec<ListItem> = self
            .providers
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
                    self.selected_index = self.providers.len().saturating_sub(1);
                } else {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % self.providers.len();
                DialogAction::None
            }
            KeyCode::Enter => DialogAction::Confirm(self.providers[self.selected_index].0.clone()),
            _ => DialogAction::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn connect_provider_dialog_confirms_openai_selection() {
        let mut dialog = ConnectProviderDialog::new(Theme::default());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("openai".into()));
    }
}
