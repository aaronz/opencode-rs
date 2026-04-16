use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug)]
pub struct CliTester {
    command: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    temp_dir: Option<TempDir>,
    capture_stdout: bool,
    capture_stderr: bool,
}

impl CliTester {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            args: Vec::new(),
            env_vars: HashMap::new(),
            working_dir: None,
            temp_dir: None,
            capture_stdout: true,
            capture_stderr: true,
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

    pub fn capture_stdout(mut self) -> Self {
        self.capture_stdout = true;
        self
    }

    pub fn capture_stderr(mut self) -> Self {
        self.capture_stderr = true;
        self
    }

    pub async fn run(&self) -> Result<CliOutput> {
        self.run_with_timeout(std::time::Duration::from_secs(30))
            .await
    }

    pub async fn run_with_timeout(&self, timeout: std::time::Duration) -> Result<CliOutput> {
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

        if self.capture_stdout {
            cmd.stdout(Stdio::piped());
        } else {
            cmd.stdout(Stdio::inherit());
        }

        if self.capture_stderr {
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stderr(Stdio::inherit());
        }

        let child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn process '{}' with args {:?}",
                self.command, self.args
            )
        })?;

        let output = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .with_context(|| "Process timed out")?
            .with_context(|| "Failed to wait for process")?;

        Ok(CliOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        })
    }

    pub async fn spawn(&self) -> Result<ChildProcess> {
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

        if self.capture_stdout {
            cmd.stdout(Stdio::piped());
        } else {
            cmd.stdout(Stdio::inherit());
        }

        if self.capture_stderr {
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stderr(Stdio::inherit());
        }

        let child = cmd.spawn().with_context(|| {
            format!(
                "Failed to spawn process '{}' with args {:?}",
                self.command, self.args
            )
        })?;

        Ok(ChildProcess { inner: child })
    }
}

impl Default for CliTester {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug)]
pub struct ChildProcess {
    inner: tokio::process::Child,
}

impl ChildProcess {
    pub async fn wait(mut self) -> Result<CliOutput> {
        let status = self.inner.wait().await?;
        let exit_code = status.code().unwrap_or(-1);
        let success = status.success();

        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();

        if let Some(ref mut stdout) = self.inner.stdout.take() {
            let mut reader = BufReader::new(stdout);
            reader.read_line(&mut stdout_buf).await.ok();
        }

        if let Some(ref mut stderr) = self.inner.stderr.take() {
            let mut reader = BufReader::new(stderr);
            reader.read_line(&mut stderr_buf).await.ok();
        }

        Ok(CliOutput {
            stdout: stdout_buf,
            stderr: stderr_buf,
            exit_code,
            success,
        })
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.inner.kill().await?;
        Ok(())
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

    #[tokio::test]
    async fn test_cli_tester_spawn_process_with_args() {
        let tester = CliTester::new("echo").arg("hello").arg("world");
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(output.stdout.contains("hello"), "Should contain 'hello'");
        assert!(output.stdout.contains("world"), "Should contain 'world'");
    }

    #[tokio::test]
    async fn test_cli_tester_exit_code_capture() {
        let tester = CliTester::new("sh").arg("-c").arg("exit 42");
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert_eq!(output.exit_code, 42, "Should capture exit code 42");
        assert!(!output.success, "Should not be successful");
    }

    #[tokio::test]
    async fn test_cli_tester_stdout_capture() {
        let tester = CliTester::new("printf").arg("test output");
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(
            output.stdout.contains("test output"),
            "Should capture stdout"
        );
    }

    #[tokio::test]
    async fn test_cli_tester_stderr_capture() {
        let tester = CliTester::new("sh").arg("-c").arg("echo error >&2");
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(output.stderr.contains("error"), "Should capture stderr");
    }

    #[tokio::test]
    async fn test_cli_tester_temp_dir_cleanup() {
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

    #[tokio::test]
    async fn test_cli_tester_working_dir() {
        let tester = CliTester::new("pwd").working_dir(PathBuf::from("/tmp"));
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(
            output.stdout.contains("/tmp"),
            "Should run in specified working dir"
        );
    }

    #[tokio::test]
    async fn test_cli_tester_env_vars() {
        let tester = CliTester::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "test_value");
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn successfully");
        let output = result.unwrap();
        assert!(
            output.stdout.contains("test_value"),
            "Should capture env var value"
        );
    }

    #[tokio::test]
    async fn test_cli_output_assertions() {
        let tester = CliTester::new("echo").arg("success");
        let output = tester.run().await.unwrap();
        output.assert_success().expect("Should assert success");
        output
            .assert_stdout_contains("success")
            .expect("Should contain 'success'");
    }

    #[tokio::test]
    async fn test_cli_output_assertion_failure() {
        let tester = CliTester::new("sh").arg("-c").arg("exit 1");
        let output = tester.run().await.unwrap();
        let result = output.assert_success();
        assert!(
            result.is_err(),
            "assert_success should fail for non-zero exit"
        );
    }

    #[tokio::test]
    async fn test_cli_output_exit_code_assertion() {
        let tester = CliTester::new("true");
        let output = tester.run().await.unwrap();
        output
            .assert_exit_code(0)
            .expect("Should assert exit code 0");
    }

    #[tokio::test]
    async fn test_cli_tester_multiple_args() {
        let tester = CliTester::new("printf").args(&["arg1", "arg2", "arg3"]);
        let result = tester.run().await;
        assert!(result.is_ok(), "Process should spawn with multiple args");
    }

    #[tokio::test]
    async fn test_cli_tester_nonexistent_command() {
        let tester = CliTester::new("nonexistent_command_xyz_12345");
        let result = tester.run().await;
        assert!(result.is_err(), "Should fail with nonexistent command");
    }

    #[tokio::test]
    async fn test_cli_tester_spawn_and_wait() {
        let tester = CliTester::new("echo").arg("spawned_process");
        let child = tester.spawn().await.expect("Should spawn process");
        let output = child.wait().await.expect("Should wait for process");
        assert!(
            output.stdout.contains("spawned_process"),
            "Should capture output"
        );
    }

    #[tokio::test]
    async fn test_cli_tester_captures_stdout_stderr() {
        let tester = CliTester::new("sh")
            .arg("-c")
            .arg("echo stdout_content && echo stderr_content >&2");
        let output = tester.run().await.expect("Should run successfully");
        assert!(
            output.stdout.contains("stdout_content"),
            "Should capture stdout"
        );
        assert!(
            output.stderr.contains("stderr_content"),
            "Should capture stderr"
        );
    }

    #[tokio::test]
    async fn test_capture_stdout_returns_self() {
        let tester = CliTester::new("echo").capture_stdout();
        let result = tester.run().await;
        assert!(result.is_ok(), "capture_stdout should return self and work");
    }

    #[tokio::test]
    async fn test_capture_stderr_returns_self() {
        let tester = CliTester::new("echo").capture_stderr();
        let result = tester.run().await;
        assert!(result.is_ok(), "capture_stderr should return self and work");
    }

    #[tokio::test]
    async fn test_capture_stdout_stderr_chainable() {
        let tester = CliTester::new("echo")
            .arg("chain_test")
            .capture_stdout()
            .capture_stderr();
        let output = tester
            .run()
            .await
            .expect("Should run with chained capture methods");
        assert!(
            output.stdout.contains("chain_test"),
            "Should capture stdout after fluent chain"
        );
    }
}
