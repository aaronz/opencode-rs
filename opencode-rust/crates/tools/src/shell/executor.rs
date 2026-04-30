use std::process::Stdio;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::timeout;

use super::traits::ShellExecutor;
use super::{CommandExecutionResult, CommandOutput, CommandRequest, CommandStatus};

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

const SHELL_INJECTION_CHARS: &[char] = &[';', '|', '&', '>', '<', '`', '\n', '\r'];

fn filter_dangerous_env_vars() -> Vec<(String, String)> {
    std::env::vars()
        .filter(|(key, _)| !DANGEROUS_ENV_VARS.iter().any(|&dangerous| key == dangerous))
        .collect()
}

fn check_shell_injection(command: &str) -> Option<String> {
    if command.contains("&&") || command.contains("||") {
        return Some("Logical operators (&&, ||) are not allowed".to_string());
    }
    if command.contains(";;") {
        return Some("Case statement operator (;;) is not allowed".to_string());
    }
    if command.contains('\n') || command.contains('\r') {
        return Some("Newline characters are not allowed".to_string());
    }

    for ch in SHELL_INJECTION_CHARS {
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

    if command.contains("${") && command.contains("}") {
        let re = regex::Regex::new(r"\$\{[^}]+\}").ok();
        if let Some(re) = re {
            if re.is_match(command) {
                let simple_var_re = regex::Regex::new(r"$[a-zA-Z_][a-zA-Z0-9_]*").ok();
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

pub struct RealShellExecutor {
    max_output_length: usize,
    default_timeout: Duration,
}

impl RealShellExecutor {
    pub fn new() -> Self {
        Self {
            max_output_length: 30_000,
            default_timeout: Duration::from_secs(120),
        }
    }

    pub fn with_max_output_length(mut self, max: usize) -> Self {
        self.max_output_length = max;
        self
    }

    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    async fn run_command(
        command: &str,
        cwd: &std::path::Path,
        timeout_duration: Duration,
        filtered_env: &[(String, String)],
    ) -> Result<(Vec<u8>, Vec<u8>, i32), String> {
        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .envs(filtered_env.iter().map(|(k, v)| (k.as_str(), v.as_str())))
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
}

impl Default for RealShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ShellExecutor for RealShellExecutor {
    async fn execute(&self, request: CommandRequest) -> CommandExecutionResult {
        let start = Instant::now();
        let filtered_env = filter_dangerous_env_vars();
        let env_slice: Vec<(String, String)> = filtered_env;

        let timeout_duration = if request.timeout.as_secs() == 0 {
            self.default_timeout
        } else {
            request.timeout
        };

        let result =
            Self::run_command(&request.command, &request.cwd, timeout_duration, &env_slice).await;

        let elapsed = start.elapsed();

        match result {
            Ok((stdout, stderr, exit_code)) => {
                let stdout_str = String::from_utf8_lossy(&stdout).to_string();
                let stderr_str = String::from_utf8_lossy(&stderr).to_string();

                let mut output_str = stdout_str.clone();
                if !stderr_str.is_empty() {
                    if !output_str.is_empty() {
                        output_str.push('\n');
                    }
                    output_str.push_str(&stderr_str);
                }

                let truncated = output_str.len() > self.max_output_length;
                if truncated {
                    output_str.truncate(self.max_output_length);
                    output_str.push_str("\n\n...(output truncated)");
                }

                CommandExecutionResult {
                    output: CommandOutput {
                        stdout: stdout_str,
                        stderr: stderr_str,
                        exit_code: Some(exit_code),
                        timed_out: false,
                        truncated,
                    },
                    status: CommandStatus::Completed,
                    duration_ms: elapsed.as_millis() as u64,
                }
            }
            Err(e) => {
                let timed_out = e.contains("timed out");
                CommandExecutionResult {
                    output: CommandOutput {
                        stdout: String::new(),
                        stderr: e.clone(),
                        exit_code: if timed_out { None } else { Some(-1) },
                        timed_out,
                        truncated: false,
                    },
                    status: if timed_out {
                        CommandStatus::TimedOut
                    } else {
                        CommandStatus::Failed
                    },
                    duration_ms: elapsed.as_millis() as u64,
                }
            }
        }
    }

    fn is_command_safe(&self, command: &str) -> Option<String> {
        check_shell_injection(command)
    }
}
