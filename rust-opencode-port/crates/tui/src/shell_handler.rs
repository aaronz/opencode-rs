use std::process::{Command, Stdio};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MAX_OUTPUT_SIZE: usize = 100 * 1024;

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ExecuteResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub truncated: bool,
}

pub struct ShellHandler {
    timeout_secs: u64,
    max_output_size: usize,
}

impl ShellHandler {
    pub fn new() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_output_size: MAX_OUTPUT_SIZE,
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_max_output(mut self, size: usize) -> Self {
        self.max_output_size = size;
        self
    }

    pub fn execute(&self, command: &str) -> ExecuteResult {
        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let output = Command::new(shell)
            .arg("-c")
            .arg(command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let exit_code = out.status.code();

                let (truncated_stdout, truncated) = self.truncate_output(stdout);
                let truncated_stderr = if truncated {
                    self.truncate_output(stderr).0
                } else {
                    stderr
                };

                ExecuteResult {
                    stdout: truncated_stdout,
                    stderr: truncated_stderr,
                    exit_code,
                    timed_out: false,
                    truncated,
                }
            }
            Err(e) => ExecuteResult {
                stdout: String::new(),
                stderr: e.to_string(),
                exit_code: None,
                timed_out: false,
                truncated: false,
            },
        }
    }

    fn truncate_output(&self, output: String) -> (String, bool) {
        if output.len() > self.max_output_size {
            let truncated = format!(
                "{}...[truncated {} bytes]",
                &output[..self.max_output_size],
                output.len() - self.max_output_size
            );
            (truncated, true)
        } else {
            (output, false)
        }
    }
}

impl Default for ShellHandler {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn handler() -> ShellHandler {
        ShellHandler::new()
    }

    #[test]
    fn test_echo_command() {
        let result = handler().execute("echo hello");
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_stderr_capture() {
        let result = handler().execute("echo error >&2");
        assert!(result.stderr.contains("error"));
    }

    #[test]
    fn test_invalid_command() {
        let result = handler().execute("nonexistent_command_xyz");
        assert!(!result.stderr.is_empty() || result.exit_code != Some(0));
    }

    #[test]
    fn test_exit_code() {
        let result = handler().execute("exit 42");
        assert_eq!(result.exit_code, Some(42));
    }

    #[test]
    fn test_output_truncation() {
        let handler = ShellHandler::new().with_max_output(100);
        let result = handler.execute("printf 'a%.0s' {1..200}");
        assert!(result.truncated);
        assert!(result.stdout.len() <= 100 + 50);
    }
}
