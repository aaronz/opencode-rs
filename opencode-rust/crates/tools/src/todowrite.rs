use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;

pub struct TodowriteTool;

#[derive(Deserialize)]
struct TodowriteArgs {
    items: Vec<TodoItem>,
}

#[derive(Deserialize)]
struct TodoItem {
    content: String,
    status: Option<String>,
    priority: Option<String>,
}

impl sealed::Sealed for TodowriteTool {}

#[async_trait]
impl Tool for TodowriteTool {
    fn name(&self) -> &str {
        "todo"
    }

    fn description(&self) -> &str {
        "Manage todo lists"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TodowriteTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: TodowriteArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let mut result = String::new();
        for item in &args.items {
            let status = item.status.as_deref().unwrap_or("pending");
            let priority = item.priority.as_deref().unwrap_or("medium");
            let checkbox = if status == "completed" { "x" } else { " " };
            result.push_str(&format!(
                "- [{}] {} ({})\n",
                checkbox, item.content, priority
            ));
        }

        Ok(ToolResult::ok(result))
    }
}
