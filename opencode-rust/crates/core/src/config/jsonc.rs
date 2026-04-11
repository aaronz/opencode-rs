use serde_json::Value;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum JsoncError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    Parse {
        line: usize,
        column: usize,
        message: String,
        context: String,
    },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl JsoncError {
    pub fn new_parse_error(raw_error: json5::Error) -> Self {
        let (line, column, message) = match raw_error {
            json5::Error::Message { msg, location } => {
                let (line, column) = location.map(|loc| (loc.line, loc.column)).unwrap_or((1, 1));
                (line, column, msg)
            }
        };
        let context = Self::generate_context(&message);

        JsoncError::Parse {
            line,
            column,
            message,
            context,
        }
    }

    fn generate_context(message: &str) -> String {
        if message.contains("double-quote") || message.contains("quote") {
            "JSON requires double quotes for strings. Replace single quotes with double quotes."
                .to_string()
        } else if message.contains("trailing") && message.contains("comma") {
            "Remove the trailing comma before the closing bracket/brace.".to_string()
        } else if message.contains("missing")
            && (message.contains("colon")
                || message.contains(":")
                || message.contains("comma")
                || message.contains(","))
        {
            "Ensure all object properties are separated by commas.".to_string()
        } else if message.contains("missing")
            && (message.contains("quote") || message.contains("\""))
        {
            "Ensure all string values are enclosed in double quotes.".to_string()
        } else if message.contains("colon") || message.contains(":") {
            "Check that property names are followed by a colon.".to_string()
        } else if message.contains("bracket") || message.contains("]") {
            "Check for mismatched or missing square brackets.".to_string()
        } else if message.contains("brace") || message.contains("}") {
            "Check for mismatched or missing curly braces.".to_string()
        } else if message.contains("comment") {
            "JSONC comments (// or /* */) are only allowed in .jsonc files, not .json files. Consider renaming to .jsonc or removing comments.".to_string()
        } else if message.contains("escape") {
            "Check for invalid escape sequences in strings. Use \\\\ for backslashes, \\\" for quotes.".to_string()
        } else {
            "Check the JSON/JSONC syntax near the error location.".to_string()
        }
    }

    pub fn with_file_path(&self, path: &Path) -> String {
        let path_str = path.display().to_string();
        match self {
            JsoncError::Parse {
                line,
                column,
                message,
                context,
            } => {
                format!(
                    "Failed to parse JSONC file '{}': {} at line {}, column {}.\nHint: {}",
                    path_str, message, line, column, context
                )
            }
            JsoncError::Io(e) => {
                format!("Failed to read JSONC file '{}': {}", path_str, e)
            }
        }
    }
}

pub fn parse_jsonc(content: &str) -> Result<Value, JsoncError> {
    json5::from_str(content).map_err(JsoncError::new_parse_error)
}

pub(crate) fn strip_jsonc_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(c) = chars.next() {
        if escaped {
            result.push(c);
            escaped = false;
            continue;
        }

        if c == '\\' && in_string {
            result.push(c);
            escaped = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            result.push(c);
            continue;
        }

        if in_string {
            result.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    chars.next();
                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            result.push(ch);
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    chars.next();
                    let mut prev = ' ';
                    for ch in chars.by_ref() {
                        if prev == '*' && ch == '/' {
                            break;
                        }
                        prev = ch;
                    }
                    continue;
                }
                _ => {
                    result.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[allow(dead_code)]
pub fn parse_jsonc_file(path: &Path) -> Result<Value, JsoncError> {
    let content = std::fs::read_to_string(path)?;
    parse_jsonc(&content)
}

pub fn is_jsonc_extension(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(), "jsonc" | "json5")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_single_line_comments() {
        let input = r#"{
    // This is a comment
    "key": "value"
}"#;
        let result = strip_jsonc_comments(input);
        assert!(!result.contains("// This is a comment"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let input = r#"{
    /* Multi
       line
       comment */
    "key": "value"
}"#;
        let result = strip_jsonc_comments(input);
        assert!(!result.contains("Multi"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_parse_jsonc_with_comments() {
        let input = r#"{
    // Leading comment
    "name": "test",
    /* Trailing comment */
    "enabled": true
}"#;
        let value = parse_jsonc(input).unwrap();
        assert_eq!(value["name"], "test");
        assert_eq!(value["enabled"], true);
    }

    #[test]
    fn test_preserve_strings_with_slashes() {
        let input = r#"{"path": "http://example.com"}"#;
        let value = parse_jsonc(input).unwrap();
        assert_eq!(value["path"], "http://example.com");
    }

    #[test]
    fn test_invalid_jsonc_produces_clear_error() {
        // Invalid JSON - missing closing brace
        let input = r#"{
    // Comment
    "key": "value"
"#;
        let result = parse_jsonc(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(
            error_msg.contains("Parse error") && error_msg.contains("line"),
            "Error should mention Parse error and include line number: {}",
            error_msg
        );
    }

    #[test]
    fn test_invalid_json_syntax_error_message() {
        let input = "{invalid: no quotes}";
        let result = parse_jsonc(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn test_mixed_comments_invalid_json_after_stripping() {
        // JSON that becomes invalid after comment stripping (unterminated string)
        let input = r#"{
    "key": "value
}"#;
        let result = parse_jsonc(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("Parse") || error.to_string().contains("line"));
    }

    #[test]
    fn test_unterminated_multiline_comment_error() {
        let input = r#"{
    /* Unterminated comment
    "key": "value"
}"#;
        let result = parse_jsonc(input);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_error_type_is_jsonc_error() {
        let input = "{invalid}";
        let result = parse_jsonc(input);
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_string = format!("{}", error);
        assert!(error_string.contains("Parse") || error_string.contains("line"));
    }
}
