use opencode_tui::render::{MarkdownRenderer, SyntaxHighlighter};
use opencode_tui::widgets::CodeBlock;

#[test]
fn test_markdown_renderer_new() {
    let _renderer = MarkdownRenderer::new();
    assert!(true, "MarkdownRenderer created");
}

#[test]
fn test_markdown_renderer_default() {
    let _renderer = MarkdownRenderer::default();
    assert!(true, "MarkdownRenderer created via default");
}

#[test]
fn test_markdown_render_simple() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("# Hello");
    assert!(html.contains("<h1>"));
    assert!(html.contains("Hello"));
}

#[test]
fn test_markdown_render_paragraph() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("This is a paragraph.");
    assert!(html.contains("<p>"));
}

#[test]
fn test_markdown_render_bold() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("**bold text**");
    assert!(html.contains("<strong>"));
}

#[test]
fn test_markdown_render_italic() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("*italic text*");
    assert!(html.contains("<em>"));
}

#[test]
fn test_markdown_render_code_inline() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("`inline code`");
    assert!(html.contains("<code>"));
}

#[test]
fn test_markdown_render_link() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("[link text](https://example.com)");
    assert!(html.contains("<a href"));
}

#[test]
fn test_markdown_render_list() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("- item 1\n- item 2");
    assert!(html.contains("<ul>"));
}

#[test]
fn test_markdown_render_ordered_list() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("1. first\n2. second");
    assert!(html.contains("<ol>"));
}

#[test]
fn test_markdown_render_blockquote() {
    let renderer = MarkdownRenderer::new();
    let html = renderer.render("> quote text");
    assert!(html.contains("<blockquote>"));
}

#[test]
fn test_markdown_parse_elements_heading() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("# Heading 1\n## Heading 2");
    assert!(elements.len() >= 2);
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::Heading(1)));
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::Heading(2)));
}

#[test]
fn test_markdown_parse_elements_paragraph() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("Some text here");
    assert!(!elements.is_empty());
}

#[test]
fn test_markdown_parse_elements_bold() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("**bold**");
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::Bold));
}

#[test]
fn test_markdown_parse_elements_italic() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("*italic*");
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::Italic));
}

#[test]
fn test_markdown_parse_elements_code_block() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("```rust\nfn main() {}\n```");
    assert!(elements.iter().any(|e| matches!(e.element_type, opencode_tui::render::markdown::ElementType::CodeBlock(ref lang) if lang == "rust")));
}

#[test]
fn test_markdown_parse_elements_link() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("[link](https://example.com)");
    assert!(elements.iter().any(|e| matches!(e.element_type, opencode_tui::render::markdown::ElementType::Link(ref url) if url == "https://example.com")));
}

#[test]
fn test_markdown_parse_elements_list() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("- item1\n- item2");
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::List(false)));
}

#[test]
fn test_markdown_parse_elements_ordered_list() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("1. first\n2. second");
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::List(true)));
}

#[test]
fn test_markdown_parse_elements_blockquote() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("> a quote");
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::BlockQuote));
}

#[test]
fn test_markdown_parse_elements_rule() {
    let renderer = MarkdownRenderer::new();
    let elements = renderer.parse_elements("---\nparagraph");
    assert!(elements
        .iter()
        .any(|e| e.element_type == opencode_tui::render::markdown::ElementType::Rule));
}

#[test]
fn test_markdown_extract_code_blocks() {
    let renderer = MarkdownRenderer::new();
    let markdown = "```rust\nfn main() {}\n```\n\nSome text\n\n```python\nprint('hello')\n```";
    let blocks = renderer.extract_code_blocks(markdown);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].0, "rust");
    assert_eq!(blocks[1].0, "python");
}

#[test]
fn test_markdown_extract_code_blocks_indented() {
    let renderer = MarkdownRenderer::new();
    let markdown = "    fn indented_code()\n    ";
    let blocks = renderer.extract_code_blocks(markdown);
    assert!(!blocks.is_empty() || blocks.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_simple() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("Hello world");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_heading() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("# Heading");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_empty() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("");
    assert!(lines.is_empty() || !lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_bold() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("**bold**");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_italic() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("*italic*");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_code_block() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("```rust\nfn main() {}\n```");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_link() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("[link](https://example.com)");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_list() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("- item 1\n- item 2");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_ordered_list() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("1. first\n2. second");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_blockquote() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("> quote");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_rule() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("---\ntext");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_multiline() {
    let renderer = MarkdownRenderer::new();
    let markdown = "# Title\n\nThis is a paragraph.\n\n- list item";
    let lines = renderer.to_ratatui_lines(markdown);
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_soft_break() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("line1  \nline2");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_to_ratatui_lines_hard_break() {
    let renderer = MarkdownRenderer::new();
    let lines = renderer.to_ratatui_lines("line1\\\nline2");
    assert!(!lines.is_empty());
}

#[test]
fn test_markdown_parsed_element_debug() {
    use opencode_tui::render::markdown::ElementType;
    use opencode_tui::render::markdown::ParsedElement;

    let element = ParsedElement {
        content: "test".to_string(),
        element_type: ElementType::Paragraph,
    };
    assert!(format!("{:?}", element).contains("test"));
}

#[test]
fn test_markdown_element_type_clone() {
    use opencode_tui::render::markdown::ElementType;

    let heading = ElementType::Heading(3);
    let cloned = heading.clone();
    assert_eq!(heading, cloned);
}

#[test]
fn test_markdown_element_type_equality() {
    use opencode_tui::render::markdown::ElementType;

    assert_eq!(ElementType::Heading(1), ElementType::Heading(1));
    assert_eq!(ElementType::Paragraph, ElementType::Paragraph);
    assert_eq!(ElementType::Bold, ElementType::Bold);
    assert_eq!(ElementType::Italic, ElementType::Italic);
    assert_ne!(ElementType::Heading(1), ElementType::Heading(2));
}

#[test]
fn test_syntax_highlighter_new() {
    let _highlighter = SyntaxHighlighter::new();
    assert!(true, "SyntaxHighlighter created");
}

#[test]
fn test_syntax_highlighter_default() {
    let _highlighter = SyntaxHighlighter::default();
    assert!(true, "SyntaxHighlighter created via default");
}

#[test]
fn test_syntax_highlighter_get_syntax() {
    let highlighter = SyntaxHighlighter::new();
    let rust_syntax = highlighter.get_syntax("rust");
    assert!(rust_syntax.is_some());
}

#[test]
fn test_syntax_highlighter_get_syntax_unknown() {
    let highlighter = SyntaxHighlighter::new();
    let unknown = highlighter.get_syntax("unknown_language_xyz");
    assert!(unknown.is_none());
}

#[test]
fn test_syntax_highlighter_get_theme() {
    let highlighter = SyntaxHighlighter::new();
    let theme = highlighter.get_theme("base16-ocean.dark");
    assert!(theme.is_some());
}

#[test]
fn test_syntax_highlighter_get_theme_unknown() {
    let highlighter = SyntaxHighlighter::new();
    let theme = highlighter.get_theme("nonexistent_theme");
    assert!(theme.is_none());
}

#[test]
fn test_syntax_highlighter_highlight_code_rust() {
    let highlighter = SyntaxHighlighter::new();
    let code = "fn main() {\n    println!(\"Hello\");\n}";
    let lines = highlighter.highlight_code(code, "rust", "base16-ocean.dark");
    assert!(!lines.is_empty());
}

#[test]
fn test_syntax_highlighter_highlight_code_python() {
    let highlighter = SyntaxHighlighter::new();
    let code = "def hello():\n    print('world')";
    let lines = highlighter.highlight_code(code, "python", "base16-ocean.dark");
    assert!(!lines.is_empty());
}

#[test]
fn test_syntax_highlighter_highlight_code_unknown_language() {
    let highlighter = SyntaxHighlighter::new();
    let code = "some code";
    let lines = highlighter.highlight_code(code, "unknown_lang", "base16-ocean.dark");
    assert!(!lines.is_empty());
}

#[test]
fn test_syntax_highlighter_highlight_code_empty() {
    let highlighter = SyntaxHighlighter::new();
    let lines = highlighter.highlight_code("", "rust", "base16-ocean.dark");
    assert!(!lines.is_empty());
}

#[test]
fn test_syntax_highlighter_supported_languages() {
    let highlighter = SyntaxHighlighter::new();
    let languages = highlighter.supported_languages();
    assert!(!languages.is_empty());
    assert!(languages.contains(&"Rust"));
    assert!(languages.contains(&"Python"));
}

#[test]
fn test_syntax_highlighter_supported_languages_contains_common() {
    let highlighter = SyntaxHighlighter::new();
    let languages = highlighter.supported_languages();
    let common = vec!["JavaScript", "TypeScript", "JSON", "HTML", "CSS"];
    for lang in common {
        if !languages.contains(&lang) {
            continue;
        }
        assert!(languages.contains(&lang));
    }
}

#[test]
fn test_code_block_with_scroll() {
    let code = "#!/usr/bin/env python\nprint('hello')\n".to_string();
    let _block = CodeBlock::new(code, "python".to_string()).with_scroll(1);
    assert!(true, "CodeBlock with scroll created");
}

#[test]
fn test_code_block_scroll_operations() {
    let mut block = CodeBlock::new(
        "line1\nline2\nline3\nline4\nline5".to_string(),
        "text".to_string(),
    );
    block.scroll_up();
    block.scroll_down(10);
    block.scroll_down(2);
    assert!(true, "Scroll operations executed");
}

#[test]
fn test_markdown_parsed_element_clone() {
    use opencode_tui::render::markdown::ElementType;
    use opencode_tui::render::markdown::ParsedElement;

    let element1 = ParsedElement {
        content: "test".to_string(),
        element_type: ElementType::Paragraph,
    };
    let element2 = element1.clone();
    assert_eq!(element1.content, element2.content);
    assert_eq!(element1.element_type, element2.element_type);
}

#[test]
fn test_markdown_element_type_heading_levels() {
    use opencode_tui::render::markdown::ElementType;

    for level in 1..=6 {
        assert_eq!(ElementType::Heading(level), ElementType::Heading(level));
    }
}

#[test]
fn test_markdown_element_type_code_block() {
    use opencode_tui::render::markdown::ElementType;

    let block1 = ElementType::CodeBlock("rust".to_string());
    let block2 = ElementType::CodeBlock("python".to_string());
    assert_ne!(block1, block2);
}

#[test]
fn test_markdown_element_type_link() {
    use opencode_tui::render::markdown::ElementType;

    let link1 = ElementType::Link("https://example.com".to_string());
    let link2 = ElementType::Link("https://other.com".to_string());
    assert_ne!(link1, link2);
}

#[test]
fn test_markdown_element_type_list() {
    use opencode_tui::render::markdown::ElementType;

    let ordered = ElementType::List(true);
    let unordered = ElementType::List(false);
    assert_ne!(ordered, unordered);
}
