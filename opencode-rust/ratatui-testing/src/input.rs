pub use crate::action::{Direction, InputMapper};
pub use crate::reducer::{test_table_driven, Reducer, ReducerError, ReducerTester, TableDrivenCase};

pub struct TerminalSizes;

impl TerminalSizes {
    pub fn very_small() -> (u16, u16) {
        (40, 10)
    }

    pub fn classic() -> (u16, u16) {
        (80, 24)
    }

    pub fn modern_wide() -> (u16, u16) {
        (120, 30)
    }

    pub fn large_monitor() -> (u16, u16) {
        (160, 40)
    }

    pub fn all() -> Vec<(u16, u16)> {
        vec![
            Self::very_small(),
            Self::classic(),
            Self::modern_wide(),
            Self::large_monitor(),
        ]
    }

    pub fn classic_and_smaller() -> Vec<(u16, u16)> {
        vec![Self::very_small(), Self::classic()]
    }
}

#[allow(dead_code)]
pub trait SizeSensitive: Clone {
    fn with_all_sizes(self) -> Vec<(u16, u16, Self)>
    where
        Self: Sized,
    {
        Self::all_sizes()
            .into_iter()
            .map(|s| (s.0, s.1, self.clone()))
            .collect()
    }

    fn all_sizes() -> Vec<(u16, u16)> {
        TerminalSizes::all()
    }
}

pub mod layout_helpers {
    use ratatui::layout::Rect;

    pub fn center_in_area(area: Rect, width: u16, height: u16) -> Rect {
        let x = area.x + (area.width.saturating_sub(width) / 2);
        let y = area.y + (area.height.saturating_sub(height) / 2);
        Rect::new(x, y, width.min(area.width), height.min(area.height))
    }

    pub fn vertical_center_in_area(area: Rect, height: u16) -> Rect {
        let y = area.y + (area.height.saturating_sub(height) / 2);
        Rect::new(area.x, y, area.width, height.min(area.height))
    }

    pub fn horizontal_center_in_area(area: Rect, width: u16) -> Rect {
        let x = area.x + (area.width.saturating_sub(width) / 2);
        Rect::new(x, area.y, width.min(area.width), area.height)
    }

    pub fn top_left(area: Rect, width: u16, height: u16) -> Rect {
        Rect::new(area.x, area.y, width.min(area.width), height.min(area.height))
    }

    pub fn bottom_right(area: Rect, width: u16, height: u16) -> Rect {
        let x = area.x + area.width.saturating_sub(width);
        let y = area.y + area.height.saturating_sub(height);
        Rect::new(x, y, width.min(area.width), height.min(area.height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn terminal_sizes_values() {
        let (w, h) = TerminalSizes::very_small();
        assert_eq!(w, 40);
        assert_eq!(h, 10);

        let (w, h) = TerminalSizes::classic();
        assert_eq!(w, 80);
        assert_eq!(h, 24);

        let (w, h) = TerminalSizes::modern_wide();
        assert_eq!(w, 120);
        assert_eq!(h, 30);

        let (w, h) = TerminalSizes::large_monitor();
        assert_eq!(w, 160);
        assert_eq!(h, 40);
    }

    #[test]
    fn terminal_sizes_all_count() {
        assert_eq!(TerminalSizes::all().len(), 4);
    }

    #[test]
    fn center_in_area_basic() {
        let area = Rect::new(0, 0, 80, 24);
        let centered = layout_helpers::center_in_area(area, 20, 10);
        assert_eq!(centered.x, 30);
        assert_eq!(centered.y, 7);
        assert_eq!(centered.width, 20);
        assert_eq!(centered.height, 10);
    }

    #[test]
    fn center_in_area_truncates_if_too_large() {
        let area = Rect::new(0, 0, 20, 10);
        let centered = layout_helpers::center_in_area(area, 100, 100);
        assert_eq!(centered.width, 20);
        assert_eq!(centered.height, 10);
    }

    #[test]
    fn bottom_right_position() {
        let area = Rect::new(0, 0, 80, 24);
        let bottom = layout_helpers::bottom_right(area, 20, 10);
        assert_eq!(bottom.x, 60);
        assert_eq!(bottom.y, 14);
    }
}