# mcp.md — MCP (Model Context Protocol) Module

## Module Overview

- **Crate**: `opencode-mcp`
- **Source**: `crates/mcp/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Implementation of the Model Context Protocol for tool providers, MCP client/server communication, connection pooling, context cost tracking, and tool bridging to the OpenCode tool registry.

---

## Crate Layout

```
crates/mcp/src/
├── lib.rs              ← Public re-exports
├── auth.rs             ← MCP authentication
├── client.rs           ← McpClient, McpTransport, StdioProcess, ConnectionState
├── context_cost.rs     ← ContextCostTracker, CostLimits, CostLevel
├── integration.rs      ← register_mcp_tools (bridge to ToolRegistry)
├── pool.rs             ← McpConnectionPool, PoolConfig, PoolStats, PooledClient
├── protocol.rs         ← JSON-RPC types, ServerCapabilities, ToolDefinition
├── registry.rs         ← McpRegistry, McpManager, McpPermission
├── server.rs           ← McpServer
└── tool_bridge.rs      ← McpToolAdapter (implements opencode_tools::Tool)
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.45", features = ["full"] }
thiserror = "2.0"
reqwest = { version = "0.12" }

opencode-core = { path = "../core" }
opencode-tools = { path = "../tools" }
opencode-permission = { path = "../permission" }
```

**Public exports from lib.rs**:
```rust
pub use context_cost::{
    ContextCostStats, ContextCostTracker, CostLevel, CostLimits, CostRecord,
    SharedContextCostTracker,
};
pub use client::{
    ConnectionState, JsonRpcMessage, McpClient, McpError, McpResource, McpTool, McpToolResult,
    McpTransport, StdioProcess,
};
pub use integration::register_mcp_tools;
pub use pool::{EndpointPoolStats, McpConnectionPool, PoolConfig, PoolStats, PooledClient};
pub use protocol::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, ResourceContent,
    ResourceDefinition, ResourcesCapability, ServerCapabilities, ServerInfo, ToolContent,
    ToolDefinition, ToolResult, ToolsCapability,
};
pub use registry::{McpManager, McpPermission, McpRegistry};
pub use server::McpServer;
pub use tool_bridge::McpToolAdapter;
```

---

## Core Types

### McpClient

```rust
pub struct McpClient { ... }

impl McpClient {
    pub async fn new(command: &[String], env: Option<&HashMap<String, String>>) -> Result<Self, McpError>;
    pub async fn initialize() -> Result<ServerCapabilities, McpError>;
    pub async fn list_tools() -> Result<Vec<ToolDefinition>, McpError>;
    pub async fn call_tool(name: &str, args: serde_json::Value) -> Result<McpToolResult, McpError>;
    pub async fn list_resources() -> Result<Vec<ResourceDefinition>, McpError>;
    pub async fn read_resource(uri: &str) -> Result<ResourceContent, McpError>;
    pub async fn shutdown() -> Result<(), McpError>;
}

pub enum McpTransport {
    Stdio(StdioProcess),
    Http(String),  // URL for remote MCP servers
}

pub struct StdioProcess { ... }
impl StdioProcess {
    pub async fn new(command: &[String], env: Option<&HashMap<String, String>>) -> Result<Self, McpError>;
    pub async fn send(&self, msg: JsonRpcMessage) -> Result<(), McpError>;
    pub async fn receive(&self) -> Result<JsonRpcMessage, McpError>;
}
```

### Protocol Types (JSON-RPC)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,  // "2.0"
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
}

pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

pub struct ResourcesCapability {
    pub list_changed: Option<bool>,
}

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

pub struct ToolContent {
    pub r#type: String,  // "text" or "image"
    pub text: Option<String>,
    pub data: Option<String>,
    pub mime_type: Option<String>,
}

pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub contents: Vec<ToolContent>,
}

pub struct ServerInfo {
    pub name: String,
    pub version: String,
}
```

### Connection Pool

```rust
pub struct PoolConfig {
    pub max_connections: usize,
    pub max_idle_per_endpoint: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

pub struct PoolStats {
    pub total_connections: usize,
    pub active_connections: usize,
    pub idle_connections: usize,
    pub waiting_requests: usize,
}

pub struct McpConnectionPool {
    config: PoolConfig,
    endpoints: HashMap<String, EndpointPool>,
}

impl McpConnectionPool {
    pub async fn get_client(&self, endpoint: &str) -> Result<PooledClient, McpError>;
    pub fn stats(&self) -> PoolStats;
}

pub struct PooledClient {
    client: McpClient,
    endpoint: String,
}

impl PooledClient {
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>, McpError>;
    pub async fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<McpToolResult, McpError>;
}
```

### Context Cost Tracking

```rust
pub struct ContextCostTracker { ... }

pub struct CostLimits {
    pub max_tokens: Option<usize>,
    pub max_cost: Option<f64>,
    pub warning_threshold: f64,  // 0.0-1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostLevel {
    Normal,
    Warning,
    Critical,
    Exceeded,
}

pub struct CostRecord {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost: f64,
    pub level: CostLevel,
}

pub struct ContextCostStats {
    pub total_tokens: u64,
    pub total_cost: f64,
    pub current_level: CostLevel,
    pub warning_triggered: bool,
}
```

### Registry and Manager

```rust
pub struct McpRegistry { ... }
impl McpRegistry {
    pub fn new() -> Self;
    pub fn register(&mut self, name: String, config: McpConfig) -> Result<(), McpError>;
    pub fn unregister(&mut self, name: &str);
    pub fn get(&self, name: &str) -> Option<&McpConfig>;
    pub fn list(&self) -> Vec<String>;
}

pub struct McpManager { ... }
impl McpManager {
    pub fn new() -> Self;
    pub async fn start_servers(&mut self) -> Result<(), McpError>;
    pub async fn stop_servers(&mut self) -> Result<(), McpError>;
    pub async fn get_tools(&self) -> Vec<McpTool>;
}

pub enum McpPermission {
    Allow,
    Deny,
    RequireApproval,
}
```

### Tool Bridge

```rust
// McpToolAdapter implements opencode_tools::Tool
pub struct McpToolAdapter { ... }

impl McpToolAdapter {
    pub fn new(tool_def: ToolDefinition, pool: Arc<McpConnectionPool>) -> Self;
}

impl opencode_tools::Tool for McpToolAdapter {
    fn name(&self) -> &str { &self.tool_def.name }
    fn description(&self) -> &str { &self.tool_def.description }
    fn clone_tool(&self) -> Box<dyn opencode_tools::Tool>;
    async fn execute(&self, args: serde_json::Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError>;
}

pub fn register_mcp_tools(
    mcp_manager: &McpManager,
    tool_registry: &ToolRegistry,
) -> Result<(), McpError>;
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-mcp` |
|---|---|
| `opencode-server` | `McpManager` to start/stop MCP servers |
| `opencode-tools` | `McpToolAdapter` via tool bridge |
| `opencode-config` | `McpConfig` deserialization |

**Dependencies of `opencode-mcp`**:
| Crate | Usage |
|---|---|
| `opencode-core` | `OpenCodeError` |
| `opencode-tools` | `Tool`, `ToolContext`, `ToolResult`, `ToolRegistry` |
| `opencode-permission` | `AgentPermissionScope` for tool permissions |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_mcp_client_initialize() { ... }

    #[tokio::test]
    async fn test_mcp_client_list_tools() { ... }

    #[tokio::test]
    async fn test_mcp_client_call_tool() { ... }

    #[tokio::test]
    async fn test_connection_pool_limits() { ... }

    #[test]
    fn test_cost_level_calculation() {
        let limits = CostLimits {
            max_tokens: Some(100000),
            max_cost: Some(10.0),
            warning_threshold: 0.8,
        };
        // Normal → Warning at 80% → Critical at 100%
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::json!(1),
            method: "tools/list".into(),
            params: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"method\":\"tools/list\""));
    }

    #[test]
    fn test_tool_definition_deserialization() {
        let json = r#"{"name": "read", "description": "Read files", "input_schema": {}}"#;
        let def: ToolDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.name, "read");
    }

    #[tokio::test]
    async fn test_context_cost_tracker_warning() { ... }

    #[tokio::test]
    async fn test_mcp_tool_adapter_execute() { ... }
}
```
