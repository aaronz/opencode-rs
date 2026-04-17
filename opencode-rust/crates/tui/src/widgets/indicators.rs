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
        let label = format!("{} ", self.label);

        // Use ratatui's Gauge widget for proper progress bar rendering
        let gauge = ratatui::widgets::Gauge::default()
            .percent(percentage as u16)
            .label(label)
            .gauge_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

        gauge.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinking_indicator_new() {
        let indicator = ThinkingIndicator::new();
        assert_eq!(indicator.label, "Thinking");
        assert_eq!(indicator.frame_index, 0);
    }

    #[test]
    fn test_thinking_indicator_with_label() {
        let indicator = ThinkingIndicator::with_label("Processing");
        assert_eq!(indicator.label, "Processing");
    }

    #[test]
    fn test_thinking_indicator_tick() {
        let mut indicator = ThinkingIndicator::new();
        indicator.tick();
        assert_eq!(indicator.frame_index, 1);
    }

    #[test]
    fn test_thinking_indicator_tick_wraps() {
        let mut indicator = ThinkingIndicator::new();
        for _ in 0..THINKING_FRAMES.len() {
            indicator.tick();
        }
        assert_eq!(indicator.frame_index, 0);
    }

    #[test]
    fn test_thinking_indicator_set_label() {
        let mut indicator = ThinkingIndicator::new();
        indicator.set_label("New Label".to_string());
        assert_eq!(indicator.label, "New Label");
    }

    #[test]
    fn test_thinking_indicator_default() {
        let indicator = ThinkingIndicator::default();
        assert_eq!(indicator.label, "Thinking");
    }

    #[test]
    fn test_thinking_frames_length() {
        assert_eq!(THINKING_FRAMES.len(), 4);
    }

    #[test]
    fn test_progress_bar_new() {
        let bar = ProgressBar::new();
        assert_eq!(bar.current, 0);
        assert!(bar.total.is_none());
        assert_eq!(bar.label, "Generating");
    }

    #[test]
    fn test_progress_bar_with_total() {
        let bar = ProgressBar::with_total(100);
        assert_eq!(bar.current, 0);
        assert_eq!(bar.total, Some(100));
    }

    #[test]
    fn test_progress_bar_increment() {
        let mut bar = ProgressBar::new();
        bar.increment();
        assert_eq!(bar.current, 1);
    }

    #[test]
    fn test_progress_bar_set_progress() {
        let mut bar = ProgressBar::new();
        bar.set_progress(50);
        assert_eq!(bar.current, 50);
    }

    #[test]
    fn test_progress_bar_set_total() {
        let mut bar = ProgressBar::new();
        bar.set_total(100);
        assert_eq!(bar.total, Some(100));
    }

    #[test]
    fn test_progress_bar_percentage_none() {
        let bar = ProgressBar::new();
        assert!(bar.percentage().is_none());
    }

    #[test]
    fn test_progress_bar_percentage_some() {
        let mut bar = ProgressBar::with_total(100);
        bar.set_progress(50);
        assert_eq!(bar.percentage(), Some(50.0));
    }

    #[test]
    fn test_progress_bar_percentage_zero() {
        let bar = ProgressBar::with_total(100);
        assert_eq!(bar.percentage(), Some(0.0));
    }

    #[test]
    fn test_progress_bar_default() {
        let bar = ProgressBar::default();
        assert_eq!(bar.current, 0);
    }
}
