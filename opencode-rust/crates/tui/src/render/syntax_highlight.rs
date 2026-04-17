use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SyntectColor, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

fn syntect_color_to_ratatui(c: SyntectColor) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        Self {
            syntax_set,
            theme_set,
        }
    }

    pub fn get_syntax(&self, lang: &str) -> Option<&syntect::parsing::SyntaxReference> {
        self.syntax_set.find_syntax_by_token(lang)
    }

    pub fn get_theme(&self, name: &str) -> Option<&syntect::highlighting::Theme> {
        self.theme_set.themes.get(name)
    }

    pub fn highlight_code(&self, code: &str, lang: &str, theme_name: &str) -> Vec<Line<'static>> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        #[expect(clippy::expect_used)]
        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .or_else(|| self.theme_set.themes.get("base16-ocean.dark"))
            .expect("at least the fallback theme 'base16-ocean.dark' should exist");

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut lines: Vec<Line<'static>> = Vec::new();

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            let spans: Vec<Span<'static>> = ranges
                .iter()
                .filter_map(|(style, text)| {
                    let fg = if style.foreground != SyntectColor::BLACK {
                        Some(syntect_color_to_ratatui(style.foreground))
                    } else {
                        None
                    };

                    let mut modifiers = Modifier::empty();
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::BOLD)
                    {
                        modifiers |= Modifier::BOLD;
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::ITALIC)
                    {
                        modifiers |= Modifier::ITALIC;
                    }
                    if style
                        .font_style
                        .contains(syntect::highlighting::FontStyle::UNDERLINE)
                    {
                        modifiers |= Modifier::UNDERLINED;
                    }

                    let mut ratatui_style = Style::default();
                    if let Some(color) = fg {
                        ratatui_style = ratatui_style.fg(color);
                    }
                    if !modifiers.is_empty() {
                        ratatui_style = ratatui_style.add_modifier(modifiers);
                    }

                    let text = text.trim_end_matches('\n').to_string();
                    if text.is_empty() {
                        None
                    } else {
                        Some(Span::styled(text, ratatui_style))
                    }
                })
                .collect();

            if spans.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(spans));
            }
        }

        lines
    }

    pub fn supported_languages(&self) -> Vec<&str> {
        self.syntax_set
            .syntaxes()
            .iter()
            .filter(|s| !s.hidden && !s.file_extensions.is_empty())
            .map(|s| s.name.as_str())
            .collect()
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntect_color_to_ratatui_black() {
        let syntect_color = SyntectColor::BLACK;
        let ratatui_color = syntect_color_to_ratatui(syntect_color);
        assert!(matches!(ratatui_color, Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn test_syntect_color_to_ratatui_white() {
        let syntect_color = SyntectColor::WHITE;
        let ratatui_color = syntect_color_to_ratatui(syntect_color);
        assert!(matches!(ratatui_color, Color::Rgb(255, 255, 255)));
    }

    #[test]
    fn test_syntax_highlighter_new() {
        let highlighter = SyntaxHighlighter::new();
        assert!(!highlighter.syntax_set.syntaxes().is_empty());
        assert!(!highlighter.theme_set.themes.is_empty());
    }

    #[test]
    fn test_syntax_highlighter_default() {
        let highlighter = SyntaxHighlighter::default();
        assert!(!highlighter.syntax_set.syntaxes().is_empty());
    }

    #[test]
    fn test_syntax_highlighter_get_syntax_rust() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.get_syntax("rust");
        assert!(syntax.is_some());
    }

    #[test]
    fn test_syntax_highlighter_get_syntax_javascript() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.get_syntax("javascript");
        assert!(syntax.is_some());
    }

    #[test]
    fn test_syntax_highlighter_get_syntax_nonexistent() {
        let highlighter = SyntaxHighlighter::new();
        let syntax = highlighter.get_syntax("nonexistent_language_xyz");
        assert!(syntax.is_none());
    }

    #[test]
    fn test_syntax_highlighter_get_theme() {
        let highlighter = SyntaxHighlighter::new();
        let theme = highlighter.get_theme("base16-ocean.dark");
        assert!(theme.is_some());
    }

    #[test]
    fn test_syntax_highlighter_get_theme_nonexistent() {
        let highlighter = SyntaxHighlighter::new();
        let theme = highlighter.get_theme("nonexistent_theme_xyz");
        assert!(theme.is_none());
    }

    #[test]
    fn test_syntax_highlighter_supported_languages() {
        let highlighter = SyntaxHighlighter::new();
        let languages = highlighter.supported_languages();
        assert!(!languages.is_empty());
        assert!(languages.iter().any(|l| *l == "Rust"));
        assert!(languages.iter().any(|l| *l == "JavaScript"));
    }

    #[test]
    fn test_syntax_highlighter_highlight_code() {
        let highlighter = SyntaxHighlighter::new();
        let code = "fn main() { println!(\"hello\"); }";
        let lines = highlighter.highlight_code(code, "rust", "base16-ocean.dark");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_syntax_highlighter_highlight_code_nonexistent_lang() {
        let highlighter = SyntaxHighlighter::new();
        let lines = highlighter.highlight_code("some code", "nonexistent", "base16-ocean.dark");
        assert!(!lines.is_empty());
    }
}
