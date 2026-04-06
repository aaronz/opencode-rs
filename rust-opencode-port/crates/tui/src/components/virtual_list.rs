pub struct VirtualList {
    pub total_items: usize,
    pub visible_count: usize,
    pub scroll_offset: usize,
    item_heights: Vec<usize>,
    estimated_item_height: usize,
}

impl VirtualList {
    pub fn new(total_items: usize, visible_count: usize) -> Self {
        let estimated_item_height = 1;
        let item_heights = vec![estimated_item_height; total_items];

        Self {
            total_items,
            visible_count,
            scroll_offset: 0,
            item_heights,
            estimated_item_height,
        }
    }

    pub fn with_estimated_height(
        total_items: usize,
        visible_count: usize,
        estimated_height: usize,
    ) -> Self {
        let item_heights = vec![estimated_height; total_items];

        Self {
            total_items,
            visible_count,
            scroll_offset: 0,
            item_heights,
            estimated_item_height: estimated_height,
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

    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_count);
    }

    pub fn page_down(&mut self) {
        let max_offset = self.total_items.saturating_sub(self.visible_count);
        self.scroll_offset = (self.scroll_offset + self.visible_count).min(max_offset);
    }

    pub fn visible_range(&self) -> (usize, usize) {
        let start = self.scroll_offset;
        let end = (start + self.visible_count).min(self.total_items);
        (start, end)
    }

    pub fn update_total_items(&mut self, total: usize) {
        let old_len = self.item_heights.len();
        self.total_items = total;

        if total > old_len {
            self.item_heights.resize(total, self.estimated_item_height);
        } else if total < old_len {
            self.item_heights.truncate(total);
        }

        let max_offset = total.saturating_sub(self.visible_count);
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    pub fn set_item_height(&mut self, index: usize, height: usize) {
        if index < self.item_heights.len() {
            self.item_heights[index] = height;
        }
    }

    pub fn get_total_height(&self) -> usize {
        self.item_heights.iter().sum()
    }

    pub fn get_visible_height(&self) -> usize {
        let (start, end) = self.visible_range();
        self.item_heights[start..end].iter().sum()
    }

    pub fn scroll_to_item(&mut self, index: usize) {
        if index >= self.total_items {
            return;
        }

        let current_top = self.scroll_offset;
        let current_bottom = current_top + self.visible_count;

        if index < current_top {
            self.scroll_offset = index;
        } else if index >= current_bottom {
            self.scroll_offset = index.saturating_sub(self.visible_count - 1);
        }
    }

    pub fn is_item_visible(&self, index: usize) -> bool {
        let (start, end) = self.visible_range();
        index >= start && index < end
    }

    pub fn can_scroll_up(&self) -> bool {
        self.scroll_offset > 0
    }

    pub fn can_scroll_down(&self) -> bool {
        self.scroll_offset < self.total_items.saturating_sub(self.visible_count)
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

    #[test]
    fn test_page_scroll() {
        let mut list = VirtualList::new(100, 10);
        list.scroll_down(5);
        list.page_down();
        assert_eq!(list.scroll_offset, 15);
        list.page_up();
        assert_eq!(list.scroll_offset, 5);
    }

    #[test]
    fn test_scroll_to_item() {
        let mut list = VirtualList::new(100, 10);

        list.scroll_to_item(50);
        assert!(list.scroll_offset <= 50);
        assert!(list.scroll_offset + list.visible_count > 50);

        list.scroll_to_item(5);
        assert!(list.scroll_offset <= 5);

        list.scroll_to_item(95);
        let max_offset = 100usize.saturating_sub(10);
        assert!(list.scroll_offset <= max_offset);
    }

    #[test]
    fn test_item_visibility() {
        let list = VirtualList::new(100, 10);
        assert!(list.is_item_visible(5));
        assert!(!list.is_item_visible(15));
    }

    #[test]
    fn test_update_total_items() {
        let mut list = VirtualList::new(10, 5);
        list.update_total_items(20);
        assert_eq!(list.total_items, 20);
        assert_eq!(list.item_heights.len(), 20);

        list.update_total_items(5);
        assert_eq!(list.total_items, 5);
        assert_eq!(list.scroll_offset, 0);
    }
}
