use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct WebfetchTool;

#[derive(Deserialize)]
struct WebfetchArgs {
    url: String,
    _format: Option<String>,
}

#[async_trait]
impl Tool for WebfetchTool {
    fn name(&self) -> &str {
        "webfetch"
    }

    fn description(&self) -> &str {
        "Fetch web content"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(WebfetchTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: WebfetchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Web fetch for '{}' (placeholder).\n\n\
            Configure webfetch API to enable web content fetching.",
            args.url
        )))
    }
}
