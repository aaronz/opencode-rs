use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

pub struct SkillsPanel {
    pub skills: Vec<SkillInfo>,
    state: ListState,
    theme: Theme,
}

impl SkillsPanel {
    pub fn new(theme: Theme) -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            skills: Vec::new(),
            state,
            theme,
        }
    }

    pub fn with_skills(theme: Theme, skills: Vec<SkillInfo>) -> Self {
        let mut panel = Self::new(theme);
        panel.set_skills(skills);
        panel
    }

    pub fn set_skills(&mut self, skills: Vec<SkillInfo>) {
        self.skills = skills;
        if self.skills.is_empty() {
            self.state.select(None);
        } else if self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = if self.skills.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No skills available",
                Style::default().fg(self.theme.muted_color()),
            )))]
        } else {
            self.skills
                .iter()
                .map(|skill| {
                    let check = if skill.enabled { "[x]" } else { "[ ]" };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{} ", check),
                            Style::default().fg(self.theme.primary_color()),
                        ),
                        Span::styled(
                            &skill.name,
                            Style::default().fg(self.theme.foreground_color()),
                        ),
                        Span::styled(
                            format!(" — {}", skill.description),
                            Style::default().fg(self.theme.muted_color()),
                        ),
                    ]))
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Skills (↑/↓ navigate, Enter toggle)")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.theme.border_color())),
            )
            .highlight_style(
                Style::default()
                    .fg(self.theme.foreground_color())
                    .add_modifier(Modifier::REVERSED),
            );

        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                true
            }
            KeyCode::Enter => {
                self.toggle_selected();
                true
            }
            _ => false,
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(idx) = self.state.selected() {
            if let Some(skill) = self.skills.get_mut(idx) {
                skill.enabled = !skill.enabled;
            }
        }
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(skill) = self
            .skills
            .iter_mut()
            .find(|skill| skill.name.eq_ignore_ascii_case(name))
        {
            skill.enabled = enabled;
            return true;
        }
        false
    }

    pub fn enabled_skill_names(&self) -> Vec<String> {
        self.skills
            .iter()
            .filter(|skill| skill.enabled)
            .map(|skill| skill.name.clone())
            .collect()
    }

    fn select_next(&mut self) {
        if self.skills.is_empty() {
            return;
        }
        let len = self.skills.len();
        let next = self
            .state
            .selected()
            .map(|idx| (idx + 1).min(len - 1))
            .unwrap_or(0);
        self.state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.skills.is_empty() {
            return;
        }
        let prev = self
            .state
            .selected()
            .map(|idx| idx.saturating_sub(1))
            .unwrap_or(0);
        self.state.select(Some(prev));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel_with_skills() -> SkillsPanel {
        SkillsPanel::with_skills(
            Theme::default(),
            vec![
                SkillInfo {
                    name: "code-review".to_string(),
                    description: "Review code".to_string(),
                    enabled: false,
                },
                SkillInfo {
                    name: "debugger".to_string(),
                    description: "Debug bugs".to_string(),
                    enabled: false,
                },
            ],
        )
    }

    #[test]
    fn toggles_selected_skill() {
        let mut panel = panel_with_skills();
        panel.toggle_selected();
        assert!(panel.skills[0].enabled);
    }

    #[test]
    fn navigation_changes_selected_index() {
        let mut panel = panel_with_skills();
        panel.select_next();
        assert_eq!(panel.selected_index(), Some(1));
        panel.select_previous();
        assert_eq!(panel.selected_index(), Some(0));
    }

    #[test]
    fn returns_enabled_skill_names() {
        let mut panel = panel_with_skills();
        panel.set_enabled("debugger", true);
        assert_eq!(panel.enabled_skill_names(), vec!["debugger".to_string()]);
    }
}
