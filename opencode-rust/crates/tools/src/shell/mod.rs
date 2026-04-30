mod executor;
mod traits;

pub use executor::RealShellExecutor;
pub use traits::ShellExecutor;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub truncated: bool,
}

impl CommandOutput {
    pub fn is_success(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out && !self.truncated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecutionResult {
    pub output: CommandOutput,
    pub status: CommandStatus,
    pub duration_ms: u64,
}

pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub timeout: Duration,
}

impl CommandRequest {
    pub fn new(command: String) -> Self {
        Self {
            command,
            args: Vec::new(),
            cwd: PathBuf::from("."),
            env: BTreeMap::new(),
            timeout: Duration::from_secs(120),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = cwd;
        self
    }

    pub fn with_env(mut self, env: BTreeMap<String, String>) -> Self {
        self.env = env;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
