use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;

pub struct PlanExitTool;

impl sealed::Sealed for PlanExitTool {}

#[async_trait]
impl Tool for PlanExitTool {
    fn name(&self) -> &str {
        "plan_exit"
    }

    fn description(&self) -> &str {
        "Use this tool when you have completed the planning phase and are ready to exit plan agent.\n\nThis tool will ask the user if they want to switch to build agent to start implementing the plan.\n\nCall this tool:\n- After you have written a complete plan to the plan file\n- After you have clarified any questions with the user\n- When you are confident the plan is ready for implementation\n\nDo NOT call this tool:\n- Before you have created or finalized the plan\n- If you still have unanswered questions about the implementation\n- If the user has indicated they want to continue planning"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(PlanExitTool)
    }

    async fn execute(
        &self,
        _args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        Ok(ToolResult::ok(
            "Plan exit: switch to build agent to start implementing",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plan_exit_tool_name() {
        let tool = PlanExitTool;
        assert_eq!(tool.name(), "plan_exit");
    }

    #[tokio::test]
    async fn test_plan_exit_tool_description() {
        let tool = PlanExitTool;
        assert!(tool.description().contains("plan agent"));
    }

    #[tokio::test]
    async fn test_plan_exit_tool_clone() {
        let tool = PlanExitTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "plan_exit");
    }

    #[tokio::test]
    async fn test_plan_exit_execute() {
        let tool = PlanExitTool;
        let args = serde_json::json!({});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("switch to build agent"));
    }

    #[tokio::test]
    async fn test_plan_exit_execute_with_args() {
        let tool = PlanExitTool;
        let args = serde_json::json!({"some": "args"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }
}
