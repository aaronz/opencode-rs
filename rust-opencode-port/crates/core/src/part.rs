use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Part {
    Text(TextPart),
    Code(CodePart),
    ToolUse(ToolUsePart),
    ToolResult(ToolResultPart),
    FileReference(FileReferencePart),
    Image(ImagePart),
    Reasoning(ReasoningPart),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPart {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodePart {
    pub code: String,
    pub language: Option<String>,
    pub filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsePart {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultPart {
    pub tool_name: String,
    pub result: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReferencePart {
    pub path: String,
    pub content_preview: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePart {
    pub reference: String,
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningPart {
    pub trace: String,
}

impl Part {
    pub fn text<S: Into<String>>(text: S) -> Self {
        Part::Text(TextPart { text: text.into() })
    }

    pub fn code<S: Into<String>>(code: S, language: Option<&str>) -> Self {
        Part::Code(CodePart {
            code: code.into(),
            language: language.map(|s| s.to_string()),
            filename: None,
        })
    }

    pub fn tool_use<S: Into<String>>(tool_name: S, arguments: serde_json::Value) -> Self {
        Part::ToolUse(ToolUsePart {
            tool_name: tool_name.into(),
            arguments,
        })
    }

    pub fn tool_result<S: Into<String>>(
        tool_name: S,
        result: serde_json::Value,
        success: bool,
    ) -> Self {
        Part::ToolResult(ToolResultPart {
            tool_name: tool_name.into(),
            result,
            success,
        })
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Part::Text(p) => Some(&p.text),
            _ => None,
        }
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Part::Text(p) => write!(f, "{}", p.text),
            Part::Code(p) => write!(f, "{}", p.code),
            Part::ToolUse(p) => write!(f, "tool:{}", p.tool_name),
            Part::ToolResult(p) => write!(f, "result:{}", p.tool_name),
            Part::FileReference(p) => write!(f, "file:{}", p.path),
            Part::Image(_) => write!(f, "[image]"),
            Part::Reasoning(p) => write!(f, "{}", p.trace),
            Part::Unknown => write!(f, "[unknown part]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_text() {
        let part = Part::text("hello world");
        assert_eq!(part.as_text(), Some("hello world"));
    }

    #[test]
    fn test_part_code() {
        let part = Part::code("fn main() {}", Some("rust"));
        assert!(matches!(part, Part::Code(_)));
    }

    #[test]
    fn test_part_tool_use() {
        let args = serde_json::json!({"path": "/tmp"});
        let part = Part::tool_use("read_file", args);
        assert!(matches!(part, Part::ToolUse(_)));
    }

    #[test]
    fn test_part_serialization() {
        let part = Part::text("test");
        let json = serde_json::to_string(&part).unwrap();
        assert!(json.contains("\"type\":\"Text\""));
        assert!(json.contains("\"text\":\"test\""));
    }
}
