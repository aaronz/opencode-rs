use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};

use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

#[derive(Debug, Clone, thiserror::Error)]
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

type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, McpError>>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error(String),
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
    connection_state: ConnectionState,
    tools: Vec<McpTool>,
    resources: Vec<McpResource>,
    runtime: Option<RuntimeConnection>,
    health_task: Option<JoinHandle<()>>,
}

#[derive(Debug)]
enum RuntimeConnection {
    Stdio {
        stdin: Arc<Mutex<ChildStdin>>,
        child: Arc<Mutex<Child>>,
        pending: PendingMap,
        reader_task: JoinHandle<()>,
    },
    Sse {
        client: reqwest::Client,
        post_url: String,
        pending: PendingMap,
        reader_task: JoinHandle<()>,
    },
}

type TransportHandler =
    Arc<dyn Fn(JsonRpcRequest) -> Result<JsonRpcResponse, McpError> + Send + Sync>;

#[derive(Clone)]
pub struct McpClient {
    transport: McpTransport,
    timeout: Duration,
    max_retries: usize,
    reconnect_base_delay: Duration,
    health_check_interval: Option<Duration>,
    state: Arc<Mutex<McpState>>,
    id_counter: Arc<AtomicU64>,
    handler: Option<TransportHandler>,
}

impl McpClient {
    pub fn new(transport: McpTransport) -> Self {
        Self {
            transport,
            timeout: Duration::from_secs(5),
            max_retries: 3,
            reconnect_base_delay: Duration::from_millis(250),
            health_check_interval: None,
            state: Arc::new(Mutex::new(McpState::default())),
            id_counter: Arc::new(AtomicU64::new(1)),
            handler: None,
        }
    }

    pub fn with_handler(transport: McpTransport, handler: TransportHandler) -> Self {
        Self {
            transport,
            timeout: Duration::from_secs(5),
            max_retries: 3,
            reconnect_base_delay: Duration::from_millis(250),
            health_check_interval: None,
            state: Arc::new(Mutex::new(McpState::default())),
            id_counter: Arc::new(AtomicU64::new(1)),
            handler: Some(handler),
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

    pub fn with_health_check_interval(mut self, interval: Option<Duration>) -> Self {
        self.health_check_interval = interval;
        self
    }

    pub fn with_reconnect_base_delay(mut self, delay: Duration) -> Self {
        self.reconnect_base_delay = delay;
        self
    }

    pub async fn connect(&self) -> Result<(), McpError> {
        {
            let state = self.state.lock().await;
            if state.connection_state == ConnectionState::Connected {
                return Ok(());
            }
        }

        {
            let mut state = self.state.lock().await;
            state.connection_state = ConnectionState::Connecting;
        }

        if self.handler.is_some() {
            self.validate_transport()?;
            let mut state = self.state.lock().await;
            state.connection_state = ConnectionState::Connected;
            self.start_health_check_locked(&mut state);
            return Ok(());
        }

        let runtime = match &self.transport {
            McpTransport::Stdio(process) => Self::connect_stdio(process.clone()).await,
            McpTransport::Sse(url) => Self::connect_sse(url.clone()).await,
        };

        let runtime = match runtime {
            Ok(runtime) => runtime,
            Err(err) => {
                let mut state = self.state.lock().await;
                state.connection_state = ConnectionState::Error(err.to_string());
                return Err(err);
            }
        };

        {
            let mut state = self.state.lock().await;
            state.runtime = Some(runtime);
            state.connection_state = ConnectionState::Connected;
            self.start_health_check_locked(&mut state);
        }

        if let Err(err) = self.initialize().await {
            self.disconnect().await?;
            let mut state = self.state.lock().await;
            state.connection_state = ConnectionState::Error(err.to_string());
            return Err(err);
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), McpError> {
        let mut state = self.state.lock().await;
        if let Some(task) = state.health_task.take() {
            task.abort();
        }

        if let Some(runtime) = state.runtime.take() {
            match runtime {
                RuntimeConnection::Stdio {
                    child,
                    pending,
                    reader_task,
                    ..
                } => {
                    reader_task.abort();
                    let mut child = child.lock().await;
                    let _ = child.kill().await;
                    fail_pending(
                        &pending,
                        McpError::ConnectionLost("stdio disconnected".to_string()),
                    )
                    .await;
                }
                RuntimeConnection::Sse {
                    pending,
                    reader_task,
                    ..
                } => {
                    reader_task.abort();
                    fail_pending(
                        &pending,
                        McpError::ConnectionLost("sse disconnected".to_string()),
                    )
                    .await;
                }
            }
        }

        state.connection_state = ConnectionState::Disconnected;
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.connection_state == ConnectionState::Connected
    }

    pub async fn connection_state(&self) -> ConnectionState {
        self.state.lock().await.connection_state.clone()
    }

    pub async fn ping(&self) -> Result<(), McpError> {
        let req = JsonRpcRequest::new("ping", None);
        self.send_request(req).await.map(|_| ())
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let response = self
            .send_request(JsonRpcRequest::new("tools/list", None))
            .await?;
        let result = response.result.ok_or_else(|| {
            McpError::Protocol("missing result in tools/list response".to_string())
        })?;
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
                    name: entry
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    description: entry
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
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

        let raw = response.result.ok_or_else(|| {
            McpError::Protocol("missing result in tools/call response".to_string())
        })?;
        let is_error = raw
            .get("isError")
            .and_then(|v| v.as_bool())
            .or_else(|| raw.get("is_error").and_then(|v| v.as_bool()))
            .unwrap_or(false);

        let content = extract_result_text(&raw);
        Ok(McpToolResult {
            content,
            raw,
            is_error,
        })
    }

    pub async fn list_resources(&self) -> Result<Vec<McpResource>, McpError> {
        let response = self
            .send_request(JsonRpcRequest::new("resources/list", None))
            .await?;
        let result = response.result.ok_or_else(|| {
            McpError::Protocol("missing result in resources/list response".to_string())
        })?;
        let resources = result
            .get("resources")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| McpResource {
                uri: entry
                    .get("uri")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                name: entry
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
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
        let result = response.result.ok_or_else(|| {
            McpError::Protocol("missing result in resources/read response".to_string())
        })?;

        result
            .get("contents")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("text"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| {
                McpError::Protocol("resources/read response had no text content".to_string())
            })
    }

    pub async fn cached_tools(&self) -> Vec<McpTool> {
        self.state.lock().await.tools.clone()
    }

    async fn initialize(&self) -> Result<(), McpError> {
        let init_id = Value::from(self.next_id() as i64);
        let init = JsonRpcRequest::new(
            "initialize",
            Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "clientInfo": {
                    "name": "opencode-rs",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": { "listChanged": true },
                    "resources": { "listChanged": true }
                }
            })),
        )
        .with_id(init_id);

        let _ = self.send_request_runtime(init).await?;
        self.send_notification("initialized", None).await?;
        Ok(())
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<(), McpError> {
        if self.handler.is_some() {
            return Ok(());
        }

        let runtime = {
            let mut state = self.state.lock().await;
            state.runtime.take()
        };

        let mut runtime = runtime.ok_or(McpError::NotConnected)?;

        let notification = JsonRpcMessage::Notification(JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        });

        let result = match &mut runtime {
            RuntimeConnection::Stdio { stdin, .. } => write_json_line(stdin, &notification).await,
            RuntimeConnection::Sse {
                client, post_url, ..
            } => {
                let response = client
                    .post(post_url.as_str())
                    .json(&notification)
                    .send()
                    .await
                    .map_err(|e| McpError::ConnectionLost(format!("sse post failed: {}", e)))?;
                if !response.status().is_success() {
                    Err(McpError::ConnectionLost(format!(
                        "sse post status {}",
                        response.status()
                    )))
                } else {
                    Ok(())
                }
            }
        };

        let mut state = self.state.lock().await;
        state.runtime = Some(runtime);
        result
    }

    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse, McpError> {
        let mut attempts = 0usize;
        let mut request = request;

        if request.id.is_none() {
            request.id = Some(Value::from(self.next_id() as i64));
        }

        loop {
            if !self.is_connected().await {
                self.connect().await?;
            }

            let outcome = if let Some(handler) = &self.handler {
                let request_clone = request.clone();
                match timeout(self.timeout, async { (handler)(request_clone) }).await {
                    Ok(inner) => inner,
                    Err(_) => Err(McpError::Timeout(self.timeout)),
                }
            } else {
                self.send_request_runtime(request.clone()).await
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
                    self.reconnect_with_backoff(attempts).await?;
                }
                Err(err) => return Err(err),
            }
        }
    }

    async fn send_request_runtime(
        &self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, McpError> {
        let runtime = {
            let mut state = self.state.lock().await;
            state.runtime.take()
        };

        let mut runtime = runtime.ok_or(McpError::NotConnected)?;

        let result = match &mut runtime {
            RuntimeConnection::Stdio { stdin, pending, .. } => {
                send_with_pending_stdio(stdin, pending, request, self.timeout).await
            }
            RuntimeConnection::Sse {
                client,
                post_url,
                pending,
                ..
            } => send_with_pending_sse(client, post_url, pending, request, self.timeout).await,
        };

        let mut state = self.state.lock().await;
        state.runtime = Some(runtime);
        result
    }

    async fn reconnect_with_backoff(&self, attempt: usize) -> Result<(), McpError> {
        self.disconnect().await?;
        let multiplier = 1u64 << (attempt.saturating_sub(1).min(8) as u32);
        let delay = self.reconnect_base_delay.saturating_mul(multiplier as u32);
        sleep(delay).await;
        self.connect().await
    }

    fn next_id(&self) -> u64 {
        self.id_counter.fetch_add(1, Ordering::SeqCst)
    }

    fn validate_transport(&self) -> Result<(), McpError> {
        match &self.transport {
            McpTransport::Stdio(process) if process.command.trim().is_empty() => Err(
                McpError::ConnectionFailed("empty stdio command".to_string()),
            ),
            McpTransport::Sse(url) if url.trim().is_empty() => {
                Err(McpError::ConnectionFailed("empty sse url".to_string()))
            }
            _ => Ok(()),
        }
    }

    fn start_health_check_locked(&self, state: &mut McpState) {
        if state.health_task.is_some() {
            return;
        }

        if let Some(interval) = self.health_check_interval {
            let client = self.clone();
            state.health_task = Some(tokio::spawn(async move {
                loop {
                    sleep(interval).await;
                    if !client.is_connected().await {
                        break;
                    }
                    if client.ping().await.is_err() {
                        let _ = client.reconnect_with_backoff(1).await;
                    }
                }
            }));
        }
    }

    async fn connect_stdio(process: StdioProcess) -> Result<RuntimeConnection, McpError> {
        if process.command.trim().is_empty() {
            return Err(McpError::ConnectionFailed(
                "empty stdio command".to_string(),
            ));
        }

        let mut command = Command::new(&process.command);
        command
            .args(&process.args)
            .envs(&process.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());

        if let Some(cwd) = process.cwd {
            command.current_dir(cwd);
        }

        let mut child = command.spawn().map_err(|e| {
            McpError::ConnectionFailed(format!("failed to spawn stdio process: {}", e))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::ConnectionFailed("failed to capture stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::ConnectionFailed("failed to capture stdout".to_string()))?;

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let pending_reader = pending.clone();

        let reader_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if let Ok(JsonRpcMessage::Response(response)) =
                            serde_json::from_str::<JsonRpcMessage>(&line)
                        {
                            dispatch_response(&pending_reader, response).await;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            fail_pending(
                &pending_reader,
                McpError::ConnectionLost("stdio stream ended".to_string()),
            )
            .await;
        });

        Ok(RuntimeConnection::Stdio {
            stdin: Arc::new(Mutex::new(stdin)),
            child: Arc::new(Mutex::new(child)),
            pending,
            reader_task,
        })
    }

    async fn connect_sse(url: String) -> Result<RuntimeConnection, McpError> {
        if url.trim().is_empty() {
            return Err(McpError::ConnectionFailed("empty sse url".to_string()));
        }

        let client = reqwest::Client::new();
        let stream_response = client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| McpError::ConnectionFailed(format!("sse stream connect failed: {}", e)))?;

        if !stream_response.status().is_success() {
            return Err(McpError::ConnectionFailed(format!(
                "sse stream returned status {}",
                stream_response.status()
            )));
        }

        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));
        let pending_reader = pending.clone();

        let reader_task = tokio::spawn(async move {
            let mut stream = stream_response.bytes_stream();
            let mut buffer = String::new();
            let mut event_data: Vec<String> = Vec::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(bytes) => bytes,
                    Err(_) => break,
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_idx) = buffer.find('\n') {
                    let mut line = buffer.drain(..=newline_idx).collect::<String>();
                    if line.ends_with('\n') {
                        line.pop();
                    }
                    if line.ends_with('\r') {
                        line.pop();
                    }

                    if line.is_empty() {
                        if !event_data.is_empty() {
                            let payload = event_data.join("\n");
                            if let Ok(JsonRpcMessage::Response(response)) =
                                serde_json::from_str::<JsonRpcMessage>(&payload)
                            {
                                dispatch_response(&pending_reader, response).await;
                            }
                            event_data.clear();
                        }
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data:") {
                        event_data.push(data.trim_start().to_string());
                    }
                }
            }

            fail_pending(
                &pending_reader,
                McpError::ConnectionLost("sse stream ended".to_string()),
            )
            .await;
        });

        let post_url = infer_sse_post_url(&url);

        Ok(RuntimeConnection::Sse {
            client,
            post_url,
            pending,
            reader_task,
        })
    }
}

async fn write_json_line(
    stdin: &Arc<Mutex<ChildStdin>>,
    message: &JsonRpcMessage,
) -> Result<(), McpError> {
    let mut payload = serde_json::to_vec(message)
        .map_err(|e| McpError::Protocol(format!("failed to serialize json-rpc message: {}", e)))?;
    payload.push(b'\n');

    let mut stdin = stdin.lock().await;
    stdin.write_all(&payload).await.map_err(|e| {
        McpError::ConnectionLost(format!("failed to write request to stdio: {}", e))
    })?;
    stdin
        .flush()
        .await
        .map_err(|e| McpError::ConnectionLost(format!("failed to flush stdio request: {}", e)))?;
    Ok(())
}

async fn send_with_pending_stdio(
    stdin: &Arc<Mutex<ChildStdin>>,
    pending: &PendingMap,
    request: JsonRpcRequest,
    timeout_duration: Duration,
) -> Result<JsonRpcResponse, McpError> {
    let id = request
        .id
        .clone()
        .ok_or_else(|| McpError::Protocol("request id missing".to_string()))?;
    let key = id_to_key(&id);

    let (tx, rx) = oneshot::channel::<Result<JsonRpcResponse, McpError>>();
    pending.lock().await.insert(key.clone(), tx);

    let msg = JsonRpcMessage::Request(request);
    if let Err(err) = write_json_line(stdin, &msg).await {
        pending.lock().await.remove(&key);
        return Err(err);
    }

    await_pending_response(pending, key, rx, timeout_duration).await
}

async fn send_with_pending_sse(
    client: &reqwest::Client,
    post_url: &str,
    pending: &PendingMap,
    request: JsonRpcRequest,
    timeout_duration: Duration,
) -> Result<JsonRpcResponse, McpError> {
    let id = request
        .id
        .clone()
        .ok_or_else(|| McpError::Protocol("request id missing".to_string()))?;
    let key = id_to_key(&id);

    let (tx, rx) = oneshot::channel::<Result<JsonRpcResponse, McpError>>();
    pending.lock().await.insert(key.clone(), tx);

    let message = JsonRpcMessage::Request(request);
    let response = match client.post(post_url).json(&message).send().await {
        Ok(resp) => resp,
        Err(err) => {
            pending.lock().await.remove(&key);
            return Err(McpError::ConnectionLost(format!(
                "sse post failed: {}",
                err
            )));
        }
    };

    if !response.status().is_success() {
        pending.lock().await.remove(&key);
        return Err(McpError::ConnectionLost(format!(
            "sse post status {}",
            response.status()
        )));
    }

    if let Ok(text) = response.text().await {
        if !text.trim().is_empty() {
            if let Ok(JsonRpcMessage::Response(resp)) =
                serde_json::from_str::<JsonRpcMessage>(&text)
            {
                dispatch_response(pending, resp).await;
            }
        }
    }

    await_pending_response(pending, key, rx, timeout_duration).await
}

async fn await_pending_response(
    pending: &PendingMap,
    key: String,
    rx: oneshot::Receiver<Result<JsonRpcResponse, McpError>>,
    timeout_duration: Duration,
) -> Result<JsonRpcResponse, McpError> {
    match timeout(timeout_duration, rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => Err(McpError::ConnectionLost(
            "response channel closed".to_string(),
        )),
        Err(_) => {
            pending.lock().await.remove(&key);
            Err(McpError::Timeout(timeout_duration))
        }
    }
}

async fn dispatch_response(pending: &PendingMap, response: JsonRpcResponse) {
    if let Some(id) = response.id.as_ref() {
        let key = id_to_key(id);
        if let Some(tx) = pending.lock().await.remove(&key) {
            let _ = tx.send(Ok(response));
        }
    }
}

async fn fail_pending(pending: &PendingMap, error: McpError) {
    let mut pending = pending.lock().await;
    let mut entries = HashMap::new();
    std::mem::swap(&mut *pending, &mut entries);
    drop(pending);

    for (_, tx) in entries {
        let _ = tx.send(Err(error.clone()));
    }
}

fn id_to_key(id: &Value) -> String {
    serde_json::to_string(id).unwrap_or_else(|_| "null".to_string())
}

fn infer_sse_post_url(stream_url: &str) -> String {
    if stream_url.ends_with("/sse") {
        let prefix = stream_url.trim_end_matches("/sse");
        format!("{}/messages", prefix)
    } else {
        stream_url.to_string()
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

        let client =
            McpClient::with_handler(McpTransport::Sse("http://mock/sse".to_string()), handler);
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

        let client =
            McpClient::with_handler(McpTransport::Sse("http://mock/sse".to_string()), handler)
                .with_max_retries(3);
        client.connect().await.unwrap();
        let _ = client.list_tools().await.unwrap();
        assert!(attempts.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn test_connection_state_machine_transitions() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "tools/list" => Ok(ok_response(serde_json::json!({ "tools": [] }))),
            "ping" => Ok(ok_response(Value::Null)),
            _ => Ok(ok_response(Value::Null)),
        });

        let client = McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        );

        assert_eq!(
            client.connection_state().await,
            ConnectionState::Disconnected
        );
        client.connect().await.unwrap();
        assert_eq!(client.connection_state().await, ConnectionState::Connected);

        let _ = client.list_tools().await.unwrap();
        client.disconnect().await.unwrap();
        assert_eq!(
            client.connection_state().await,
            ConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn test_connection_stability_100_connections() {
        // Test: 100 sequential connections with > 99% success rate (at most 1 failure allowed)
        const CONNECTION_COUNT: usize = 100;
        const MAX_ALLOWED_FAILURES: usize = 1; // 99% = at most 1 failure out of 100

        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }))),
            "tools/list" => Ok(ok_response(serde_json::json!({ "tools": [] }))),
            "ping" => Ok(ok_response(Value::Null)),
            _ => Ok(ok_response(Value::Null)),
        });

        let mut failures = 0;
        let mut successes = 0;

        for i in 0..CONNECTION_COUNT {
            let client = McpClient::with_handler(
                McpTransport::Sse(format!("http://mock-{}/sse", i)),
                handler.clone(),
            )
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(2);

            match client.connect().await {
                Ok(()) => {
                    // Verify we can perform operations after connect
                    match client.list_tools().await {
                        Ok(_) => successes += 1,
                        Err(_) => failures += 1,
                    }
                }
                Err(_) => failures += 1,
            }

            // Small delay between connections to avoid overwhelming
            if i < CONNECTION_COUNT - 1 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        let success_rate = (successes as f64) / (CONNECTION_COUNT as f64) * 100.0;
        assert!(
            failures <= MAX_ALLOWED_FAILURES,
            "Connection stability test failed: {} failures out of {} (success rate: {:.2}%, required: >99%)",
            failures,
            CONNECTION_COUNT,
            success_rate
        );
        assert_eq!(successes + failures, CONNECTION_COUNT);
    }

    #[tokio::test]
    async fn test_concurrent_connection_stability() {
        // Test: Multiple concurrent connections should all succeed
        const CONCURRENT_CONNECTIONS: usize = 20;
        const TIMEOUT_SECS: u64 = 30;

        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }))),
            "tools/list" => Ok(ok_response(serde_json::json!({ "tools": [] }))),
            "ping" => Ok(ok_response(Value::Null)),
            _ => Ok(ok_response(Value::Null)),
        });

        let clients: Vec<_> = (0..CONCURRENT_CONNECTIONS)
            .map(|i| {
                McpClient::with_handler(
                    McpTransport::Sse(format!("http://mock-concurrent-{}/sse", i)),
                    handler.clone(),
                )
                .with_timeout(Duration::from_secs(5))
            })
            .collect();

        let results = tokio::time::timeout(
            Duration::from_secs(TIMEOUT_SECS),
            futures_util::future::join_all(clients.iter().map(|client| async {
                client.connect().await?;
                client.list_tools().await
            })),
        )
        .await;

        match results {
            Ok(connection_results) => {
                let failures: Vec<_> = connection_results
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| r.is_err())
                    .collect();
                assert!(
                    failures.is_empty(),
                    "Concurrent connection test had {} failures: {:?}",
                    failures.len(),
                    failures
                );
            }
            Err(_) => {
                panic!(
                    "Concurrent connection test timed out after {} seconds",
                    TIMEOUT_SECS
                );
            }
        }
    }

    #[tokio::test]
    async fn test_rapid_connect_disconnect_cycle() {
        // Test: Rapid connect/disconnect cycles should not leak resources
        const CYCLES: usize = 50;

        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }))),
            "tools/list" => Ok(ok_response(serde_json::json!({ "tools": [] }))),
            "ping" => Ok(ok_response(Value::Null)),
            _ => Ok(ok_response(Value::Null)),
        });

        for i in 0..CYCLES {
            let client = McpClient::with_handler(
                McpTransport::Sse(format!("http://mock-cycle-{}/sse", i)),
                handler.clone(),
            )
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(1);

            // Connect
            assert!(
                client.connect().await.is_ok(),
                "Cycle {}: connect failed",
                i
            );

            // Verify connected state
            assert_eq!(
                client.connection_state().await,
                ConnectionState::Connected,
                "Cycle {}: not in Connected state after connect",
                i
            );

            // Disconnect
            assert!(
                client.disconnect().await.is_ok(),
                "Cycle {}: disconnect failed",
                i
            );

            // Verify disconnected state
            assert_eq!(
                client.connection_state().await,
                ConnectionState::Disconnected,
                "Cycle {}: not in Disconnected state after disconnect",
                i
            );
        }
    }
}
