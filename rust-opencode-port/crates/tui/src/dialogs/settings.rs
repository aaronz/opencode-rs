use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    Keybinds,
    Models,
    Providers,
}

impl SettingsTab {
    fn as_str(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Keybinds => "Keybinds",
            SettingsTab::Models => "Models",
            SettingsTab::Providers => "Providers",
        }
    }
}

pub struct SettingsDialog {
    active_tab: SettingsTab,
    theme: Theme,
}

impl SettingsDialog {
    pub fn new(theme: Theme) -> Self {
        Self {
            active_tab: SettingsTab::General,
            theme,
        }
    }

    fn draw_general_tab(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Theme: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("default"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Font Size: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("14"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Editor: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("vim"),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }

    fn draw_keybinds_tab(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Ctrl+P ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("- Command Palette"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+T ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("- Timeline"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+, ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("- Settings"),
            ]),
            Line::from(vec![
                Span::styled("Ctrl+M ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw("- Model Selection"),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }

    fn draw_models_tab(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from("Default Model: gpt-4o"),
            Line::from(""),
            Line::from("Available Models:"),
            Line::from("  - gpt-4o (OpenAI)"),
            Line::from("  - claude-3-5-sonnet (Anthropic)"),
            Line::from("  - llama3.1 (Ollama)"),
        ];

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }

    fn draw_providers_tab(&self, f: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from("OpenAI: Connected"),
            Line::from("Anthropic: Connected"),
            Line::from("Ollama: Disconnected"),
        ];

        let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, area);
    }
}

impl Dialog for SettingsDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 70.min(area.width.saturating_sub(4));
        let dialog_height = 25.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Settings")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(inner_area);

        let tab_titles: Vec<Line> = vec![
            SettingsTab::General.as_str(),
            SettingsTab::Keybinds.as_str(),
            SettingsTab::Models.as_str(),
            SettingsTab::Providers.as_str(),
        ]
        .into_iter()
        .map(|t| Line::from(t))
        .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .select(match self.active_tab {
                SettingsTab::General => 0,
                SettingsTab::Keybinds => 1,
                SettingsTab::Models => 2,
                SettingsTab::Providers => 3,
            })
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(tabs, chunks[0]);

        match self.active_tab {
            SettingsTab::General => self.draw_general_tab(f, chunks[1]),
            SettingsTab::Keybinds => self.draw_keybinds_tab(f, chunks[1]),
            SettingsTab::Models => self.draw_models_tab(f, chunks[1]),
            SettingsTab::Providers => self.draw_providers_tab(f, chunks[1]),
        }
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Tab => {
                self.active_tab = match self.active_tab {
                    SettingsTab::General => SettingsTab::Keybinds,
                    SettingsTab::Keybinds => SettingsTab::Models,
                    SettingsTab::Models => SettingsTab::Providers,
                    SettingsTab::Providers => SettingsTab::General,
                };
                DialogAction::None
            }
            KeyCode::BackTab => {
                self.active_tab = match self.active_tab {
                    SettingsTab::General => SettingsTab::Providers,
                    SettingsTab::Keybinds => SettingsTab::General,
                    SettingsTab::Models => SettingsTab::Keybinds,
                    SettingsTab::Providers => SettingsTab::Models,
                };
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}
