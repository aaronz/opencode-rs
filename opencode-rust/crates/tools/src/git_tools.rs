use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::process::Command;

pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }

    fn description(&self) -> &str {
        "Show git repository status"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GitStatusTool)
    }

    async fn execute(
        &self,
        _args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        if result.is_empty() {
            return Ok(ToolResult::ok("Working tree clean".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}

pub struct GitDiffTool;

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn description(&self) -> &str {
        "Show uncommitted changes"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GitDiffTool)
    }

    async fn execute(
        &self,
        _args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let output = Command::new("git")
            .args(["diff", "--color=never"])
            .output()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        if result.is_empty() {
            return Ok(ToolResult::ok("No changes".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}

pub struct GitLogTool;

#[derive(Deserialize)]
struct GitLogArgs {
    #[serde(default = "default_limit")]
    limit: Option<usize>,
    file: Option<String>,
    path: Option<String>,
}

fn default_limit() -> Option<usize> {
    Some(10)
}

#[async_trait]
impl Tool for GitLogTool {
    fn name(&self) -> &str {
        "git_log"
    }

    fn description(&self) -> &str {
        "Show commit history"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GitLogTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: GitLogArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let mut cmd = Command::new("git");
        cmd.args(["log", "--oneline", "--decorate"]);

        if let Some(limit) = args.limit {
            cmd.args(["-n", &limit.to_string()]);
        }

        if let Some(file) = &args.file {
            cmd.args(["--", file]);
        }

        if let Some(path) = &args.path {
            cmd.arg("--follow").arg("--").arg(path);
        }

        let output = cmd
            .output()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        if result.is_empty() {
            return Ok(ToolResult::ok("No commits found".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}

pub struct GitShowTool;

#[derive(Deserialize)]
struct GitShowArgs {
    #[serde(default)]
    commit: Option<String>,
    #[serde(default)]
    file: Option<String>,
}

#[async_trait]
impl Tool for GitShowTool {
    fn name(&self) -> &str {
        "git_show"
    }

    fn description(&self) -> &str {
        "Show commit details or file at specific commit"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GitShowTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: GitShowArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let commit = args.commit.unwrap_or_else(|| "HEAD".to_string());
        let mut cmd = Command::new("git");
        cmd.arg("show").arg(&commit);

        if let Some(file) = args.file {
            cmd.arg("--");
            cmd.arg(file);
        } else {
            cmd.arg("--stat");
        }

        let output = cmd
            .output()
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        if result.is_empty() {
            return Ok(ToolResult::ok("No content found".to_string()));
        }

        Ok(ToolResult::ok(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_git_status_tool() {
        let tool = GitStatusTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_diff_tool() {
        let tool = GitDiffTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_log_tool_default() {
        let tool = GitLogTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_log_tool_with_limit() {
        let tool = GitLogTool;
        let result = tool.execute(serde_json::json!({"limit": 5}), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_log_tool_with_file() {
        let tool = GitLogTool;
        let result = tool
            .execute(serde_json::json!({"file": "Cargo.toml"}), None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_show_tool_head() {
        let tool = GitShowTool;
        let result = tool.execute(serde_json::json!({}), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_show_tool_with_commit() {
        let tool = GitShowTool;
        let result = tool
            .execute(serde_json::json!({"commit": "HEAD"}), None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_git_show_tool_with_file() {
        let tool = GitShowTool;
        let result = tool
            .execute(
                serde_json::json!({"commit": "HEAD", "file": "Cargo.toml"}),
                None,
            )
            .await;
        assert!(result.is_ok());
    }
}
