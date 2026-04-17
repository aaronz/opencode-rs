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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollbar_new() {
        let scrollbar = Scrollbar::new(100);
        assert_eq!(scrollbar.viewport_content_height, 100);
        assert!(matches!(scrollbar.scroll_state, ScrollState::Auto));
    }

    #[test]
    fn test_scrollbar_with_position() {
        let scrollbar = Scrollbar::with_position(50);
        assert_eq!(scrollbar.viewport_content_height, 0);
        match scrollbar.scroll_state {
            ScrollState::Manual(pos) => assert_eq!(pos, 50),
            _ => panic!("Expected Manual state"),
        }
    }

    #[test]
    fn test_scroll_state_auto() {
        assert!(matches!(ScrollState::Auto, ScrollState::Auto));
    }

    #[test]
    fn test_scroll_state_manual() {
        assert!(matches!(ScrollState::Manual(10), ScrollState::Manual(10)));
        assert!(matches!(ScrollState::Manual(0), ScrollState::Manual(0)));
    }

    #[test]
    fn test_scroll_state_clone() {
        let state = ScrollState::Manual(42);
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_scroll_state_equality() {
        assert_eq!(ScrollState::Auto, ScrollState::Auto);
        assert_eq!(ScrollState::Manual(10), ScrollState::Manual(10));
        assert_ne!(ScrollState::Auto, ScrollState::Manual(0));
        assert_ne!(ScrollState::Manual(10), ScrollState::Manual(20));
    }
}
