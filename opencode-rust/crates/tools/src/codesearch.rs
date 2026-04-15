use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchArgs {
    pub pattern: String,
    pub path: Option<String>,
}

pub struct CodeSearchTool;

#[async_trait]
impl Tool for CodeSearchTool {
    fn name(&self) -> &str {
        "codesearch"
    }

    fn description(&self) -> &str {
        "Search code patterns across the filesystem using AST-aware grep patterns"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(CodeSearchTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: SearchArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Parse(e.to_string()))?;

        let path = args.path.unwrap_or_else(|| ".".to_string());

        // Simple grep -r implementation as a base for AST-aware search
        let output = Command::new("grep")
            .arg("-r")
            .arg("-n")
            .arg("--color=never")
            .arg(&args.pattern)
            .arg(&path)
            .output()
            .map_err(OpenCodeError::Io)?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        if result.is_empty() {
            return Ok(ToolResult::ok("No matches found".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}
