use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use ratatui_testing::{BufferDiff, IgnoreOptions};

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
fn test_buffer_diff_identical_buffers() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(3, 2, &["abc", "def"]);
    let buf2 = create_buffer(3, 2, &["abc", "def"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0);
    assert!(result.differences.is_empty());
    assert!(result.passed);
}

#[test]
fn test_buffer_diff_different_symbols() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(3, 2, &["abc", "def"]);
    let buf2 = create_buffer(3, 2, &["axc", "def"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    assert_eq!(result.differences[0].x, 1);
    assert_eq!(result.differences[0].y, 0);
    assert_eq!(result.differences[0].expected_symbol(), "b");
    assert_eq!(result.differences[0].actual_symbol(), "x");
}

#[test]
fn test_buffer_diff_multiple_differences() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(3, 2, &["abc", "def"]);
    let buf2 = create_buffer(3, 2, &["xbc", "daf"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 2);
    assert_eq!(result.differences[0].x, 0);
    assert_eq!(result.differences[0].y, 0);
    assert_eq!(result.differences[1].x, 1);
    assert_eq!(result.differences[1].y, 1);
}

#[test]
fn test_buffer_diff_different_sizes() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(3, 2, &["abc", "def"]);
    let buf2 = create_buffer(4, 2, &["abcd", "defg"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    assert!(result.differences[0].expected_symbol().contains("3x2"));
    assert!(result.differences[0].actual_symbol().contains("4x2"));
}

#[test]
fn test_diff_result_passed_when_no_diffs() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["ab"]);
    let result = diff.diff(&buf1, &buf2);
    assert!(
        result.passed,
        "DiffResult.passed should be true when total_diffs == 0"
    );
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_diff_result_not_passed_when_diffs_exist() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["xx"]);
    let result = diff.diff(&buf1, &buf2);
    assert!(
        !result.passed,
        "DiffResult.passed should be false when total_diffs > 0"
    );
    assert_eq!(result.total_diffs, 2);
}

#[test]
fn test_diff_result_expected_reference() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["xx"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.expected, buf1);
}

#[test]
fn test_diff_result_actual_reference() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["xx"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.actual, buf2);
}

#[test]
fn test_buffer_diff_ignore_foreground() {
    let diff = BufferDiff::new().ignore_foreground();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].fg = Color::Red;
    buf2.content[0].fg = Color::Blue;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_buffer_diff_ignore_background() {
    let diff = BufferDiff::new().ignore_background();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].bg = Color::Red;
    buf2.content[0].bg = Color::Blue;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_buffer_diff_ignore_attributes() {
    let diff = BufferDiff::new().ignore_attributes();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].modifier = Modifier::BOLD;
    buf2.content[0].modifier = Modifier::ITALIC;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_cell_diff_uses_ratatui_cell_type() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["b"]);
    buf1.content[0].fg = Color::Red;
    buf1.content[0].bg = Color::Blue;
    buf1.content[0].modifier = Modifier::BOLD;
    buf2.content[0].fg = Color::Green;
    buf2.content[0].bg = Color::Yellow;
    buf2.content[0].modifier = Modifier::ITALIC;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected.symbol(), "a");
    assert_eq!(cell_diff.actual.symbol(), "b");
    assert_eq!(cell_diff.expected.fg, Color::Red);
    assert_eq!(cell_diff.actual.fg, Color::Green);
    assert_eq!(cell_diff.expected.bg, Color::Blue);
    assert_eq!(cell_diff.actual.bg, Color::Yellow);
    assert_eq!(cell_diff.expected.modifier, Modifier::BOLD);
    assert_eq!(cell_diff.actual.modifier, Modifier::ITALIC);
}

#[test]
fn test_cell_diff_position_accessible() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(5, 3, &["hello", "world", "test"]);
    let buf2 = create_buffer(5, 3, &["hello", "w4rld", "test"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.x, 1);
    assert_eq!(cell_diff.y, 1);
}

#[test]
fn test_cell_diff_single_cell_foreground_difference() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].fg = Color::Red;
    buf2.content[0].fg = Color::Blue;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected_foreground(), Color::Red);
    assert_eq!(cell_diff.actual_foreground(), Color::Blue);
    assert_eq!(cell_diff.expected_symbol(), "a");
    assert_eq!(cell_diff.actual_symbol(), "a");
}

#[test]
fn test_cell_diff_single_cell_background_difference() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].bg = Color::Black;
    buf2.content[0].bg = Color::White;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected_background(), Color::Black);
    assert_eq!(cell_diff.actual_background(), Color::White);
}

#[test]
fn test_cell_diff_single_cell_modifier_difference() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].modifier = Modifier::BOLD;
    buf2.content[0].modifier = Modifier::ITALIC;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected_modifier(), Modifier::BOLD);
    assert_eq!(cell_diff.actual_modifier(), Modifier::ITALIC);
}

#[test]
fn test_cell_diff_helper_methods() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["b"]);
    buf1.content[0].fg = Color::Red;
    buf1.content[0].bg = Color::Blue;
    buf1.content[0].modifier = Modifier::BOLD;
    buf2.content[0].fg = Color::Green;
    buf2.content[0].bg = Color::Yellow;
    buf2.content[0].modifier = Modifier::ITALIC;
    let result = diff.diff(&buf1, &buf2);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected_symbol(), "a");
    assert_eq!(cell_diff.actual_symbol(), "b");
    assert_eq!(cell_diff.expected_foreground(), Color::Red);
    assert_eq!(cell_diff.actual_foreground(), Color::Green);
    assert_eq!(cell_diff.expected_background(), Color::Blue);
    assert_eq!(cell_diff.actual_background(), Color::Yellow);
    assert_eq!(cell_diff.expected_modifier(), Modifier::BOLD);
    assert_eq!(cell_diff.actual_modifier(), Modifier::ITALIC);
    let (exp_sym, act_sym) = cell_diff.symbol();
    assert_eq!(exp_sym, "a");
    assert_eq!(act_sym, "b");
    let (exp_fg, act_fg) = cell_diff.foreground();
    assert_eq!(exp_fg, Color::Red);
    assert_eq!(act_fg, Color::Green);
    let (exp_bg, act_bg) = cell_diff.background();
    assert_eq!(exp_bg, Color::Blue);
    assert_eq!(act_bg, Color::Yellow);
    let (exp_mod, act_mod) = cell_diff.modifier();
    assert_eq!(exp_mod, Modifier::BOLD);
    assert_eq!(act_mod, Modifier::ITALIC);
}

#[test]
fn test_cell_diff_human_readable_output() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(2, 1, &["ab"]);
    let mut buf2 = create_buffer(2, 1, &["ax"]);
    buf1.content[0].fg = Color::Red;
    buf2.content[0].fg = Color::Blue;
    let result = diff.diff(&buf1, &buf2);
    let output = result.to_string();
    assert!(output.contains("BufferDiff"));
    assert!(output.contains("difference(s) found"));
    assert!(output.contains("Position: (1, 0)"));
    assert!(output.contains("Expected:"));
    assert!(output.contains("Actual:"));
}

#[test]
fn test_diff_str_identical_strings() {
    let diff = BufferDiff::new();
    let result = diff.diff_str("hello", "hello");
    assert!(result.passed);
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_diff_str_different_strings() {
    let diff = BufferDiff::new();
    let result = diff.diff_str("hello", "world");
    assert!(!result.passed);
    assert_eq!(result.total_diffs, 4);
}

#[test]
fn test_diff_str_multi_line_strings() {
    let diff = BufferDiff::new();
    let expected = "line1\nline2\nline3";
    let actual = "line1\nline2\nline4";
    let result = diff.diff_str(expected, actual);
    assert!(!result.passed);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.y, 2);
    assert_eq!(cell_diff.x, 4);
}

#[test]
fn test_diff_str_empty_strings() {
    let diff = BufferDiff::new();
    let result = diff.diff_str("", "");
    assert!(result.passed);
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_diff_str_one_empty() {
    let diff = BufferDiff::new();
    let result = diff.diff_str("", "content");
    assert!(!result.passed);
}

#[test]
fn test_diff_to_string() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["ax"]);
    let output = diff.diff_to_string(&buf1, &buf2);
    assert!(output.contains("1 difference"));
    assert!(output.contains("Position: (1, 0)"));
}

#[test]
fn test_ignore_options_builder() {
    let options = IgnoreOptions::default()
        .ignore_foreground()
        .ignore_background()
        .ignore_attributes();
    assert!(options.ignore_foreground);
    assert!(options.ignore_background);
    assert!(options.ignore_attributes);
}

#[test]
fn test_buffer_diff_with_options_struct() {
    let options = IgnoreOptions::default().ignore_foreground();
    let diff = BufferDiff::with_options(options);
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].fg = Color::Red;
    buf2.content[0].fg = Color::Blue;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0);
}

#[test]
fn test_buffer_diff_strict_comparison() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["a"]);
    let mut buf2 = create_buffer(1, 1, &["a"]);
    buf1.content[0].fg = Color::Red;
    buf2.content[0].fg = Color::Blue;
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 1);
    assert_eq!(result.differences[0].expected_foreground(), Color::Red);
    assert_eq!(result.differences[0].actual_foreground(), Color::Blue);
}

#[test]
fn test_diff_display_shows_buffers_identical() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["ab"]);
    let result = diff.diff(&buf1, &buf2);
    let output = result.to_string();
    assert!(output.contains("identical"));
}

#[test]
fn test_diff_display_shows_differences() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(2, 1, &["ab"]);
    let buf2 = create_buffer(2, 1, &["xx"]);
    let result = diff.diff(&buf1, &buf2);
    let output = result.to_string();
    assert!(output.contains("BufferDiff"));
    assert!(output.contains("2 difference(s)"));
}

#[test]
fn test_diff_str_longer_strings() {
    let diff = BufferDiff::new();
    let expected = "The quick brown fox";
    let actual = "The slow blue box";
    let result = diff.diff_str(expected, actual);
    assert!(!result.passed);
    assert!(result.total_diffs > 0);
}

#[test]
fn test_buffer_diff_position_exact() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(5, 3, &["abcde", "fghij", "klmno"]);
    let buf2 = create_buffer(5, 3, &["abcde", "fghij", "klmno"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.total_diffs, 0);
    assert!(result.passed);
}

#[test]
fn test_cell_diff_expected_actual_cells() {
    let diff = BufferDiff::new();
    let mut buf1 = create_buffer(1, 1, &["X"]);
    let mut buf2 = create_buffer(1, 1, &["Y"]);
    buf1.content[0].fg = Color::Cyan;
    buf2.content[0].fg = Color::Magenta;
    let result = diff.diff(&buf1, &buf2);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected.symbol(), "X");
    assert_eq!(cell_diff.actual.symbol(), "Y");
    assert_eq!(cell_diff.expected.fg, Color::Cyan);
    assert_eq!(cell_diff.actual.fg, Color::Magenta);
}

#[test]
fn test_diff_result_has_expected_and_actual_buffers() {
    let diff = BufferDiff::new();
    let buf1 = create_buffer(3, 1, &["abc"]);
    let buf2 = create_buffer(3, 1, &["xyz"]);
    let result = diff.diff(&buf1, &buf2);
    assert_eq!(result.expected.area.width, 3);
    assert_eq!(result.actual.area.width, 3);
    assert_eq!(result.expected.area.height, 1);
    assert_eq!(result.actual.area.height, 1);
}

#[test]
fn test_buffer_diff_zero_width_height() {
    let diff = BufferDiff::new();
    let buf1 = Buffer::empty(Rect::new(0, 0, 0, 0));
    let buf2 = Buffer::empty(Rect::new(0, 0, 0, 0));
    let result = diff.diff(&buf1, &buf2);
    assert!(result.passed);
}

#[test]
fn test_diff_str_single_char_diff() {
    let diff = BufferDiff::new();
    let result = diff.diff_str("a", "b");
    assert!(!result.passed);
    assert_eq!(result.total_diffs, 1);
    let cell_diff = &result.differences[0];
    assert_eq!(cell_diff.expected_symbol(), "a");
    assert_eq!(cell_diff.actual_symbol(), "b");
}
