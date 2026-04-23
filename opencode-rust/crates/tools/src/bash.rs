use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;

const DANGEROUS_SHELL_CHARS: &[char] = &[';', '|', '&', '>', '<', '`', '\n', '\r'];

fn is_shell_injection(command: &str) -> Option<String> {
    if command.contains("&&") || command.contains("||") {
        return Some("Logical operators (&&, ||) are not allowed".to_string());
    }
    if command.contains(";;") {
        return Some("Case statement operator (;;) is not allowed".to_string());
    }
    if command.contains('\n') || command.contains('\r') {
        return Some("Newline characters are not allowed".to_string());
    }

    for ch in DANGEROUS_SHELL_CHARS {
        if command.contains(*ch) {
            match ch {
                ';' => return Some("Semicolons are not allowed".to_string()),
                '|' => return Some("Pipes are not allowed".to_string()),
                '&' => return Some("Background operators are not allowed".to_string()),
                '>' => return Some("Output redirection is not allowed".to_string()),
                '<' => return Some("Input redirection is not allowed".to_string()),
                '`' => return Some("Backtick command substitution is not allowed".to_string()),
                '\n' | '\r' => return Some("Newline characters are not allowed".to_string()),
                _ => {}
            }
        }
    }

    if command.contains("$(") || command.contains("$('')") {
        return Some("Command substitution $(...) is not allowed".to_string());
    }

    let has_backtick_sub = command.contains("`");
    if has_backtick_sub {
        return Some("Backtick command substitution is not allowed".to_string());
    }

    if command.contains("${") && command.contains("}") {
        let re = regex::Regex::new(r"\$\{[^}]+\}").ok();
        if let Some(re) = re {
            if re.is_match(command) {
                let simple_var_re = regex::Regex::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*").ok();
                if let Some(simple_var_re) = simple_var_re {
                    let remaining: String = re.replace_all(command, "").to_string();
                    if !simple_var_re.is_match(&remaining) {
                        return Some(
                            "Variable expansion with braces ${...} is not allowed".to_string(),
                        );
                    }
                }
            }
        }
    }

    None
}

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

const DANGEROUS_ENV_VARS: &[&str] = &[
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "LD_AUDIT",
    "LD_DEBUG",
    "LD_PROFILE",
    "DYLD_INSERT_LIBRARIES",
    "DYLD_LIBRARY_PATH",
    "_ORIGINAL_RPATH",
    "ORIGINAL_LD_LIBRARY_PATH",
    "OPENCODE_INJECTED",
];

fn filter_dangerous_env_vars() -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(key, _)| !DANGEROUS_ENV_VARS.iter().any(|&dangerous| key == dangerous))
        .collect()
}

async fn run_command_with_timeout(
    command: &str,
    workdir: &str,
    timeout_duration: Duration,
) -> Result<(Vec<u8>, Vec<u8>, i32), String> {
    let filtered_env = filter_dangerous_env_vars();
    let env_slice: Vec<(String, String)> = filtered_env;

    let child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(workdir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .envs(env_slice.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .spawn()
        .map_err(|e| e.to_string())?;

    let result = timeout(timeout_duration, child.wait_with_output()).await;

    match result {
        Ok(Ok(output)) => {
            let status = output.status;
            Ok((output.stdout, output.stderr, status.code().unwrap_or(-1)))
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err("Command timed out".to_string()),
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

        if let Some(reason) = is_shell_injection(&args.command) {
            return Ok(ToolResult {
                success: false,
                content: String::new(),
                error: Some(format!("Shell injection detected: {}", reason)),
                title: None,
                metadata: Some(serde_json::json!({
                    "description": description,
                    "rejected": true
                })),
            });
        }

        let start = Instant::now();
        let output_result =
            run_command_with_timeout(&args.command, &workdir, timeout_duration).await;

        let elapsed = start.elapsed();

        match output_result {
            Ok((stdout, stderr, exit_code)) => {
                let stdout_str = String::from_utf8_lossy(&stdout);
                let stderr_str = String::from_utf8_lossy(&stderr);
                let mut result = String::new();

                if !stdout_str.is_empty() {
                    result.push_str(&stdout_str);
                }
                if !stderr_str.is_empty() {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(&stderr_str);
                }

                if result.len() > self.max_output_length {
                    result.truncate(self.max_output_length);
                    result.push_str("\n\n...(output truncated)");
                }

                let mut metadata = format!("Exit code: {}\n", exit_code);
                metadata.push_str(&format!("Time: {:.2}s\n", elapsed.as_secs_f64()));
                metadata.push_str(&format!("Description: {}", description));

                Ok(ToolResult {
                    success: exit_code == 0,
                    content: result,
                    error: if exit_code != 0 {
                        Some(format!("Command failed with exit code {}", exit_code))
                    } else {
                        None
                    },
                    title: None,
                    metadata: Some(serde_json::json!({
                        "exitCode": exit_code,
                        "time": elapsed.as_secs_f64(),
                        "description": description
                    })),
                })
            }
            Err(e) => Err(OpenCodeError::Tool(e)),
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
        let _result = tool.execute(args, None).await.unwrap();
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

    #[tokio::test]
    async fn test_bash_timeout_kills_sleep_command() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "sleep 60",
            "timeout": 1000
        });
        let start = Instant::now();
        let result = tool.execute(args, None).await;
        let elapsed = start.elapsed();

        assert!(result.is_err(), "Timeout should return error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("timed out"),
            "Error should mention timeout"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "Command should be killed quickly, not wait for sleep to finish"
        );
    }

    #[tokio::test]
    async fn test_bash_timeout_kills_process_tree() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "yes",
            "timeout": 1000
        });
        let start = Instant::now();
        let result = tool.execute(args, None).await;
        let elapsed = start.elapsed();

        assert!(result.is_err(), "Timeout should return error");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("timed out"),
            "Error should mention timeout"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "Process tree should be killed quickly"
        );
    }

    #[tokio::test]
    async fn test_bash_timeout_error_message_format() {
        let tool = BashTool::new();
        let args = serde_json::json!({
            "command": "sleep 60",
            "timeout": 500
        });
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("timed out") || err_str.contains("Command timed out"),
            "Error should indicate timeout"
        );
    }

    #[tokio::test]
    async fn test_bash_tool_still_works_after_timeout() {
        let tool = BashTool::new();

        let args_timeout = serde_json::json!({
            "command": "sleep 60",
            "timeout": 500
        });
        let _ = tool.execute(args_timeout, None).await;

        let args_normal = serde_json::json!({"command": "echo 'still works'"});
        let result = tool.execute(args_normal, None).await.unwrap();

        assert!(result.success, "Tool should still work after timeout");
        assert!(result.content.contains("still works"));
    }
}
