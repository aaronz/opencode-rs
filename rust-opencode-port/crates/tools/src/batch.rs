use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct BatchTool;

#[derive(Deserialize)]
struct BatchArgs {
    invocations: Vec<ToolInvocation>,
}

#[derive(Deserialize)]
struct ToolInvocation {
    tool_name: String,
    _input: serde_json::Value,
}

#[async_trait]
impl Tool for BatchTool {
    fn name(&self) -> &str {
        "batch"
    }

    fn description(&self) -> &str {
        "Execute multiple tools in parallel"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(BatchTool)
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: BatchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let mut results = Vec::new();
        let mut errors = Vec::new();

        for invocation in &args.invocations {
            let result = Ok::<_, OpenCodeError>(ToolResult::ok(format!(
                "Executed {} (placeholder)",
                invocation.tool_name
            )));
            match result {
                Ok(r) => results.push(r),
                Err(e) => errors.push(format!("{}: {}", invocation.tool_name, e)),
            }
        }

        if !errors.is_empty() {
            return Ok(ToolResult::err(format!(
                "Some tools failed: {}",
                errors.join(", ")
            )));
        }

        let combined = results
            .iter()
            .map(|r| r.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(ToolResult::ok(combined))
    }
}
