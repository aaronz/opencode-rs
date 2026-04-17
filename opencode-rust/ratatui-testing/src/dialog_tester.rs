use ratatui::backend::TestBackend;
use ratatui::Terminal;

pub struct DialogRenderTester;

impl DialogRenderTester {
    pub fn new() -> Self {
        Self
    }

    pub fn with_backend(width: u16, height: u16) -> TestBackend {
        TestBackend::new(width, height)
    }

    pub fn terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        Terminal::new(Self::with_backend(width, height)).unwrap()
    }

    pub fn has_border(buffer: &ratatui::buffer::Buffer) -> bool {
        buffer
            .content
            .iter()
            .any(|cell| cell.symbol() == "─" || cell.symbol() == "│")
    }

    pub fn has_content(buffer: &ratatui::buffer::Buffer) -> bool {
        buffer.content.iter().any(|cell| cell.symbol() != " ")
    }

    pub fn count_lines_with_content(buffer: &ratatui::buffer::Buffer) -> usize {
        buffer
            .content
            .chunks(buffer.area.width as usize)
            .filter(|line| line.iter().any(|cell| cell.symbol() != " "))
            .count()
    }

    pub fn has_title(buffer: &ratatui::buffer::Buffer, title: &str) -> bool {
        let width = buffer.area.width as usize;
        let height = buffer.area.height as usize;

        for y in 1..height {
            let start = y * width;
            let end = (start + width).min(buffer.content.len());
            if start >= buffer.content.len() {
                break;
            }
            let line_text: String = buffer.content[start..end]
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim()
                .to_string();
            if line_text.contains(title) {
                return true;
            }
        }
        false
    }

    pub fn has_specific_content(buffer: &ratatui::buffer::Buffer, content: &str) -> bool {
        let width = buffer.area.width as usize;
        let height = buffer.area.height as usize;
        let buffer_width = width.min(buffer.content.len());

        for y in 0..height {
            let start = y * width;
            let end = (start + buffer_width).min(buffer.content.len());
            if start >= buffer.content.len() {
                break;
            }
            let line_text: String = buffer.content[start..end]
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim()
                .to_string();
            if line_text.contains(content) {
                return true;
            }
        }
        false
    }
}

impl Default for DialogRenderTester {
    fn default() -> Self {
        Self::new()
    }
}

pub fn assert_render_result(buffer: &ratatui::buffer::Buffer) {
    let has_border = DialogRenderTester::has_border(buffer);
    let has_content = DialogRenderTester::has_content(buffer);
    assert!(has_border, "Dialog should render with border");
    assert!(has_content, "Dialog should render with content");
}

pub fn assert_empty_state(buffer: &ratatui::buffer::Buffer) {
    let has_border = DialogRenderTester::has_border(buffer);
    assert!(has_border, "Empty dialog should still render border");
}
