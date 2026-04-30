//! E2E workflow tests for TUI applications using PTY.
//!
//! This module provides integration tests that spawn the opencode-rs TUI binary
//! and verify user workflows through a pseudo-terminal.

use std::time::Duration;

use crate::PtySimulator;
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Builder for TUI E2E test scenarios.
pub struct TuiWorkflowBuilder {
    binary_path: String,
    args: Vec<String>,
    env_vars: Vec<(String, String)>,
    terminal_size: (u16, u16),
    timeout_secs: u64,
}

impl TuiWorkflowBuilder {
    pub fn new(binary_path: &str) -> Self {
        Self {
            binary_path: binary_path.to_string(),
            args: Vec::new(),
            env_vars: Vec::new(),
            terminal_size: (120, 40),
            timeout_secs: 30,
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_size(mut self, cols: u16, rows: u16) -> Self {
        self.terminal_size = (cols, rows);
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Spawn the TUI and return a handle for interaction.
    pub async fn spawn(self) -> Result<TuiSession> {
        let mut cmd_args = vec![self.binary_path.as_str()];
        cmd_args.extend(self.args.iter().map(|s| s.as_str()));

        let mut pty =
            PtySimulator::new_with_command(&cmd_args).context("Failed to create PTY for TUI")?;

        pty.resize(self.terminal_size.0, self.terminal_size.1)
            .context("Failed to resize PTY")?;

        Ok(TuiSession {
            pty,
            timeout: Duration::from_secs(self.timeout_secs),
        })
    }
}

/// Active TUI session for interacting with a spawned terminal.
pub struct TuiSession {
    pty: PtySimulator,
    timeout: Duration,
}

impl TuiSession {
    /// Wait for a specific pattern to appear in the terminal output.
    pub async fn wait_for(&mut self, pattern: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let mut buffer = String::new();

        while start.elapsed() < self.timeout {
            let output = self
                .pty
                .read_output(Duration::from_millis(100))
                .context("Failed to read PTY output")?;

            buffer.push_str(&output);

            if buffer.contains(pattern) {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        anyhow::bail!(
            "Timeout waiting for pattern '{}'. Buffer contents:\n{}",
            pattern,
            buffer
        );
    }

    /// Send a line of input to the TUI.
    pub async fn send_line(&mut self, line: &str) -> Result<()> {
        self.pty
            .write_input(&format!("{}\r", line))
            .context("Failed to send line")?;
        Ok(())
    }

    /// Send a key event to the TUI.
    pub async fn send_key(&mut self, key: KeyEvent) -> Result<()> {
        self.pty
            .inject_key_event(key)
            .context("Failed to inject key event")?;
        Ok(())
    }

    /// Send Ctrl+C to interrupt.
    pub async fn send_ctrl_c(&mut self) -> Result<()> {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        self.send_key(ctrl_c).await
    }

    /// Send Escape key.
    pub async fn send_escape(&mut self) -> Result<()> {
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        self.send_key(esc).await
    }

    /// Read current terminal output without waiting.
    pub fn read_output(&mut self) -> Result<String> {
        self.pty.read_output(Duration::from_millis(50))
    }

    /// Check if the child process is still running.
    pub fn is_alive(&self) -> bool {
        self.pty.is_child_running()
    }

    /// Get the exit status of the child process.
    pub async fn wait_for_exit(mut self) -> Result<ExitStatus> {
        let mut buffer = String::new();

        while self.pty.is_child_running() {
            let output = self
                .pty
                .read_output(Duration::from_millis(100))
                .unwrap_or_default();
            buffer.push_str(&output);
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Drain remaining output
        loop {
            let output = self.pty.read_output(Duration::from_millis(100))?;
            if output.is_empty() {
                break;
            }
            buffer.push_str(&output);
        }

        Ok(ExitStatus { output: buffer })
    }
}

/// Exit status information from a TUI session.
#[derive(Debug)]
pub struct ExitStatus {
    pub output: String,
}

/// Helper to create a basic workflow test.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_workflow_builder_basic() {
        let builder = TuiWorkflowBuilder::new("echo");
        assert_eq!(builder.binary_path, "echo");
    }

    #[test]
    fn test_tui_session_read_output() {
        let mut pty = PtySimulator::new_with_command(&["echo", "test"]).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        let output = pty.read_output(Duration::from_millis(200)).unwrap();
        assert!(output.contains("test") || output.contains("ready"));
        drop(pty);
    }
}
