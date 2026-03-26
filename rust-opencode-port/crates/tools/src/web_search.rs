use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct WebSearchTool;

#[derive(Deserialize)]
struct SearchArgs {
    query: String,
    num_results: Option<usize>,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(WebSearchTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: SearchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let _num_results = args.num_results.unwrap_or(5);

        Ok(ToolResult::ok(format!(
            "Web search placeholder for '{}'.\n\n\
            To enable web search, configure a search API key in settings.\n\
            For now, please use external browser or manually navigate to search engines.",
            args.query
        )))
    }
}
