use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;

#[allow(dead_code)]
pub struct InvalidTool;

#[derive(Deserialize)]
#[allow(dead_code)]
struct InvalidArgs {
    #[allow(dead_code)]
    tool: String,
    error: String,
}

impl sealed::Sealed for InvalidTool {}

#[async_trait]
impl Tool for InvalidTool {
    fn name(&self) -> &str {
        "invalid"
    }

    fn description(&self) -> &str {
        "Do not use"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(InvalidTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: InvalidArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "The arguments provided to the tool are invalid: {}",
            args.error
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_tool_name() {
        let tool = InvalidTool;
        assert_eq!(tool.name(), "invalid");
    }

    #[tokio::test]
    async fn test_invalid_tool_description() {
        let tool = InvalidTool;
        assert_eq!(tool.description(), "Do not use");
    }

    #[tokio::test]
    async fn test_invalid_tool_clone() {
        let tool = InvalidTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "invalid");
    }

    #[tokio::test]
    async fn test_invalid_tool_execute() {
        let tool = InvalidTool;
        let args = serde_json::json!({
            "tool": "some_tool",
            "error": "Invalid argument"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("Invalid argument"));
    }

    #[tokio::test]
    async fn test_invalid_tool_invalid_args() {
        let tool = InvalidTool;
        let args = serde_json::json!({"not_error": "test"});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
