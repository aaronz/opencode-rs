use crate::command::CommandRegistry;
use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub struct SlashCommandOverlay {
    input: String,
    filtered_commands: Vec<String>,
    selected_index: usize,
    theme: Theme,
}

impl SlashCommandOverlay {
    pub fn new(theme: Theme) -> Self {
        Self {
            input: String::new(),
            filtered_commands: Vec::new(),
            selected_index: 0,
            theme,
        }
    }

    pub fn update_input(&mut self, registry: &CommandRegistry, input: &str) {
        self.input = input.to_string();
        self.filtered_commands = registry
            .find(input)
            .iter()
            .map(|cmd| cmd.name.clone())
            .collect();
        self.selected_index = 0;
    }

    pub fn get_selected_command(&self) -> Option<String> {
        self.filtered_commands.get(self.selected_index).cloned()
    }

    pub fn filtered_commands(&self) -> &[String] {
        &self.filtered_commands
    }
}

impl sealed::Sealed for SlashCommandOverlay {}

impl Dialog for SlashCommandOverlay {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 15.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Commands")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);

        let input_widget = ratatui::widgets::Paragraph::new(Line::from(vec![
            Span::styled("/", Style::default().fg(self.theme.accent_color())),
            Span::styled(&self.input, Style::default()),
        ]))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(input_widget, inner_area);

        let list_area = Rect::new(
            inner_area.x,
            inner_area.y + 3,
            inner_area.width,
            inner_area.height.saturating_sub(4),
        );

        if self.filtered_commands.is_empty() {
            let empty_msg = Paragraph::new("No commands match")
                .style(Style::default().fg(self.theme.muted_color()));
            f.render_widget(empty_msg, list_area);
            return;
        }

        let items: Vec<ListItem> = self
            .filtered_commands
            .iter()
            .map(|cmd| ListItem::new(cmd.clone()))
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(self.theme.primary_color())
                    .fg(Color::Black)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(
            self.selected_index
                .min(self.filtered_commands.len().saturating_sub(1)),
        ));
        f.render_stateful_widget(list, list_area, &mut state);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                if self.filtered_commands.is_empty() {
                    DialogAction::Close
                } else if let Some(cmd_name) = self.get_selected_command() {
                    DialogAction::Confirm(cmd_name)
                } else {
                    DialogAction::Close
                }
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                let max = self.filtered_commands.len().saturating_sub(1);
                if self.selected_index < max {
                    self.selected_index += 1;
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

    #[test]
    fn test_slash_command_overlay_new() {
        let theme = crate::theme::Theme::default();
        let overlay = SlashCommandOverlay::new(theme);
        assert!(overlay.input.is_empty());
        assert_eq!(overlay.selected_index, 0);
    }

    #[test]
    fn test_slash_command_overlay_update() {
        let theme = crate::theme::Theme::default();
        let mut overlay = SlashCommandOverlay::new(theme);
        let registry = CommandRegistry::new();

        overlay.update_input(&registry, "p");
        assert!(!overlay.filtered_commands.is_empty());
    }

    #[test]
    fn test_slash_command_empty_filter_closes_dialog() {
        let theme = crate::theme::Theme::default();
        let mut overlay = SlashCommandOverlay::new(theme);
        let registry = CommandRegistry::new();

        overlay.update_input(&registry, "zzzz_no_command_exists");
        assert!(overlay.filtered_commands.is_empty());

        let action = overlay.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }

    #[test]
    fn test_slash_command_enter_confirms_selected() {
        let theme = crate::theme::Theme::default();
        let mut overlay = SlashCommandOverlay::new(theme);
        let registry = CommandRegistry::new();

        overlay.update_input(&registry, "p");
        let cmd = overlay.get_selected_command().unwrap();

        let action = overlay.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Confirm(cmd));
    }

    #[test]
    fn test_slash_command_escape_closes() {
        let theme = crate::theme::Theme::default();
        let mut overlay = SlashCommandOverlay::new(theme);

        let action = overlay.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, DialogAction::Close);
    }

    #[test]
    fn test_slash_command_up_navigation() {
        let theme = crate::theme::Theme::default();
        let mut overlay = SlashCommandOverlay::new(theme);
        let registry = CommandRegistry::new();

        overlay.update_input(&registry, "");
        let first_cmd = overlay.get_selected_command().unwrap();

        overlay.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        let after_up_cmd = overlay.get_selected_command().unwrap();

        assert_eq!(first_cmd, after_up_cmd);
    }
}
