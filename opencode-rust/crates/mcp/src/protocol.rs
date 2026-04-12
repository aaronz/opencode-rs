use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

pub struct SchemaCache {
    cache: HashMap<String, CachedToolSchema>,
    max_age: Duration,
}

struct CachedToolSchema {
    tools: Vec<ToolDefinition>,
    cached_at: Instant,
}

impl SchemaCache {
    pub fn new(max_age_hours: u64) -> Self {
        Self {
            cache: HashMap::new(),
            max_age: Duration::from_secs(max_age_hours * 3600),
        }
    }

    pub fn get(&self, server_name: &str) -> Option<Vec<ToolDefinition>> {
        self.cache.get(server_name).and_then(|cached| {
            if cached.cached_at.elapsed() < self.max_age {
                Some(cached.tools.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&mut self, server_name: String, tools: Vec<ToolDefinition>) {
        self.cache.insert(
            server_name,
            CachedToolSchema {
                tools,
                cached_at: Instant::now(),
            },
        );
    }

    pub fn invalidate(&mut self, server_name: &str) {
        self.cache.remove(server_name);
    }

    pub fn invalidate_all(&mut self) {
        self.cache.clear();
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new(24)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpServerType {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum McpPermission {
    #[default]
    Ask,
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: McpServerType,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub url: Option<String>,
    pub enabled: bool,
    pub permission: McpPermission,
}

impl McpServerConfig {
    pub fn local(name: String, command: String, args: Vec<String>) -> Self {
        Self {
            name,
            server_type: McpServerType::Local,
            command: Some(command),
            args,
            env: HashMap::new(),
            url: None,
            enabled: true,
            permission: McpPermission::Allow,
        }
    }

    pub fn remote(name: String, url: String) -> Self {
        Self {
            name,
            server_type: McpServerType::Remote,
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            url: Some(url),
            enabled: true,
            permission: McpPermission::Ask,
        }
    }

    pub fn with_permission(mut self, permission: McpPermission) -> Self {
        self.permission = permission;
        self
    }
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.to_string(),
            params,
        }
    }

    pub fn with_id(mut self, id: Value) -> Self {
        self.id = Some(id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_roundtrip() {
        let request = JsonRpcRequest::new(
            "tools/call",
            Some(serde_json::json!({"name": "echo", "arguments": {"v": 1}})),
        )
        .with_id(serde_json::json!(1));

        let encoded = serde_json::to_string(&request).unwrap();
        let decoded: JsonRpcRequest = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.jsonrpc, "2.0");
        assert_eq!(decoded.method, "tools/call");
        assert_eq!(decoded.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn test_jsonrpc_response_roundtrip() {
        let response = JsonRpcResponse::success(
            Some(serde_json::json!(2)),
            serde_json::json!({"tools": [{"name": "search"}]}),
        );

        let encoded = serde_json::to_string(&response).unwrap();
        let decoded: JsonRpcResponse = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.id, Some(serde_json::json!(2)));
        assert!(decoded.error.is_none());
        assert!(decoded.result.is_some());
    }
}
