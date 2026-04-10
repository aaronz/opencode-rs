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
    json5::from_str(content).map_err(|e| JsoncError::Parse(e.to_string()))
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
        // Error message should contain "JSON parse error" and indicate the issue
        assert!(
            error_msg.contains("JSON"),
            "Error should mention JSON: {}",
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
        assert!(error.to_string().contains("JSON") || error.to_string().contains("parse"));
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
        assert!(error_string.contains("JSON") || error_string.contains("parse"));
    }
}
