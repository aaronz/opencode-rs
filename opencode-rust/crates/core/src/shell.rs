use std::process::{Command, Stdio};

pub struct Shell;

fn contains_shell_injection(cmd: &str) -> bool {
    let chars: Vec<char> = cmd.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        match c {
            ';' | '|' | '&' | '<' | '>' | '#' | '\n' => {
                return true;
            }
            '$' => {
                if i + 1 < chars.len() {
                    let next = chars[i + 1];
                    if next == '(' || next == '{' {
                        return true;
                    }
                }
            }
            '`' => {
                return true;
            }
            _ => {}
        }
    }

    if cmd.contains("&&") || cmd.contains("||") {
        return true;
    }

    let lower = cmd.to_lowercase();
    if lower.contains("rm -rf") || lower.contains("dd if=") || lower.contains(":(){:|:&}:") {
        return true;
    }

    false
}

impl Shell {
    pub fn execute(cmd: &str) -> Result<String, crate::OpenCodeError> {
        if contains_shell_injection(cmd) {
            return Err(crate::OpenCodeError::Tool(
                "Shell command contains forbidden characters or patterns".to_string(),
            ));
        }

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

    #[test]
    fn test_shell_injection_blocked_semicolon() {
        let result = Shell::execute("echo hello; rm -rf /");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("forbidden"));
    }

    #[test]
    fn test_shell_injection_blocked_pipe() {
        let result = Shell::execute("echo hello | cat /etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_backtick() {
        let result = Shell::execute("echo `whoami`");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_dollar_substitution() {
        let result = Shell::execute("echo $(whoami)");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_and_and() {
        let result = Shell::execute("echo hello && ls");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_or_or() {
        let result = Shell::execute("echo hello || ls");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_newline() {
        let result = Shell::execute("echo hello\nwhoami");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_redirect() {
        let result = Shell::execute("echo hello > /tmp/test.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_injection_blocked_rm_rf() {
        let result = Shell::execute("rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_safe_commands_allowed() {
        assert!(Shell::execute("echo hello").is_ok());
        assert!(Shell::execute("ls /tmp").is_ok());
        assert!(Shell::execute("pwd").is_ok());
    }

    #[test]
    fn test_contains_shell_injection_detects_dangerous() {
        assert!(contains_shell_injection("; rm -rf /"));
        assert!(contains_shell_injection("`id`"));
        assert!(contains_shell_injection("$(whoami)"));
        assert!(contains_shell_injection("hello && ls"));
        assert!(contains_shell_injection("hello || ls"));
        assert!(contains_shell_injection("echo hello\nwhoami"));
        assert!(contains_shell_injection("echo test > file"));
        assert!(contains_shell_injection("rm -rf /"));
        assert!(contains_shell_injection(":(){:|:&}:")); // fork bomb
        assert!(contains_shell_injection("echo `whoami`"));
    }

    #[test]
    fn test_contains_shell_injection_allows_safe() {
        assert!(!contains_shell_injection("echo hello"));
        assert!(!contains_shell_injection("ls /tmp"));
        assert!(!contains_shell_injection("pwd"));
        assert!(!contains_shell_injection("cat /tmp/test.txt"));
        assert!(!contains_shell_injection("echo $HOME"));
        assert!(!contains_shell_injection("echo $PATH"));
    }

    #[test]
    fn test_shell_injection_blocked_environment_var() {
        let result = Shell::execute("echo $HOME");
        assert!(result.is_ok());
    }
}
