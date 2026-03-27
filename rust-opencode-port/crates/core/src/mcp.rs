use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl McpTool {
    pub fn validate_args(&self, args: &serde_json::Value) -> Result<(), String> {
        let schema = &self.parameters;

        if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
            let mut missing = Vec::new();
            for field in required {
                if let Some(field_name) = field.as_str() {
                    if args.get(field_name).is_none() {
                        missing.push(field_name.to_string());
                    }
                }
            }
            if !missing.is_empty() {
                return Err(format!("Missing required fields: {}", missing.join(", ")));
            }
        }

        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (field, field_schema) in properties {
                if let Some(value) = args.get(field) {
                    if let Some(expected_type) = field_schema.get("type").and_then(|t| t.as_str()) {
                        let actual_type = json_type_name(value);
                        if actual_type != expected_type && value != &serde_json::Value::Null {
                            return Err(format!(
                                "Field '{}' expected type '{}', got '{}'",
                                field, expected_type, actual_type
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn json_type_name(val: &serde_json::Value) -> &'static str {
    match val {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<McpPromptArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub name: String,
    pub url: String,
    pub tools: Vec<McpTool>,
    pub resources: Vec<McpResource>,
    pub prompts: Vec<McpPrompt>,
    pub capabilities: McpCapabilities,
}

impl McpServer {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
            capabilities: McpCapabilities {
                tools: false,
                resources: false,
                prompts: false,
            },
        }
    }

    pub fn find_resource(&self, uri: &str) -> Option<&McpResource> {
        self.resources.iter().find(|r| r.uri == uri)
    }

    pub fn find_prompt(&self, name: &str) -> Option<&McpPrompt> {
        self.prompts.iter().find(|p| p.name == name)
    }
}

pub struct McpManager {
    servers: HashMap<String, McpServer>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub fn register_server(&mut self, server: McpServer) {
        self.servers.insert(server.name.clone(), server);
    }

    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    pub fn get_server_mut(&mut self, name: &str) -> Option<&mut McpServer> {
        self.servers.get_mut(name)
    }

    pub fn list_servers(&self) -> Vec<&McpServer> {
        self.servers.values().collect()
    }

    pub fn list_tools(&self) -> Vec<&McpTool> {
        self.servers.values().flat_map(|s| s.tools.iter()).collect()
    }

    pub fn find_tool(&self, name: &str) -> Option<&McpTool> {
        self.servers
            .values()
            .flat_map(|s| s.tools.iter())
            .find(|t| t.name == name)
    }

    pub fn list_resources(&self) -> Vec<&McpResource> {
        self.servers
            .values()
            .flat_map(|s| s.resources.iter())
            .collect()
    }

    pub fn find_resource(&self, uri: &str) -> Option<&McpResource> {
        self.servers
            .values()
            .flat_map(|s| s.resources.iter())
            .find(|r| r.uri == uri)
    }

    pub fn list_prompts(&self) -> Vec<&McpPrompt> {
        self.servers
            .values()
            .flat_map(|s| s.prompts.iter())
            .collect()
    }

    pub fn validate_tool_args(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Result<(), String> {
        let tool = self
            .find_tool(tool_name)
            .ok_or_else(|| format!("Tool '{}' not found in any MCP server", tool_name))?;
        tool.validate_args(args)
    }

    pub fn update_server_discovery(
        &mut self,
        name: &str,
        tools: Vec<McpTool>,
        resources: Vec<McpResource>,
        prompts: Vec<McpPrompt>,
    ) -> bool {
        if let Some(server) = self.servers.get_mut(name) {
            server.capabilities.tools = !tools.is_empty();
            server.capabilities.resources = !resources.is_empty();
            server.capabilities.prompts = !prompts.is_empty();
            server.tools = tools;
            server.resources = resources;
            server.prompts = prompts;
            true
        } else {
            false
        }
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_tool_validate_args_ok() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: json!({
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
        };
        assert!(tool.validate_args(&json!({"path": "/foo"})).is_ok());
    }

    #[test]
    fn test_mcp_tool_validate_args_missing_required() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: json!({
                "required": ["path"],
                "properties": { "path": { "type": "string" } }
            }),
        };
        let result = tool.validate_args(&json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path"));
    }

    #[test]
    fn test_mcp_tool_validate_args_wrong_type() {
        let tool = McpTool {
            name: "test".to_string(),
            description: "test".to_string(),
            parameters: json!({
                "properties": { "count": { "type": "number" } }
            }),
        };
        let result = tool.validate_args(&json!({"count": "not-a-number"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_mcp_manager_discovery_update() {
        let mut mgr = McpManager::new();
        mgr.register_server(McpServer::new("s1", "http://localhost"));
        let tools = vec![McpTool {
            name: "t1".to_string(),
            description: "d".to_string(),
            parameters: json!({}),
        }];
        let updated = mgr.update_server_discovery("s1", tools, vec![], vec![]);
        assert!(updated);
        assert!(mgr.get_server("s1").unwrap().capabilities.tools);
        assert_eq!(mgr.list_tools().len(), 1);
    }
}
