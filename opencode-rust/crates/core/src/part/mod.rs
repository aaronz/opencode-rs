mod types;

pub use types::{
    CodePart, FileReferencePart, ImagePart, Part, ReasoningPart, TextPart, ToolResultPart,
    ToolUsePart,
};

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
        let json =
            r#"{"type":"FileReference","data":{"path":"/tmp/test.rs","line_start":1,"line_end":10}}"#;
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