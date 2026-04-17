use serde::de::{self, IntoDeserializer, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Part {
    Text(TextPart),
    Code(CodePart),
    ToolUse(ToolUsePart),
    ToolResult(ToolResultPart),
    FileReference(FileReferencePart),
    Image(ImagePart),
    Reasoning(ReasoningPart),
    Unknown,
}

const PART_VARIANTS: &[&str] = &[
    "Text",
    "Code",
    "ToolUse",
    "ToolResult",
    "FileReference",
    "Image",
    "Reasoning",
];

impl<'de> Deserialize<'de> for Part {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Part", PART_VARIANTS, PartVisitor)
    }
}

struct PartVisitor;

impl<'de> Visitor<'de> for PartVisitor {
    type Value = Part;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a Part variant")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut type_variant: Option<String> = None;
        let mut data: Option<serde_json::Value> = None;

        while let Some(key) = map.next_key()? {
            match key {
                "type" => {
                    if type_variant.is_some() {
                        return Err(de::Error::custom("duplicate 'type' field"));
                    }
                    type_variant = Some(map.next_value()?);
                }
                "data" => {
                    if data.is_some() {
                        return Err(de::Error::custom("duplicate 'data' field"));
                    }
                    data = Some(map.next_value()?);
                }
                _ => {
                    return Err(de::Error::custom("unknown field"));
                }
            }
        }

        let type_variant = type_variant.ok_or_else(|| de::Error::custom("missing 'type' field"))?;
        let data = data.ok_or_else(|| de::Error::custom("missing 'data' field"))?;

        let deserializer = data.into_deserializer();
        match type_variant.as_str() {
            "Text" => {
                let part: TextPart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid Text data: {}", e)))?;
                Ok(Part::Text(part))
            }
            "Code" => {
                let part: CodePart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid Code data: {}", e)))?;
                Ok(Part::Code(part))
            }
            "ToolUse" => {
                let part: ToolUsePart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid ToolUse data: {}", e)))?;
                Ok(Part::ToolUse(part))
            }
            "ToolResult" => {
                let part: ToolResultPart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid ToolResult data: {}", e)))?;
                Ok(Part::ToolResult(part))
            }
            "FileReference" => {
                let part: FileReferencePart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid FileReference data: {}", e)))?;
                Ok(Part::FileReference(part))
            }
            "Image" => {
                let part: ImagePart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid Image data: {}", e)))?;
                Ok(Part::Image(part))
            }
            "Reasoning" => {
                let part: ReasoningPart = de::Deserialize::deserialize(deserializer)
                    .map_err(|e| de::Error::custom(format!("invalid Reasoning data: {}", e)))?;
                Ok(Part::Reasoning(part))
            }
            _ => Err(de::Error::custom(format!(
                "unknown part type: {}. Expected one of: {}",
                type_variant,
                PART_VARIANTS.join(", ")
            ))),
        }
    }
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

    #[test]
    fn test_part_deserialization() {
        let json = r#"{"type":"Text","data":{"text":"hello"}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert_eq!(part.as_text(), Some("hello"));
    }

    #[test]
    fn test_part_unknown_type_error() {
        let json = r#"{"type":"UnknownType","data":{"foo":"bar"}}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("unknown part type: UnknownType"));
    }

    #[test]
    fn test_part_missing_type_error() {
        let json = r#"{"data":{"text":"hello"}}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_missing_data_error() {
        let json = r#"{"type":"Text"}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_unknown_field_error() {
        let json = r#"{"type":"Text","data":{"text":"hello"},"extra":"field"}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_tool_result() {
        let result = serde_json::json!({"output": "success"});
        let part = Part::tool_result("read_file", result, true);
        assert!(matches!(part, Part::ToolResult(_)));
    }

    #[test]
    fn test_part_display_text() {
        let part = Part::text("hello");
        assert_eq!(format!("{}", part), "hello");
    }

    #[test]
    fn test_part_display_code() {
        let part = Part::code("fn main() {}", Some("rust"));
        assert_eq!(format!("{}", part), "fn main() {}");
    }

    #[test]
    fn test_part_display_tool_use() {
        let part = Part::tool_use("read_file", serde_json::json!({}));
        assert_eq!(format!("{}", part), "tool:read_file");
    }

    #[test]
    fn test_part_display_tool_result() {
        let part = Part::tool_result("read_file", serde_json::json!({}), true);
        assert_eq!(format!("{}", part), "result:read_file");
    }

    #[test]
    fn test_part_display_file_reference() {
        let part = Part::FileReference(FileReferencePart {
            path: "/tmp/test.rs".to_string(),
            content_preview: None,
            line_start: None,
            line_end: None,
        });
        assert_eq!(format!("{}", part), "file:/tmp/test.rs");
    }

    #[test]
    fn test_part_display_image() {
        let part = Part::Image(ImagePart {
            reference: "ref123".to_string(),
            alt_text: Some("test image".to_string()),
        });
        assert_eq!(format!("{}", part), "[image]");
    }

    #[test]
    fn test_part_display_reasoning() {
        let part = Part::Reasoning(ReasoningPart {
            trace: "thinking...".to_string(),
        });
        assert_eq!(format!("{}", part), "thinking...");
    }

    #[test]
    fn test_part_display_unknown() {
        let part = Part::Unknown;
        assert_eq!(format!("{}", part), "[unknown part]");
    }

    #[test]
    fn test_part_deserialization_code() {
        let json = r#"{"type":"Code","data":{"code":"fn main() {}","language":"rust"}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert!(matches!(part, Part::Code(_)));
    }

    #[test]
    fn test_part_deserialization_tool_use() {
        let json = r#"{"type":"ToolUse","data":{"tool_name":"read","arguments":{}}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert!(matches!(part, Part::ToolUse(_)));
    }

    #[test]
    fn test_part_deserialization_tool_result() {
        let json =
            r#"{"type":"ToolResult","data":{"tool_name":"read","result":{},"success":true}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert!(matches!(part, Part::ToolResult(_)));
    }

    #[test]
    fn test_part_deserialization_file_reference() {
        let json = r#"{"type":"FileReference","data":{"path":"/tmp/test.rs","line_start":1,"line_end":10}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert!(matches!(part, Part::FileReference(_)));
    }

    #[test]
    fn test_part_deserialization_image() {
        let json = r#"{"type":"Image","data":{"reference":"img123","alt_text":"test"}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert!(matches!(part, Part::Image(_)));
    }

    #[test]
    fn test_part_deserialization_reasoning() {
        let json = r#"{"type":"Reasoning","data":{"trace":"thinking..."}}"#;
        let part: Part = serde_json::from_str(json).unwrap();
        assert!(matches!(part, Part::Reasoning(_)));
    }

    #[test]
    fn test_part_duplicate_type_field_error() {
        let json = r#"{"type":"Text","type":"Code","data":{"text":"hello"}}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_duplicate_data_field_error() {
        let json = r#"{"type":"Text","data":{"text":"hello"},"data":{"text":"world"}}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_invalid_text_data_error() {
        let json = r#"{"type":"Text","data":{"not_text":"hello"}}"#;
        let result: Result<Part, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_part_code_with_filename() {
        let part = Part::Code(CodePart {
            code: "fn main() {}".to_string(),
            language: Some("rust".to_string()),
            filename: Some("main.rs".to_string()),
        });
        assert!(matches!(part, Part::Code(_)));
    }

    #[test]
    fn test_part_file_reference_with_preview() {
        let part = Part::FileReference(FileReferencePart {
            path: "/tmp/test.rs".to_string(),
            content_preview: Some("fn main() {}".to_string()),
            line_start: Some(1),
            line_end: Some(10),
        });
        assert!(matches!(part, Part::FileReference(_)));
    }
}
