use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::sync::oneshot;
use opencode_core::OpenCodeError;

use crate::types::{Diagnostic, Location, CompletionItem};

pub struct LspClient {
    process: Option<Child>,
    request_id: u64,
    _pending: HashMap<u64, oneshot::Sender<String>>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            process: None,
            request_id: 0,
            _pending: HashMap::new(),
        }
    }

    pub async fn start(&mut self, server_command: &str, root_path: &PathBuf) -> Result<(), OpenCodeError> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(server_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(root_path);

        let child = cmd.spawn()
            .map_err(|e| OpenCodeError::Tui(format!("Failed to spawn LSP server: {}", e)))?;

        self.process = Some(child);
        self.request_id = 0;

        Ok(())
    }

    pub fn detect_language_server(root: &PathBuf) -> Option<String> {
        if root.join("Cargo.toml").exists() {
            return Some("rust-analyzer".to_string());
        }
        if root.join("package.json").exists() {
            if root.join("tsconfig.json").exists() {
                return Some("typescript-language-server --stdio".to_string());
            }
            return Some("node_modules/.bin/typescript-language-server --stdio".to_string());
        }
        if root.join("go.mod").exists() {
            return Some("gopls".to_string());
        }
        if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            return Some("pylsp".to_string());
        }
        None
    }

    pub async fn initialize(&mut self, root_path: &PathBuf) -> Result<(), OpenCodeError> {
        if let Some(server) = Self::detect_language_server(root_path) {
            self.start(&server, root_path).await?;
        }
        Ok(())
    }

    pub async fn get_diagnostics(&mut self, _uri: &str) -> Result<Vec<Diagnostic>, OpenCodeError> {
        Ok(Vec::new())
    }

    pub async fn goto_definition(&mut self, _uri: &str, _line: u32, _col: u32) -> Result<Option<Location>, OpenCodeError> {
        Ok(None)
    }

    pub async fn find_references(&mut self, _uri: &str, _line: u32, _col: u32) -> Result<Vec<Location>, OpenCodeError> {
        Ok(Vec::new())
    }

    pub async fn completion(&mut self, _uri: &str, _line: u32, _col: u32) -> Result<Vec<CompletionItem>, OpenCodeError> {
        Ok(Vec::new())
    }

    pub async fn shutdown(&mut self) -> Result<(), OpenCodeError> {
        if let Some(mut process) = self.process.take() {
            process.kill().await.ok();
        }
        Ok(())
    }
}

impl Default for LspClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        if let Some(ref mut process) = self.process {
            let _ = process.kill();
        }
    }
}
