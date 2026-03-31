use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use opencode_tools::{Tool, ToolResult};
use opencode_core::OpenCodeError;

use crate::protocol::ToolDefinition;
use crate::client::McpClient;

pub struct McpToolBridge {
    client: Arc<McpClient>,
    tool_name: String,
    description: String,
    input_schema: Value,
}

impl McpToolBridge {
    pub fn new(client: Arc<McpClient>, tool_def: &ToolDefinition) -> Self {
        Self {
            client,
            tool_name: tool_def.name.clone(),
            description: tool_def.description.clone(),
            input_schema: tool_def.input_schema.clone(),
        }
    }
}

#[async_trait]
impl Tool for McpToolBridge {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(McpToolBridge {
            client: self.client.clone(),
            tool_name: self.tool_name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
        })
    }

    async fn execute(&self, args: Value, _ctx: Option<opencode_tools::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let request = self.client.call_tool(&self.tool_name, args).await;
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| OpenCodeError::Tool(format!("Failed to serialize MCP request: {}", e)))?;

        let response: crate::protocol::JsonRpcResponse = serde_json::from_str(&request_json)
            .map_err(|e| OpenCodeError::Tool(format!("Failed to parse MCP response: {}", e)))?;

        match self.client.handle_tool_call_response(response).await {
            Ok(result) => {
                let content = result.content.iter()
                    .map(|c| match c {
                        crate::protocol::ToolContent::Text { text } => text.clone(),
                        crate::protocol::ToolContent::Image { mime_type, .. } => format!("[Image: {}]", mime_type),
                        crate::protocol::ToolContent::Resource { resource } => {
                            format!("[Resource: {}]", resource.uri)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                if result.is_error.unwrap_or(false) {
                    Ok(ToolResult::err(content))
                } else {
                    Ok(ToolResult::ok(content))
                }
            }
            Err(e) => Err(OpenCodeError::Tool(format!("MCP tool execution failed: {}", e))),
        }
    }
}
