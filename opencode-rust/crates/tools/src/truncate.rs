use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;

pub struct TruncateTool {
    max_lines: usize,
    max_bytes: usize,
}

#[derive(Deserialize)]
struct TruncateArgs {
    content: String,
    max_lines: Option<usize>,
    max_bytes: Option<usize>,
}

impl TruncateTool {
    pub fn new() -> Self {
        Self {
            max_lines: 2000,
            max_bytes: 51200,
        }
    }
}

impl Default for TruncateTool {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for TruncateTool {}

#[async_trait]
impl Tool for TruncateTool {
    fn name(&self) -> &str {
        "truncate"
    }

    fn description(&self) -> &str {
        "Truncate large outputs"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TruncateTool::new())
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: TruncateArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let max_lines = args.max_lines.unwrap_or(self.max_lines);
        let max_bytes = args.max_bytes.unwrap_or(self.max_bytes);

        let lines: Vec<&str> = args.content.lines().take(max_lines).collect();
        let mut result = lines.join("\n");

        if result.len() > max_bytes {
            result.truncate(max_bytes);
            result.push_str("\n\n...(truncated)");
        }

        if args.content.lines().count() > max_lines {
            result.push_str(&format!(
                "\n\n... ({} more lines)",
                args.content.lines().count() - max_lines
            ));
        }

        Ok(ToolResult::ok(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_truncate_tool_name() {
        let tool = TruncateTool::new();
        assert_eq!(tool.name(), "truncate");
    }

    #[tokio::test]
    async fn test_truncate_tool_description() {
        let tool = TruncateTool::new();
        assert_eq!(tool.description(), "Truncate large outputs");
    }

    #[tokio::test]
    async fn test_truncate_tool_clone() {
        let tool = TruncateTool::new();
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "truncate");
    }

    #[tokio::test]
    async fn test_truncate_content_within_limits() {
        let tool = TruncateTool::new();
        let args = serde_json::json!({
            "content": "line1\nline2\nline3",
            "max_lines": 10,
            "max_bytes": 10000
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("line1"));
    }

    #[tokio::test]
    async fn test_truncate_exceeds_max_lines() {
        let tool = TruncateTool::new();
        let content = (0..3000)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let args = serde_json::json!({
            "content": content,
            "max_lines": 100,
            "max_bytes": 1000000
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("... ("));
        assert!(result.content.contains("more lines"));
    }

    #[tokio::test]
    async fn test_truncate_exceeds_max_bytes() {
        let tool = TruncateTool::new();
        let content = "x".repeat(100000);
        let args = serde_json::json!({
            "content": content,
            "max_lines": 1000000,
            "max_bytes": 1000
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("...(truncated)"));
    }

    #[tokio::test]
    async fn test_truncate_default_limits() {
        let tool = TruncateTool::new();
        let content = (0..3000)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let args = serde_json::json!({"content": content});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("... ("));
    }

    #[tokio::test]
    async fn test_truncate_invalid_args() {
        let tool = TruncateTool::new();
        let args = serde_json::json!({"content": "test", "max_lines": "not_a_number"});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
