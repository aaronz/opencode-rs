use ratatui::buffer::{Buffer, Cell};
use ratatui::style::{Color, Modifier};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IgnoreOptions {
    pub ignore_foreground: bool,
    pub ignore_background: bool,
    pub ignore_attributes: bool,
}

impl IgnoreOptions {
    pub fn ignore_foreground(mut self) -> Self {
        self.ignore_foreground = true;
        self
    }

    pub fn ignore_background(mut self) -> Self {
        self.ignore_background = true;
        self
    }

    pub fn ignore_attributes(mut self) -> Self {
        self.ignore_attributes = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellDiff {
    pub x: u16,
    pub y: u16,
    pub expected: Cell,
    pub actual: Cell,
}

impl CellDiff {
    pub fn expected_symbol(&self) -> &str {
        self.expected.symbol()
    }

    pub fn actual_symbol(&self) -> &str {
        self.actual.symbol()
    }

    pub fn expected_foreground(&self) -> Color {
        self.expected.fg
    }

    pub fn actual_foreground(&self) -> Color {
        self.actual.fg
    }

    pub fn expected_background(&self) -> Color {
        self.expected.bg
    }

    pub fn actual_background(&self) -> Color {
        self.actual.bg
    }

    pub fn expected_modifier(&self) -> Modifier {
        self.expected.modifier
    }

    pub fn actual_modifier(&self) -> Modifier {
        self.actual.modifier
    }

    pub fn symbol(&self) -> (&str, &str) {
        (self.expected.symbol(), self.actual.symbol())
    }

    pub fn foreground(&self) -> (Color, Color) {
        (self.expected.fg, self.actual.fg)
    }

    pub fn background(&self) -> (Color, Color) {
        (self.expected.bg, self.actual.bg)
    }

    pub fn modifier(&self) -> (Modifier, Modifier) {
        (self.expected.modifier, self.actual.modifier)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub passed: bool,
    pub expected: Buffer,
    pub actual: Buffer,
    pub differences: Vec<CellDiff>,
    pub total_diffs: usize,
}

impl fmt::Display for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.passed {
            return write!(f, "Buffers are identical");
        }

        writeln!(f, "BufferDiff: {} difference(s) found", self.total_diffs)?;
        writeln!(f, "{}", "=".repeat(60))?;

        for diff in &self.differences {
            writeln!(f, "Position: ({}, {})", diff.x, diff.y)?;
            writeln!(f, "  Expected: {:?}", diff.expected_symbol().escape_debug())?;
            writeln!(f, "  Actual:   {:?}", diff.actual_symbol().escape_debug())?;

            if diff.expected_foreground() != diff.actual_foreground() {
                writeln!(
                    f,
                    "  Foreground: expected={:?}, actual={:?}",
                    diff.expected_foreground(),
                    diff.actual_foreground()
                )?;
            }
            if diff.expected_background() != diff.actual_background() {
                writeln!(
                    f,
                    "  Background: expected={:?}, actual={:?}",
                    diff.expected_background(),
                    diff.actual_background()
                )?;
            }
            if diff.expected_modifier() != diff.actual_modifier() {
                writeln!(
                    f,
                    "  Modifier: expected={:?}, actual={:?}",
                    diff.expected_modifier(),
                    diff.actual_modifier()
                )?;
            }
            writeln!(f, "{}", "-".repeat(40))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BufferDiff {
    options: IgnoreOptions,
}

impl BufferDiff {
    pub fn new() -> Self {
        Self {
            options: IgnoreOptions::default(),
        }
    }

    pub fn with_options(options: IgnoreOptions) -> Self {
        Self { options }
    }

    pub fn ignore_foreground(mut self) -> Self {
        self.options.ignore_foreground = true;
        self
    }

    pub fn ignore_background(mut self) -> Self {
        self.options.ignore_background = true;
        self
    }

    pub fn ignore_attributes(mut self) -> Self {
        self.options.ignore_attributes = true;
        self
    }

    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> DiffResult {
        let mut differences = Vec::new();

        let expected_area = expected.area;
        let actual_area = actual.area;

        if expected_area != actual_area {
            let mut expected_cell = Cell::default();
            expected_cell.set_symbol(&format!(
                "Buffer size: {}x{}",
                expected_area.width, expected_area.height
            ));
            let mut actual_cell = Cell::default();
            actual_cell.set_symbol(&format!(
                "Buffer size: {}x{}",
                actual_area.width, actual_area.height
            ));
            differences.push(CellDiff {
                x: 0,
                y: 0,
                expected: expected_cell,
                actual: actual_cell,
            });
            let total_diffs = differences.len();
            return DiffResult {
                passed: total_diffs == 0,
                expected: expected.clone(),
                actual: actual.clone(),
                total_diffs,
                differences,
            };
        }

        let width = expected_area.width as usize;
        let height = expected_area.height as usize;

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let expected_cell = &expected.content[idx];
                let actual_cell = &actual.content[idx];

                if self.cells_differ(expected_cell, actual_cell) {
                    differences.push(CellDiff {
                        x: x as u16,
                        y: y as u16,
                        expected: expected_cell.clone(),
                        actual: actual_cell.clone(),
                    });
                }
            }
        }

        DiffResult {
            passed: differences.is_empty(),
            expected: expected.clone(),
            actual: actual.clone(),
            total_diffs: differences.len(),
            differences,
        }
    }

    fn cells_differ(&self, expected: &Cell, actual: &Cell) -> bool {
        if expected.symbol() != actual.symbol() {
            return true;
        }

        if !self.options.ignore_foreground && expected.fg != actual.fg {
            return true;
        }

        if !self.options.ignore_background && expected.bg != actual.bg {
            return true;
        }

        if !self.options.ignore_attributes && expected.modifier != actual.modifier {
            return true;
        }

        false
    }

    pub fn diff_to_string(&self, expected: &Buffer, actual: &Buffer) -> String {
        let result = self.diff(expected, actual);
        result.to_string()
    }

    pub fn diff_str(&self, expected: &str, actual: &str) -> DiffResult {
        let expected_buffer = Self::parse_str_to_buffer(expected);
        let actual_buffer = Self::parse_str_to_buffer(actual);
        self.diff(&expected_buffer, &actual_buffer)
    }

    fn parse_str_to_buffer(content: &str) -> Buffer {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            let area = ratatui::layout::Rect::new(0, 0, 0, 0);
            return Buffer::empty(area);
        }

        let max_width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let height = lines.len() as u16;
        let area = ratatui::layout::Rect::new(0, 0, max_width, height);
        let mut buffer = Buffer::empty(area);

        for (y, line) in lines.iter().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let idx = y * max_width as usize + x;
                if idx < buffer.content.len() {
                    buffer.content[idx].set_symbol(c.to_string().as_str());
                }
            }
        }

        buffer
    }
}

impl Default for BufferDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn create_buffer_with_content(width: u16, height: u16, content: &[&str]) -> Buffer {
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
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(3, 2, &["abc", "def"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 0);
        assert!(result.differences.is_empty());
        assert!(result.passed);
    }

    #[test]
    fn test_diff_result_passed_when_no_diffs() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let buf2 = create_buffer_with_content(2, 1, &["ab"]);

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
        let buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let buf2 = create_buffer_with_content(2, 1, &["xx"]);

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
        let buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let buf2 = create_buffer_with_content(2, 1, &["xx"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(
            result.expected, buf1,
            "DiffResult.expected should match expected Buffer"
        );
    }

    #[test]
    fn test_diff_result_actual_reference() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let buf2 = create_buffer_with_content(2, 1, &["xx"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(
            result.actual, buf2,
            "DiffResult.actual should match actual Buffer"
        );
    }

    #[test]
    fn test_buffer_diff_different_symbols() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(3, 2, &["axc", "def"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 1);

        let diff_cell = &result.differences[0];
        assert_eq!(diff_cell.x, 1);
        assert_eq!(diff_cell.y, 0);
        assert_eq!(diff_cell.expected_symbol(), "b");
        assert_eq!(diff_cell.actual_symbol(), "x");
    }

    #[test]
    fn test_buffer_diff_position_reporting() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(5, 3, &["hello", "world", "test"]);
        let buf2 = create_buffer_with_content(5, 3, &["hello", "wxrld", "test"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 1);

        let diff_cell = &result.differences[0];
        assert_eq!(diff_cell.x, 1);
        assert_eq!(diff_cell.y, 1);
    }

    #[test]
    fn test_buffer_diff_ignore_foreground() {
        let diff = BufferDiff::new().ignore_foreground();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].fg = Color::Red;
        buf2.content[0].fg = Color::Blue;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_ignore_background() {
        let diff = BufferDiff::new().ignore_background();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].bg = Color::Red;
        buf2.content[0].bg = Color::Blue;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_ignore_attributes() {
        let diff = BufferDiff::new().ignore_attributes();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].modifier = Modifier::BOLD;
        buf2.content[0].modifier = Modifier::ITALIC;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_multiple_differences() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(3, 2, &["xbc", "daf"]);

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
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(4, 2, &["abcd", "defg"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 1);
        assert!(result.differences[0].expected_symbol().contains("3x2"));
        assert!(result.differences[0].actual_symbol().contains("4x2"));
    }

    #[test]
    fn test_buffer_diff_to_string() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let buf2 = create_buffer_with_content(2, 1, &["ax"]);

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

        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].fg = Color::Red;
        buf2.content[0].fg = Color::Blue;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_detects_single_cell_differences() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let mut buf2 = create_buffer_with_content(3, 2, &["abc", "def"]);

        buf2.content[3].set_symbol("x");

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(
            result.total_diffs, 1,
            "Should detect exactly one cell difference"
        );

        let cell_diff = &result.differences[0];
        assert_eq!(cell_diff.x, 0, "x position should be 0");
        assert_eq!(cell_diff.y, 1, "y position should be 1");
        assert_eq!(
            cell_diff.expected_symbol(),
            "d",
            "Expected symbol should be 'd'"
        );
        assert_eq!(
            cell_diff.actual_symbol(),
            "x",
            "Actual symbol should be 'x'"
        );
    }

    #[test]
    fn test_buffer_diff_calculates_correct_diff_statistics() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(4, 3, &["abcd", "efgh", "ijkl"]);
        let buf2 = create_buffer_with_content(4, 3, &["axcd", "efyh", "ijkl"]);

        let result = diff.diff(&buf1, &buf2);

        assert_eq!(
            result.total_diffs, 2,
            "Should calculate exactly 2 differences"
        );

        assert_eq!(
            result.differences.len(),
            2,
            "differences vec should have 2 entries"
        );

        let diff1 = &result.differences[0];
        assert_eq!(diff1.x, 1);
        assert_eq!(diff1.y, 0);
        assert_eq!(diff1.expected_symbol(), "b");
        assert_eq!(diff1.actual_symbol(), "x");

        let diff2 = &result.differences[1];
        assert_eq!(diff2.x, 2);
        assert_eq!(diff2.y, 1);
        assert_eq!(diff2.expected_symbol(), "g");
        assert_eq!(diff2.actual_symbol(), "y");
    }

    #[test]
    fn test_buffer_diff_single_cell_foreground_difference() {
        let diff = BufferDiff::new();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].fg = Color::Red;
        buf2.content[0].fg = Color::Blue;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(
            result.total_diffs, 1,
            "Should detect single cell foreground difference"
        );

        let cell_diff = &result.differences[0];
        assert_eq!(cell_diff.expected_foreground(), Color::Red);
        assert_eq!(cell_diff.actual_foreground(), Color::Blue);
        assert_eq!(cell_diff.expected_symbol(), "a");
        assert_eq!(cell_diff.actual_symbol(), "a");
    }

    #[test]
    fn test_buffer_diff_single_cell_background_difference() {
        let diff = BufferDiff::new();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].bg = Color::Black;
        buf2.content[0].bg = Color::White;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(
            result.total_diffs, 1,
            "Should detect single cell background difference"
        );

        let cell_diff = &result.differences[0];
        assert_eq!(cell_diff.expected_background(), Color::Black);
        assert_eq!(cell_diff.actual_background(), Color::White);
    }

    #[test]
    fn test_buffer_diff_single_cell_modifier_difference() {
        let diff = BufferDiff::new();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].modifier = Modifier::BOLD;
        buf2.content[0].modifier = Modifier::ITALIC;

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(
            result.total_diffs, 1,
            "Should detect single cell modifier difference"
        );

        let cell_diff = &result.differences[0];
        assert_eq!(cell_diff.expected_modifier(), Modifier::BOLD);
        assert_eq!(cell_diff.actual_modifier(), Modifier::ITALIC);
    }

    #[test]
    fn test_cell_diff_uses_ratatui_cell_type() {
        let diff = BufferDiff::new();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["b"]);

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
        let buf1 = create_buffer_with_content(5, 3, &["hello", "world", "test"]);
        let buf2 = create_buffer_with_content(5, 3, &["hello", "w4rld", "test"]);

        let result = diff.diff(&buf1, &buf2);
        assert_eq!(result.total_diffs, 1);

        let cell_diff = &result.differences[0];
        assert_eq!(cell_diff.x, 1, "x position should be accessible");
        assert_eq!(cell_diff.y, 1, "y position should be accessible");
    }

    #[test]
    fn test_cell_diff_human_readable_output() {
        let diff = BufferDiff::new();
        let mut buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let mut buf2 = create_buffer_with_content(2, 1, &["ax"]);

        buf1.content[0].fg = Color::Red;
        buf2.content[0].fg = Color::Blue;

        let result = diff.diff(&buf1, &buf2);
        let output = result.to_string();

        assert!(output.contains("BufferDiff"));
        assert!(output.contains("difference(s) found"));
        assert!(output.contains("Position: (1, 0)"));
        assert!(output.contains("Expected:"));
        assert!(output.contains("Actual:"));
        assert!(output.contains("'x'"));
    }

    #[test]
    fn test_cell_diff_helper_methods() {
        let diff = BufferDiff::new();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["b"]);

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
    fn test_diff_str_accepts_string_inputs() {
        let diff = BufferDiff::new();
        let expected = "hello";
        let actual = "hello";
        let result = diff.diff_str(expected, actual);
        assert!(result.passed);
    }

    #[test]
    fn test_diff_str_returns_passed_true_for_identical_content() {
        let diff = BufferDiff::new();
        let expected = "hello\nworld";
        let actual = "hello\nworld";
        let result = diff.diff_str(expected, actual);
        assert!(
            result.passed,
            "diff_str should return passed: true for identical content"
        );
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_diff_str_returns_passed_false_for_different_content() {
        let diff = BufferDiff::new();
        let expected = "hello";
        let actual = "world";
        let result = diff.diff_str(expected, actual);
        assert!(
            !result.passed,
            "diff_str should return passed: false for different content"
        );
        assert_eq!(
            result.total_diffs, 4,
            "Expected 4 differences between 'hello' and 'world'"
        );
    }

    #[test]
    fn test_diff_str_parses_multi_line_strings_correctly() {
        let diff = BufferDiff::new();
        let expected = "line1\nline2\nline3";
        let actual = "line1\nline2\nline4";
        let result = diff.diff_str(expected, actual);
        assert!(!result.passed);
        assert_eq!(result.total_diffs, 1);
        let cell_diff = &result.differences[0];
        assert_eq!(cell_diff.y, 2, "Difference should be on line 3 (y=2)");
        assert_eq!(cell_diff.x, 4, "Difference should be at char 4");
    }
}
