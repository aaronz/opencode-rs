use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarSectionType {
    Files,
    Diagnostics,
    LspStatus,
}

impl SidebarSectionType {
    pub fn title(&self) -> &'static str {
        match self {
            SidebarSectionType::Files => "Files",
            SidebarSectionType::Diagnostics => "Diagnostics",
            SidebarSectionType::LspStatus => "LSP Status",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SidebarSectionType::Files => SidebarSectionType::Diagnostics,
            SidebarSectionType::Diagnostics => SidebarSectionType::LspStatus,
            SidebarSectionType::LspStatus => SidebarSectionType::Files,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SidebarSectionType::Files => SidebarSectionType::LspStatus,
            SidebarSectionType::Diagnostics => SidebarSectionType::Files,
            SidebarSectionType::LspStatus => SidebarSectionType::Diagnostics,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarSectionState {
    pub section_type: SidebarSectionType,
    pub collapsed: bool,
}

impl SidebarSectionState {
    pub fn new(section_type: SidebarSectionType) -> Self {
        Self {
            section_type,
            collapsed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SidebarSection {
    pub section_type: SidebarSectionType,
    pub collapsed: bool,
    pub file_tree: Option<SidebarFileTree>,
    pub diagnostics: Vec<String>,
    pub lsp_status: String,
}

impl SidebarSection {
    pub fn new(section_type: SidebarSectionType) -> Self {
        Self {
            section_type,
            collapsed: false,
            file_tree: None,
            diagnostics: Vec::new(),
            lsp_status: String::new(),
        }
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    pub fn title(&self) -> &'static str {
        self.section_type.title()
    }
}

#[derive(Debug, Clone)]
pub struct SidebarFileTree {
    root_path: PathBuf,
    items: Vec<SidebarFileTreeItem>,
    state: ListState,
}

#[derive(Debug, Clone)]
pub struct SidebarFileTreeItem {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
}

#[derive(Clone)]
struct SectionDrawInfo {
    section_type: SidebarSectionType,
    collapsed: bool,
    diagnostics: Vec<String>,
    lsp_status: String,
    file_tree_items: Vec<SidebarFileTreeItem>,
    file_tree_selected: Option<usize>,
}

impl SidebarFileTree {
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

    fn collect_items(&mut self, path: &PathBuf, depth: usize) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = path.is_dir();

        self.items.push(SidebarFileTreeItem {
            path: path.clone(),
            name,
            is_dir,
            is_expanded: false,
            depth,
        });

        if is_dir {
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
}

pub struct Sidebar {
    sections: Vec<SidebarSection>,
    active_section: usize,
    theme: Theme,
    pub collapsed: bool,
}

impl Sidebar {
    pub fn new(theme: Theme) -> Self {
        let mut sections = vec![
            SidebarSection::new(SidebarSectionType::Files),
            SidebarSection::new(SidebarSectionType::Diagnostics),
            SidebarSection::new(SidebarSectionType::LspStatus),
        ];

        if let Ok(cwd) = std::env::current_dir() {
            sections[0].file_tree = Some(SidebarFileTree::new(cwd));
        }

        Self {
            sections,
            active_section: 0,
            theme,
            collapsed: false,
        }
    }

    pub fn sections(&self) -> &[SidebarSection] {
        &self.sections
    }

    pub fn sections_mut(&mut self) -> &mut [SidebarSection] {
        &mut self.sections
    }

    pub fn active_section(&self) -> &SidebarSection {
        &self.sections[self.active_section]
    }

    pub fn active_section_mut(&mut self) -> &mut SidebarSection {
        &mut self.sections[self.active_section]
    }

    pub fn active_section_index(&self) -> usize {
        self.active_section
    }

    pub fn set_active_section(&mut self, index: usize) {
        if index < self.sections.len() {
            self.active_section = index;
        }
    }

    pub fn next_section(&mut self) {
        self.active_section = (self.active_section + 1) % self.sections.len();
    }

    pub fn prev_section(&mut self) {
        self.active_section = if self.active_section == 0 {
            self.sections.len() - 1
        } else {
            self.active_section - 1
        };
    }

    pub fn toggle_active_collapse(&mut self) {
        self.sections[self.active_section].toggle_collapse();
    }

    pub fn collapse_section(&mut self, section_type: SidebarSectionType) {
        if let Some(section) = self
            .sections
            .iter_mut()
            .find(|s| s.section_type == section_type)
        {
            section.set_collapsed(true);
        }
    }

    pub fn expand_section(&mut self, section_type: SidebarSectionType) {
        if let Some(section) = self
            .sections
            .iter_mut()
            .find(|s| s.section_type == section_type)
        {
            section.set_collapsed(false);
        }
    }

    pub fn toggle_section(&mut self, section_type: SidebarSectionType) {
        if let Some(section) = self
            .sections
            .iter_mut()
            .find(|s| s.section_type == section_type)
        {
            section.toggle_collapse();
        }
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<String>) {
        if let Some(section) = self
            .sections
            .iter_mut()
            .find(|s| s.section_type == SidebarSectionType::Diagnostics)
        {
            section.diagnostics = diagnostics;
        }
    }

    pub fn set_lsp_status(&mut self, status: String) {
        if let Some(section) = self
            .sections
            .iter_mut()
            .find(|s| s.section_type == SidebarSectionType::LspStatus)
        {
            section.lsp_status = status;
        }
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn get_section_state(&self) -> Vec<SidebarSectionState> {
        self.sections
            .iter()
            .map(|s| SidebarSectionState {
                section_type: s.section_type,
                collapsed: s.collapsed,
            })
            .collect()
    }

    pub fn set_section_state(&mut self, states: &[SidebarSectionState]) {
        for state in states {
            if let Some(section) = self
                .sections
                .iter_mut()
                .find(|s| s.section_type == state.section_type)
            {
                section.set_collapsed(state.collapsed);
            }
        }
    }

    pub fn load_from_file(&mut self, path: &PathBuf) -> std::io::Result<()> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            if let Ok(states) = serde_json::from_str::<Vec<SidebarSectionState>>(&content) {
                self.set_section_state(&states);
            }
        }
        Ok(())
    }

    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        let states = self.get_section_state();
        let content = serde_json::to_string_pretty(&states)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        if self.collapsed {
            let block = Block::default()
                .title("Sidebar (collapsed)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.border_color()));
            f.render_widget(block, area);
            return;
        }

        let block = Block::default()
            .title("Sidebar")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_color()));
        let inner = block.inner(area);
        f.render_widget(block, area);

        let section_count = self.sections.len() as u16;
        let section_height = inner.height.saturating_sub(section_count) / section_count;
        let header_height = 1;

        let mut current_y = inner.y;
        let theme = self.theme.clone();
        let active_section = self.active_section;

        let section_infos: Vec<SectionDrawInfo> = self
            .sections
            .iter()
            .map(|section| SectionDrawInfo {
                section_type: section.section_type,
                collapsed: section.collapsed,
                diagnostics: section.diagnostics.clone(),
                lsp_status: section.lsp_status.clone(),
                file_tree_items: section
                    .file_tree
                    .as_ref()
                    .map(|ft| ft.items.clone())
                    .unwrap_or_default(),
                file_tree_selected: section
                    .file_tree
                    .as_ref()
                    .and_then(|ft| ft.state.selected()),
            })
            .collect();

        for (i, section_info) in section_infos.iter().enumerate() {
            let is_active = i == active_section;

            let section_area = Rect::new(
                inner.x,
                current_y,
                inner.width,
                section_height + header_height,
            );

            Self::draw_section_static(f, section_area, section_info, is_active, &theme);

            current_y += section_height + header_height;
        }
    }

    fn draw_section_static(
        f: &mut Frame,
        area: Rect,
        section: &SectionDrawInfo,
        is_active: bool,
        theme: &Theme,
    ) {
        let collapse_indicator = if section.collapsed { "[+]" } else { "[-]" };
        let title = format!("{} {}", collapse_indicator, section.section_type.title());

        let border_style = if is_active {
            Style::default().fg(theme.primary_color())
        } else {
            Style::default().fg(theme.border_color())
        };

        let block = Block::default()
            .title(title.as_str())
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        f.render_widget(block, area);

        if section.collapsed {
            return;
        }

        match section.section_type {
            SidebarSectionType::Files => {
                Self::draw_file_tree_static(f, inner, section, theme);
            }
            SidebarSectionType::Diagnostics => {
                Self::draw_diagnostics_static(f, inner, section, theme);
            }
            SidebarSectionType::LspStatus => {
                Self::draw_lsp_status_static(f, inner, section, theme);
            }
        }
    }

    fn draw_file_tree_static(f: &mut Frame, area: Rect, section: &SectionDrawInfo, theme: &Theme) {
        if section.file_tree_items.is_empty() {
            let paragraph = Paragraph::new("No file tree available");
            f.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = section
            .file_tree_items
            .iter()
            .take((area.height as usize).saturating_sub(2))
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
                    ratatui::text::Span::raw(format!("{}{} ", indent, icon)),
                    ratatui::text::Span::raw(&item.name),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .fg(theme.primary_color()),
            );

        let mut state = ListState::default();
        state.select(section.file_tree_selected);
        f.render_stateful_widget(list, area, &mut state);
    }

    fn draw_diagnostics_static(
        f: &mut Frame,
        area: Rect,
        section: &SectionDrawInfo,
        theme: &Theme,
    ) {
        let content = if section.diagnostics.is_empty() {
            "No diagnostics".to_string()
        } else {
            section
                .diagnostics
                .iter()
                .take((area.height as usize).saturating_sub(1))
                .map(|d: &String| d.clone())
                .collect::<Vec<_>>()
                .join("\n")
        };

        let paragraph =
            Paragraph::new(content).style(Style::default().fg(theme.foreground_color()));
        f.render_widget(paragraph, area);
    }

    fn draw_lsp_status_static(f: &mut Frame, area: Rect, section: &SectionDrawInfo, theme: &Theme) {
        let content = if section.lsp_status.is_empty() {
            "LSP not connected".to_string()
        } else {
            section.lsp_status.clone()
        };

        let paragraph =
            Paragraph::new(content).style(Style::default().fg(theme.foreground_color()));
        f.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_section_collapse() {
        let theme = Theme::default();
        let sidebar = Sidebar::new(theme);

        assert!(!sidebar.sections[0].collapsed);

        let mut sidebar = sidebar;
        sidebar.sections[0].toggle_collapse();
        assert!(sidebar.sections[0].collapsed);
    }

    #[test]
    fn test_sidebar_section_navigation() {
        let theme = Theme::default();
        let mut sidebar = Sidebar::new(theme);

        assert_eq!(sidebar.active_section_index(), 0);

        sidebar.next_section();
        assert_eq!(sidebar.active_section_index(), 1);

        sidebar.next_section();
        assert_eq!(sidebar.active_section_index(), 2);

        sidebar.next_section();
        assert_eq!(sidebar.active_section_index(), 0);

        sidebar.prev_section();
        assert_eq!(sidebar.active_section_index(), 2);

        sidebar.prev_section();
        assert_eq!(sidebar.active_section_index(), 1);
    }

    #[test]
    fn test_sidebar_section_type_titles() {
        assert_eq!(SidebarSectionType::Files.title(), "Files");
        assert_eq!(SidebarSectionType::Diagnostics.title(), "Diagnostics");
        assert_eq!(SidebarSectionType::LspStatus.title(), "LSP Status");
    }

    #[test]
    fn test_sidebar_section_state_serialization() {
        let state = SidebarSectionState::new(SidebarSectionType::Files);
        let json = serde_json::to_string(&state).unwrap();
        let decoded: SidebarSectionState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.section_type, SidebarSectionType::Files);
        assert_eq!(decoded.collapsed, state.collapsed);
    }

    #[test]
    fn test_toggle_section() {
        let theme = Theme::default();
        let mut sidebar = Sidebar::new(theme);

        assert!(!sidebar.sections[0].collapsed);
        sidebar.toggle_section(SidebarSectionType::Files);
        assert!(sidebar.sections[0].collapsed);
        sidebar.toggle_section(SidebarSectionType::Files);
        assert!(!sidebar.sections[0].collapsed);
    }

    #[test]
    fn test_collapse_expand_section() {
        let theme = Theme::default();
        let mut sidebar = Sidebar::new(theme);

        sidebar.collapse_section(SidebarSectionType::Files);
        assert!(sidebar.sections[0].collapsed);

        sidebar.expand_section(SidebarSectionType::Files);
        assert!(!sidebar.sections[0].collapsed);
    }
}
