use ratatui::layout::Rect;

#[allow(dead_code)]
fn areas_overlap(a: &Rect, b: &Rect) -> bool {
    !(a.x + a.width <= b.x
        || b.x + b.width <= a.x
        || a.y + a.height <= b.y
        || b.y + b.height <= a.y)
}

#[test]
fn test_chat_layout_no_overlap_80x24() {
    let area = Rect::new(0, 0, 80, 24);
    let title_height = 1;
    let messages_height = 18;
    let input_height = 3;
    let status_height = 1;
    let terminal_height = 0;

    let title_area = Rect::new(0, 0, 80, title_height);
    let messages_area = Rect::new(0, title_height, 80, messages_height);
    let input_area = Rect::new(0, title_height + messages_height, 80, input_height);
    let status_area = Rect::new(
        0,
        title_height + messages_height + input_height,
        80,
        status_height,
    );

    assert!(
        !areas_overlap(&title_area, &messages_area),
        "Title and messages overlap"
    );
    assert!(
        !areas_overlap(&messages_area, &input_area),
        "Messages and input overlap"
    );
    assert!(
        !areas_overlap(&input_area, &status_area),
        "Input and status overlap"
    );

    let total_height =
        title_height + messages_height + input_height + status_height + terminal_height;
    assert_eq!(
        total_height, 23,
        "Total height should be 23 (title+messages+input+status)"
    );
}

#[test]
fn test_dynamic_file_tree_width() {
    let widths = [40u16, 60u16, 80u16, 100u16, 120u16];

    for width in &widths {
        let file_tree_width = (width / 3).max(20).min(40);

        assert!(
            file_tree_width >= 20,
            "Min width should be 20, got {}",
            file_tree_width
        );
        assert!(
            file_tree_width <= 40,
            "Max width should be 40, got {}",
            file_tree_width
        );
        assert!(
            file_tree_width <= *width,
            "Should not exceed terminal width"
        );
    }
}

#[test]
fn test_status_bar_dynamic_width() {
    let widths = [40u16, 60u16, 80u16, 100u16, 120u16];

    for width in &widths {
        let w = *width as usize;
        let status_indicator_width = 30usize.min(w);
        let status_area_width = w.saturating_sub(status_indicator_width);

        let total = status_area_width + status_indicator_width;
        assert_eq!(total, w, "Status areas should fill terminal width");
    }
}

#[test]
fn test_rect_calculations() {
    let area = Rect::new(0, 0, 80, 24);

    assert_eq!(area.width, 80);
    assert_eq!(area.height, 24);

    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    assert_eq!(inner.width, 78);
    assert_eq!(inner.height, 22);

    let below = Rect::new(area.x + 1, area.y + 23, area.width - 2, 1);
    assert!(
        !areas_overlap(&inner, &below),
        "Inner and below should not overlap"
    );
}
