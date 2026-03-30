#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Plain,
    FileRef,
    Shell,
    Command,
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InputType::Plain => "Plain",
            InputType::FileRef => "FileRef",
            InputType::Shell => "Shell",
            InputType::Command => "Command",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub input_type: InputType,
    pub raw: String,
    pub content: String,
    pub command: Option<String>,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidInput(String),
}

pub struct InputParser {
    max_content_length: usize,
}

impl InputParser {
    pub fn new() -> Self {
        Self {
            max_content_length: 100_000,
        }
    }

    pub fn with_max_length(max_length: usize) -> Self {
        Self {
            max_content_length: max_length,
        }
    }

    pub fn parse(&self, input: &str) -> ParseResult {
        let trimmed = input.trim_start();

        if trimmed.is_empty() {
            return ParseResult {
                input_type: InputType::Plain,
                raw: input.to_string(),
                content: input.to_string(),
                command: None,
                args: vec![],
            };
        }

        let first_char = trimmed.chars().next().unwrap_or(' ');

        if first_char == '@' {
            self.parse_file_ref(input, trimmed)
        } else if first_char == '!' {
            self.parse_shell(input, trimmed)
        } else if first_char == '/' {
            self.parse_command(input, trimmed)
        } else {
            ParseResult {
                input_type: InputType::Plain,
                raw: input.to_string(),
                content: input.to_string(),
                command: None,
                args: vec![],
            }
        }
    }

    fn parse_file_ref(&self, raw: &str, trimmed: &str) -> ParseResult {
        let content = trimmed[1..].trim_start().to_string();
        let content = if content.len() > self.max_content_length {
            content[..self.max_content_length].to_string()
        } else {
            content
        };

        ParseResult {
            input_type: InputType::FileRef,
            raw: raw.to_string(),
            content,
            command: None,
            args: vec![],
        }
    }

    fn parse_shell(&self, raw: &str, trimmed: &str) -> ParseResult {
        let content = trimmed[1..].trim_start().to_string();
        let content = if content.len() > self.max_content_length {
            content[..self.max_content_length].to_string()
        } else {
            content
        };

        ParseResult {
            input_type: InputType::Shell,
            raw: raw.to_string(),
            content,
            command: None,
            args: vec![],
        }
    }

    fn parse_command(&self, raw: &str, trimmed: &str) -> ParseResult {
        let content = trimmed[1..].trim_start().to_string();
        let parts: Vec<&str> = content.splitn(2, ' ').collect();
        let command = if parts[0].is_empty() {
            None
        } else {
            Some(parts[0].to_lowercase())
        };

        let args = if parts.len() > 1 {
            self.parse_args(parts[1])
        } else {
            vec![]
        };

        ParseResult {
            input_type: InputType::Command,
            raw: raw.to_string(),
            content,
            command,
            args,
        }
    }

    fn parse_args(&self, args_str: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';

        for c in args_str.chars() {
            if !in_quotes && (c == '"' || c == '\'') {
                in_quotes = true;
                quote_char = c;
            } else if in_quotes && c == quote_char {
                in_quotes = false;
                quote_char = ' ';
            } else if !in_quotes && c == ' ' {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            } else {
                current.push(c);
            }
        }

        if !current.is_empty() {
            args.push(current);
        }

        args
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> InputParser {
        InputParser::new()
    }

    #[test]
    fn test_plain_text() {
        let result = parser().parse("Hello world");
        assert_eq!(result.input_type, InputType::Plain);
        assert_eq!(result.raw, "Hello world");
        assert_eq!(result.content, "Hello world");
    }

    #[test]
    fn test_plain_text_with_leading_space() {
        let result = parser().parse("  Hello world");
        assert_eq!(result.input_type, InputType::Plain);
        assert_eq!(result.content, "  Hello world");
    }

    #[test]
    fn test_file_reference() {
        let result = parser().parse("@src/main.rs");
        assert_eq!(result.input_type, InputType::FileRef);
        assert_eq!(result.raw, "@src/main.rs");
        assert_eq!(result.content, "src/main.rs");
    }

    #[test]
    fn test_file_reference_with_space() {
        let result = parser().parse("@ src/main.rs");
        assert_eq!(result.input_type, InputType::FileRef);
        assert_eq!(result.content, "src/main.rs");
    }

    #[test]
    fn test_file_reference_empty() {
        let result = parser().parse("@");
        assert_eq!(result.input_type, InputType::FileRef);
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_shell_command() {
        let result = parser().parse("!ls -la");
        assert_eq!(result.input_type, InputType::Shell);
        assert_eq!(result.raw, "!ls -la");
        assert_eq!(result.content, "ls -la");
    }

    #[test]
    fn test_shell_command_with_space() {
        let result = parser().parse("! ls -la");
        assert_eq!(result.input_type, InputType::Shell);
        assert_eq!(result.content, "ls -la");
    }

    #[test]
    fn test_shell_command_empty() {
        let result = parser().parse("!");
        assert_eq!(result.input_type, InputType::Shell);
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_slash_command() {
        let result = parser().parse("/help");
        assert_eq!(result.input_type, InputType::Command);
        assert_eq!(result.raw, "/help");
        assert_eq!(result.content, "help");
        assert_eq!(result.command, Some("help".to_string()));
    }

    #[test]
    fn test_slash_command_uppercase() {
        let result = parser().parse("/HELP");
        assert_eq!(result.input_type, InputType::Command);
        assert_eq!(result.command, Some("help".to_string()));
    }

    #[test]
    fn test_slash_command_with_arg() {
        let result = parser().parse("/model gpt-4");
        assert_eq!(result.input_type, InputType::Command);
        assert_eq!(result.command, Some("model".to_string()));
        assert_eq!(result.args, vec!["gpt-4"]);
    }

    #[test]
    fn test_slash_command_empty() {
        let result = parser().parse("/");
        assert_eq!(result.input_type, InputType::Command);
        assert_eq!(result.command, None);
    }

    #[test]
    fn test_slash_command_with_quoted_arg() {
        let result = parser().parse("/command \"arg with spaces\"");
        assert_eq!(result.input_type, InputType::Command);
        assert_eq!(result.command, Some("command".to_string()));
        assert_eq!(result.args, vec!["arg with spaces"]);
    }

    #[test]
    fn test_empty_input() {
        let result = parser().parse("");
        assert_eq!(result.input_type, InputType::Plain);
        assert_eq!(result.raw, "");
        assert_eq!(result.content, "");
    }

    #[test]
    fn test_prefix_in_middle() {
        let result = parser().parse("Hello @world");
        assert_eq!(result.input_type, InputType::Plain);
    }
}
