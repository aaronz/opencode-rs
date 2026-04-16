use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui_testing::DialogRenderTester;

fn create_buffer(width: u16, height: u16, content: &[&str]) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);
    for (y, line) in content.iter().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let idx = y * width as usize + x;
            if idx < buffer.content.len() {
                buffer.content[idx].set_symbol(c.to_string().as_str());
            }
        }
    }
    buffer
}

#[test]
fn test_has_title_finds_matching_title() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(DialogRenderTester::has_title(&buffer, "Test Dialog"));
}

#[test]
fn test_has_title_returns_false_for_non_matching_title() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(!DialogRenderTester::has_title(&buffer, "Wrong Title"));
}

#[test]
fn test_has_title_partial_match() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   My Dialog Box  │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(DialogRenderTester::has_title(&buffer, "Dialog Box"));
}

#[test]
fn test_has_specific_content_finds_content() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(DialogRenderTester::has_specific_content(
        &buffer,
        "Content here"
    ));
}

#[test]
fn test_has_specific_content_returns_false_when_missing() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(!DialogRenderTester::has_specific_content(
        &buffer,
        "Not Present"
    ));
}

#[test]
fn test_has_specific_content_finds_partial_content() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Error: 404      │",
            "└──────────────────┘",
        ],
    );
    assert!(DialogRenderTester::has_specific_content(&buffer, "404"));
}

#[test]
fn test_has_border_detects_border() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(DialogRenderTester::has_border(&buffer));
}

#[test]
fn test_has_content_detects_content() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert!(DialogRenderTester::has_content(&buffer));
}

#[test]
fn test_count_lines_with_content_counts_correctly() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│   Test Dialog    │",
            "│                  │",
            "│   Content here   │",
            "└──────────────────┘",
        ],
    );
    assert_eq!(DialogRenderTester::count_lines_with_content(&buffer), 4);
}

#[test]
fn test_empty_buffer_has_no_content() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
        ],
    );
    assert!(!DialogRenderTester::has_content(&buffer));
}

#[test]
fn test_empty_buffer_has_no_title() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "                    ",
            "                    ",
            "                    ",
            "                    ",
            "                    ",
        ],
    );
    assert!(!DialogRenderTester::has_title(&buffer, "Anything"));
}
