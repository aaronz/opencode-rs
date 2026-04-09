use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

pub struct Scrollbar {
    #[allow(dead_code)]
    viewport_content_height: u16,
    #[allow(dead_code)]
    scroll_state: ScrollState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollState {
    Auto,
    Manual(u16),
}

impl Scrollbar {
    pub fn new(content_height: u16) -> Self {
        Self {
            viewport_content_height: content_height,
            scroll_state: ScrollState::Auto,
        }
    }

    pub fn with_position(position: u16) -> Self {
        Self {
            viewport_content_height: 0,
            scroll_state: ScrollState::Manual(position),
        }
    }
}

impl Widget for Scrollbar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 1 {
            return;
        }
        let track = "│";
        let thumb = "┃";
        for y in 0..area.height {
            let symbol = if y == area.height / 2 { thumb } else { track };
            buf.set_string(area.x, area.y + y, symbol, ratatui::style::Style::default());
        }
    }
}
