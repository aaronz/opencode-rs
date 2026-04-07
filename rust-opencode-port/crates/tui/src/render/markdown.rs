use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub struct MarkdownRenderer {
    options: Options,
}

#[derive(Debug, Clone)]
pub struct ParsedElement {
    pub content: String,
    pub element_type: ElementType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Paragraph,
    Heading(u8),
    Bold,
    Italic,
    CodeBlock(String),
    Link(String),
    List(bool),
    BlockQuote,
    Rule,
}

#[derive(Debug, Clone, PartialEq)]
enum ParseState {
    Paragraph,
    Heading(u8),
    Bold,
    Italic,
    #[allow(dead_code)]
    BoldItalic,
    CodeBlock(String),
    Link(String),
    List(bool),
    BlockQuote,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let options = Options::all();
        Self { options }
    }

    pub fn render(&self, markdown: &str) -> String {
        let parser = Parser::new_ext(markdown, self.options);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        html_output
    }

    fn heading_level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
        match level {
            pulldown_cmark::HeadingLevel::H1 => 1,
            pulldown_cmark::HeadingLevel::H2 => 2,
            pulldown_cmark::HeadingLevel::H3 => 3,
            pulldown_cmark::HeadingLevel::H4 => 4,
            pulldown_cmark::HeadingLevel::H5 => 5,
            pulldown_cmark::HeadingLevel::H6 => 6,
        }
    }

    pub fn parse_elements(&self, markdown: &str) -> Vec<ParsedElement> {
        let mut elements = Vec::new();

        for event in Parser::new_ext(markdown, self.options) {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::Heading(Self::heading_level_to_u8(level)),
                    });
                }
                Event::End(TagEnd::Heading(_)) => {}
                Event::Start(Tag::Strong) => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::Bold,
                    });
                }
                Event::End(TagEnd::Strong) => {}
                Event::Start(Tag::Emphasis) => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::Italic,
                    });
                }
                Event::End(TagEnd::Emphasis) => {}
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::CodeBlock(lang),
                    });
                }
                Event::End(TagEnd::CodeBlock) => {}
                Event::Start(Tag::Link { dest_url, .. }) => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::Link(dest_url.to_string()),
                    });
                }
                Event::End(TagEnd::Link) => {}
                Event::Start(Tag::List(ordered)) => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::List(ordered.is_some()),
                    });
                }
                Event::End(TagEnd::List(_)) => {}
                Event::Start(Tag::BlockQuote(_)) => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::BlockQuote,
                    });
                }
                Event::End(TagEnd::BlockQuote(_)) => {}
                Event::Rule => {
                    elements.push(ParsedElement {
                        content: String::new(),
                        element_type: ElementType::Rule,
                    });
                }
                Event::Text(text) => {
                    if let Some(last) = elements.last_mut() {
                        last.content.push_str(&text);
                    }
                }
                _ => {}
            }
        }

        elements
    }

    pub fn extract_code_blocks(&self, markdown: &str) -> Vec<(String, String)> {
        let mut blocks = Vec::new();
        let mut current_lang = String::new();
        let mut current_code = String::new();
        let mut in_code = false;

        for event in Parser::new_ext(markdown, self.options) {
            match event {
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code = true;
                    current_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    current_code.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !current_code.is_empty() {
                        blocks.push((current_lang.clone(), current_code.clone()));
                    }
                    in_code = false;
                }
                Event::Text(text) if in_code => {
                    current_code.push_str(&text);
                }
                _ => {}
            }
        }

        blocks
    }

    pub fn to_ratatui_lines(&self, markdown: &str) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut state_stack: Vec<ParseState> = vec![ParseState::Paragraph];
        let mut current_text = String::new();
        let mut list_counter: u32 = 0;

        let parser = Parser::new_ext(markdown, self.options);

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    if !current_text.is_empty() {
                        lines.push(self.finalize_current_text(
                            &state_stack,
                            &mut current_text,
                            None,
                        ));
                    }
                    state_stack.push(ParseState::Heading(Self::heading_level_to_u8(level)));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if !current_text.is_empty() {
                        lines.push(self.finalize_current_text(
                            &state_stack,
                            &mut current_text,
                            None,
                        ));
                    }
                    state_stack.pop();
                }
                Event::Start(Tag::Strong) => {
                    state_stack.push(ParseState::Bold);
                }
                Event::End(TagEnd::Strong) => {
                    state_stack.pop();
                }
                Event::Start(Tag::Emphasis) => {
                    state_stack.push(ParseState::Italic);
                }
                Event::End(TagEnd::Emphasis) => {
                    state_stack.pop();
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    if !current_text.is_empty() {
                        lines.push(self.finalize_current_text(
                            &state_stack,
                            &mut current_text,
                            None,
                        ));
                    }
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    state_stack.push(ParseState::CodeBlock(lang));
                }
                Event::End(TagEnd::CodeBlock) => {
                    if !current_text.is_empty() {
                        lines.push(self.finalize_current_text(
                            &state_stack,
                            &mut current_text,
                            None,
                        ));
                    }
                    state_stack.pop();
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    state_stack.push(ParseState::Link(dest_url.to_string()));
                }
                Event::End(TagEnd::Link) => {
                    state_stack.pop();
                }
                Event::Start(Tag::List(ordered)) => {
                    list_counter = 0;
                    state_stack.push(ParseState::List(ordered.is_some()));
                }
                Event::End(TagEnd::List(_)) => {
                    state_stack.pop();
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    state_stack.push(ParseState::BlockQuote);
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    state_stack.pop();
                }
                Event::Start(Tag::Paragraph) => {
                    state_stack.push(ParseState::Paragraph);
                }
                Event::End(TagEnd::Paragraph) => {
                    if !current_text.is_empty() {
                        let counter = if matches!(state_stack.last(), Some(ParseState::List(true)))
                        {
                            list_counter += 1;
                            Some(list_counter)
                        } else {
                            None
                        };
                        lines.push(self.finalize_current_text(
                            &state_stack,
                            &mut current_text,
                            counter,
                        ));
                    }
                    state_stack.pop();
                }
                Event::Text(text) => {
                    current_text.push_str(&text);
                }
                Event::Code(code) => {
                    current_text.push_str(&code);
                }
                Event::SoftBreak => {
                    current_text.push(' ');
                }
                Event::HardBreak => {
                    if !current_text.is_empty() {
                        let counter = if matches!(state_stack.last(), Some(ParseState::List(true)))
                        {
                            list_counter += 1;
                            Some(list_counter)
                        } else {
                            None
                        };
                        lines.push(self.finalize_current_text(
                            &state_stack,
                            &mut current_text,
                            counter,
                        ));
                    }
                }
                Event::Rule => {
                    lines.push(Line::from(vec![Span::styled(
                        "─".repeat(40),
                        Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
                    )]));
                }
                _ => {}
            }
        }

        if !current_text.is_empty() {
            let counter = if matches!(state_stack.last(), Some(ParseState::List(true))) {
                list_counter += 1;
                Some(list_counter)
            } else {
                None
            };
            lines.push(self.finalize_current_text(&state_stack, &mut current_text, counter));
        }

        lines
    }

    fn finalize_current_text(
        &self,
        state_stack: &[ParseState],
        text: &mut String,
        list_number: Option<u32>,
    ) -> Line<'static> {
        let t = std::mem::take(text);
        if t.is_empty() {
            return Line::from("");
        }

        let current_state = state_stack.last().cloned().unwrap_or(ParseState::Paragraph);

        match current_state {
            ParseState::Heading(level) => {
                let prefix = match level {
                    1 => "# ",
                    2 => "## ",
                    3 => "### ",
                    4 => "#### ",
                    5 => "##### ",
                    _ => "###### ",
                };
                Line::from(vec![Span::styled(
                    format!("{}{}", prefix, t.trim()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )])
            }
            ParseState::Bold => Line::from(vec![Span::styled(
                t,
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            ParseState::Italic => Line::from(vec![Span::styled(
                t,
                Style::default().add_modifier(Modifier::ITALIC),
            )]),
            ParseState::BoldItalic => Line::from(vec![Span::styled(
                t,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            )]),
            ParseState::CodeBlock(ref lang) => {
                let lang_label = if lang.is_empty() {
                    "code".to_string()
                } else {
                    lang.clone()
                };
                let mut spans = vec![Span::styled(
                    format!("[{}] ", lang_label),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )];
                for line in t.lines() {
                    spans.push(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Green),
                    ));
                }
                Line::from(spans)
            }
            ParseState::Link(ref url) => Line::from(vec![
                Span::styled(
                    t,
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(
                    format!(" ({})", url),
                    Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
                ),
            ]),
            ParseState::List(ordered) => {
                let prefix = if ordered {
                    format!("{}. ", list_number.unwrap_or(1))
                } else {
                    "• ".to_string()
                };
                Line::from(vec![
                    Span::styled(
                        prefix,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(t.trim().to_string()),
                ])
            }
            ParseState::BlockQuote => Line::from(vec![
                Span::styled(
                    "│ ",
                    Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    t,
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]),
            ParseState::Paragraph => Line::from(vec![Span::raw(t)]),
        }
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}
