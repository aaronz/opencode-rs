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

        let plan_path = worktree.join(".opencode").join("plan.md");

        // Create .opencode directory if it doesn't exist
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
