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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_todowrite_tool_name() {
        let tool = TodowriteTool;
        assert_eq!(tool.name(), "todo");
    }

    #[tokio::test]
    async fn test_todowrite_tool_description() {
        let tool = TodowriteTool;
        assert_eq!(tool.description(), "Manage todo lists");
    }

    #[tokio::test]
    async fn test_todowrite_tool_clone() {
        let tool = TodowriteTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "todo");
    }

    #[tokio::test]
    async fn test_todowrite_single_item() {
        let tool = TodowriteTool;
        let args = serde_json::json!({
            "items": [{"content": "Test task"}]
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("[ ] Test task"));
        assert!(result.content.contains("(medium)"));
    }

    #[tokio::test]
    async fn test_todowrite_completed_item() {
        let tool = TodowriteTool;
        let args = serde_json::json!({
            "items": [{"content": "Done task", "status": "completed"}]
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("[x] Done task"));
    }

    #[tokio::test]
    async fn test_todowrite_with_priority() {
        let tool = TodowriteTool;
        let args = serde_json::json!({
            "items": [{"content": "High priority", "priority": "high"}]
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("(high)"));
    }

    #[tokio::test]
    async fn test_todowrite_multiple_items() {
        let tool = TodowriteTool;
        let args = serde_json::json!({
            "items": [
                {"content": "Task 1"},
                {"content": "Task 2", "status": "completed"},
                {"content": "Task 3", "priority": "low"}
            ]
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("[ ] Task 1"));
        assert!(result.content.contains("[x] Task 2"));
        assert!(result.content.contains("(low)"));
    }

    #[tokio::test]
    async fn test_todowrite_empty_items() {
        let tool = TodowriteTool;
        let args = serde_json::json!({"items": []});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "");
    }

    #[tokio::test]
    async fn test_todowrite_invalid_args() {
        let tool = TodowriteTool;
        let args = serde_json::json!({"not_items": []});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
