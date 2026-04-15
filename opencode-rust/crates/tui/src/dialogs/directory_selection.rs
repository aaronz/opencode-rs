use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

pub struct DirectorySelectionDialog {
    current_dir: PathBuf,
    entries: Vec<DirEntry>,
    selected_index: usize,
    theme: Theme,
}

struct DirEntry {
    name: String,
    _is_dir: bool,
}

impl DirectorySelectionDialog {
    pub fn new(theme: Theme) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let entries = Self::read_dir(&current_dir);

        Self {
            current_dir,
            entries,
            selected_index: 0,
            theme,
        }
    }

    fn read_dir(path: &PathBuf) -> Vec<DirEntry> {
        let mut entries = vec![];

        if path.parent().is_some() {
            entries.push(DirEntry {
                name: "..".to_string(),
                _is_dir: true,
            });
        }

        if let Ok(dir_entries) = std::fs::read_dir(path) {
            for entry in dir_entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

                if is_dir {
                    entries.push(DirEntry {
                        name,
                        _is_dir: is_dir,
                    });
                }
            }
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries
    }
}

impl sealed::Sealed for DirectorySelectionDialog {}

impl Dialog for DirectorySelectionDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 70.min(area.width.saturating_sub(4));
        let dialog_height = 25.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(format!("Select Directory - {}", self.current_dir.display()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|entry| {
                let icon = if entry.name == ".." { "⬆" } else { "📁" };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} ", icon)),
                    Span::raw(entry.name.clone()),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));
        f.render_stateful_widget(list, chunks[0], &mut state);

        let help_text = Paragraph::new("↑↓: Navigate | Enter: Open | Space: Select | Esc: Cancel")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(help_text, chunks[1]);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                if let Some(entry) = self.entries.get(self.selected_index) {
                    let new_path = if entry.name == ".." {
                        self.current_dir
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| self.current_dir.clone())
                    } else {
                        self.current_dir.join(&entry.name)
                    };
                    self.current_dir = new_path;
                    self.entries = Self::read_dir(&self.current_dir);
                    self.selected_index = 0;
                }
                DialogAction::None
            }
            KeyCode::Char(' ') => {
                DialogAction::Confirm(self.current_dir.to_string_lossy().to_string())
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                if self.selected_index < self.entries.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                DialogAction::None
            }
            _ => DialogAction::None,
        }
    }
}
