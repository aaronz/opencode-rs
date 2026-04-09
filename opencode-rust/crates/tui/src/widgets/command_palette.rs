use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub struct CommandPalette {
    commands: Vec<CommandItem>,
    selected_index: usize,
    filter_text: String,
}

#[derive(Debug, Clone)]
pub struct CommandItem {
    pub name: String,
    pub description: String,
    pub shortcut: Option<String>,
}

impl CommandPalette {
    pub fn new(commands: Vec<CommandItem>) -> Self {
        Self {
            commands,
            selected_index: 0,
            filter_text: String::new(),
        }
    }

    pub fn filter(&mut self, query: &str) {
        self.filter_text = query.to_string();
        self.selected_index = 0;
    }

    pub fn filtered_commands(&self) -> Vec<&CommandItem> {
        if self.filter_text.is_empty() {
            return self.commands.iter().collect();
        }

        let matcher = SkimMatcherV2::default();
        let mut scored: Vec<(i64, &CommandItem)> = self
            .commands
            .iter()
            .filter_map(|cmd| {
                let search_text = format!("{} {}", cmd.name, cmd.description);
                matcher
                    .fuzzy_match(&search_text, &self.filter_text)
                    .map(|score| (score, cmd))
            })
            .collect();

        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, cmd)| cmd).collect()
    }

    pub fn selected(&self) -> Option<&CommandItem> {
        let filtered = self.filtered_commands();
        filtered.get(self.selected_index).copied()
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let len = self.filtered_commands().len();
        if len > 0 {
            self.selected_index = (self.selected_index + 1).min(len.saturating_sub(1));
        }
    }
}

impl Widget for CommandPalette {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 10 || area.height < 3 {
            return;
        }

        let block = Block::default()
            .title(format!("Command Palette ({})", self.filter_text))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let filtered = self.filtered_commands();
        if filtered.is_empty() {
            let no_results = Paragraph::new(Line::from(Span::styled(
                "No commands match",
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
            )));
            no_results.render(inner, buf);
            return;
        }

        let max_height = inner.height as usize;
        let visible_count = filtered.len().min(max_height);

        let start = if self.selected_index >= visible_count {
            self.selected_index - visible_count + 1
        } else {
            0
        };

        let visible_items: Vec<Line> = filtered
            .iter()
            .skip(start)
            .take(visible_count)
            .enumerate()
            .map(|(i, cmd)| {
                let global_idx = start + i;
                let is_selected = global_idx == self.selected_index;

                let prefix = if is_selected { "> " } else { "  " };
                let shortcut = cmd
                    .shortcut
                    .as_deref()
                    .map(|s| format!(" [{}]", s))
                    .unwrap_or_default();

                let name_style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let desc_style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
                };

                let shortcut_style = if is_selected {
                    Style::default().fg(Color::Yellow).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(&cmd.name, name_style),
                    Span::styled(" - ", desc_style),
                    Span::styled(&cmd.description, desc_style),
                    Span::styled(shortcut, shortcut_style),
                ])
            })
            .collect();

        let content = Paragraph::new(visible_items);
        content.render(inner, buf);
    }
}
