use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use serde_json::Value;
use tokio::sync::RwLock;

use opencode_core::ToolRegistry;

use crate::client::{ConnectionState, McpClient, McpError, McpResource, McpTool, McpTransport};
use crate::tool_bridge::McpToolAdapter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpPermission {
    Allow,
    Ask,
    Deny,
}

impl Default for McpPermission {
    fn default() -> Self {
        Self::Ask
    }
}

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub transport: McpTransport,
    pub timeout: Duration,
    pub auto_connect: bool,
    pub health_check_interval: Option<Duration>,
    pub permission: McpPermission,
}

impl McpServerConfig {
    pub fn new(transport: McpTransport) -> Self {
        Self {
            transport,
            timeout: Duration::from_secs(5),
            auto_connect: true,
            health_check_interval: None,
            permission: McpPermission::default(),
        }
    }

    pub fn with_permission(mut self, permission: McpPermission) -> Self {
        self.permission = permission;
        self
    }
}

pub struct McpRegistry {
    servers: HashMap<String, McpServerConfig>,
    clients: HashMap<String, Arc<McpClient>>,
    discovered_tools: HashMap<String, Vec<McpTool>>,
    discovered_resources: HashMap<String, Vec<McpResource>>,
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            clients: HashMap::new(),
            discovered_tools: HashMap::new(),
            discovered_resources: HashMap::new(),
        }
    }

    pub fn add_server(&mut self, name: &str, config: McpServerConfig) {
        self.servers.insert(name.to_string(), config);
    }

    pub async fn connect_all(&mut self) -> Result<Vec<String>, McpError> {
        let mut connected = Vec::new();

        for (name, cfg) in self.servers.clone() {
            let client = Arc::new(
                McpClient::new(cfg.transport.clone())
                    .with_timeout(cfg.timeout)
                    .with_health_check_interval(cfg.health_check_interval),
            );

            if cfg.auto_connect {
                client.connect().await?;
                let tools = client.list_tools().await.unwrap_or_default();
                let resources = client.list_resources().await.unwrap_or_default();
                self.discovered_tools.insert(name.clone(), tools);
                self.discovered_resources.insert(name.clone(), resources);
                connected.push(name.clone());
            }

            self.clients.insert(name, client);
        }

        Ok(connected)
    }

    pub fn bridge_to_tool_registry(&self, tool_registry: &mut ToolRegistry) {
        for (server_name, tools) in &self.discovered_tools {
            let Some(client) = self.clients.get(server_name) else {
                continue;
            };

            let requires_approval = self
                .servers
                .get(server_name)
                .map(|cfg| cfg.permission == McpPermission::Ask)
                .unwrap_or(false);

            for tool in tools {
                let adapter = McpToolAdapter::new(client.clone(), tool.clone(), server_name)
                    .with_requires_approval(requires_approval);
                adapter.register_into(tool_registry);
            }
        }
    }

    pub async fn disconnect_all(&self) {
        for client in self.clients.values() {
            let _ = client.disconnect().await;
        }
    }

    pub fn clients(&self) -> &HashMap<String, Arc<McpClient>> {
        &self.clients
    }

    pub fn tools_for_server(&self, name: &str) -> Option<&Vec<McpTool>> {
        self.discovered_tools.get(name)
    }

    pub fn resources_for_server(&self, name: &str) -> Option<&Vec<McpResource>> {
        self.discovered_resources.get(name)
    }
}

pub struct McpManager {
    registry: RwLock<McpRegistry>,
}

impl McpManager {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<McpManager> = OnceLock::new();
        INSTANCE.get_or_init(|| McpManager {
            registry: RwLock::new(McpRegistry::new()),
        })
    }

    pub async fn add_server(&self, name: &str, config: McpServerConfig) {
        self.registry.write().await.add_server(name, config);
    }

    pub async fn connect_all(&self) -> Result<Vec<String>, McpError> {
        self.registry.write().await.connect_all().await
    }

    pub async fn get_tools(&self) -> Vec<McpTool> {
        let registry = self.registry.read().await;
        registry
            .discovered_tools
            .values()
            .flat_map(|tools| tools.iter().cloned())
            .collect()
    }

    pub async fn get_resources(&self) -> Vec<McpResource> {
        let registry = self.registry.read().await;
        registry
            .discovered_resources
            .values()
            .flat_map(|items| items.iter().cloned())
            .collect()
    }

    pub async fn call_tool(&self, tool_name: &str, args: Value) -> Result<String, McpError> {
        let registry = self.registry.read().await;
        for (server_name, tools) in &registry.discovered_tools {
            if tools.iter().any(|t| t.name == tool_name) {
                if let Some(client) = registry.clients.get(server_name) {
                    let result = client.call_tool(tool_name, &args).await?;
                    return Ok(result.content);
                }
            }
        }
        Err(McpError::Other(format!(
            "MCP tool not found: {}",
            tool_name
        )))
    }

    pub async fn bridge_to_tool_registry(&self, tool_registry: &mut ToolRegistry) {
        self.registry
            .read()
            .await
            .bridge_to_tool_registry(tool_registry);
    }

    pub async fn disconnect_all(&self) {
        self.registry.read().await.disconnect_all().await;
    }

    pub async fn connection_states(&self) -> HashMap<String, ConnectionState> {
        let registry = self.registry.read().await;
        let mut states = HashMap::new();
        for (name, client) in registry.clients() {
            states.insert(name.clone(), client.connection_state().await);
        }
        states
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::JsonRpcResponse;

    fn ok_response(result: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(result),
            error: None,
        }
    }

    #[tokio::test]
    async fn test_registry_add_server() {
        let mut registry = McpRegistry::new();
        registry.add_server(
            "local",
            McpServerConfig::new(McpTransport::Stdio(crate::client::StdioProcess::new(
                "cmd",
                vec![],
            ))),
        );
        assert_eq!(registry.servers.len(), 1);
    }

    #[tokio::test]
    async fn test_manager_global_singleton() {
        let a = McpManager::global() as *const _;
        let b = McpManager::global() as *const _;
        assert_eq!(a, b);
    }

    #[tokio::test]
    async fn test_bridge_to_tool_registry_registers_tools() {
        let mut registry = McpRegistry::new();
        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(
            |request: crate::protocol::JsonRpcRequest| match request.method.as_str() {
                "tools/list" => Ok(ok_response(serde_json::json!({
                    "tools": [{
                        "name": "search_docs",
                        "description": "Search docs",
                        "inputSchema": {"type": "object"}
                    }]
                }))),
                "tools/call" => Ok(ok_response(serde_json::json!({
                    "content": [{"type": "text", "text": "ok"}],
                    "isError": false
                }))),
                _ => Ok(ok_response(Value::Null)),
            },
        );

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://mock/sse".to_string()),
            handler,
        ));
        client.connect().await.unwrap();

        registry.clients.insert("mock".to_string(), client.clone());
        registry
            .discovered_tools
            .insert("mock".to_string(), client.list_tools().await.unwrap());

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        assert!(tool_registry.contains("mock_search_docs"));
    }

    #[tokio::test]
    async fn test_remote_mcp_tools_require_approval_by_default() {
        let mut registry = McpRegistry::new();

        registry.add_server(
            "remote-server",
            McpServerConfig::new(McpTransport::Sse("http://remote/sse".to_string()))
                .with_permission(McpPermission::Ask),
        );

        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/list" => Ok(ok_response(serde_json::json!({
                "tools": [{
                    "name": "remote_tool",
                    "description": "A tool from remote server",
                    "inputSchema": {"type": "object"}
                }]
            }))),
            "tools/call" => Ok(ok_response(serde_json::json!({
                "content": [{"type": "text", "text": "result"}],
                "isError": false
            }))),
            _ => Ok(ok_response(Value::Null)),
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://remote/sse".to_string()),
            handler,
        ));
        client.connect().await.unwrap();

        registry
            .clients
            .insert("remote-server".to_string(), client.clone());
        registry.discovered_tools.insert(
            "remote-server".to_string(),
            client.list_tools().await.unwrap(),
        );

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        let def = tool_registry.get("remote-server_remote_tool").unwrap();
        assert!(
            def.requires_approval,
            "Remote MCP tools should require approval by default"
        );
    }

    #[tokio::test]
    async fn test_local_mcp_tools_allow_by_default() {
        let mut registry = McpRegistry::new();

        registry.add_server(
            "local-server",
            McpServerConfig::new(McpTransport::Stdio(crate::client::StdioProcess::new(
                "cmd",
                vec![],
            )))
            .with_permission(McpPermission::Allow),
        );

        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/list" => Ok(ok_response(serde_json::json!({
                "tools": [{
                    "name": "local_tool",
                    "description": "A tool from local server",
                    "inputSchema": {"type": "object"}
                }]
            }))),
            "tools/call" => Ok(ok_response(serde_json::json!({
                "content": [{"type": "text", "text": "result"}],
                "isError": false
            }))),
            _ => Ok(ok_response(Value::Null)),
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(crate::client::StdioProcess::new("cmd", vec![])),
            handler,
        ));
        client.connect().await.unwrap();

        registry
            .clients
            .insert("local-server".to_string(), client.clone());
        registry.discovered_tools.insert(
            "local-server".to_string(),
            client.list_tools().await.unwrap(),
        );

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        let def = tool_registry.get("local-server_local_tool").unwrap();
        assert!(
            !def.requires_approval,
            "Local MCP tools with Allow permission should not require approval"
        );
    }

    #[tokio::test]
    async fn tool_discovery() {
        let mut registry = McpRegistry::new();

        registry.add_server(
            "docs-server",
            McpServerConfig::new(McpTransport::Stdio(crate::client::StdioProcess::new(
                "npx",
                vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
            )))
            .with_permission(McpPermission::Allow),
        );

        registry.add_server(
            "remote-server",
            McpServerConfig::new(McpTransport::Sse("http://remote:8080/sse".to_string()))
                .with_permission(McpPermission::Ask),
        );

        let docs_handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/list" => Ok(ok_response(serde_json::json!({
                "tools": [
                    {
                        "name": "search_docs",
                        "description": "Search documentation",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string"}
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "list_files",
                        "description": "List files in directory",
                        "inputSchema": {"type": "object"}
                    }
                ]
            }))),
            "tools/call" => Ok(ok_response(serde_json::json!({
                "content": [{"type": "text", "text": "discovery-test-ok"}],
                "isError": false
            }))),
            _ => Ok(ok_response(Value::Null)),
        });

        let remote_handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/list" => Ok(ok_response(serde_json::json!({
                "tools": [
                    {
                        "name": "remote_search",
                        "description": "Search remote resources",
                        "inputSchema": {"type": "object"}
                    }
                ]
            }))),
            "tools/call" => Ok(ok_response(serde_json::json!({
                "content": [{"type": "text", "text": "remote-discovery-ok"}],
                "isError": false
            }))),
            _ => Ok(ok_response(Value::Null)),
        });

        let docs_client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(crate::client::StdioProcess::new("mock-docs", vec![])),
            docs_handler,
        ));
        let remote_client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://mock-remote/sse".to_string()),
            remote_handler,
        ));

        docs_client.connect().await.unwrap();
        remote_client.connect().await.unwrap();

        registry.clients.insert("docs-server".to_string(), docs_client.clone());
        registry.clients.insert("remote-server".to_string(), remote_client.clone());

        registry.discovered_tools.insert(
            "docs-server".to_string(),
            docs_client.list_tools().await.unwrap(),
        );
        registry.discovered_tools.insert(
            "remote-server".to_string(),
            remote_client.list_tools().await.unwrap(),
        );

        let docs_tools = registry.tools_for_server("docs-server");
        assert!(docs_tools.is_some());
        let docs_tools = docs_tools.unwrap();
        assert_eq!(docs_tools.len(), 2);
        assert!(docs_tools.iter().any(|t| t.name == "search_docs"));
        assert!(docs_tools.iter().any(|t| t.name == "list_files"));

        let remote_tools = registry.tools_for_server("remote-server");
        assert!(remote_tools.is_some());
        assert_eq!(remote_tools.unwrap().len(), 1);

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        assert!(tool_registry.contains("docs-server_search_docs"));
        assert!(tool_registry.contains("docs-server_list_files"));
        assert!(tool_registry.contains("remote-server_remote_search"));

        let executor = tool_registry.get_executor("docs-server_search_docs");
        assert!(executor.is_some());

        let local_def = tool_registry.get("docs-server_search_docs");
        assert!(local_def.is_some());
        assert!(
            !local_def.unwrap().requires_approval,
            "Local MCP tools should not require approval"
        );

        let remote_def = tool_registry.get("remote-server_remote_search");
        assert!(remote_def.is_some());
        assert!(
            remote_def.unwrap().requires_approval,
            "Remote MCP tools should require approval"
        );

        let all_tools: Vec<&McpTool> = registry
            .discovered_tools
            .values()
            .flat_map(|tools| tools.iter())
            .collect();
        assert!(!all_tools.is_empty());
        assert_eq!(all_tools.len(), 3);

        let call_result = docs_client
            .call_tool("search_docs", &serde_json::json!({"query": "test"}))
            .await;
        assert!(call_result.is_ok());
        assert_eq!(call_result.unwrap().content, "discovery-test-ok");
    }
}
