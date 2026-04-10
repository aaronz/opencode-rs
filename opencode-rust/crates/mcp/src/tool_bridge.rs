use std::future::Future;
use std::sync::{Arc, Mutex};

use opencode_core::{TokenCounter, ToolDefinition, ToolExecutor, ToolParameter, ToolRegistry};

use crate::client::{McpClient, McpTool};

/// Delimiter used to separate server name from tool name in qualified names
const TOOL_NAME_DELIMITER: &str = "_";

#[derive(Clone)]
pub struct McpToolAdapter {
    client: Arc<McpClient>,
    tool: McpTool,
    server_name: String,
    token_counter: Option<Arc<Mutex<TokenCounter>>>,
    requires_approval: bool,
}

impl McpToolAdapter {
    pub fn with_token_counter(mut self, counter: Arc<Mutex<TokenCounter>>) -> Self {
        self.token_counter = Some(counter);
        self
    }

    pub fn with_requires_approval(mut self, requires_approval: bool) -> Self {
        self.requires_approval = requires_approval;
        self
    }
}

impl McpToolAdapter {
    pub fn new(client: Arc<McpClient>, tool: McpTool, server_name: &str) -> Self {
        Self {
            client,
            tool,
            server_name: server_name.to_string(),
            token_counter: None,
            requires_approval: false,
        }
    }

    pub fn qualified_name(&self) -> String {
        format!(
            "{}{}{}",
            self.server_name, TOOL_NAME_DELIMITER, self.tool.name
        )
    }

    pub fn name(&self) -> &str {
        &self.tool.name
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.qualified_name(),
            description: self.tool.description.clone(),
            parameters: vec![ToolParameter {
                name: "args".to_string(),
                description: "Arguments object for MCP tool invocation".to_string(),
                required: false,
                schema: self.tool.input_schema.clone(),
            }],
            requires_approval: self.requires_approval,
        }
    }

    pub fn executor(&self) -> ToolExecutor {
        let client = self.client.clone();
        let tool_name = self.tool.name.clone();
        let token_counter = self.token_counter.clone();

        Arc::new(move |args| {
            let input_tokens = args.to_string().chars().count().div_ceil(4);

            let result = run_async(async { client.call_tool(&tool_name, &args).await });

            if let Some(counter) = &token_counter {
                if let Ok(output) = &result {
                    let output_tokens = output.content.chars().count().div_ceil(4);
                    if let Ok(mut guard) = counter.lock() {
                        guard.record_tokens("gpt-4o", input_tokens, output_tokens);
                    }
                }
            }

            result
                .map(|result| result.content)
                .map_err(|e| format!("MCP tool call failed: {}", e))
        })
    }

    pub fn register_into(&self, registry: &mut ToolRegistry) {
        registry.register(self.definition(), self.executor());
    }
}

fn run_async<T>(future: impl Future<Output = T>) -> T {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(future))
    } else {
        tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime for MCP tool adapter")
            .block_on(future)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::{McpError, McpTransport, StdioProcess};
    use crate::protocol::JsonRpcResponse;

    fn ok_response(result: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(result),
            error: None,
        }
    }

    #[test]
    fn test_tool_adapter_definition_and_execution() {
        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(
            |request: crate::protocol::JsonRpcRequest| match request.method.as_str() {
                "tools/call" => Ok(ok_response(json!({
                    "content": [{"type": "text", "text": "adapter-ok"}],
                    "isError": false
                }))),
                _ => Err(McpError::Other("unexpected method".to_string())),
            },
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let adapter = McpToolAdapter::new(
            client,
            McpTool {
                name: "echo".to_string(),
                description: "Echo tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "mock_server",
        );

        let def = adapter.definition();
        assert_eq!(def.name, "mock_server_echo");
        assert_eq!(def.parameters.len(), 1);

        let mut registry = ToolRegistry::new();
        adapter.register_into(&mut registry);

        let exec = registry.get_executor("mock_server_echo").unwrap();
        let out = exec(json!({"message": "hi"})).unwrap();
        assert_eq!(out, "adapter-ok");
    }

    #[test]
    fn test_mcp_tool_qualification() {
        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "qualified-test-ok"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let adapter = McpToolAdapter::new(
            client,
            McpTool {
                name: "search".to_string(),
                description: "Search tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "docs_server",
        )
        .with_requires_approval(true);

        assert_eq!(adapter.qualified_name(), "docs_server_search");
        assert_eq!(adapter.name(), "search");
        assert_eq!(adapter.server_name(), "docs_server");

        let def = adapter.definition();
        assert_eq!(def.name, "docs_server_search");
        assert_eq!(def.description, "Search tool");
        assert!(def.requires_approval);
    }

    #[test]
    fn test_mcp_tool_qualification_no_conflicts() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let search_handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "search-result"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let docs_client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock-docs", vec![])),
            search_handler.clone(),
        ));
        rt.block_on(docs_client.connect()).unwrap();

        let web_client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock-web", vec![])),
            search_handler,
        ));
        rt.block_on(web_client.connect()).unwrap();

        let docs_adapter = McpToolAdapter::new(
            docs_client,
            McpTool {
                name: "search".to_string(),
                description: "Search docs".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "docs_server",
        );

        let web_adapter = McpToolAdapter::new(
            web_client,
            McpTool {
                name: "search".to_string(),
                description: "Search web".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "web_server",
        );

        assert_eq!(docs_adapter.qualified_name(), "docs_server_search");
        assert_eq!(web_adapter.qualified_name(), "web_server_search");
        assert_ne!(docs_adapter.qualified_name(), web_adapter.qualified_name());

        let mut registry = ToolRegistry::new();
        docs_adapter.register_into(&mut registry);
        web_adapter.register_into(&mut registry);

        assert!(registry.contains("docs_server_search"));
        assert!(registry.contains("web_server_search"));
        assert!(!registry.contains("search"));
    }

    #[test]
    fn test_mcp_tool_naming_convention() {
        let tool = McpTool {
            name: "myTool".to_string(),
            description: "Test tool".to_string(),
            input_schema: json!({"type": "object"}),
        };

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            Arc::new(|_| Err(McpError::Other("not used".to_string()))),
        ));

        let adapter = McpToolAdapter::new(client, tool, "my-server");

        let qualified = adapter.qualified_name();
        assert!(qualified.starts_with("my-server_"));
        assert!(qualified.contains("myTool"));
        assert!(!qualified.contains(" "));
        assert!(!qualified.contains("/"));
    }

    #[test]
    fn test_mcp_tool_qualification_collision_handling() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handler1: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "server1-result"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let handler2: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "server2-result"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let client1 = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock1", vec![])),
            handler1,
        ));
        let client2 = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock2", vec![])),
            handler2,
        ));
        rt.block_on(client1.connect()).unwrap();
        rt.block_on(client2.connect()).unwrap();

        let adapter1 = McpToolAdapter::new(
            client1,
            McpTool {
                name: "echo".to_string(),
                description: "Echo from server1".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "server1",
        );

        let adapter2 = McpToolAdapter::new(
            client2,
            McpTool {
                name: "echo".to_string(),
                description: "Echo from server2".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "server2",
        );

        assert_eq!(adapter1.qualified_name(), "server1_echo");
        assert_eq!(adapter2.qualified_name(), "server2_echo");
        assert_ne!(adapter1.qualified_name(), adapter2.qualified_name());

        let mut registry = ToolRegistry::new();
        adapter1.register_into(&mut registry);
        adapter2.register_into(&mut registry);

        let exec1 = registry.get_executor("server1_echo").unwrap();
        let exec2 = registry.get_executor("server2_echo").unwrap();
        assert!(exec1(json!({})).is_ok());
        assert!(exec2(json!({})).is_ok());
    }
}
