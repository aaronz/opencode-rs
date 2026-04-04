use std::future::Future;
use std::sync::Arc;

use opencode_core::{ToolDefinition, ToolExecutor, ToolParameter, ToolRegistry};

use crate::client::McpClient;

pub fn register_mcp_tools(client: &McpClient, registry: &mut ToolRegistry) {
    let client = client.clone();
    let tools = match run_async(async { client.list_tools().await }) {
        Ok(tools) => tools,
        Err(_) => return,
    };

    for tool in tools {
        let name = tool.name.clone();
        let description = tool.description.clone();
        let schema = tool.input_schema.clone();
        let call_name = name.clone();
        let call_client = client.clone();

        let definition = ToolDefinition {
            name,
            description,
            parameters: vec![ToolParameter {
                name: "args".to_string(),
                description: "MCP tool arguments object".to_string(),
                required: false,
                schema: schema.clone(),
            }],
        };

        let executor: ToolExecutor = Arc::new(move |args| {
            run_async(async { call_client.call_tool(&call_name, &args).await })
                .map(|result| result.content)
                .map_err(|e| e.to_string())
        });

        registry.register(definition, executor);
    }
}

fn run_async<T>(future: impl Future<Output = T>) -> T {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(future))
    } else {
        tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime for mcp tool registration")
            .block_on(future)
    }
}
