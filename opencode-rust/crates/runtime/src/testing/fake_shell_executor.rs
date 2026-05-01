use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use opencode_tools::shell::{
    CommandExecutionResult, CommandOutput, CommandRequest, CommandStatus, ShellExecutor,
};

pub struct FakeShellExecutor {
    commands: Arc<RwLock<HashMap<String, CommandOutput>>>,
    should_error: Arc<RwLock<HashMap<String, String>>>,
    executed: Arc<RwLock<Vec<String>>>,
}

impl Default for FakeShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeShellExecutor {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            should_error: Arc::new(RwLock::new(HashMap::new())),
            executed: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_command(
        self: Arc<Self>,
        cmd: impl Into<String>,
        output: CommandOutput,
    ) -> Arc<Self> {
        self.commands.write().unwrap().insert(cmd.into(), output);
        self
    }

    pub fn with_error(
        self: Arc<Self>,
        cmd: impl Into<String>,
        error: impl Into<String>,
    ) -> Arc<Self> {
        self.should_error
            .write()
            .unwrap()
            .insert(cmd.into(), error.into());
        self
    }

    pub fn executed_commands(&self) -> Vec<String> {
        self.executed.read().unwrap().clone()
    }

    pub fn was_executed(&self, cmd: &str) -> bool {
        self.executed.read().unwrap().contains(&cmd.to_string())
    }
}

#[async_trait]
impl ShellExecutor for FakeShellExecutor {
    async fn execute(&self, request: CommandRequest) -> CommandExecutionResult {
        self.executed.write().unwrap().push(request.command.clone());

        if let Some(error) = self.should_error.read().unwrap().get(&request.command) {
            return CommandExecutionResult {
                output: CommandOutput {
                    stdout: String::new(),
                    stderr: error.clone(),
                    exit_code: Some(1),
                    timed_out: false,
                    truncated: false,
                },
                status: CommandStatus::Failed,
                duration_ms: 0,
            };
        }

        let output = self
            .commands
            .read()
            .unwrap()
            .get(&request.command)
            .cloned()
            .unwrap_or(CommandOutput {
                stdout: format!("output for: {}", request.command),
                stderr: String::new(),
                exit_code: Some(0),
                timed_out: false,
                truncated: false,
            });

        CommandExecutionResult {
            output,
            status: CommandStatus::Completed,
            duration_ms: 0,
        }
    }

    fn is_command_safe(&self, command: &str) -> Option<String> {
        if command.contains("rm -rf") {
            Some("Dangerous command".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_shell_executes_configured_command() {
        let executor = Arc::new(FakeShellExecutor::new()).with_command(
            "echo hello",
            CommandOutput {
                stdout: "hello".to_string(),
                stderr: "".to_string(),
                exit_code: Some(0),
                timed_out: false,
                truncated: false,
            },
        );

        let request = CommandRequest::new("echo hello".to_string());
        let result = executor.execute(request).await;
        assert_eq!(result.output.stdout, "hello");
        assert_eq!(result.status, CommandStatus::Completed);
    }

    #[tokio::test]
    async fn test_fake_shell_returns_default_for_unconfigured() {
        let executor = Arc::new(FakeShellExecutor::new());
        let request = CommandRequest::new("echo hello".to_string());
        let result = executor.execute(request).await;
        assert_eq!(result.output.stdout, "output for: echo hello");
        assert_eq!(result.status, CommandStatus::Completed);
    }

    #[tokio::test]
    async fn test_fake_shell_returns_configured_error() {
        let executor =
            Arc::new(FakeShellExecutor::new()).with_error("echo hello", "Permission denied");

        let request = CommandRequest::new("echo hello".to_string());
        let result = executor.execute(request).await;
        assert_eq!(result.status, CommandStatus::Failed);
        assert_eq!(result.output.stderr, "Permission denied");
    }

    #[tokio::test]
    async fn test_fake_shell_tracks_executed_commands() {
        let executor = Arc::new(FakeShellExecutor::new()).with_command(
            "echo hello",
            CommandOutput {
                stdout: "hello".to_string(),
                stderr: "".to_string(),
                exit_code: Some(0),
                timed_out: false,
                truncated: false,
            },
        );

        let request1 = CommandRequest::new("echo hello".to_string());
        let request2 = CommandRequest::new("echo hello".to_string());
        executor.execute(request1).await;
        executor.execute(request2).await;

        let executed = executor.executed_commands();
        assert_eq!(executed.len(), 2);
        assert!(executor.was_executed("echo hello"));
        assert!(!executor.was_executed("echo goodbye"));
    }

    #[tokio::test]
    async fn test_is_command_safe() {
        let executor = Arc::new(FakeShellExecutor::new());
        assert!(executor.is_command_safe("ls").is_none());
        assert!(executor.is_command_safe("rm -rf /").is_some());
    }
}
