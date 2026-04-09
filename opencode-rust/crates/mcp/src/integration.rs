use std::future::Future;

use opencode_core::ToolRegistry;

use crate::client::McpClient;
use crate::tool_bridge::McpToolAdapter;

pub fn register_mcp_tools(client: &McpClient, registry: &mut ToolRegistry) {
    let client = client.clone();
    let tools = match run_async(async { client.list_tools().await }) {
        Ok(tools) => tools,
        Err(_) => return,
    };

    for tool in tools {
        let adapter = McpToolAdapter::new(std::sync::Arc::new(client.clone()), tool);
        adapter.register_into(registry);
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
