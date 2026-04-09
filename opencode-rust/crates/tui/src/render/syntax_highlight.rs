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

        let theme = self
            .theme_set
            .themes
            .get(theme_name)
            .unwrap_or_else(|| self.theme_set.themes.get("base16-ocean.dark").unwrap());

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
