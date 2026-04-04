use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputToken {
    Text(String),
    FileRef(PathBuf),
    ShellCommand(String),
    SlashCommand { name: String, args: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputResult {
    pub tokens: Vec<InputToken>,
    pub raw: String,
    pub has_files: bool,
    pub has_shell: bool,
    pub has_command: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InputParser;

impl InputParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, input: &str) -> InputResult {
        if let Some(command) = self.parse_slash_command(input) {
            return InputResult {
                tokens: vec![command],
                raw: input.to_string(),
                has_files: false,
                has_shell: false,
                has_command: true,
            };
        }

        let mut tokens = Vec::new();
        let mut text_buf = String::new();
        let mut has_files = false;
        let mut has_shell = false;

        let chars: Vec<(usize, char)> = input.char_indices().collect();
        let mut i = 0;

        while i < chars.len() {
            let (idx, ch) = chars[i];

            if ch == '\\' {
                if i + 1 < chars.len() {
                    let next = chars[i + 1].1;
                    if matches!(next, '@' | '!' | '/' | '\\') {
                        text_buf.push(next);
                        i += 2;
                        continue;
                    }
                }
                text_buf.push(ch);
                i += 1;
                continue;
            }

            let token_start =
                idx == 0 || input[..idx].chars().last().is_some_and(char::is_whitespace);

            if token_start && ch == '@' {
                let end = find_token_end(&chars, i + 1, input.len());
                let path_raw = input[chars[i].0 + 1..end].trim();
                if !path_raw.is_empty() {
                    if !text_buf.is_empty() {
                        tokens.push(InputToken::Text(text_buf.clone()));
                        text_buf.clear();
                    }
                    has_files = true;
                    tokens.push(InputToken::FileRef(PathBuf::from(unescape(path_raw))));
                    i = index_after_byte(&chars, end);
                    continue;
                }
            }

            if token_start && ch == '!' {
                let command = input[idx + 1..].trim_start().to_string();
                if !text_buf.is_empty() {
                    tokens.push(InputToken::Text(text_buf.clone()));
                    text_buf.clear();
                }
                tokens.push(InputToken::ShellCommand(command));
                has_shell = true;
                break;
            }

            text_buf.push(ch);
            i += 1;
        }

        if !text_buf.is_empty() {
            tokens.push(InputToken::Text(text_buf));
        }

        InputResult {
            tokens,
            raw: input.to_string(),
            has_files,
            has_shell,
            has_command: false,
        }
    }

    fn parse_slash_command(&self, input: &str) -> Option<InputToken> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('/') {
            return None;
        }

        if input.starts_with("\\/") {
            return None;
        }

        let body = trimmed.trim_start_matches('/');
        let mut parts = body.splitn(2, char::is_whitespace);
        let name = parts.next().unwrap_or_default().trim().to_string();
        let args = parts.next().unwrap_or_default().trim().to_string();
        if name.is_empty() {
            return None;
        }

        Some(InputToken::SlashCommand { name, args })
    }
}

fn find_token_end(chars: &[(usize, char)], start: usize, default_end: usize) -> usize {
    for (byte_idx, ch) in chars.iter().skip(start) {
        if ch.is_whitespace() {
            return *byte_idx;
        }
    }
    default_end
}

fn index_after_byte(chars: &[(usize, char)], byte_offset: usize) -> usize {
    chars
        .iter()
        .position(|(idx, _)| *idx >= byte_offset)
        .unwrap_or(chars.len())
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut escaped = false;
    for ch in text.chars() {
        if escaped {
            out.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            out.push(ch);
        }
    }
    if escaped {
        out.push('\\');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_text() {
        let parser = InputParser::new();
        let result = parser.parse("hello world");
        assert_eq!(
            result.tokens,
            vec![InputToken::Text("hello world".to_string())]
        );
        assert!(!result.has_files);
        assert!(!result.has_shell);
        assert!(!result.has_command);
    }

    #[test]
    fn parses_multi_file_refs_and_text() {
        let parser = InputParser::new();
        let result = parser.parse("check @src/lib.rs and @README.md now");
        assert_eq!(
            result.tokens,
            vec![
                InputToken::Text("check ".to_string()),
                InputToken::FileRef(PathBuf::from("src/lib.rs")),
                InputToken::Text(" and ".to_string()),
                InputToken::FileRef(PathBuf::from("README.md")),
                InputToken::Text(" now".to_string())
            ]
        );
        assert!(result.has_files);
    }

    #[test]
    fn parses_shell_as_rest_of_input() {
        let parser = InputParser::new();
        let result = parser.parse("run !cargo test --workspace");
        assert_eq!(
            result.tokens,
            vec![
                InputToken::Text("run ".to_string()),
                InputToken::ShellCommand("cargo test --workspace".to_string())
            ]
        );
        assert!(result.has_shell);
    }

    #[test]
    fn parses_slash_command() {
        let parser = InputParser::new();
        let result = parser.parse("/model openai/gpt-4.1");
        assert_eq!(
            result.tokens,
            vec![InputToken::SlashCommand {
                name: "model".to_string(),
                args: "openai/gpt-4.1".to_string()
            }]
        );
        assert!(result.has_command);
    }

    #[test]
    fn escaped_tokens_stay_text() {
        let parser = InputParser::new();
        let result = parser.parse("\\@file \\!ls \\/help");
        assert_eq!(
            result.tokens,
            vec![InputToken::Text("@file !ls /help".to_string())]
        );
    }

    #[test]
    fn edge_empty_input() {
        let parser = InputParser::new();
        let result = parser.parse("");
        assert!(result.tokens.is_empty());
    }
}
