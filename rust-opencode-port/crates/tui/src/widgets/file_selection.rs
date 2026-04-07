use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, Widget};
use std::path::PathBuf;

pub struct FileSelectionList {
    files: Vec<FileItem>,
    selected: usize,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub display_name: String,
    pub size: Option<u64>,
    pub preview_lines: Vec<String>,
}

impl FileSelectionList {
    pub fn new(files: Vec<FileItem>) -> Self {
        Self { files, selected: 0 }
    }

    pub fn selected_file(&self) -> Option<&FileItem> {
        self.files.get(self.selected)
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        self.selected = (self.selected + 1).min(self.files.len().saturating_sub(1));
    }

    pub fn select(&mut self, index: usize) {
        if index < self.files.len() {
            self.selected = index;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

impl Widget for FileSelectionList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.files.is_empty() {
            return;
        }

        let items: Vec<ListItem> = self
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let prefix = if i == self.selected { ">" } else { " " };
                let size_str = file
                    .size
                    .map(|s| format!(" ({} bytes)", s))
                    .unwrap_or_default();
                let line = format!("{} {}{}", prefix, file.display_name, size_str);
                ListItem::new(Line::from(Span::raw(line)))
            })
            .collect();

        let list = List::new(items);
        list.render(area, buf);
    }
}
