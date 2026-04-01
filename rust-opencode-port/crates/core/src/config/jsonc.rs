use serde_json::Value;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum JsoncError {
    #[error("JSON parse error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn parse_jsonc(content: &str) -> Result<Value, JsoncError> {
    let stripped = strip_jsonc_comments(content);
    serde_json::from_str(&stripped).map_err(|e| JsoncError::Parse(e.to_string()))
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
                    while let Some(ch) = chars.next() {
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
                    while let Some(ch) = chars.next() {
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
        let input = r#"
{
    // This is a comment
    "key": "value"
}
"#;
        let result = strip_jsonc_comments(input);
        assert!(!result.contains("// This is a comment"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_strip_multi_line_comments() {
        let input = r#"
{
    /* Multi
       line
       comment */
    "key": "value"
}
"#;
        let result = strip_jsonc_comments(input);
        assert!(!result.contains("Multi"));
        assert!(result.contains("\"key\": \"value\""));
    }

    #[test]
    fn test_parse_jsonc_with_comments() {
        let input = r#"
{
    // Leading comment
    "name": "test",
    /* Trailing comment */
    "enabled": true
}
"#;
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
}
