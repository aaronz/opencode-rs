use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui_testing::{assert_empty_state, assert_render_result, DialogRenderTester};

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
    assert_eq!(DialogRenderTester::count_lines_with_content(&buffer), 5);
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

#[test]
fn test_assert_render_result_with_valid_dialog() {
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
    assert_render_result(&buffer);
}

#[test]
fn test_assert_render_result_panics_without_border() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "                    ",
            "   Test Dialog      ",
            "                    ",
            "   Content here    ",
            "                    ",
        ],
    );
    let result = std::panic::catch_unwind(|| {
        assert_render_result(&buffer);
    });
    assert!(
        result.is_err(),
        "assert_render_result should panic when border is missing"
    );
}

#[test]
fn test_assert_render_result_panics_when_completely_empty() {
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
    let result = std::panic::catch_unwind(|| {
        assert_render_result(&buffer);
    });
    assert!(
        result.is_err(),
        "assert_render_result should panic when buffer is completely empty"
    );
}

#[test]
fn test_assert_empty_state_with_border_only() {
    let buffer = create_buffer(
        20,
        5,
        &[
            "┌──────────────────┐",
            "│                  │",
            "│                  │",
            "│                  │",
            "└──────────────────┘",
        ],
    );
    assert_empty_state(&buffer);
}

#[test]
fn test_assert_render_result_verifies_border_presence() {
    let buffer_with_border = create_buffer(
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
    assert!(
        DialogRenderTester::has_border(&buffer_with_border),
        "Buffer should have border"
    );
    assert_render_result(&buffer_with_border);

    let buffer_without_border = create_buffer(
        20,
        5,
        &[
            "                    ",
            "   Test Dialog      ",
            "                    ",
            "   Content here    ",
            "                    ",
        ],
    );
    assert!(
        !DialogRenderTester::has_border(&buffer_without_border),
        "Buffer should not have border"
    );
}

#[test]
fn test_assert_render_result_verifies_content_presence() {
    let buffer_with_content = create_buffer(
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
    assert!(
        DialogRenderTester::has_content(&buffer_with_content),
        "Buffer should have content"
    );
    assert_render_result(&buffer_with_content);

    let buffer_without_content = create_buffer(
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
    assert!(
        !DialogRenderTester::has_content(&buffer_without_content),
        "Buffer should not have content"
    );
}
