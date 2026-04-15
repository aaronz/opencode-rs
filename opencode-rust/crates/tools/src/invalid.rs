use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;

pub struct InvalidTool;

#[derive(Deserialize)]
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
