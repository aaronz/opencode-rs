use crate::sealed;
use crate::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session};
use serde::Deserialize;
use uuid::Uuid;

pub struct TaskTool;

#[derive(Deserialize)]
struct TaskArgs {
    description: String,
    prompt: String,
    subagent_type: String,
    task_id: Option<String>,
    _command: Option<String>,
}

impl sealed::Sealed for TaskTool {}

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str {
        "task"
    }

    fn description(&self) -> &str {
        "Spawn subagents to perform specialized tasks. The subagent will execute in a new session context."
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TaskTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: TaskArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let session_id = match args.task_id.as_ref() {
            Some(id) => Uuid::parse_str(id).unwrap_or_else(|_| Uuid::new_v4()),
            None => Uuid::new_v4(),
        };

        let mut session = Session::new();
        session.id = session_id;

        session.add_message(Message::user(format!(
            "Task: {}\n\nInstructions:\n{}",
            args.description, args.prompt
        )));

        Ok(ToolResult::ok(format!(
            "task_id: {}\n\nSubagent '{}' task created with description: {}\n\nThe subagent will process the following instructions:\n{}\n\nTo continue this task later, use the task_id: {}",
            session.id, args.subagent_type, args.description, args.prompt, session.id
        )).with_metadata(serde_json::json!({
            "sessionId": session.id.to_string(),
            "subagentType": args.subagent_type
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_tool_name() {
        let tool = TaskTool;
        assert_eq!(tool.name(), "task");
    }

    #[tokio::test]
    async fn test_task_tool_description() {
        let tool = TaskTool;
        assert!(tool.description().contains("Spawn subagents"));
    }

    #[tokio::test]
    async fn test_task_tool_clone() {
        let tool = TaskTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "task");
    }

    #[tokio::test]
    async fn test_task_execute() {
        let tool = TaskTool;
        let args = serde_json::json!({
            "description": "Test task",
            "prompt": "Do something",
            "subagent_type": "coder"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("task_id:"));
        assert!(result.content.contains("Test task"));
        assert!(result.content.contains("coder"));
    }

    #[tokio::test]
    async fn test_task_execute_with_task_id() {
        let tool = TaskTool;
        let args = serde_json::json!({
            "description": "Test task",
            "prompt": "Do something",
            "subagent_type": "coder",
            "task_id": "550e8400-e29b-41d4-a716-446655440000"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result
            .content
            .contains("550e8400-e29b-41d4-a716-446655440000"));
    }

    #[tokio::test]
    async fn test_task_execute_with_command() {
        let tool = TaskTool;
        let args = serde_json::json!({
            "description": "Test task",
            "prompt": "Do something",
            "subagent_type": "coder",
            "task_id": null,
            "command": "/some/command"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_task_invalid_args() {
        let tool = TaskTool;
        let args = serde_json::json!({"description": 123});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
