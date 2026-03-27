use async_trait::async_trait;
use serde::Deserialize;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

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

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: BashArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let timeout_duration = args.timeout
            .map(Duration::from_millis)
            .unwrap_or(self.default_timeout);

        let workdir = args.workdir.unwrap_or_else(|| ".".to_string());
        let description = args.description.unwrap_or_else(|| "Execute command".to_string());

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
        }).await;

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
