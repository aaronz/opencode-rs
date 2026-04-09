use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::render::SyntaxHighlighter;

pub struct CodeBlock {
    code: String,
    language: String,
    scroll_offset: u16,
    highlighter: Option<SyntaxHighlighter>,
}

impl CodeBlock {
    pub fn new(code: String, language: String) -> Self {
        Self {
            code,
            language,
            scroll_offset: 0,
            highlighter: None,
        }
    }

    pub fn with_highlighter(mut self, highlighter: SyntaxHighlighter) -> Self {
        self.highlighter = Some(highlighter);
        self
    }

    pub fn with_scroll(mut self, offset: u16) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, max_offset: u16) {
        self.scroll_offset = (self.scroll_offset + 1).min(max_offset);
    }

    pub fn set_language(&mut self, lang: String) {
        self.language = lang;
    }
}

impl Widget for CodeBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 4 || area.height < 2 {
            return;
        }

        let title = if self.language.is_empty() {
            "Code".to_string()
        } else {
            format!(" {} ", self.language.to_uppercase())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 1 {
            return;
        }

        let lines = if let Some(ref highlighter) = self.highlighter {
            highlighter.highlight_code(&self.code, &self.language, "base16-ocean.dark")
        } else {
            self.code
                .lines()
                .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::LightYellow))))
                .collect()
        };

        let total_lines = lines.len();
        let max_scroll = total_lines.saturating_sub(inner.height as usize) as u16;
        let scroll = self.scroll_offset.min(max_scroll as u16);

        let visible: Vec<Line> = lines
            .into_iter()
            .skip(scroll as usize)
            .take(inner.height as usize)
            .collect();

        let paragraph = Paragraph::new(visible);
        paragraph.render(inner, buf);
    }
}
