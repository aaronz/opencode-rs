#![allow(clippy::redundant_closure, clippy::unwrap_or_default)]

use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::Instance;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::path::PathBuf;

pub struct PlanTool;

#[derive(Deserialize)]
struct PlanArgs {
    content: Option<String>,
}

impl sealed::Sealed for PlanTool {}

#[async_trait]
impl Tool for PlanTool {
    fn name(&self) -> &str {
        "plan"
    }

    fn description(&self) -> &str {
        "Plan tool - create and manage implementation plans"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(PlanTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: PlanArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let worktree = Instance::worktree().unwrap_or_else(|| PathBuf::from("."));

        // Plan tool allows creating/editing plan files for the plan agent
        let content = args.content.unwrap_or_else(|| "".to_string());

        let plan_path = worktree.join(".opencode-rs").join("plan.md");

        // Create .opencode-rs directory if it doesn't exist
        if let Some(parent) = plan_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OpenCodeError::Tool(e.to_string()))?;
        }

        std::fs::write(&plan_path, &content).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Plan saved to {}",
            plan_path.display()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plan_tool_name() {
        let tool = PlanTool;
        assert_eq!(tool.name(), "plan");
    }

    #[tokio::test]
    async fn test_plan_tool_description() {
        let tool = PlanTool;
        assert_eq!(
            tool.description(),
            "Plan tool - create and manage implementation plans"
        );
    }

    #[tokio::test]
    async fn test_plan_tool_clone() {
        let tool = PlanTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "plan");
    }

    #[tokio::test]
    async fn test_plan_with_content() {
        let tool = PlanTool;
        let args = serde_json::json!({"content": "# Test Plan\n\nSome content"});
        let result = tool.execute(args, None).await;
        assert!(result.is_ok() || result.as_ref().is_ok());
    }

    #[tokio::test]
    async fn test_plan_without_content() {
        let tool = PlanTool;
        let args = serde_json::json!({});
        let result = tool.execute(args, None).await;
        assert!(result.is_ok() || result.as_ref().is_ok());
    }

    #[tokio::test]
    async fn test_plan_invalid_args() {
        let tool = PlanTool;
        let args = serde_json::json!({"content": 123});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
