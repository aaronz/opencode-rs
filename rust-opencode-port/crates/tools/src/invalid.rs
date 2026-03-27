use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct InvalidTool;

#[derive(Deserialize)]
struct InvalidArgs {
    tool: String,
    error: String,
}

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

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: InvalidArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "The arguments provided to the tool are invalid: {}",
            args.error
        )))
    }
}