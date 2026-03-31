use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use tracing::{info, debug, warn};

use crate::protocol::*;

pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn execute(&self, arguments: Value) -> Result<ToolResult, String>;
}

pub trait ResourceHandler: Send + Sync {
    fn uri(&self) -> &str;
    fn name(&self) -> &str;
    fn mime_type(&self) -> Option<&str>;
    fn read(&self) -> Result<String, String>;
}

pub struct McpServer {
    name: String,
    version: String,
    tools: Arc<RwLock<HashMap<String, Box<dyn ToolHandler>>>>,
    resources: Arc<RwLock<HashMap<String, Box<dyn ResourceHandler>>>>,
    initialized: Arc<RwLock<bool>>,
}

impl McpServer {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn register_tool(&self, tool: Box<dyn ToolHandler>) {
        let name = tool.name().to_string();
        self.tools.write().await.insert(name.clone(), tool);
        debug!("Registered MCP tool: {}", name);
    }

    pub async fn register_resource(&self, resource: Box<dyn ResourceHandler>) {
        let uri = resource.uri().to_string();
        self.resources.write().await.insert(uri.clone(), resource);
        debug!("Registered MCP resource: {}", uri);
    }

    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone();
        
        match request.method.as_str() {
            "initialize" => self.handle_initialize(id, request.params).await,
            "initialized" => self.handle_initialized(id).await,
            "tools/list" => self.handle_tools_list(id).await,
            "tools/call" => self.handle_tools_call(id, request.params).await,
            "resources/list" => self.handle_resources_list(id).await,
            "resources/read" => self.handle_resources_read(id, request.params).await,
            _ => JsonRpcResponse::error(id, METHOD_NOT_FOUND, format!("Method not found: {}", request.method)),
        }
    }

    async fn handle_initialize(&self, id: Option<Value>, _params: Option<Value>) -> JsonRpcResponse {
        info!("MCP server initialize");
        
        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": self.name,
                "version": self.version
            }
        });
        
        JsonRpcResponse::success(id, result)
    }

    async fn handle_initialized(&self, id: Option<Value>) -> JsonRpcResponse {
        *self.initialized.write().await = true;
        info!("MCP server initialized");
        JsonRpcResponse::success(id, Value::Null)
    }

    async fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools = self.tools.read().await;
        let tool_list: Vec<Value> = tools.values().map(|tool| {
            serde_json::json!({
                "name": tool.name(),
                "description": tool.description(),
                "inputSchema": tool.input_schema()
            })
        }).collect();
        
        JsonRpcResponse::success(id, serde_json::json!({ "tools": tool_list }))
    }

    async fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string()),
        };

        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(name) => name.to_string(),
            None => return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing tool name".to_string()),
        };

        let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

        let tools = self.tools.read().await;
        match tools.get(&tool_name) {
            Some(tool) => {
                match tool.execute(arguments) {
                    Ok(result) => {
                        let result_value = serde_json::to_value(result).unwrap_or(Value::Null);
                        JsonRpcResponse::success(id, result_value)
                    }
                    Err(e) => {
                        JsonRpcResponse::success(id, serde_json::json!({
                            "content": [{ "type": "text", "text": e }],
                            "isError": true
                        }))
                    }
                }
            }
            None => JsonRpcResponse::error(id, METHOD_NOT_FOUND, format!("Tool not found: {}", tool_name)),
        }
    }

    async fn handle_resources_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let resources = self.resources.read().await;
        let resource_list: Vec<Value> = resources.values().map(|resource| {
            serde_json::json!({
                "uri": resource.uri(),
                "name": resource.name(),
                "mimeType": resource.mime_type()
            })
        }).collect();
        
        JsonRpcResponse::success(id, serde_json::json!({ "resources": resource_list }))
    }

    async fn handle_resources_read(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing params".to_string()),
        };

        let uri = match params.get("uri").and_then(|u| u.as_str()) {
            Some(uri) => uri.to_string(),
            None => return JsonRpcResponse::error(id, INVALID_PARAMS, "Missing resource URI".to_string()),
        };

        let resources = self.resources.read().await;
        match resources.get(&uri) {
            Some(resource) => {
                match resource.read() {
                    Ok(content) => {
                        JsonRpcResponse::success(id, serde_json::json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": resource.mime_type(),
                                "text": content
                            }]
                        }))
                    }
                    Err(e) => {
                        JsonRpcResponse::error(id, INTERNAL_ERROR, e)
                    }
                }
            }
            None => JsonRpcResponse::error(id, INVALID_PARAMS, format!("Resource not found: {}", uri)),
        }
    }
}
