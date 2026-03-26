use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct TaskTool;

#[derive(Deserialize)]
struct TaskArgs {
    description: String,
    _prompt: String,
}

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "Spawn subagents"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TaskTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: TaskArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Task '{}' spawned (placeholder).\n\n\
            Subagents provide specialized task execution.",
            args.description
        )))
    }
}
