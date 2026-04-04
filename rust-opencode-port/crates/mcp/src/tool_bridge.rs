use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use opencode_tools::{Tool, ToolResult};
use opencode_core::OpenCodeError;

use crate::client::{McpClient, McpTool};

pub struct McpToolBridge {
    client: Arc<McpClient>,
    tool_name: String,
    description: String,
    input_schema: Value,
}

impl McpToolBridge {
    pub fn new(client: Arc<McpClient>, tool_def: &McpTool) -> Self {
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
        match self.client.call_tool(&self.tool_name, &args).await {
            Ok(result) => {
                if result.is_error {
                    Ok(ToolResult::err(result.content))
                } else {
                    Ok(ToolResult::ok(result.content))
                }
            }
            Err(e) => Err(OpenCodeError::Tool(format!("MCP tool execution failed: {}", e))),
        }
    }
}
