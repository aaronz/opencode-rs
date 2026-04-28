use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub struct ConnectProviderDialog {
    selected_index: usize,
    scroll_offset: usize,
    providers: Vec<(String, String)>,
    theme: Theme,
}

impl ConnectProviderDialog {
    pub fn new(theme: Theme) -> Self {
        Self {
            selected_index: 0,
            scroll_offset: 0,
            providers: Self::all_providers(),
            theme,
        }
    }

    pub fn set_providers(&mut self, providers: Vec<(String, String)>) {
        self.providers = providers;
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn all_providers() -> Vec<(String, String)> {
        vec![
            ("openai".to_string(), "OpenAI".to_string()),
            ("anthropic".to_string(), "Anthropic".to_string()),
            ("google".to_string(), "Google".to_string()),
            ("ollama".to_string(), "Ollama".to_string()),
            ("lmstudio".to_string(), "LM Studio".to_string()),
            ("azure".to_string(), "Azure".to_string()),
            ("openrouter".to_string(), "OpenRouter".to_string()),
            ("mistral".to_string(), "Mistral".to_string()),
            ("groq".to_string(), "Groq".to_string()),
            ("deepinfra".to_string(), "DeepInfra".to_string()),
            ("cerebras".to_string(), "Cerebras".to_string()),
            ("cohere".to_string(), "Cohere".to_string()),
            ("togetherai".to_string(), "Together AI".to_string()),
            ("perplexity".to_string(), "Perplexity".to_string()),
            ("xai".to_string(), "xAI".to_string()),
            ("huggingface".to_string(), "Hugging Face".to_string()),
            ("copilot".to_string(), "GitHub Copilot".to_string()),
            ("ai21".to_string(), "AI21".to_string()),
            ("minimax".to_string(), "MiniMax".to_string()),
            (
                "minimax-cn".to_string(),
                "MiniMax CN (minimaxi.com)".to_string(),
            ),
            ("qwen".to_string(), "Qwen".to_string()),
        ]
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

        let mut state = ListState::default();
        state = state.with_selected(Some(self.selected_index));
        state = state.with_offset(self.scroll_offset);
        f.render_stateful_widget(List::new(items), inner, &mut state);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        let visible_height = 12usize;

        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Up => {
                if self.providers.is_empty() {
                    DialogAction::Close
                } else {
                    if self.selected_index == 0 {
                        self.selected_index = self.providers.len().saturating_sub(1);
                    } else {
                        self.selected_index -= 1;
                    }
                    self.scroll_offset = self.scroll_offset.saturating_sub(
                        if self.selected_index < self.scroll_offset {
                            1
                        } else {
                            0
                        },
                    );
                    DialogAction::None
                }
            }
            KeyCode::Down => {
                if !self.providers.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.providers.len();
                    if self.selected_index >= self.scroll_offset + visible_height {
                        self.scroll_offset =
                            (self.selected_index + 1).saturating_sub(visible_height);
                    }
                }
                DialogAction::None
            }
            KeyCode::Enter => {
                if self.providers.is_empty() {
                    DialogAction::Close
                } else {
                    DialogAction::Confirm(self.providers[self.selected_index].0.clone())
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
    fn connect_provider_dialog_confirms_openai_selection() {
        let mut dialog = ConnectProviderDialog::new(Theme::default());
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm("openai".into()));
    }

    #[test]
    fn connect_provider_dialog_navigation() {
        let mut dialog = ConnectProviderDialog::new(Theme::default());
        assert_eq!(dialog.providers.len(), 21);

        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(dialog.selected_index, 1);

        dialog.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(dialog.selected_index, 0);
    }

    #[test]
    fn connect_provider_dialog_empty_list_closes() {
        let mut dialog = ConnectProviderDialog::new(Theme::default());
        dialog.set_providers(vec![]);
        let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }
}
