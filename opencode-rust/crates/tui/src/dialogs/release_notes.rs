use crate::dialogs::sealed;
use crate::dialogs::{Dialog, DialogAction};
use crate::theme::Theme;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct ReleaseNotesDialog {
    version: String,
    notes: Vec<String>,
    theme: Theme,
}

impl ReleaseNotesDialog {
    pub fn new(theme: Theme) -> Self {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let notes = vec![
            "✨ New Features".to_string(),
            "  • Added settings dialog with tabs".to_string(),
            "  • Added model selection dialog".to_string(),
            "  • Added provider management".to_string(),
            "  • Added file and directory selection dialogs".to_string(),
            "".to_string(),
            "🔧 Improvements".to_string(),
            "  • Enhanced TUI with new dialogs".to_string(),
            "  • Better keyboard navigation".to_string(),
            "".to_string(),
            "🐛 Bug Fixes".to_string(),
            "  • Various UI improvements".to_string(),
        ];

        Self {
            version,
            notes,
            theme,
        }
    }
}

impl sealed::Sealed for ReleaseNotesDialog {}

impl Dialog for ReleaseNotesDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 20.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        f.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title(format!("Release Notes - v{}", self.version))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.primary_color()));
        f.render_widget(block.clone(), dialog_area);

        let inner_area = block.inner(dialog_area);

        let lines: Vec<Line> = self
            .notes
            .iter()
            .map(|note| {
                if note.starts_with("✨") || note.starts_with("🔧") || note.starts_with("🐛") {
                    Line::from(Span::styled(
                        note.clone(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(note.clone())
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::NONE));
        f.render_widget(paragraph, inner_area);
    }

    fn handle_input(&mut self, _key: KeyEvent) -> DialogAction {
        DialogAction::Close
    }
}
