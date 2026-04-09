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
use std::path::PathBuf;

pub struct FileSelectionDialog {
    current_dir: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    selected_indices: Vec<usize>,
    filter: String,
    theme: Theme,
}

struct FileEntry {
    name: String,
    is_dir: bool,
    is_hidden: bool,
    path: Option<PathBuf>,
}

impl FileSelectionDialog {
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.selected_index = 0;
    }

    pub fn new(theme: Theme) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let entries = Self::read_dir(&current_dir);

        Self {
            current_dir,
            entries,
            selected_index: 0,
            selected_indices: Vec::new(),
            filter: String::new(),
            theme,
        }
    }

    pub fn with_initial_entries(theme: Theme, entries: Vec<(PathBuf, bool)>) -> Self {
        let file_entries: Vec<FileEntry> = entries
            .into_iter()
            .map(|(path, is_dir)| FileEntry {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                is_dir,
                is_hidden: false,
                path: Some(path),
            })
            .collect();

        Self {
            current_dir: PathBuf::from("."),
            entries: file_entries,
            selected_index: 0,
            selected_indices: Vec::new(),
            filter: String::new(),
            theme,
        }
    }

    pub fn selected_paths(&self) -> Vec<String> {
        let filtered = self.filtered_entries();
        self.selected_indices
            .iter()
            .filter_map(|&idx| filtered.get(idx))
            .filter(|e| !e.is_dir && e.name != "..")
            .filter_map(|e| e.path.as_ref())
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    fn read_dir(path: &PathBuf) -> Vec<FileEntry> {
        let mut entries = vec![];

        if path.parent().is_some() {
            entries.push(FileEntry {
                name: "..".to_string(),
                is_dir: true,
                is_hidden: false,
                path: Some(path.parent().unwrap().to_path_buf()),
            });
        }

        if let Ok(dir_entries) = std::fs::read_dir(path) {
            for entry in dir_entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_hidden = name.starts_with('.');
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

                entries.push(FileEntry {
                    name,
                    is_dir,
                    is_hidden,
                    path: Some(entry.path()),
                });
            }
        }

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        entries
    }

    fn filtered_entries(&self) -> Vec<&FileEntry> {
        self.entries
            .iter()
            .filter(|e| {
                self.filter.is_empty()
                    || e.name.to_lowercase().contains(&self.filter.to_lowercase())
            })
            .collect()
    }
}

impl Dialog for FileSelectionDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 70.min(area.width.saturating_sub(4));
        let dialog_height = 25.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(format!("Select File - {}", self.current_dir.display()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(inner_area);

        let filtered = self.filtered_entries();
        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let icon = if entry.name == ".." {
                    "⬆"
                } else if entry.is_dir {
                    "📁"
                } else {
                    "📄"
                };

                let is_selected = self.selected_indices.contains(&idx);
                let checkbox = if entry.is_dir || entry.name == ".." {
                    "  "
                } else if is_selected {
                    "[x]"
                } else {
                    "[ ]"
                };

                let style = if entry.is_hidden {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };

                let checkbox_style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    style
                };

                ListItem::new(Line::from(vec![
                    Span::styled(checkbox.to_string(), checkbox_style),
                    Span::styled(format!("{} ", icon), style),
                    Span::styled(entry.name.clone(), style),
                ]))
            })
            .collect();

        let selection_hint = if self.selected_indices.is_empty() {
            " Space: toggle | Enter: confirm".to_string()
        } else {
            format!(
                " {} selected | Enter/Ctrl+Enter: confirm",
                self.selected_indices.len()
            )
        };

        let hint_text = format!(
            "{}{}",
            if self.filter.is_empty() {
                "Filter: type to search..."
            } else {
                "Filter: ..."
            },
            selection_hint
        );
        let filter_widget = Paragraph::new(hint_text).block(Block::default().borders(Borders::ALL));
        f.render_widget(filter_widget, chunks[0]);

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_index));
        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        match key.code {
            KeyCode::Esc => DialogAction::Close,
            KeyCode::Enter => {
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    let selected = self.selected_paths();
                    if selected.is_empty() {
                        let filtered = self.filtered_entries();
                        if let Some(entry) = filtered.get(self.selected_index) {
                            if !entry.is_dir && entry.name != ".." {
                                let full_path = entry
                                    .path
                                    .clone()
                                    .unwrap_or_else(|| self.current_dir.join(&entry.name));
                                return DialogAction::Confirm(
                                    full_path.to_string_lossy().to_string(),
                                );
                            }
                        }
                    }
                    if !selected.is_empty() {
                        return DialogAction::ConfirmMultiple(selected);
                    }
                }
                let filtered = self.filtered_entries();
                if let Some(entry) = filtered.get(self.selected_index) {
                    if entry.is_dir {
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
                        DialogAction::None
                    } else if self.selected_indices.contains(&self.selected_index) {
                        let selected = self.selected_paths();
                        if selected.len() == 1 {
                            DialogAction::Confirm(selected.into_iter().next().unwrap())
                        } else if !selected.is_empty() {
                            DialogAction::ConfirmMultiple(selected)
                        } else {
                            DialogAction::None
                        }
                    } else {
                        let full_path = entry
                            .path
                            .clone()
                            .unwrap_or_else(|| self.current_dir.join(&entry.name));
                        DialogAction::Confirm(full_path.to_string_lossy().to_string())
                    }
                } else {
                    DialogAction::None
                }
            }
            KeyCode::Char(' ') => {
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    let filtered = self.filtered_entries();
                    if let Some(entry) = filtered.get(self.selected_index) {
                        if !entry.is_dir && entry.name != ".." {
                            if let Some(idx) = self
                                .selected_indices
                                .iter()
                                .position(|&i| i == self.selected_index)
                            {
                                self.selected_indices.remove(idx);
                            } else {
                                self.selected_indices.push(self.selected_index);
                            }
                        }
                    }
                } else {
                    if let Some(idx) = self
                        .selected_indices
                        .iter()
                        .position(|&i| i == self.selected_index)
                    {
                        self.selected_indices.remove(idx);
                    } else {
                        self.selected_indices.push(self.selected_index);
                    }
                }
                DialogAction::None
            }
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                DialogAction::None
            }
            KeyCode::Down => {
                let max = self.filtered_entries().len().saturating_sub(1);
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
