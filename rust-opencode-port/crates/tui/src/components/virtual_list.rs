pub struct VirtualList {
    pub total_items: usize,
    pub visible_count: usize,
    pub scroll_offset: usize,
}

impl VirtualList {
    pub fn new(total_items: usize, visible_count: usize) -> Self {
        Self {
            total_items,
            visible_count,
            scroll_offset: 0,
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        let max_offset = self.total_items.saturating_sub(self.visible_count);
        self.scroll_offset = (self.scroll_offset + amount).min(max_offset);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_items.saturating_sub(self.visible_count);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.visible_count).min(self.total_items);
        (start, end)
    }

    pub fn update_total_items(&mut self, total: usize) {
        self.total_items = total;
        let max_offset = total.saturating_sub(self.visible_count);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_list_new() {
        let list = VirtualList::new(100, 10);
        assert_eq!(list.total_items, 100);
        assert_eq!(list.visible_count, 10);
        assert_eq!(list.scroll_offset, 0);
    }

    #[test]
    fn test_virtual_list_scroll() {
        let mut list = VirtualList::new(100, 10);
        list.scroll_down(5);
        assert_eq!(list.scroll_offset, 5);

        list.scroll_up(2);
        assert_eq!(list.scroll_offset, 3);
    }

    #[test]
    fn test_virtual_list_bounds() {
        let mut list = VirtualList::new(20, 10);
        list.scroll_down(100);
        assert_eq!(list.scroll_offset, 10);
    }

    #[test]
    fn test_virtual_list_visible_range() {
        let mut list = VirtualList::new(100, 10);
        list.scroll_down(50);
        let (start, end) = list.visible_range();
        assert_eq!(start, 50);
        assert_eq!(end, 60);
    }
}
