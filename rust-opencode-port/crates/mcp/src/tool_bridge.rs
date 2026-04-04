use std::future::Future;
use std::sync::Arc;

use opencode_core::{ToolDefinition, ToolExecutor, ToolParameter, ToolRegistry};

use crate::client::{McpClient, McpTool};

#[derive(Clone)]
pub struct McpToolAdapter {
    client: Arc<McpClient>,
    tool: McpTool,
}

impl McpToolAdapter {
    pub fn new(client: Arc<McpClient>, tool: McpTool) -> Self {
        Self { client, tool }
    }

    pub fn name(&self) -> &str {
        &self.tool.name
    }

    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.tool.name.clone(),
            description: self.tool.description.clone(),
            parameters: vec![ToolParameter {
                name: "args".to_string(),
                description: "Arguments object for MCP tool invocation".to_string(),
                required: false,
                schema: self.tool.input_schema.clone(),
            }],
        }
    }

    pub fn executor(&self) -> ToolExecutor {
        let client = self.client.clone();
        let tool_name = self.tool.name.clone();

        Arc::new(move |args| {
            run_async(async { client.call_tool(&tool_name, &args).await })
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
        let handler: Arc<dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError> + Send + Sync> = Arc::new(|request: crate::protocol::JsonRpcRequest| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "adapter-ok"}],
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
                name: "echo".to_string(),
                description: "Echo tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
        );

        let def = adapter.definition();
        assert_eq!(def.name, "echo");
        assert_eq!(def.parameters.len(), 1);

        let mut registry = ToolRegistry::new();
        adapter.register_into(&mut registry);

        let exec = registry.get_executor("echo").unwrap();
        let out = exec(json!({"message": "hi"})).unwrap();
        assert_eq!(out, "adapter-ok");
    }
}
