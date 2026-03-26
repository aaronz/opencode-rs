use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct CodesearchTool;

#[derive(Deserialize)]
struct CodesearchArgs {
    pattern: String,
    _file_pattern: Option<String>,
    _context_lines: Option<usize>,
}

#[async_trait]
impl Tool for CodesearchTool {
    fn name(&self) -> &str {
        "codesearch"
    }

    fn description(&self) -> &str {
        "Search code across repositories"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(CodesearchTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: CodesearchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Code search for '{}' (placeholder implementation).\n\n\
            Use grep tool for local searches.",
            args.pattern
        )))
    }
}
