use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const MAX_OUTPUT_SIZE: usize = 100 * 1024;

const DANGEROUS_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "mkfs",
    "dd if=",
    ":(){:|:&};:",
    "chmod -R 777 /",
    "chmod -R 000 /",
    "chown -R",
    "sudo rm",
    "sudo chmod",
    "sudo dd",
    "wget http",
    "curl http",
];

#[derive(Debug, Clone, Default)]
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
    working_dir: PathBuf,
    allowed_commands: Option<Vec<String>>,
}

impl ShellHandler {
    pub fn new() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_output_size: MAX_OUTPUT_SIZE,
            working_dir: std::env::current_dir().unwrap_or_default(),
            allowed_commands: None,
        }
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = dir;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_max_output(mut self, size: usize) -> Self {
        self.max_output_size = size;
        self
    }

    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = Some(commands);
        self
    }

    pub fn is_dangerous(&self, command: &str) -> Option<&'static str> {
        let cmd_lower = command.to_lowercase();

        for dangerous in DANGEROUS_COMMANDS {
            if cmd_lower.contains(dangerous) {
                return Some(dangerous);
            }
        }

        if cmd_lower.contains("rm ") && cmd_lower.contains("-rf") {
            return Some("rm -rf (recursive force delete)");
        }

        if cmd_lower.starts_with("sudo") {
            return Some("sudo (privilege escalation)");
        }

        None
    }

    pub fn is_within_working_dir(&self, command: &str) -> bool {
        if command.contains("..") {
            return false;
        }

        true
    }

    pub fn validate_command(&self, command: &str) -> Result<(), String> {
        if let Some(reason) = self.is_dangerous(command) {
            return Err(format!(
                "Dangerous command detected: {}\n\
                Command: {}\n\
                \nThis command may harm your system. Please review and modify if needed.",
                reason, command
            ));
        }

        if !self.is_within_working_dir(command) {
            return Err(format!(
                "Command attempts to access paths outside working directory\n\
                Working directory: {}\n\
                Command: {}",
                self.working_dir.display(),
                command
            ));
        }

        if let Some(ref allowed) = self.allowed_commands {
            let cmd_name = command.split_whitespace().next().unwrap_or("");
            if !allowed.iter().any(|a| a == cmd_name || a == command) {
                return Err(format!(
                    "Command not allowed: {}\n\
                    Allowed commands: {}",
                    cmd_name,
                    allowed.join(", ")
                ));
            }
        }

        Ok(())
    }

    pub fn execute(&self, command: &str) -> ExecuteResult {
        if let Err(e) = self.validate_command(command) {
            return ExecuteResult {
                stdout: String::new(),
                stderr: e,
                exit_code: Some(1),
                timed_out: false,
                truncated: false,
            };
        }

        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };

        let child = Command::new(shell)
            .arg("-c")
            .arg(command)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match child {
            Ok(mut child) => {
                let timeout = Duration::from_secs(self.timeout_secs);
                match child.wait_timeout(timeout) {
                    Ok(Some(status)) => {
                        let exit_code = status.code();
                        let stdout = match child.stdout.take() {
                            Some(mut out) => {
                                let mut buf = String::new();
                                let _ = out.read_to_string(&mut buf);
                                buf
                            }
                            None => String::new(),
                        };
                        let stderr = match child.stderr.take() {
                            Some(mut err) => {
                                let mut buf = String::new();
                                let _ = err.read_to_string(&mut buf);
                                buf
                            }
                            None => String::new(),
                        };

                        let stdout = String::from_utf8_lossy(&stdout.into_bytes()).to_string();
                        let stderr = String::from_utf8_lossy(&stderr.into_bytes()).to_string();

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
                    Ok(None) => {
                        let _ = child.kill();
                        ExecuteResult {
                            stdout: String::new(),
                            stderr: format!(
                                "Command timed out after {} seconds",
                                self.timeout_secs
                            ),
                            exit_code: None,
                            timed_out: true,
                            truncated: false,
                        }
                    }
                    Err(_) => ExecuteResult {
                        stdout: String::new(),
                        stderr: "Failed to wait for process".to_string(),
                        exit_code: None,
                        timed_out: false,
                        truncated: false,
                    },
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

use std::io::Read;

trait WaitTimeout {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl WaitTimeout for std::process::Child {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        use std::thread;
        use std::time::Instant;

        let start = Instant::now();
        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
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
        let result = handler.execute("python3 -c \"print('a' * 200)\"");
        assert!(result.truncated);
        assert!(result.stdout.len() <= 100 + 50);
    }

    #[test]
    fn test_dangerous_command_detection() {
        let handler = handler();

        assert!(handler.is_dangerous("rm -rf /").is_some());
        assert!(handler.is_dangerous("sudo rm -rf /tmp").is_some());
        assert!(handler.is_dangerous("mkfs.ext4 /dev/sda").is_some());
        assert!(handler.is_dangerous("echo hello").is_none());
    }

    #[test]
    fn test_command_validation_blocks_dangerous() {
        let handler = handler();

        assert!(handler.validate_command("rm -rf /").is_err());
        assert!(handler.validate_command("sudo chmod 777 /").is_err());
        assert!(handler.validate_command("echo safe").is_ok());
    }

    #[test]
    fn test_working_directory_restriction() {
        let handler = ShellHandler::new().with_working_dir(PathBuf::from("/tmp"));

        assert!(handler.validate_command("ls -la").is_ok());
        assert!(!handler.is_within_working_dir("cat ../../etc/passwd"));
    }
}
