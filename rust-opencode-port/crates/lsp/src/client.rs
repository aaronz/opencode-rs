use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Command, Child};
use tokio::sync::{oneshot, Mutex};
use std::sync::Arc;
use opencode_core::OpenCodeError;

use crate::types::{Diagnostic, Location, CompletionItem, Range, Position};

pub struct LspClient {
    process: Option<Child>,
    request_id: u64,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<String>>>>,
    stdin: Option<tokio::process::ChildStdin>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            process: None,
            request_id: 0,
            pending: Arc::new(Mutex::new(HashMap::new())),
            stdin: None,
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

        let mut child = cmd.spawn()
            .map_err(|e| OpenCodeError::Tui(format!("Failed to spawn LSP server: {}", e)))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        self.process = Some(child);
        self.stdin = stdin;
        self.request_id = 0;

        if let Some(stdout) = stdout {
            let pending = self.pending.clone();
            tokio::spawn(async move {
                let mut stdout = stdout;
                let mut buf = Vec::new();
                loop {
                    let mut temp = [0u8; 8192];
                    match stdout.read(&mut temp).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&temp[..n]);
                            loop {
                                if let Some(msg) = extract_jsonrpc_message(&buf) {
                                    let len = msg.len();
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg) {
                                        if let Some(id) = v.get("id").and_then(|id| id.as_u64()) {
                                            let mut p = pending.lock().await;
                                            if let Some(tx) = p.remove(&id) {
                                                let _ = tx.send(msg);
                                            }
                                        }
                                    }
                                    buf = buf.split_at(len.min(buf.len())).1.to_vec();
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }

        self.send_initialize().await?;

        Ok(())
    }

    async fn send_initialize(&mut self) -> Result<(), OpenCodeError> {
        let params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": null,
            "capabilities": {}
        });
        let _ = self.send_request("initialize", params).await;
        let _ = self.send_notification("initialized", serde_json::json!({})).await;
        Ok(())
    }

    async fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<u64, OpenCodeError> {
        let id = self.request_id;
        self.request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let msg = serde_json::to_string(&request).map_err(|e| OpenCodeError::Tui(e.to_string()))?;
        let msg = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);

        if let Some(ref mut stdin) = self.stdin {
            stdin.write_all(msg.as_bytes()).await.map_err(|e| OpenCodeError::Tui(e.to_string()))?;
            stdin.flush().await.map_err(|e| OpenCodeError::Tui(e.to_string()))?;
        }

        Ok(id)
    }

    pub async fn wait_for_response(&mut self, id: u64, timeout_secs: u64) -> Result<serde_json::Value, OpenCodeError> {
        let (tx, rx) = oneshot::channel::<String>();
        {
            let mut p = self.pending.lock().await;
            p.insert(id, tx);
        }

        let resp = match tokio::time::timeout(tokio::time::Duration::from_secs(timeout_secs), rx).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => return Err(OpenCodeError::Tool(format!("LSP request {} failed: {}", id, e))),
            Err(_) => return Err(OpenCodeError::Tool(format!("LSP request {} timed out", id))),
        };

        serde_json::from_str(&resp).map_err(|e| OpenCodeError::Tool(format!("Invalid LSP response: {}", e)))
    }

    async fn send_notification(&mut self, method: &str, params: serde_json::Value) -> Result<(), OpenCodeError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let msg = serde_json::to_string(&notification).map_err(|e| OpenCodeError::Tui(e.to_string()))?;
        let msg = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);

        if let Some(ref mut stdin) = self.stdin {
            stdin.write_all(msg.as_bytes()).await.map_err(|e| OpenCodeError::Tui(e.to_string()))?;
            stdin.flush().await.map_err(|e| OpenCodeError::Tui(e.to_string()))?;
        }

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

    pub async fn goto_definition(&mut self, uri: &str, line: u32, col: u32) -> Result<Option<Location>, OpenCodeError> {
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col }
        });

        let id = self.send_request("textDocument/definition", params).await?;
        let resp = self.wait_for_response(id, 5).await?;

        if let Some(result) = resp.get("result") {
            if result.is_null() {
                return Ok(None);
            }
            return Ok(parse_location(result));
        }

        Ok(None)
    }

    pub async fn find_references(&mut self, uri: &str, line: u32, col: u32) -> Result<Vec<Location>, OpenCodeError> {
        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col },
            "context": { "includeDeclaration": true }
        });

        let id = self.send_request("textDocument/references", params).await?;
        let resp = self.wait_for_response(id, 5).await?;

        if let Some(result) = resp.get("result") {
            if let Some(arr) = result.as_array() {
                let locations: Vec<Location> = arr.iter().filter_map(parse_location).collect();
                return Ok(locations);
            }
        }

        Ok(Vec::new())
    }

    pub async fn completion(&mut self, _uri: &str, _line: u32, _col: u32) -> Result<Vec<CompletionItem>, OpenCodeError> {
        Ok(Vec::new())
    }

    pub async fn shutdown(&mut self) -> Result<(), OpenCodeError> {
        let _ = self.send_notification("shutdown", serde_json::json!({})).await;
        if let Some(mut process) = self.process.take() {
            process.kill().await.ok();
        }
        Ok(())
    }
}

fn extract_jsonrpc_message(buf: &[u8]) -> Option<String> {
    let header = b"Content-Length: ";
    let idx = buf.windows(header.len()).position(|w| w == header)?;

    let after_header = &buf[idx + header.len()..];
    let end_of_headers = after_header.windows(4).position(|w| w == b"\r\n\r\n")?;

    let len_str = std::str::from_utf8(&after_header[..end_of_headers])
        .ok()?.trim();
    let len: usize = len_str.parse().ok()?;

    let msg_start = idx + header.len() + end_of_headers + 4;
    if buf.len() < msg_start + len {
        return None;
    }

    let msg = std::str::from_utf8(&buf[msg_start..msg_start + len]).ok()?;
    Some(msg.to_string())
}

fn parse_location(v: &serde_json::Value) -> Option<Location> {
    let obj = v.as_object()?;
    let uri = obj.get("uri")?.as_str()?.to_string();
    let range = obj.get("range")?.as_object()?;
    let start = range.get("start")?.as_object()?;
    let end = range.get("end")?.as_object()?;

    let line = start.get("line")?.as_u64()? as u32;
    let character = start.get("character")?.as_u64()? as u32;
    let end_line = end.get("line")?.as_u64()? as u32;
    let end_character = end.get("character")?.as_u64()? as u32;

    Some(Location {
        uri,
        range: Range {
            start: Position { line, character },
            end: Position { line: end_line, character: end_character },
        },
    })
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
