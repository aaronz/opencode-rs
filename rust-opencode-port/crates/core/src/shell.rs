use std::process::{Command, Stdio};

pub struct Shell;

impl Shell {
    pub fn execute(cmd: &str) -> Result<String, crate::OpenCodeError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| crate::OpenCodeError::Tool(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Err(crate::OpenCodeError::Tool(stderr.to_string()))
        }
    }

    pub fn which(program: &str) -> Option<String> {
        Self::execute(&format!("which {}", program)).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_execute_simple() {
        let result = Shell::execute("echo hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_shell_execute_failure() {
        let result = Shell::execute("exit 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_which_exists() {
        let result = Shell::which("ls");
        assert!(result.is_some());
    }

    #[test]
    fn test_shell_which_not_exists() {
        let result = Shell::which("nonexistent_command_12345");
        assert!(result.is_none());
    }
}
