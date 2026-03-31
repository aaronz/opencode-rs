use serde_json::Value;
use tracing::{info, debug};

use crate::protocol::*;

pub struct McpClient {
    server_name: Option<String>,
    server_version: Option<String>,
    tools: Vec<ToolDefinition>,
    resources: Vec<ResourceDefinition>,
    initialized: bool,
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            server_name: None,
            server_version: None,
            tools: Vec::new(),
            resources: Vec::new(),
            initialized: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<JsonRpcResponse, String> {
        let request = JsonRpcRequest::new("initialize", Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "clientInfo": {
                "name": "opencode-rs",
                "version": "0.1.0"
            }
        })));
        
        info!("MCP client initialize");
        Ok(JsonRpcResponse::success(None, Value::Null))
    }

    pub async fn handle_initialize_response(&mut self, response: JsonRpcResponse) -> Result<(), String> {
        if let Some(result) = response.result {
            if let Some(server_info) = result.get("serverInfo") {
                self.server_name = server_info.get("name").and_then(|n| n.as_str()).map(String::from);
                self.server_version = server_info.get("version").and_then(|v| v.as_str()).map(String::from);
            }
            self.initialized = true;
            info!("MCP client initialized with server: {:?}", self.server_name);
            Ok(())
        } else if let Some(error) = response.error {
            Err(format!("Initialize failed: {}", error.message))
        } else {
            Err("Invalid initialize response".to_string())
        }
    }

    pub async fn send_initialized(&self) -> JsonRpcRequest {
        JsonRpcRequest::new("initialized", None)
    }

    pub async fn list_tools(&self) -> JsonRpcRequest {
        JsonRpcRequest::new("tools/list", None)
    }

    pub async fn handle_tools_list_response(&mut self, response: JsonRpcResponse) -> Result<Vec<ToolDefinition>, String> {
        if let Some(result) = response.result {
            if let Some(tools_array) = result.get("tools").and_then(|t| t.as_array()) {
                self.tools = tools_array.iter().filter_map(|t| {
                    serde_json::from_value(t.clone()).ok()
                }).collect();
                debug!("Loaded {} MCP tools", self.tools.len());
                Ok(self.tools.clone())
            } else {
                Ok(Vec::new())
            }
        } else if let Some(error) = response.error {
            Err(format!("List tools failed: {}", error.message))
        } else {
            Err("Invalid tools/list response".to_string())
        }
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> JsonRpcRequest {
        JsonRpcRequest::new("tools/call", Some(serde_json::json!({
            "name": name,
            "arguments": arguments
        })))
    }

    pub async fn handle_tool_call_response(&self, response: JsonRpcResponse) -> Result<ToolResult, String> {
        if let Some(result) = response.result {
            serde_json::from_value(result).map_err(|e| format!("Failed to parse tool result: {}", e))
        } else if let Some(error) = response.error {
            Err(format!("Tool call failed: {}", error.message))
        } else {
            Err("Invalid tools/call response".to_string())
        }
    }

    pub async fn list_resources(&self) -> JsonRpcRequest {
        JsonRpcRequest::new("resources/list", None)
    }

    pub async fn handle_resources_list_response(&mut self, response: JsonRpcResponse) -> Result<Vec<ResourceDefinition>, String> {
        if let Some(result) = response.result {
            if let Some(resources_array) = result.get("resources").and_then(|r| r.as_array()) {
                self.resources = resources_array.iter().filter_map(|r| {
                    serde_json::from_value(r.clone()).ok()
                }).collect();
                debug!("Loaded {} MCP resources", self.resources.len());
                Ok(self.resources.clone())
            } else {
                Ok(Vec::new())
            }
        } else if let Some(error) = response.error {
            Err(format!("List resources failed: {}", error.message))
        } else {
            Err("Invalid resources/list response".to_string())
        }
    }

    pub async fn read_resource(&self, uri: &str) -> JsonRpcRequest {
        JsonRpcRequest::new("resources/read", Some(serde_json::json!({
            "uri": uri
        })))
    }

    pub async fn handle_resource_read_response(&self, response: JsonRpcResponse) -> Result<String, String> {
        if let Some(result) = response.result {
            if let Some(contents) = result.get("contents").and_then(|c| c.as_array()) {
                if let Some(first) = contents.first() {
                    if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                        return Ok(text.to_string());
                    }
                }
            }
            Err("No content in resource response".to_string())
        } else if let Some(error) = response.error {
            Err(format!("Read resource failed: {}", error.message))
        } else {
            Err("Invalid resources/read response".to_string())
        }
    }

    pub fn get_tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    pub fn get_resources(&self) -> &[ResourceDefinition] {
        &self.resources
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}
