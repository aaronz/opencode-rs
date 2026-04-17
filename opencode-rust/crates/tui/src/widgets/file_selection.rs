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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_selection_list_new() {
        let files = vec![FileItem {
            path: PathBuf::from("/a"),
            display_name: "a.txt".to_string(),
            size: Some(100),
            preview_lines: Vec::new(),
        }];
        let list = FileSelectionList::new(files);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_file_selection_list_selected_file() {
        let files = vec![
            FileItem {
                path: PathBuf::from("/a"),
                display_name: "a.txt".to_string(),
                size: Some(100),
                preview_lines: Vec::new(),
            },
            FileItem {
                path: PathBuf::from("/b"),
                display_name: "b.txt".to_string(),
                size: Some(200),
                preview_lines: Vec::new(),
            },
        ];
        let list = FileSelectionList::new(files);
        assert!(list.selected_file().is_some());
        assert_eq!(list.selected_file().unwrap().display_name, "a.txt");
    }

    #[test]
    fn test_file_selection_list_move_up() {
        let files = vec![
            FileItem {
                path: PathBuf::from("/a"),
                display_name: "a.txt".to_string(),
                size: Some(100),
                preview_lines: Vec::new(),
            },
            FileItem {
                path: PathBuf::from("/b"),
                display_name: "b.txt".to_string(),
                size: Some(200),
                preview_lines: Vec::new(),
            },
        ];
        let mut list = FileSelectionList::new(files);
        list.move_up();
        assert_eq!(list.selected, 0);
    }

    #[test]
    fn test_file_selection_list_move_down() {
        let files = vec![
            FileItem {
                path: PathBuf::from("/a"),
                display_name: "a.txt".to_string(),
                size: Some(100),
                preview_lines: Vec::new(),
            },
            FileItem {
                path: PathBuf::from("/b"),
                display_name: "b.txt".to_string(),
                size: Some(200),
                preview_lines: Vec::new(),
            },
        ];
        let mut list = FileSelectionList::new(files);
        list.move_down();
        assert_eq!(list.selected, 1);
    }

    #[test]
    fn test_file_selection_list_select() {
        let files = vec![
            FileItem {
                path: PathBuf::from("/a"),
                display_name: "a.txt".to_string(),
                size: Some(100),
                preview_lines: Vec::new(),
            },
            FileItem {
                path: PathBuf::from("/b"),
                display_name: "b.txt".to_string(),
                size: Some(200),
                preview_lines: Vec::new(),
            },
        ];
        let mut list = FileSelectionList::new(files);
        list.select(1);
        assert_eq!(list.selected, 1);
        assert_eq!(list.selected_file().unwrap().display_name, "b.txt");
    }

    #[test]
    fn test_file_selection_list_select_out_of_bounds() {
        let files = vec![FileItem {
            path: PathBuf::from("/a"),
            display_name: "a.txt".to_string(),
            size: Some(100),
            preview_lines: Vec::new(),
        }];
        let mut list = FileSelectionList::new(files);
        list.select(10);
        assert_eq!(list.selected, 0);
    }

    #[test]
    fn test_file_item_debug() {
        let item = FileItem {
            path: PathBuf::from("/test"),
            display_name: "test.txt".to_string(),
            size: Some(42),
            preview_lines: vec!["line1".to_string()],
        };
        let debug = format!("{:?}", item);
        assert!(debug.contains("test.txt"));
    }
}
