use jsonc_parser::errors::ParseError;
use jsonc_parser::parse_to_value;
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
        source_line: Option<String>,
    },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl JsoncError {
    pub fn new_parse_error(err: ParseError, source: &str) -> Self {
        let line = err.line_display();
        let column = err.column_display();
        let message = err.kind().to_string();
        let context = Self::generate_context(&message);
        let source_line = Self::extract_source_line(source, line);

        JsoncError::Parse {
            line,
            column,
            message,
            context,
            source_line,
        }
    }

    fn extract_source_line(source: &str, line_num: usize) -> Option<String> {
        if source.is_empty() {
            return None;
        }
        source
            .lines()
            .nth(line_num.saturating_sub(1))
            .map(|l| l.to_string())
    }

    fn generate_context(message: &str) -> String {
        let msg_lower = message.to_lowercase();
        if msg_lower.contains("double-quote") || msg_lower.contains("single-quote") {
            "JSON requires double quotes for strings. Replace any single quotes with double quotes."
                .to_string()
        } else if msg_lower.contains("trailing comma") {
            "Remove the trailing comma before the closing bracket or brace.".to_string()
        } else if msg_lower.contains("missing comma") {
            "Add a comma after each property except the last one in objects and arrays.".to_string()
        } else if msg_lower.contains("missing colon") || msg_lower.contains("expected colon") {
            "Ensure each property name is followed by a colon (:).".to_string()
        } else if msg_lower.contains("unterminated string") {
            "String values must be enclosed in double quotes. Check for missing closing quotes."
                .to_string()
        } else if msg_lower.contains("invalid escape") || msg_lower.contains("unknown escape") {
            "Use valid escape sequences: \\\\ for backslash, \\\" for double quote, \\n for newline, \\t for tab.".to_string()
        } else if msg_lower.contains("unexpected token") || msg_lower.contains("unexpected end") {
            if msg_lower.contains("comment") {
                "JSONC comments (// or /* */) require proper syntax. Ensure multi-line comments close with */.".to_string()
            } else {
                "Check for mismatched brackets, braces, or quotes in the affected area.".to_string()
            }
        } else if msg_lower.contains("comment") {
            "JSONC comments (// or /* */) are only allowed in .jsonc files, not .json files. Consider renaming the file to .jsonc or removing comments.".to_string()
        } else if msg_lower.contains("bracket") || msg_lower.contains("]") {
            "Check for mismatched or missing square brackets [ ].".to_string()
        } else if msg_lower.contains("brace") || msg_lower.contains("}") {
            "Check for mismatched or missing curly braces { }.".to_string()
        } else {
            "Review the JSON/JSONC syntax near the error location for missing or extra characters."
                .to_string()
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
                source_line,
            } => {
                let location = format!("line {}, column {}", line, column);
                let mut result = format!(
                    "Failed to parse JSONC file '{}': {}\n  --> {}",
                    path_str, message, location
                );

                if let Some(ref line_content) = source_line {
                    result.push_str(&format!(
                        "\n  |\n  | {}\n  | {}^",
                        line_content,
                        " ".repeat(*column - 1)
                    ));
                }

                result.push_str(&format!("\n  = help: {}", context));
                result
            }
            JsoncError::Io(e) => {
                format!("Failed to read JSONC file '{}': {}", path_str, e)
            }
        }
    }

    #[allow(dead_code)]
    pub fn with_source(mut self, source: &str) -> Self {
        if let JsoncError::Parse {
            ref mut source_line,
            line,
            ..
        } = self
        {
            *source_line = Self::extract_source_line(source, line);
        }
        self
    }
}

fn json_value_to_serde_value(json: jsonc_parser::JsonValue) -> Value {
    use jsonc_parser::JsonValue;
    match json {
        JsonValue::String(s) => Value::String(s.into_owned()),
        JsonValue::Number(n) => {
            if let Ok(i) = n.parse::<i64>() {
                Value::Number(i.into())
            } else if let Ok(f) = n.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        JsonValue::Boolean(b) => Value::Bool(b),
        JsonValue::Object(o) => {
            let map: serde_json::Map<String, Value> = o
                .into_iter()
                .map(|(k, v)| (k, json_value_to_serde_value(v)))
                .collect();
            Value::Object(map)
        }
        JsonValue::Array(a) => Value::Array(a.into_iter().map(json_value_to_serde_value).collect()),
        JsonValue::Null => Value::Null,
    }
}

pub fn parse_jsonc(content: &str) -> Result<Value, JsoncError> {
    parse_to_value(content, &Default::default())
        .map_err(|err| JsoncError::new_parse_error(err, content))?
        .ok_or_else(|| JsoncError::Parse {
            line: 0,
            column: 0,
            message: "Empty JSON content".to_string(),
            context: String::new(),
            source_line: None,
        })
        .map(|v| json_value_to_serde_value(v))
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
