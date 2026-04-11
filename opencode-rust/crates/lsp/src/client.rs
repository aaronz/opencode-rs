use crate::error::{
    CrashCause, FailureHandlingConfig, LspError, ProtocolViolationType, UnhealthyReason,
};
use crate::types::{CompletionItem, Diagnostic, Location, Position, Range, Severity};
use opencode_core::OpenCodeError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::{oneshot, Mutex};

pub struct LspClient {
    process: Option<Child>,
    request_id: u64,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<String, LspError>>>>>,
    stdin: Option<tokio::process::ChildStdin>,
    diagnostics: Arc<Mutex<HashMap<String, Vec<Diagnostic>>>>,
    server_name: String,
    is_running: Arc<AtomicBool>,
    consecutive_errors: Arc<AtomicU32>,
    config: FailureHandlingConfig,
}

impl LspClient {
    pub fn new() -> Self {
        Self::with_config(FailureHandlingConfig::default())
    }

    pub fn with_config(config: FailureHandlingConfig) -> Self {
        Self {
            process: None,
            request_id: 0,
            pending: Arc::new(Mutex::new(HashMap::new())),
            stdin: None,
            diagnostics: Arc::new(Mutex::new(HashMap::new())),
            server_name: String::new(),
            is_running: Arc::new(AtomicBool::new(false)),
            consecutive_errors: Arc::new(AtomicU32::new(0)),
            config,
        }
    }

    pub async fn start(
        &mut self,
        server_command: &str,
        root_path: &PathBuf,
    ) -> Result<(), OpenCodeError> {
        self.start_with_name(server_command, root_path, server_command.to_string())
            .await
    }

    pub async fn start_with_name(
        &mut self,
        server_command: &str,
        root_path: &PathBuf,
        server_name: String,
    ) -> Result<(), OpenCodeError> {
        if let Some(mut old_process) = self.process.take() {
            let _ = old_process.kill().await;
        }

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(server_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(root_path);

        let mut child = cmd
            .spawn()
            .map_err(|e| OpenCodeError::Tui(format!("Failed to spawn LSP server: {}", e)))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        self.process = Some(child);
        self.stdin = stdin;
        self.request_id = 0;
        self.server_name = server_name;
        self.is_running.store(true, Ordering::SeqCst);
        self.consecutive_errors.store(0, Ordering::SeqCst);

        if let Some(stdout) = stdout {
            let pending = self.pending.clone();
            let diagnostics = self.diagnostics.clone();
            let is_running = self.is_running.clone();
            let consecutive_errors = self.consecutive_errors.clone();
            let max_errors = self.config.max_consecutive_errors;
            let server_name = self.server_name.clone();

            tokio::spawn(async move {
                let mut stdout = stdout;
                let mut buf = Vec::new();
                loop {
                    let mut temp = [0u8; 8192];
                    match stdout.read(&mut temp).await {
                        Ok(0) => {
                            is_running.store(false, Ordering::SeqCst);
                            break;
                        }
                        Err(_) => {
                            is_running.store(false, Ordering::SeqCst);
                            break;
                        }
                        Ok(n) => {
                            buf.extend_from_slice(&temp[..n]);
                            loop {
                                match extract_jsonrpc_message(&buf) {
                                    Ok(msg) => {
                                        let len = msg.len();
                                        match serde_json::from_str::<serde_json::Value>(&msg) {
                                            Ok(v) => {
                                                consecutive_errors.store(0, Ordering::SeqCst);
                                                if let Some(id) =
                                                    v.get("id").and_then(|id| id.as_u64())
                                                {
                                                    let mut p = pending.lock().await;
                                                    if let Some(tx) = p.remove(&id) {
                                                        let _ = tx.send(Ok(msg));
                                                    }
                                                } else if let Some(method) =
                                                    v.get("method").and_then(|m| m.as_str())
                                                {
                                                    if method == "textDocument/publishDiagnostics" {
                                                        if let Some(params) = v
                                                            .get("params")
                                                            .and_then(|p| p.as_object())
                                                        {
                                                            if let Some(uri) = params
                                                                .get("uri")
                                                                .and_then(|u| u.as_str())
                                                            {
                                                                if let Some(diagnostics_arr) =
                                                                    params
                                                                        .get("diagnostics")
                                                                        .and_then(|d| d.as_array())
                                                                {
                                                                    let parsed_diagnostics: Vec<
                                                                        Diagnostic,
                                                                    > = diagnostics_arr
                                                                        .iter()
                                                                        .filter_map(
                                                                            parse_diagnostic,
                                                                        )
                                                                        .collect();
                                                                    let mut diags =
                                                                        diagnostics.lock().await;
                                                                    diags.insert(
                                                                        uri.to_string(),
                                                                        parsed_diagnostics,
                                                                    );
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                let err = LspError::ProtocolViolation {
                                                    violation: ProtocolViolationType::InvalidJson,
                                                    detail: e.to_string(),
                                                };
                                                let mut p = pending.lock().await;
                                                for (_, tx) in p.drain() {
                                                    let _ = tx.send(Err(err.clone()));
                                                }
                                            }
                                        }
                                        if len >= buf.len() {
                                            buf.clear();
                                        } else {
                                            buf = buf.split_at(len.min(buf.len())).1.to_vec();
                                        }
                                    }
                                    Err(LspError::ProtocolViolation { violation, detail }) => {
                                        consecutive_errors.fetch_add(1, Ordering::SeqCst);
                                        if consecutive_errors.load(Ordering::SeqCst) >= max_errors {
                                            is_running.store(false, Ordering::SeqCst);
                                            let err = LspError::ServerUnhealthy {
                                                server_name: server_name.clone(),
                                                reason: UnhealthyReason::ErrorThresholdExceeded,
                                            };
                                            let mut p = pending.lock().await;
                                            for (_, tx) in p.drain() {
                                                let _ = tx.send(Err(err.clone()));
                                            }
                                            break;
                                        }
                                        let _ = violation;
                                        let _ = detail;
                                        if buf.len() > len_after_header(&buf).unwrap_or(buf.len()) {
                                            buf = buf
                                                .split_at(
                                                    len_after_header(&buf).unwrap_or(buf.len()),
                                                )
                                                .1
                                                .to_vec();
                                        } else {
                                            buf.clear();
                                        }
                                    }
                                    Err(e) => {
                                        let mut p = pending.lock().await;
                                        for (_, tx) in p.drain() {
                                            let _ = tx.send(Err(e.clone()));
                                        }
                                        break;
                                    }
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
        let _ = self
            .send_notification("initialized", serde_json::json!({}))
            .await;
        Ok(())
    }

    async fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<u64, OpenCodeError> {
        let id = self.request_id;
        self.request_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        self.send_jsonrpc_message(&request).await?;

        Ok(id)
    }

    async fn send_jsonrpc_message(&mut self, msg: &serde_json::Value) -> Result<(), OpenCodeError> {
        let msg_str = serde_json::to_string(msg).map_err(|e| OpenCodeError::Tui(e.to_string()))?;
        let msg = format!("Content-Length: {}\r\n\r\n{}", msg_str.len(), msg_str);

        if let Some(ref mut stdin) = self.stdin {
            stdin.write_all(msg.as_bytes()).await.map_err(|e| {
                self.is_running.store(false, Ordering::SeqCst);
                OpenCodeError::Tui(format!(
                    "Failed to write to LSP server: {} (server may have crashed)",
                    e
                ))
            })?;
            stdin
                .flush()
                .await
                .map_err(|e| OpenCodeError::Tui(e.to_string()))?;
        } else {
            return Err(OpenCodeError::Tui("LSP stdin not available".to_string()));
        }

        Ok(())
    }

    pub async fn wait_for_response(
        &mut self,
        id: u64,
        timeout_secs: u64,
    ) -> Result<serde_json::Value, OpenCodeError> {
        let timeout = Duration::from_secs(timeout_secs);
        let (tx, rx) = oneshot::channel::<Result<String, LspError>>();

        {
            let mut p = self.pending.lock().await;
            p.insert(id, tx);
        }

        let resp = match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(Ok(resp))) => resp,
            Ok(Ok(Err(lsp_err))) => return Err(lsp_err.into_opencode_error()),
            Ok(Err(e)) => {
                return Err(OpenCodeError::Tool(format!(
                    "LSP request {} failed: {}",
                    id, e
                )))
            }
            Err(_) => {
                return Err(LspError::RequestTimeout {
                    method: "unknown".to_string(),
                    timeout_ms: timeout_secs * 1000,
                }
                .into_opencode_error())
            }
        };

        serde_json::from_str(&resp)
            .map_err(|e| OpenCodeError::Tool(format!("Invalid LSP response: {}", e)))
    }

    async fn send_notification(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), OpenCodeError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_jsonrpc_message(&notification).await
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

    pub async fn get_diagnostics(&mut self, uri: &str) -> Result<Vec<Diagnostic>, OpenCodeError> {
        self.check_health()?;
        let diags = self.diagnostics.lock().await;
        Ok(diags.get(uri).cloned().unwrap_or_default())
    }

    pub async fn goto_definition(
        &mut self,
        uri: &str,
        line: u32,
        col: u32,
    ) -> Result<Option<Location>, OpenCodeError> {
        self.check_health()?;

        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col }
        });

        let id = self.send_request("textDocument/definition", params).await?;
        let resp = self
            .wait_for_response(id, self.config.default_request_timeout_ms / 1000)
            .await?;

        if let Some(result) = resp.get("result") {
            if result.is_null() {
                return Ok(None);
            }
            return Ok(parse_location(result));
        }

        Ok(None)
    }

    pub async fn find_references(
        &mut self,
        uri: &str,
        line: u32,
        col: u32,
    ) -> Result<Vec<Location>, OpenCodeError> {
        self.check_health()?;

        let params = serde_json::json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": col },
            "context": { "includeDeclaration": true }
        });

        let id = self.send_request("textDocument/references", params).await?;
        let resp = self
            .wait_for_response(id, self.config.default_request_timeout_ms / 1000)
            .await?;

        if let Some(result) = resp.get("result") {
            if let Some(arr) = result.as_array() {
                let locations: Vec<Location> = arr.iter().filter_map(parse_location).collect();
                return Ok(locations);
            }
        }

        Ok(Vec::new())
    }

    pub async fn completion(
        &mut self,
        _uri: &str,
        _line: u32,
        _col: u32,
    ) -> Result<Vec<CompletionItem>, OpenCodeError> {
        self.check_health()?;
        Ok(Vec::new())
    }

    pub async fn shutdown(&mut self) -> Result<(), OpenCodeError> {
        let _ = self
            .send_notification("shutdown", serde_json::json!({}))
            .await;
        if let Some(mut process) = self.process.take() {
            process.kill().await.ok();
        }
        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_healthy(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
            && self.consecutive_errors.load(Ordering::SeqCst) < self.config.max_consecutive_errors
    }

    fn check_health(&self) -> Result<(), OpenCodeError> {
        if !self.is_healthy() {
            return Err(LspError::ServerUnhealthy {
                server_name: self.server_name.clone(),
                reason: UnhealthyReason::NotResponding,
            }
            .into_opencode_error());
        }
        Ok(())
    }

    pub async fn health_check(&mut self) -> Result<bool, OpenCodeError> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Ok(false);
        }

        let params = serde_json::json!({});
        match self.send_request("window/showMessageRequest", params).await {
            Ok(id) => {
                match tokio::time::timeout(Duration::from_secs(5), self.wait_for_response(id, 5))
                    .await
                {
                    Ok(Ok(_)) => Ok(true),
                    _ => Ok(false),
                }
            }
            Err(_) => Ok(false),
        }
    }

    pub fn get_server_name(&self) -> &str {
        &self.server_name
    }

    pub fn get_consecutive_error_count(&self) -> u32 {
        self.consecutive_errors.load(Ordering::SeqCst)
    }

    pub fn get_config(&self) -> &FailureHandlingConfig {
        &self.config
    }
}

fn len_after_header(buf: &[u8]) -> Option<usize> {
    let header = b"Content-Length: ";
    let idx = buf.windows(header.len()).position(|w| w == header)?;
    let after_header = &buf[idx + header.len()..];
    let end_of_headers = after_header.windows(4).position(|w| w == b"\r\n\r\n")?;
    let len_str = std::str::from_utf8(&after_header[..end_of_headers])
        .ok()?
        .trim();
    let len: usize = len_str.parse().ok()?;
    let msg_start = idx + header.len() + end_of_headers + 4;
    Some(msg_start + len)
}

fn extract_jsonrpc_message(buf: &[u8]) -> Result<String, LspError> {
    let header = b"Content-Length: ";
    let idx =
        buf.windows(header.len())
            .position(|w| w == header)
            .ok_or(LspError::ProtocolViolation {
                violation: ProtocolViolationType::MissingContentLength,
                detail: "Content-Length header not found".to_string(),
            })?;

    let after_header = &buf[idx + header.len()..];
    let end_of_headers = after_header
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or(LspError::ProtocolViolation {
            violation: ProtocolViolationType::InvalidContentLength,
            detail: "Invalid header format".to_string(),
        })?;

    let len_str = std::str::from_utf8(&after_header[..end_of_headers])
        .map_err(|_| LspError::ProtocolViolation {
            violation: ProtocolViolationType::InvalidContentLength,
            detail: "Content-Length is not valid UTF-8".to_string(),
        })?
        .trim();

    let len: usize = len_str.parse().map_err(|_| LspError::ProtocolViolation {
        violation: ProtocolViolationType::InvalidContentLength,
        detail: format!("Content-Length '{}' is not a valid number", len_str),
    })?;

    let msg_start = idx + header.len() + end_of_headers + 4;
    if buf.len() < msg_start + len {
        return Err(LspError::ProtocolViolation {
            violation: ProtocolViolationType::InvalidContentLength,
            detail: format!(
                "Buffer underflow: expected {} bytes, got {}",
                len,
                buf.len().saturating_sub(msg_start)
            ),
        });
    }

    let msg = std::str::from_utf8(&buf[msg_start..msg_start + len]).map_err(|_| {
        LspError::ProtocolViolation {
            violation: ProtocolViolationType::InvalidJson,
            detail: "Message is not valid UTF-8".to_string(),
        }
    })?;

    Ok(msg.to_string())
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
            end: Position {
                line: end_line,
                character: end_character,
            },
        },
    })
}

fn parse_diagnostic(v: &serde_json::Value) -> Option<Diagnostic> {
    let obj = v.as_object()?;

    let range = obj.get("range")?.as_object()?;
    let start = range.get("start")?.as_object()?;
    let end = range.get("end")?.as_object()?;

    let start_line = start.get("line")?.as_u64()? as u32;
    let start_char = start.get("character")?.as_u64()? as u32;
    let end_line = end.get("line")?.as_u64()? as u32;
    let end_char = end.get("character")?.as_u64()? as u32;

    let severity = obj
        .get("severity")
        .and_then(|s| s.as_i64())
        .map(|s| Severity::from(s as i32))
        .unwrap_or(Severity::Warning);

    let message = obj.get("message")?.as_str()?.to_string();

    let source = obj.get("source").and_then(|s| s.as_str()).map(String::from);

    Some(Diagnostic {
        severity,
        message,
        range: Range {
            start: Position {
                line: start_line,
                character: start_char,
            },
            end: Position {
                line: end_line,
                character: end_char,
            },
        },
        source,
        file_path: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = LspClient::new();
        assert!(!client.is_healthy());
        assert_eq!(client.get_consecutive_error_count(), 0);
    }

    #[tokio::test]
    async fn test_client_with_config() {
        let config = FailureHandlingConfig::default();
        let client = LspClient::with_config(config);
        assert!(!client.is_healthy());
    }

    #[test]
    fn test_extract_jsonrpc_message_valid() {
        let buf = b"Content-Length: 38\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"ok\"}";
        let result = extract_jsonrpc_message(buf);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"jsonrpc":"2.0","id":1,"result":"ok"}"#);
    }

    #[test]
    fn test_extract_jsonrpc_message_missing_header() {
        let buf = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"ok\"}";
        let result = extract_jsonrpc_message(buf);
        assert!(matches!(
            result,
            Err(LspError::ProtocolViolation {
                violation: ProtocolViolationType::MissingContentLength,
                ..
            })
        ));
    }

    #[test]
    fn test_extract_jsonrpc_message_invalid_length() {
        let buf = b"Content-Length: abc\r\n\r\n{\"jsonrpc\":\"2.0\"}";
        let result = extract_jsonrpc_message(buf);
        assert!(matches!(
            result,
            Err(LspError::ProtocolViolation {
                violation: ProtocolViolationType::InvalidContentLength,
                ..
            })
        ));
    }

    #[test]
    fn test_extract_jsonrpc_message_buffer_underflow() {
        let buf = b"Content-Length: 100\r\n\r\n{\"jsonrpc\":\"2.0\"}";
        let result = extract_jsonrpc_message(buf);
        assert!(matches!(
            result,
            Err(LspError::ProtocolViolation {
                violation: ProtocolViolationType::InvalidContentLength,
                ..
            })
        ));
    }

    #[test]
    fn test_len_after_header() {
        let buf = b"Content-Length: 17\r\n\r\n{\"jsonrpc\":\"2.0\"}";
        let len = len_after_header(buf);
        assert_eq!(len, Some(22 + 17));
    }

    #[tokio::test]
    async fn test_health_check_not_started() {
        let mut client = LspClient::new();
        let result = client.health_check().await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_server_name_empty_when_not_started() {
        let client = LspClient::new();
        assert_eq!(client.get_server_name(), "");
    }

    #[test]
    fn test_failure_handling_config() {
        let config = FailureHandlingConfig::default();
        assert_eq!(config.default_request_timeout_ms, 30_000);
        assert_eq!(config.max_consecutive_errors, 5);
        assert!(config.auto_restart);
    }

    #[test]
    fn test_protocol_violation_types() {
        assert_eq!(
            format!("{:?}", ProtocolViolationType::InvalidJson),
            "InvalidJson"
        );
        assert_eq!(
            format!("{:?}", ProtocolViolationType::MissingContentLength),
            "MissingContentLength"
        );
    }

    #[test]
    fn test_crash_cause() {
        let cause = CrashCause::ProcessExited { code: 1 };
        assert!(cause.to_string().contains("exited"));
        assert!(cause.to_string().contains("1"));
    }
}
