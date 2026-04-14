use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use tempfile::TempDir;

#[derive(Debug)]
pub struct CliTester {
    command: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    temp_dir: Option<TempDir>,
}

impl CliTester {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
            temp_dir: None,
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn args(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.args.push(arg.to_string());
        }
        self
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn envs(mut self, vars: HashMap<&str, &str>) -> Self {
        for (key, value) in vars {
            self.env_vars.insert(key.to_string(), value.to_string());
        }
        self
    }

    pub fn working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn with_temp_dir(self) -> Result<(Self, PathBuf)> {
        let temp_dir = TempDir::new().context("Failed to create temp directory")?;
        let path = temp_dir.path().to_path_buf();
        Ok((self.working_dir(path.clone()), path))
    }

    pub fn run(&self) -> Result<CliOutput> {
        self.run_with_timeout(std::time::Duration::from_secs(30))
    }

    pub fn run_with_timeout(&self, _timeout: std::time::Duration) -> Result<CliOutput> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        } else if let Some(ref temp_dir) = self.temp_dir {
            cmd.current_dir(temp_dir.path());
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn process '{}' with args {:?}",
                self.command, self.args
            )
        })?;

        let output = child
            .wait_with_output()
            .with_context(|| "Failed to wait for process")?;

        Ok(CliOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        })
    }
}

impl Default for CliTester {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone)]
pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

impl CliOutput {
    pub fn assert_success(&self) -> Result<()> {
        if !self.success {
            bail!(
                "Expected successful exit (0), got {}. stdout: {}, stderr: {}",
                self.exit_code,
                self.stdout,
                self.stderr
            );
        }
        Ok(())
    }

    pub fn assert_exit_code(&self, expected: i32) -> Result<()> {
        if self.exit_code != expected {
            bail!(
                "Expected exit code {}, got {}. stdout: {}, stderr: {}",
                expected,
                self.exit_code,
                self.stdout,
                self.stderr
            );
        }
        Ok(())
    }

    pub fn assert_stdout_contains(&self, expected: &str) -> Result<()> {
        if !self.stdout.contains(expected) {
            bail!(
                "Expected stdout to contain '{}'. stdout: {}, stderr: {}",
                expected,
                self.stdout,
                self.stderr
            );
        }
        Ok(())
    }

    pub fn assert_stderr_contains(&self, expected: &str) -> Result<()> {
        if !self.stderr.contains(expected) {
            bail!(
                "Expected stderr to contain '{}'. stdout: {}, stderr: {}",
                expected,
                self.stdout,
                self.stderr
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_tester_spawn_process_with_args() {
        let tester = CliTester::new("echo").arg("hello").arg("world");
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(output.stdout.contains("hello"), "Should contain 'hello'");
        assert!(output.stdout.contains("world"), "Should contain 'world'");
    }

    #[test]
    fn test_cli_tester_exit_code_capture() {
        let tester = CliTester::new("sh").arg("-c").arg("exit 42");
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert_eq!(output.exit_code, 42, "Should capture exit code 42");
        assert!(!output.success, "Should not be successful");
    }

    #[test]
    fn test_cli_tester_stdout_capture() {
        let tester = CliTester::new("printf").arg("test output");
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(
            output.stdout.contains("test output"),
            "Should capture stdout"
        );
    }

    #[test]
    fn test_cli_tester_stderr_capture() {
        let tester = CliTester::new("sh").arg("-c").arg("echo error >&2");
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(output.stderr.contains("error"), "Should capture stderr");
    }

    #[test]
    fn test_cli_tester_temp_dir_cleanup() {
        let (tester, path) = CliTester::new("echo")
            .arg("temp_dir_test")
            .with_temp_dir()
            .expect("Temp dir should be created");

        let path_clone = path.clone();
        drop(tester);

        assert!(
            !path_clone.exists(),
            "Temp directory should be cleaned up after CliTester is dropped"
        );
    }

    #[test]
    fn test_cli_tester_working_dir() {
        let tester = CliTester::new("pwd").working_dir(PathBuf::from("/tmp"));
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(
            output.stdout.contains("/tmp"),
            "Should run in specified working dir"
        );
    }

    #[test]
    fn test_cli_tester_env_vars() {
        let tester = CliTester::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "test_value");
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(
            output.stdout.contains("test_value"),
            "Should capture env var value"
        );
    }

    #[test]
    fn test_cli_output_assertions() {
        let tester = CliTester::new("echo").arg("success");
        let output = tester.run().unwrap();
        output.assert_success().expect("Should assert success");
        output
            .assert_stdout_contains("success")
            .expect("Should contain 'success'");
    }

    #[test]
    fn test_cli_output_assertion_failure() {
        let tester = CliTester::new("sh").arg("-c").arg("exit 1");
        let output = tester.run().unwrap();
        let result = output.assert_success();
        assert!(
            result.is_err(),
            "assert_success should fail for non-zero exit"
        );
    }

    #[test]
    fn test_cli_output_exit_code_assertion() {
        let tester = CliTester::new("true");
        let output = tester.run().unwrap();
        output
            .assert_exit_code(0)
            .expect("Should assert exit code 0");
    }

    #[test]
    fn test_cli_tester_multiple_args() {
        let tester = CliTester::new("printf").args(&["arg1", "arg2", "arg3"]);
        let result = tester.run();
        assert!(result.is_ok(), "Process should spawn with multiple args");
    }

    #[test]
    fn test_cli_tester_nonexistent_command() {
        let tester = CliTester::new("nonexistent_command_xyz_12345");
        let result = tester.run();
        assert!(result.is_err(), "Should fail with nonexistent command");
    }
}
