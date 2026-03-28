use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileTreeItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
}

impl FileTreeItem {
    pub fn new(path: PathBuf, depth: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = path.is_dir();

        Self {
            path,
            name,
            is_dir,
            is_expanded: false,
            depth,
        }
    }
}

pub struct FileTree {
    root_path: PathBuf,
    items: Vec<FileTreeItem>,
    state: ListState,
}

impl FileTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut tree = Self {
            root_path: root_path.clone(),
            items: Vec::new(),
            state: ListState::default(),
        };
        tree.refresh_items();
        tree
    }

    fn refresh_items(&mut self) {
        self.items.clear();
        self.collect_items(&self.root_path.clone(), 0);
        if !self.items.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    fn collect_items(&mut self, path: &Path, depth: usize) {
        let item = FileTreeItem::new(path.to_path_buf(), depth);
        let is_expanded = item.is_expanded;
        let is_dir = item.is_dir;
        self.items.push(item);

        if is_dir && is_expanded {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();

                paths.sort_by(|a, b| {
                    let a_is_dir = a.is_dir();
                    let b_is_dir = b.is_dir();
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => a.file_name().cmp(&b.file_name()),
                    }
                });

                for child_path in paths {
                    self.collect_items(&child_path, depth + 1);
                }
            }
        }
    }

    pub fn toggle_current(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(item) = self.items.get_mut(idx) {
                if item.is_dir {
                    item.is_expanded = !item.is_expanded;
                    self.refresh_items();
                }
            }
        }
    }

    pub fn select_next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let next = self
            .state
            .selected()
            .map(|i| (i + 1).min(len - 1))
            .unwrap_or(0);
        self.state.select(Some(next));
    }

    pub fn select_previous(&mut self) {
        let prev = self
            .state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.state.select(Some(prev));
    }

    pub fn get_selected_path(&self) -> Option<&PathBuf> {
        self.state
            .selected()
            .and_then(|idx| self.items.get(idx))
            .map(|item| &item.path)
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect, title: &str) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| {
                let indent = "  ".repeat(item.depth);
                let icon = if item.is_dir {
                    if item.is_expanded {
                        "📂"
                    } else {
                        "📁"
                    }
                } else {
                    "📄"
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!("{}{} ", indent, icon)),
                    Span::raw(&item.name),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(list, area, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_tree_new() {
        let tmp = TempDir::new().unwrap();
        let tree = FileTree::new(tmp.path().to_path_buf());
        assert!(!tree.items.is_empty());
    }

    #[test]
    fn test_file_tree_navigation() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("subdir")).unwrap();

        let mut tree = FileTree::new(tmp.path().to_path_buf());
        tree.select_next();
        assert!(tree.state.selected().is_some());
    }
}
