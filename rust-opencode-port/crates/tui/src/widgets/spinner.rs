use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::Widget;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    label: String,
    state: SpinnerState,
    frame_index: usize,
    color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpinnerState {
    InProgress,
    Completed,
    Error,
}

impl Spinner {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            state: SpinnerState::InProgress,
            frame_index: 0,
            color: None,
        }
    }

    pub fn with_state(label: &str, state: SpinnerState) -> Self {
        Self {
            label: label.to_string(),
            state,
            frame_index: 0,
            color: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn tick(&mut self) {
        self.frame_index = (self.frame_index + 1) % SPINNER_FRAMES.len();
    }

    pub fn set_completed(&mut self) {
        self.state = SpinnerState::Completed;
    }

    pub fn set_error(&mut self) {
        self.state = SpinnerState::Error;
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let symbol = match self.state {
            SpinnerState::InProgress => SPINNER_FRAMES[self.frame_index],
            SpinnerState::Completed => "✔",
            SpinnerState::Error => "✗",
        };
        let text = format!("{} {}", symbol, self.label);
        let style = match self.state {
            SpinnerState::InProgress => {
                let color = self.color.unwrap_or(Color::Cyan);
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            }
            SpinnerState::Completed => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            SpinnerState::Error => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        };
        let span = Span::styled(text, style);
        buf.set_span(area.x, area.y, &span, area.width);
    }
}
