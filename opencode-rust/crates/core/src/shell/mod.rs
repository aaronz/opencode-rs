mod types;

pub use types::Shell;

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
        assert!(types::contains_shell_injection("; rm -rf /"));
        assert!(types::contains_shell_injection("`id`"));
        assert!(types::contains_shell_injection("$(whoami)"));
        assert!(types::contains_shell_injection("hello && ls"));
        assert!(types::contains_shell_injection("hello || ls"));
        assert!(types::contains_shell_injection("echo hello\nwhoami"));
        assert!(types::contains_shell_injection("echo test > file"));
        assert!(types::contains_shell_injection("rm -rf /"));
        assert!(types::contains_shell_injection(":(){:|:&}:")); // fork bomb
        assert!(types::contains_shell_injection("echo `whoami`"));
    }

    #[test]
    fn test_contains_shell_injection_allows_safe() {
        assert!(!types::contains_shell_injection("echo hello"));
        assert!(!types::contains_shell_injection("ls /tmp"));
        assert!(!types::contains_shell_injection("pwd"));
        assert!(!types::contains_shell_injection("cat /tmp/test.txt"));
        assert!(!types::contains_shell_injection("echo $HOME"));
        assert!(!types::contains_shell_injection("echo $PATH"));
    }

    #[test]
    fn test_shell_injection_blocked_environment_var() {
        let result = Shell::execute("echo $HOME");
        assert!(result.is_ok());
    }
}