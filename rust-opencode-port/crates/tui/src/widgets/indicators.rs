use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub struct ThinkingIndicator {
    label: String,
    frame_index: usize,
}

const THINKING_FRAMES: &[&str] = &["  ", "· ", "··", "···"];

impl ThinkingIndicator {
    pub fn new() -> Self {
        Self {
            label: "Thinking".to_string(),
            frame_index: 0,
        }
    }

    pub fn with_label(label: &str) -> Self {
        Self {
            label: label.to_string(),
            frame_index: 0,
        }
    }

    pub fn tick(&mut self) {
        self.frame_index = (self.frame_index + 1) % THINKING_FRAMES.len();
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }
}

impl Default for ThinkingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ThinkingIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame = THINKING_FRAMES[self.frame_index];
        let text = format!("🧠 {} {}", self.label, frame);
        buf.set_string(
            area.x,
            area.y,
            text,
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
        );
    }
}

pub struct ProgressBar {
    current: u32,
    total: Option<u32>,
    label: String,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            current: 0,
            total: None,
            label: "Generating".to_string(),
        }
    }

    pub fn with_total(total: u32) -> Self {
        Self {
            current: 0,
            total: Some(total),
            label: "Generating".to_string(),
        }
    }

    pub fn increment(&mut self) {
        self.current += 1;
    }

    pub fn set_progress(&mut self, current: u32) {
        self.current = current;
    }

    pub fn set_total(&mut self, total: u32) {
        self.total = Some(total);
    }

    pub fn percentage(&self) -> Option<f32> {
        self.total.map(|t| (self.current as f32 / t as f32) * 100.0)
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let percentage = self.percentage().unwrap_or(0.0);
        let filled = ((area.width as f32 - 10.0) * (percentage / 100.0)) as usize;
        let empty = (area.width as usize).saturating_sub(filled + 10);

        let bar = format!(
            "{} [{}{}] {:.1}%",
            self.label,
            "█".repeat(filled),
            "░".repeat(empty),
            percentage
        );
        buf.set_string(
            area.x,
            area.y,
            bar,
            ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
        );
    }
}
