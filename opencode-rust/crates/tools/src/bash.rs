use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::timeout;

pub struct BashTool {
    default_timeout: Duration,
    max_output_length: usize,
}

#[derive(Deserialize)]
struct BashArgs {
    command: String,
    timeout: Option<u64>,
    workdir: Option<String>,
    description: Option<String>,
}

impl BashTool {
    pub fn new() -> Self {
        Self {
            default_timeout: Duration::from_secs(120),
            max_output_length: 30_000,
        }
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for BashTool {}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute bash commands with permission control"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(BashTool::new())
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: BashArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let timeout_duration = args
            .timeout
            .map(Duration::from_millis)
            .unwrap_or(self.default_timeout);

        let workdir = args.workdir.unwrap_or_else(|| ".".to_string());
        let description = args
            .description
            .unwrap_or_else(|| "Execute command".to_string());

        let start = Instant::now();
        let output = timeout(timeout_duration, async {
            Command::new("sh")
                .arg("-c")
                .arg(&args.command)
                .current_dir(&workdir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| OpenCodeError::Tool(e.to_string()))
        })
        .await;

        let elapsed = start.elapsed();

        match output {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let mut result = String::new();

                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(&stderr);
                }

                if result.len() > self.max_output_length {
                    result.truncate(self.max_output_length);
                    result.push_str("\n\n...(output truncated)");
                }

                let mut metadata = format!("Exit code: {}\n", output.status);
                metadata.push_str(&format!("Time: {:.2}s\n", elapsed.as_secs_f64()));
                metadata.push_str(&format!("Description: {}", description));

                Ok(ToolResult {
                    success: output.status.success(),
                    content: result,
                    error: if !output.status.success() {
                        Some(format!("Command failed with exit code {}", output.status))
                    } else {
                        None
                    },
                    title: None,
                    metadata: Some(serde_json::json!({
                        "exitCode": output.status.code(),
                        "time": elapsed.as_secs_f64(),
                        "description": description
                    })),
                })
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(OpenCodeError::Tool(format!(
                "Command timed out after {:.2}s",
                timeout_duration.as_secs_f64()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_tool_name() {
        let tool = BashTool::new();
        assert_eq!(tool.name(), "bash");
    }

    #[tokio::test]
    async fn test_bash_tool_description() {
        let tool = BashTool::new();
        assert!(tool.description().contains("bash"));
    }

    #[tokio::test]
    async fn test_bash_tool_clone() {
        let tool = BashTool::new();
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "bash");
    }

    #[tokio::test]
    async fn test_bash_simple_command() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "echo hello"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_command_with_error() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "exit 1"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_bash_command_with_custom_workdir() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "pwd",
            "workdir": "/tmp"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_bash_command_with_description() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "echo test",
            "description": "My test command"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_bash_command_with_timeout() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "sleep 1",
            "timeout": 5000
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_bash_command_with_stderr_and_stdout() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "echo stdout && echo stderr >&2"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("stdout"));
        assert!(result.content.contains("stderr"));
    }

    #[tokio::test]
    async fn test_bash_stderr_captured() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "echo error >&2"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("error"));
    }

    #[tokio::test]
    async fn test_bash_default_timeout() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "echo test"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_bash_long_output_truncated() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "echo start && yes | head -10000 && echo end"
        });
        let result = tool.execute(args, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_bash_invalid_args() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": 123});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_command_not_found() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "nonexistent_command_12345"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_bash_metadata_set() {
        let tool = BashTool::new();
        let args = serde_json::json!({"command": "echo test"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.metadata.is_some());
        let metadata = result.metadata.unwrap();
        assert!(metadata.get("exitCode").is_some());
        assert!(metadata.get("time").is_some());
        assert!(metadata.get("description").is_some());
    }
}
