use anyhow::Result;
use ratatui::buffer::{Buffer, Cell};
use ratatui::style::{Color, Modifier};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IgnoreOptions {
    pub ignore_foreground: bool,
    pub ignore_background: bool,
    pub ignore_attributes: bool,
}

impl Default for IgnoreOptions {
    fn default() -> Self {
        Self {
            ignore_foreground: false,
            ignore_background: false,
            ignore_attributes: false,
        }
    }
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
    pub expected_symbol: String,
    pub actual_symbol: String,
    pub expected_foreground: Option<Color>,
    pub actual_foreground: Option<Color>,
    pub expected_background: Option<Color>,
    pub actual_background: Option<Color>,
    pub expected_modifier: Option<Modifier>,
    pub actual_modifier: Option<Modifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffResult {
    pub differences: Vec<CellDiff>,
    pub total_diffs: usize,
}

impl fmt::Display for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.differences.is_empty() {
            return write!(f, "Buffers are identical");
        }

        writeln!(f, "BufferDiff: {} difference(s) found", self.total_diffs)?;
        writeln!(f, "{}", "=".repeat(60))?;

        for diff in &self.differences {
            writeln!(f, "Position: ({}, {})", diff.x, diff.y)?;
            writeln!(f, "  Expected: {:?}", diff.expected_symbol.escape_debug())?;
            writeln!(f, "  Actual:   {:?}", diff.actual_symbol.escape_debug())?;

            if diff.expected_foreground != diff.actual_foreground {
                writeln!(
                    f,
                    "  Foreground: expected={:?}, actual={:?}",
                    diff.expected_foreground, diff.actual_foreground
                )?;
            }
            if diff.expected_background != diff.actual_background {
                writeln!(
                    f,
                    "  Background: expected={:?}, actual={:?}",
                    diff.expected_background, diff.actual_background
                )?;
            }
            if diff.expected_modifier != diff.actual_modifier {
                writeln!(
                    f,
                    "  Modifier: expected={:?}, actual={:?}",
                    diff.expected_modifier, diff.actual_modifier
                )?;
            }
            writeln!(f, "{}", "-".repeat(40))?;
        }

        Ok(())
    }
}

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

    pub fn diff(&self, expected: &Buffer, actual: &Buffer) -> Result<DiffResult> {
        let mut differences = Vec::new();

        let expected_area = expected.area;
        let actual_area = actual.area;

        if expected_area != actual_area {
            differences.push(CellDiff {
                x: 0,
                y: 0,
                expected_symbol: format!(
                    "Buffer size: {}x{}",
                    expected_area.width, expected_area.height
                ),
                actual_symbol: format!("Buffer size: {}x{}", actual_area.width, actual_area.height),
                expected_foreground: None,
                actual_foreground: None,
                expected_background: None,
                actual_background: None,
                expected_modifier: None,
                actual_modifier: None,
            });
            return Ok(DiffResult {
                total_diffs: differences.len(),
                differences,
            });
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
                        expected_symbol: expected_cell.symbol().to_string(),
                        actual_symbol: actual_cell.symbol().to_string(),
                        expected_foreground: if self.options.ignore_foreground {
                            None
                        } else {
                            Some(expected_cell.fg)
                        },
                        actual_foreground: if self.options.ignore_foreground {
                            None
                        } else {
                            Some(actual_cell.fg)
                        },
                        expected_background: if self.options.ignore_background {
                            None
                        } else {
                            Some(expected_cell.bg)
                        },
                        actual_background: if self.options.ignore_background {
                            None
                        } else {
                            Some(actual_cell.bg)
                        },
                        expected_modifier: if self.options.ignore_attributes {
                            None
                        } else {
                            Some(expected_cell.modifier)
                        },
                        actual_modifier: if self.options.ignore_attributes {
                            None
                        } else {
                            Some(actual_cell.modifier)
                        },
                    });
                }
            }
        }

        Ok(DiffResult {
            total_diffs: differences.len(),
            differences,
        })
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

    pub fn diff_to_string(&self, expected: &Buffer, actual: &Buffer) -> Result<String> {
        let result = self.diff(expected, actual)?;
        Ok(result.to_string())
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

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 0);
        assert!(result.differences.is_empty());
    }

    #[test]
    fn test_buffer_diff_different_symbols() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(3, 2, &["axc", "def"]);

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 1);

        let diff_cell = &result.differences[0];
        assert_eq!(diff_cell.x, 1);
        assert_eq!(diff_cell.y, 0);
        assert_eq!(diff_cell.expected_symbol, "b");
        assert_eq!(diff_cell.actual_symbol, "x");
    }

    #[test]
    fn test_buffer_diff_position_reporting() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(5, 3, &["hello", "world", "test"]);
        let buf2 = create_buffer_with_content(5, 3, &["hello", "wxrld", "test"]);

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 1);

        let diff_cell = &result.differences[0];
        assert_eq!(diff_cell.x, 2);
        assert_eq!(diff_cell.y, 1);
    }

    #[test]
    fn test_buffer_diff_ignore_foreground() {
        let diff = BufferDiff::new().ignore_foreground();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].fg = Color::Red;
        buf2.content[0].fg = Color::Blue;

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_ignore_background() {
        let diff = BufferDiff::new().ignore_background();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].bg = Color::Red;
        buf2.content[0].bg = Color::Blue;

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_ignore_attributes() {
        let diff = BufferDiff::new().ignore_attributes();
        let mut buf1 = create_buffer_with_content(1, 1, &["a"]);
        let mut buf2 = create_buffer_with_content(1, 1, &["a"]);

        buf1.content[0].modifier = Modifier::BOLD;
        buf2.content[0].modifier = Modifier::ITALIC;

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 0);
    }

    #[test]
    fn test_buffer_diff_multiple_differences() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(3, 2, &["xbc", "daf"]);

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 2);

        assert_eq!(result.differences[0].x, 0);
        assert_eq!(result.differences[0].y, 0);
        assert_eq!(result.differences[1].x, 2);
        assert_eq!(result.differences[1].y, 1);
    }

    #[test]
    fn test_buffer_diff_different_sizes() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(3, 2, &["abc", "def"]);
        let buf2 = create_buffer_with_content(4, 2, &["abcd", "defg"]);

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 1);
        assert!(result.differences[0].expected_symbol.contains("3x2"));
        assert!(result.differences[0].actual_symbol.contains("4x2"));
    }

    #[test]
    fn test_buffer_diff_to_string() {
        let diff = BufferDiff::new();
        let buf1 = create_buffer_with_content(2, 1, &["ab"]);
        let buf2 = create_buffer_with_content(2, 1, &["ax"]);

        let output = diff.diff_to_string(&buf1, &buf2).unwrap();
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

        let result = diff.diff(&buf1, &buf2).unwrap();
        assert_eq!(result.total_diffs, 0);
    }
}
