use async_trait::async_trait;

use super::{CommandExecutionResult, CommandOutput, CommandRequest};

#[async_trait]
pub trait ShellExecutor: Send + Sync {
    async fn execute(&self, request: CommandRequest) -> CommandExecutionResult;

    fn is_command_safe(&self, command: &str) -> Option<String>;
}

#[allow(dead_code)]
pub struct FakeShellExecutor {
    canned_output: Option<CommandOutput>,
    should_fail: bool,
}

impl FakeShellExecutor {
    pub fn new() -> Self {
        Self {
            canned_output: None,
            should_fail: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_canned_output(mut self, output: CommandOutput) -> Self {
        self.canned_output = Some(output);
        self
    }

    #[allow(dead_code)]
    pub fn failing(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl Default for FakeShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ShellExecutor for FakeShellExecutor {
    async fn execute(&self, _request: CommandRequest) -> CommandExecutionResult {
        if self.should_fail {
            return CommandExecutionResult {
                output: CommandOutput {
                    stdout: String::new(),
                    stderr: "Fake shell execution failed".to_string(),
                    exit_code: Some(1),
                    timed_out: false,
                    truncated: false,
                },
                status: super::CommandStatus::Failed,
                duration_ms: 0,
            };
        }

        let output = self.canned_output.clone().unwrap_or(CommandOutput {
            stdout: "fake output".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            timed_out: false,
            truncated: false,
        });

        CommandExecutionResult {
            output,
            status: super::CommandStatus::Completed,
            duration_ms: 0,
        }
    }

    fn is_command_safe(&self, _command: &str) -> Option<String> {
        None
    }
}
