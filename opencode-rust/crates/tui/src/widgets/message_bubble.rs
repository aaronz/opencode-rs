use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

pub struct MessageBubble {
    content: String,
    role: MessageRole,
}

impl MessageBubble {
    pub fn new(content: String, role: MessageRole) -> Self {
        Self { content, role }
    }

    pub fn user(content: String) -> Self {
        Self {
            content,
            role: MessageRole::User,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            content,
            role: MessageRole::Assistant,
        }
    }

    fn style(&self) -> ratatui::style::Style {
        match self.role {
            MessageRole::User => ratatui::style::Style::default()
                .fg(ratatui::style::Color::White)
                .bg(ratatui::style::Color::DarkGray),
            MessageRole::Assistant => ratatui::style::Style::default()
                .fg(ratatui::style::Color::White)
                .bg(ratatui::style::Color::Blue),
            MessageRole::System => ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bg(ratatui::style::Color::Black),
        }
    }

    fn prefix(&self) -> &'static str {
        match self.role {
            MessageRole::User => "You",
            MessageRole::Assistant => "AI",
            MessageRole::System => "System",
        }
    }
}

impl Widget for MessageBubble {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines: Vec<Line> = self
            .content
            .lines()
            .map(|line| {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", self.prefix()),
                        self.style().add_modifier(ratatui::style::Modifier::BOLD),
                    ),
                    Span::styled(line, self.style()),
                ])
            })
            .collect();

        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(self.style())
            .title(self.prefix());

        let paragraph = ratatui::widgets::Paragraph::new(lines).block(block);
        paragraph.render(area, buf);
    }
}
