use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("not connected")]
    NotConnected,
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
    #[error("connection lost: {0}")]
    ConnectionLost(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("timeout after {0:?}")]
    Timeout(Duration),
    #[error("operation failed: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolResult {
    pub content: String,
    pub raw: Value,
    pub is_error: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdioProcess {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<PathBuf>,
}

impl StdioProcess {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            env: HashMap::new(),
            cwd: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpTransport {
    Stdio(StdioProcess),
    Sse(String),
}

#[derive(Debug, Default)]
struct McpState {
    connected: bool,
    tools: Vec<McpTool>,
    resources: Vec<McpResource>,
}

type TransportHandler = Arc<dyn Fn(JsonRpcRequest) -> Result<JsonRpcResponse, McpError> + Send + Sync>;

#[derive(Clone)]
pub struct McpClient {
    transport: McpTransport,
    timeout: Duration,
    max_retries: usize,
    state: Arc<Mutex<McpState>>,
    handler: TransportHandler,
}

impl McpClient {
    pub fn new(transport: McpTransport) -> Self {
        Self::with_handler(transport, Arc::new(|_req| Err(McpError::Other("transport handler not configured".to_string()))))
    }

    pub fn with_handler(transport: McpTransport, handler: TransportHandler) -> Self {
        Self {
            transport,
            timeout: Duration::from_secs(5),
            max_retries: 3,
            state: Arc::new(Mutex::new(McpState::default())),
            handler,
        }
    }

    pub fn with_timeout(mut self, timeout_value: Duration) -> Self {
        self.timeout = timeout_value;
        self
    }

    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    pub async fn connect(&self) -> Result<(), McpError> {
        let mut state = self.state.lock().await;
        if state.connected {
            return Ok(());
        }

        match &self.transport {
            McpTransport::Stdio(process) if process.command.trim().is_empty() => {
                return Err(McpError::ConnectionFailed("empty stdio command".to_string()));
            }
            McpTransport::Sse(url) if url.trim().is_empty() => {
                return Err(McpError::ConnectionFailed("empty sse url".to_string()));
            }
            _ => {
                state.connected = true;
            }
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), McpError> {
        let mut state = self.state.lock().await;
        state.connected = false;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.connected
    }

    pub async fn ping(&self) -> Result<(), McpError> {
        let req = JsonRpcRequest::new("ping", None);
        self.send_request(req).await.map(|_| ())
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let response = self.send_request(JsonRpcRequest::new("tools/list", None)).await?;
        let result = response.result.ok_or_else(|| McpError::Protocol("missing result in tools/list response".to_string()))?;
        let tools = result
            .get("tools")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| {
                let input_schema = entry
                    .get("inputSchema")
                    .cloned()
                    .or_else(|| entry.get("input_schema").cloned())
                    .unwrap_or(Value::Null);
                McpTool {
                    name: entry.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    description: entry.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    input_schema,
                }
            })
            .collect::<Vec<_>>();

        self.state.lock().await.tools = tools.clone();
        Ok(tools)
    }

    pub async fn call_tool(&self, name: &str, args: &Value) -> Result<McpToolResult, McpError> {
        let response = self
            .send_request(JsonRpcRequest::new(
                "tools/call",
                Some(serde_json::json!({
                    "name": name,
                    "arguments": args,
                })),
            ))
            .await?;

        let raw = response
            .result
            .ok_or_else(|| McpError::Protocol("missing result in tools/call response".to_string()))?;
        let is_error = raw
            .get("isError")
            .and_then(|v| v.as_bool())
            .or_else(|| raw.get("is_error").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        let content = extract_result_text(&raw);
        Ok(McpToolResult { content, raw, is_error })
    }

    pub async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> {
        let response = self.send_request(JsonRpcRequest::new("resources/list", None)).await?;
        let result = response.result.ok_or_else(|| McpError::Protocol("missing result in resources/list response".to_string()))?;
        let resources = result
            .get("resources")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| McpResource {
                uri: entry.get("uri").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                name: entry.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                description: entry
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                mime_type: entry
                    .get("mimeType")
                    .and_then(|v| v.as_str())
                    .or_else(|| entry.get("mime_type").and_then(|v| v.as_str()))
                    .map(str::to_string),
            })
            .collect::<Vec<_>>();

        self.state.lock().await.resources = resources.clone();
        Ok(resources)
    }

    pub async fn read_resource(&self, uri: &str) -> Result<String, McpError> {
        let response = self
            .send_request(JsonRpcRequest::new(
                "resources/read",
                Some(serde_json::json!({ "uri": uri })),
            ))
            .await?;
        let result = response
            .result
            .ok_or_else(|| McpError::Protocol("missing result in resources/read response".to_string()))?;

        result
            .get("contents")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("text"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| McpError::Protocol("resources/read response had no text content".to_string()))
    }

    pub async fn cached_tools(&self) -> Vec<McpTool> {
        self.state.lock().await.tools.clone()
    }

    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        let mut attempts = 0usize;

        loop {
            if !self.is_connected().await {
                self.connect().await?;
            }

            let request_clone = request.clone();
            let response = timeout(self.timeout, async { (self.handler)(request_clone) }).await;

            let outcome = match response {
                Ok(inner) => inner,
                Err(_) => return Err(McpError::Timeout(self.timeout)),
            };

            match outcome {
                Ok(resp) => {
                    if let Some(err) = resp.error {
                        return Err(McpError::Protocol(err.message));
                    }
                    return Ok(resp);
                }
                Err(McpError::ConnectionLost(_)) if attempts < self.max_retries => {
                    attempts += 1;
                    self.disconnect().await?;
                    self.connect().await?;
                }
                Err(err) => return Err(err),
            }
        }
    }
}

fn extract_result_text(raw: &Value) -> String {
    raw.get("content")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("text")
                        .and_then(|t| t.as_str())
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| raw.to_string())
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new(McpTransport::Sse("http://localhost:3000/sse".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn ok_response(result: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(result),
            error: None,
        }
    }

    #[tokio::test]
    async fn test_list_tools_and_call_tool() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "tools/list" => Ok(ok_response(serde_json::json!({
                "tools": [{
                    "name": "search",
                    "description": "search docs",
                    "inputSchema": {"type": "object"}
                }]
            }))),
            "tools/call" => Ok(ok_response(serde_json::json!({
                "content": [{"type": "text", "text": "done"}],
                "isError": false
            }))),
            "ping" => Ok(ok_response(Value::Null)),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let client = McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        );
        client.connect().await.unwrap();

        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "search");

        let result = client
            .call_tool("search", &serde_json::json!({"q": "rust"}))
            .await
            .unwrap();
        assert_eq!(result.content, "done");
        assert!(!result.is_error);

        client.ping().await.unwrap();
    }

    #[tokio::test]
    async fn test_resources_list_and_read() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "resources/list" => Ok(ok_response(serde_json::json!({
                "resources": [{
                    "uri": "file:///README.md",
                    "name": "README",
                    "description": "project readme",
                    "mimeType": "text/markdown"
                }]
            }))),
            "resources/read" => Ok(ok_response(serde_json::json!({
                "contents": [{"uri": "file:///README.md", "text": "hello"}]
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let client = McpClient::with_handler(McpTransport::Sse("http://mock/sse".to_string()), handler);
        client.connect().await.unwrap();

        let resources = client.list_resources().await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].name, "README");

        let body = client.read_resource("file:///README.md").await.unwrap();
        assert_eq!(body, "hello");
    }

    #[tokio::test]
    async fn test_auto_reconnect_on_connection_loss() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_cloned = attempts.clone();
        let handler: TransportHandler = Arc::new(move |request| {
            let call_no = attempts_cloned.fetch_add(1, Ordering::SeqCst);
            if request.method == "tools/list" && call_no == 0 {
                return Err(McpError::ConnectionLost("dropped".to_string()));
            }
            Ok(ok_response(serde_json::json!({ "tools": [] })))
        });

        let client = McpClient::with_handler(McpTransport::Sse("http://mock/sse".to_string()), handler)
            .with_max_retries(3);
        client.connect().await.unwrap();
        let _ = client.list_tools().await.unwrap();
        assert!(attempts.load(Ordering::SeqCst) >= 2);
    }
}
